//! Finding, downloading and verifying release archives on GitHub

use anyhow::{Context, Result, bail};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Rust target triple this installer was built for, the release archive must match
pub const TARGET: &str = env!("TARGET");

const USER_AGENT: &str = concat!("fonketh-installer/", env!("CARGO_PKG_VERSION"));

/// Archive extension for this platform
pub const ARCHIVE_EXT: &str = if cfg!(windows) { "zip" } else { "tar.gz" };

/// File name of the release archive for a version on this platform
pub fn archive_name(version: &str) -> String {
    format!("fonketh-{version}-{TARGET}.{ARCHIVE_EXT}")
}

/// Recovers the version from an archive file name, `fonketh-<tag>-<target>.<ext>`
///
/// Tags may contain dashes (`v0.2.0-rc1`), so the known target is stripped
/// rather than splitting on `-`.
pub fn version_from_archive_name(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let stem = name.strip_suffix(&format!(".{ARCHIVE_EXT}"))?;
    let tag = stem
        .strip_prefix("fonketh-")?
        .strip_suffix(&format!("-{TARGET}"))?;
    tag.starts_with('v').then(|| tag.to_string())
}

/// Normalises `0.2.0` to `v0.2.0`
pub fn normalize_version(version: &str) -> String {
    let version = version.trim();
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

/// Environment variable pointing at a GitHub-compatible mirror (`https://github.com` by default)
///
/// The server must answer `/<repo>/releases/latest` with a redirect to
/// `/<repo>/releases/tag/<tag>` and serve `/<repo>/releases/download/<tag>/<file>`.
pub const BASE_URL_ENV: &str = "FONKETH_RELEASES_URL";

/// Talks to GitHub without the API, so unauthenticated installs never hit rate limits
pub struct GitHub {
    base: String,
    repo: String,
    /// Follows redirects, used for downloads
    client: Client,
    /// Stops at the first redirect, used to read where `releases/latest` points
    peek: Client,
}

impl GitHub {
    pub fn new(repo: &str) -> Result<Self> {
        let base = || {
            Client::builder()
                .user_agent(USER_AGENT)
                .connect_timeout(Duration::from_secs(20))
                .timeout(Duration::from_secs(600))
        };
        let base_url = std::env::var(BASE_URL_ENV).unwrap_or_else(|_| "https://github.com".into());
        Ok(Self {
            base: base_url.trim_end_matches('/').to_string(),
            repo: repo.to_string(),
            client: base().build()?,
            peek: base().redirect(Policy::none()).build()?,
        })
    }

    /// Latest release tag: `github.com/<repo>/releases/latest` redirects to `/releases/tag/<tag>`
    pub fn latest_version(&self) -> Result<String> {
        let url = format!("{}/{}/releases/latest", self.base, self.repo);
        let response = self
            .peek
            .head(&url)
            .send()
            .context("could not reach GitHub to look up the latest release")?;
        let location = response
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        match location.rsplit_once("/tag/") {
            Some((_, tag)) if tag.starts_with('v') => Ok(tag.to_string()),
            _ => bail!("could not determine the latest release (GitHub answered '{location}')"),
        }
    }

    fn asset_url(&self, version: &str, name: &str) -> String {
        format!(
            "{}/{}/releases/download/{version}/{name}",
            self.base, self.repo
        )
    }

    /// Downloads a release asset to `dest` with a progress bar, `Ok(false)` when it does not exist
    pub fn download(&self, version: &str, name: &str, dest: &Path, quiet: bool) -> Result<bool> {
        let response = self
            .client
            .get(self.asset_url(version, name))
            .send()
            .with_context(|| format!("downloading {name}"))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(false);
        }
        let mut response = response
            .error_for_status()
            .with_context(|| format!("downloading {name}"))?;

        let mut file = File::create(dest)?;
        if quiet {
            io::copy(&mut response, &mut file)?;
            return Ok(true);
        }

        let bar = match response.content_length() {
            Some(len) => ProgressBar::new(len).with_style(
                ProgressStyle::with_template(
                    "  {bar:30.cyan/dim} {bytes:>9}/{total_bytes:9} {eta:>4}",
                )?
                .progress_chars("━━╌"),
            ),
            None => ProgressBar::new_spinner(),
        };
        io::copy(&mut bar.wrap_read(&mut response), &mut file)?;
        bar.finish_and_clear();
        Ok(true)
    }
}

/// Where a release archive came from
pub enum Source {
    /// Fetched from GitHub, checksum verified against `SHA256SUMS.txt` when present
    Download,
    /// Supplied with `--archive`
    Local,
}

/// Fetches (or accepts) the archive for `version` and returns its path
pub fn obtain_archive(
    github: &GitHub,
    version: &str,
    local: Option<&Path>,
    workdir: &Path,
) -> Result<(PathBuf, Source)> {
    if let Some(path) = local {
        anyhow::ensure!(path.is_file(), "archive not found: {}", path.display());
        return Ok((path.to_path_buf(), Source::Local));
    }

    let name = archive_name(version);
    let dest = workdir.join(&name);
    eprintln!("  Downloading {name}");
    if !github.download(version, &name, &dest, false)? {
        bail!(
            "release {version} has no build for {TARGET}.\n  \
             Releases older than the installer shipped bare binaries; pick a newer one with\n  \
             --version, or build from source:\n    \
             git clone https://github.com/{} && cd fonketh && cargo run -p game_app -F interface --release",
            github.repo
        );
    }
    crate::ui::Ui::ok(format!("Downloaded {}", human_size(dest.metadata()?.len())));

    let sums = workdir.join("SHA256SUMS.txt");
    if github.download(version, "SHA256SUMS.txt", &sums, true)? {
        verify_checksum(&dest, &name, &std::fs::read_to_string(&sums)?)?;
    } else {
        crate::ui::Ui::warn("Release has no SHA256SUMS.txt, skipping checksum verification");
    }
    Ok((dest, Source::Download))
}

/// Checks `file` against the `sha256sum` style line for `name` in `sums`
pub fn verify_checksum(file: &Path, name: &str, sums: &str) -> Result<()> {
    let Some(expected) = expected_checksum(sums, name) else {
        crate::ui::Ui::warn("Archive missing from SHA256SUMS.txt, skipping checksum verification");
        return Ok(());
    };
    let mut hasher = Sha256::new();
    io::copy(&mut File::open(file)?, &mut hasher)?;
    let actual = hex::encode(hasher.finalize());
    if actual != expected {
        bail!("checksum mismatch for {name}\n  expected {expected}\n  got      {actual}");
    }
    crate::ui::Ui::ok("Checksum verified");
    Ok(())
}

/// Finds the hash for `name` in `sha256sum` output (`<hex>  <name>` per line)
pub fn expected_checksum(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let file = parts.next()?.trim_start_matches('*');
        (file == name).then(|| hash.to_ascii_lowercase())
    })
}

fn human_size(bytes: u64) -> String {
    const MB: f64 = 1024.0 * 1024.0;
    format!("{:.1} MB", bytes as f64 / MB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_round_trips_through_archive_name() {
        let path = PathBuf::from(archive_name("v0.2.0-rc1"));
        assert_eq!(
            version_from_archive_name(&path).as_deref(),
            Some("v0.2.0-rc1")
        );
    }

    #[test]
    fn foreign_archive_names_are_rejected() {
        assert_eq!(
            version_from_archive_name(Path::new("fonketh-v0.2.0-some-other-target.tar.gz")),
            None
        );
        assert_eq!(version_from_archive_name(Path::new("random.tar.gz")), None);
    }

    #[test]
    fn checksum_lookup_matches_exact_name() {
        let sums = "abc  fonketh-v1-x.tar.gz\nDEF *fonketh-v1-y.zip\n";
        assert_eq!(
            expected_checksum(sums, "fonketh-v1-y.zip").as_deref(),
            Some("def")
        );
        assert_eq!(expected_checksum(sums, "fonketh-v1-z.zip"), None);
    }

    #[test]
    fn versions_get_a_v_prefix() {
        assert_eq!(normalize_version("0.2.0"), "v0.2.0");
        assert_eq!(normalize_version(" v0.2.0 "), "v0.2.0");
    }
}
