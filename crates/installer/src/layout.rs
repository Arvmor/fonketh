//! Where the game lives on disk and how it is wired into the system
//!
//! ```text
//! <home>/                    ~/.fonketh  or  %LOCALAPPDATA%\Programs\Fonketh
//! ├── bin/fonketh(.exe)      the game, replaced wholesale on update
//! ├── assets/                textures, replaced on update (composites are regenerated)
//! ├── fonketh-launcher(.exe) this program; checks for updates, then starts the game
//! ├── private.key            miner key, never touched by updates
//! ├── launcher.conf          auto-update setting
//! ├── VERSION
//! └── env                    (unix) sourced by shell startup files to extend PATH
//! ```
//!
//! `~/.local/bin/fonketh` (or `fonketh.cmd` next to the launcher on Windows)
//! is a one-line script that runs the launcher, which starts the game from
//! `<home>` so the key and sprite composites land there. The launcher lives
//! outside `bin/` so it can replace that directory while running.

use crate::ui::Ui;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

/// Name of the game binary inside `bin/`
pub const GAME_BIN: &str = if cfg!(windows) {
    "fonketh.exe"
} else {
    "fonketh"
};
/// Name of the launcher/updater binary in the home directory (and in release archives)
pub const LAUNCHER_BIN: &str = if cfg!(windows) {
    "fonketh-launcher.exe"
} else {
    "fonketh-launcher"
};

/// Install locations
#[derive(Debug, Clone)]
pub struct Layout {
    /// Directory holding `bin/`, `assets/` and the key
    pub home: PathBuf,
    /// Directory the launcher goes into (unix only; on Windows it sits in `home`)
    pub bin_dir: PathBuf,
}

impl Layout {
    /// Platform defaults, overridable through `FONKETH_HOME` / `FONKETH_BIN_DIR`
    pub fn default_paths() -> Result<Self> {
        let home = match std::env::var_os("FONKETH_HOME") {
            Some(dir) => PathBuf::from(dir),
            None if cfg!(windows) => dirs::data_local_dir()
                .context("no local app data directory")?
                .join("Programs")
                .join("Fonketh"),
            None => dirs::home_dir()
                .context("no home directory")?
                .join(".fonketh"),
        };
        let bin_dir = match std::env::var_os("FONKETH_BIN_DIR") {
            Some(dir) => PathBuf::from(dir),
            None if cfg!(windows) => home.clone(),
            None => dirs::home_dir()
                .context("no home directory")?
                .join(".local")
                .join("bin"),
        };
        Ok(Self { home, bin_dir })
    }

    /// Makes relative paths absolute against the current directory
    pub fn absolute(mut self) -> Result<Self> {
        let cwd = std::env::current_dir()?;
        if self.home.is_relative() {
            self.home = cwd.join(&self.home);
        }
        if self.bin_dir.is_relative() {
            self.bin_dir = cwd.join(&self.bin_dir);
        }
        Ok(self)
    }

    pub fn game_binary(&self) -> PathBuf {
        self.home.join("bin").join(GAME_BIN)
    }

    pub fn key_file(&self) -> PathBuf {
        self.home.join("private.key")
    }

    pub fn version_file(&self) -> PathBuf {
        self.home.join("VERSION")
    }

    pub fn env_file(&self) -> PathBuf {
        self.home.join("env")
    }

    pub fn settings_file(&self) -> PathBuf {
        self.home.join("launcher.conf")
    }

    /// The launcher/updater binary (a copy of this installer)
    pub fn launcher_bin(&self) -> PathBuf {
        self.home.join(LAUNCHER_BIN)
    }

    /// The `fonketh` command users type; a tiny script that runs the launcher
    pub fn launcher(&self) -> PathBuf {
        if cfg!(windows) {
            self.home.join("fonketh.cmd")
        } else {
            self.bin_dir.join("fonketh")
        }
    }

    /// Version currently installed, if any
    pub fn installed_version(&self) -> Option<String> {
        fs::read_to_string(self.version_file())
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    /// Unpacks `archive` and moves `bin/` and `assets/` into place, keeping everything else
    pub fn install_files(&self, archive: &Path, version: &str, workdir: &Path) -> Result<()> {
        let unpack = workdir.join("unpack");
        fs::create_dir_all(&unpack)?;
        extract(archive, &unpack)?;

        // Archives contain a single top-level folder named after the release
        let src = fs::read_dir(&unpack)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.is_dir())
            .context("unexpected archive layout (no top-level folder)")?;
        if !src.join("bin").join(GAME_BIN).is_file() {
            bail!("unexpected archive layout (no bin/{GAME_BIN})");
        }
        if !src.join("assets").is_dir() {
            bail!("unexpected archive layout (no assets/)");
        }

        fs::create_dir_all(&self.home)?;
        // Swap the new directories in, restoring the old ones if anything fails
        let mut backups = Vec::new();
        for dir in ["bin", "assets"] {
            let dest = self.home.join(dir);
            let backup = self.home.join(format!("{dir}.old"));
            if backup.exists() {
                fs::remove_dir_all(&backup)?;
            }
            if dest.exists() {
                fs::rename(&dest, &backup)?;
                backups.push((backup, dest.clone()));
            }
            if let Err(e) = move_dir(&src.join(dir), &dest) {
                for (backup, dest) in backups {
                    let _ = fs::remove_dir_all(&dest);
                    let _ = fs::rename(backup, dest);
                }
                return Err(e).with_context(|| format!("installing {dir}/"));
            }
        }
        for (backup, _) in backups {
            let _ = fs::remove_dir_all(backup);
        }
        // Newer releases ship the launcher too
        let shipped_launcher = src.join(LAUNCHER_BIN);
        if shipped_launcher.is_file() {
            self.install_launcher_bin(&shipped_launcher)?;
        }
        let readme = src.join("README.md");
        if readme.is_file() {
            fs::copy(&readme, self.home.join("README.md"))?;
        }
        let shipped = fs::read_to_string(src.join("VERSION")).unwrap_or_default();
        let shipped = shipped.trim();
        fs::write(
            self.version_file(),
            format!("{}\n", if shipped.is_empty() { version } else { shipped }),
        )?;

        make_executable(&self.game_binary())?;
        unquarantine(&self.game_binary());
        Ui::ok(format!("Game files installed to {}", self.home.display()));
        Ok(())
    }

    /// Writes the launcher that starts the game from its home directory
    pub fn install_launcher(&self) -> Result<()> {
        let launcher = self.launcher();
        fs::create_dir_all(launcher.parent().context("launcher has no parent")?)?;
        let contents = if cfg!(windows) {
            format!(
                "@echo off\r\n\
                 rem Fonketh, generated by fonketh-installer: update if needed, then play\r\n\
                 \"%~dp0{LAUNCHER_BIN}\" launch %*\r\n"
            )
        } else {
            format!(
                "#!/bin/sh\n\
                 # Fonketh, generated by fonketh-installer: update if needed, then play\n\
                 FONKETH_HOME=\"${{FONKETH_HOME:-{home}}}\"\n\
                 exec \"$FONKETH_HOME/{LAUNCHER_BIN}\" launch \"$@\"\n",
                home = self.home.display()
            )
        };
        fs::write(&launcher, contents)?;
        make_executable(&launcher)?;
        Ui::ok(format!("Launcher installed to {}", launcher.display()));
        Ok(())
    }

    /// Copies `source` (this installer, or the one shipped in a release) over the launcher binary
    ///
    /// The running launcher may be the file being replaced: renaming it away
    /// first works on every platform, including Windows where a running
    /// executable cannot be deleted or overwritten.
    pub fn install_launcher_bin(&self, source: &Path) -> Result<()> {
        let dest = self.launcher_bin();
        if let Ok(true) = same_file(source, &dest) {
            return Ok(());
        }
        let stale = self.stale_launcher_bin();
        let _ = fs::remove_file(&stale);
        if dest.exists() {
            fs::rename(&dest, &stale)?;
        }
        fs::copy(source, &dest).with_context(|| format!("installing {}", dest.display()))?;
        make_executable(&dest)?;
        unquarantine(&dest);
        // Gone on unix; on Windows it lingers while running and is removed next start
        let _ = fs::remove_file(&stale);
        Ok(())
    }

    /// Previous launcher binary left behind by a self-update
    pub fn stale_launcher_bin(&self) -> PathBuf {
        self.home.join(format!("{LAUNCHER_BIN}.old"))
    }

    /// Directory that has to be on PATH for `fonketh` to resolve
    pub fn path_dir(&self) -> &Path {
        if cfg!(windows) {
            &self.home
        } else {
            &self.bin_dir
        }
    }

    /// Removes everything the installer created; the key is left alone unless `purge`
    pub fn uninstall(&self, purge: bool) -> Result<()> {
        for dir in ["bin", "assets"] {
            let p = self.home.join(dir);
            if p.exists() {
                fs::remove_dir_all(&p)?;
            }
        }
        for file in [
            self.home.join("README.md"),
            self.version_file(),
            self.env_file(),
            self.settings_file(),
            self.launcher(),
            self.stale_launcher_bin(),
        ] {
            if file.exists() {
                fs::remove_file(&file)?;
            }
        }
        // The launcher may be the program running this; Windows will not delete a
        // running executable, so fall back to renaming it out of the way
        let launcher = self.launcher_bin();
        if launcher.exists() && fs::remove_file(&launcher).is_err() {
            let _ = fs::rename(&launcher, self.stale_launcher_bin());
        }
        platform::remove_from_path(self)?;
        platform::remove_shortcuts();
        if purge && self.key_file().exists() {
            fs::remove_file(self.key_file())?;
        }
        // Only disappears when nothing (like the key) is left
        let _ = fs::remove_dir(&self.home);
        Ok(())
    }
}

/// Unpacks a `.tar.gz` (unix) or `.zip` (Windows) into `dest`
fn extract(archive: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive)?;
    let name = archive.to_string_lossy();
    if name.ends_with(".zip") {
        zip::ZipArchive::new(file)?.extract(dest)?;
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        tar::Archive::new(flate2::read::GzDecoder::new(file)).unpack(dest)?;
    } else {
        bail!("unsupported archive type: {name}");
    }
    Ok(())
}

/// Whether two paths point at the same file
fn same_file(a: &Path, b: &Path) -> Result<bool> {
    if !a.exists() || !b.exists() {
        return Ok(false);
    }
    Ok(a.canonicalize()? == b.canonicalize()?)
}

/// Rename when possible, copy across filesystems otherwise
fn move_dir(from: &Path, to: &Path) -> Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    copy_dir(from, to)?;
    fs::remove_dir_all(from)?;
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Drops the "downloaded from the internet" mark so the OS complains less
fn unquarantine(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("xattr")
            .args(["-dr", "com.apple.quarantine"])
            .arg(path)
            .output();
    }
    #[cfg(windows)]
    {
        let mut ads = path.as_os_str().to_owned();
        ads.push(":Zone.Identifier");
        let _ = fs::remove_file(ads);
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    let _ = path;
}

/// Restricts a file to the current user
pub fn make_private(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(windows)]
    {
        let user = std::env::var("USERNAME").unwrap_or_default();
        let status = std::process::Command::new("icacls")
            .arg(path)
            .args(["/inheritance:r", "/grant:r", &format!("{user}:F")])
            .output();
        if !status.map(|o| o.status.success()).unwrap_or(false) {
            Ui::warn("Could not restrict permissions on the key file");
        }
    }
    Ok(())
}

/// PATH and shortcut handling, one implementation per platform family
#[cfg(unix)]
pub mod platform {
    use super::Layout;
    use crate::ui::Ui;
    use anyhow::{Context, Result};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn on_path(dir: &Path) -> bool {
        std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).any(|d| d == dir))
            .unwrap_or(false)
    }

    fn home() -> Result<PathBuf> {
        dirs::home_dir().context("no home directory")
    }

    fn startup_files() -> Result<Vec<PathBuf>> {
        let home = home()?;
        Ok(
            [".profile", ".bashrc", ".bash_profile", ".zshrc", ".zshenv"]
                .iter()
                .map(|f| home.join(f))
                .collect(),
        )
    }

    fn fish_conf() -> Option<PathBuf> {
        dirs::config_dir().map(|c| c.join("fish").join("conf.d").join("fonketh.fish"))
    }

    /// Adds the launcher directory to PATH via a rustup-style `env` file
    pub fn add_to_path(layout: &Layout, ui: &Ui, allowed: bool) -> Result<()> {
        let dir = layout.path_dir();
        if on_path(dir) {
            Ui::ok(format!("{} is already on your PATH", dir.display()));
            return Ok(());
        }

        let env_file = layout.env_file();
        fs::write(
            &env_file,
            format!(
                "#!/bin/sh\n\
                 # Fonketh: add the launcher directory to PATH\n\
                 case \":${{PATH}}:\" in\n\
                 \x20   *:\"{dir}\":*) ;;\n\
                 \x20   *) export PATH=\"{dir}:$PATH\" ;;\n\
                 esac\n",
                dir = dir.display()
            ),
        )?;
        let line = format!(". \"{}\"", env_file.display());

        if !allowed {
            Ui::warn(format!(
                "{} is not on your PATH. Add this to your shell startup file:",
                dir.display()
            ));
            Ui::say(format!("    {line}"));
            return Ok(());
        }
        if !ui.confirm(
            &format!(
                "Add {} to your PATH (edits shell startup files)?",
                dir.display()
            ),
            true,
        )? {
            Ui::warn("Skipped. To run the game later add this to your shell startup file:");
            Ui::say(format!("    {line}"));
            return Ok(());
        }

        let marker = env_file.display().to_string();
        let mut updated = Vec::new();
        for rc in startup_files()? {
            if !rc.is_file() {
                continue;
            }
            if fs::read_to_string(&rc)?.contains(&marker) {
                continue;
            }
            append(&rc, &line)?;
            updated.push(rc);
        }
        if updated.is_empty() {
            // Nothing existed yet: zsh users get a .zshrc, everyone else a .profile
            let shell = std::env::var("SHELL").unwrap_or_default();
            let rc = home()?.join(if shell.ends_with("zsh") {
                ".zshrc"
            } else {
                ".profile"
            });
            append(&rc, &line)?;
            updated.push(rc);
        }
        if let Some(fish) = fish_conf()
            && which("fish")
        {
            if let Some(parent) = fish.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(
                &fish,
                format!("fish_add_path --prepend --global \"{}\"\n", dir.display()),
            )?;
            updated.push(fish);
        }
        let names: Vec<String> = updated.iter().map(|p| p.display().to_string()).collect();
        Ui::ok(format!("PATH updated in: {}", names.join(", ")));
        Ui::note(format!("Open a new terminal, or run: {line}"));
        Ok(())
    }

    /// Undoes `add_to_path`
    pub fn remove_from_path(layout: &Layout) -> Result<()> {
        let marker = layout.env_file().display().to_string();
        for rc in startup_files()? {
            let Ok(contents) = fs::read_to_string(&rc) else {
                continue;
            };
            if !contents.contains(&marker) {
                continue;
            }
            let kept: Vec<&str> = contents.lines().filter(|l| !l.contains(&marker)).collect();
            fs::write(&rc, format!("{}\n", kept.join("\n")))?;
        }
        if let Some(fish) = fish_conf()
            && fish.exists()
        {
            fs::remove_file(fish)?;
        }
        Ok(())
    }

    /// No desktop shortcuts on unix, the launcher on PATH is the entry point
    pub fn create_shortcuts(_layout: &Layout, _ui: &Ui) -> Result<()> {
        Ok(())
    }

    pub fn remove_shortcuts() {}

    /// Platform-specific hints shown at the end
    pub fn hints() -> Vec<&'static str> {
        if cfg!(target_os = "macos") {
            vec![
                "macOS: the game is not notarized. If Gatekeeper blocks it, run `fonketh` from a terminal once.",
            ]
        } else {
            vec!["Linux: the game needs ALSA, udev, Wayland/X11 and xkbcommon runtime libraries."]
        }
    }

    fn append(path: &Path, line: &str) -> Result<()> {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "\n{line}")?;
        Ok(())
    }

    fn which(bin: &str) -> bool {
        std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).any(|d| d.join(bin).is_file()))
            .unwrap_or(false)
    }
}

#[cfg(windows)]
pub mod platform {
    use super::Layout;
    use crate::ui::Ui;
    use anyhow::Result;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use winreg::RegKey;
    use winreg::enums::*;

    fn user_path() -> Result<(RegKey, Vec<String>)> {
        let key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
        let current: String = key.get_value("Path").unwrap_or_default();
        let parts = current
            .split(';')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect();
        Ok((key, parts))
    }

    fn write_user_path(key: &RegKey, parts: &[String]) -> Result<()> {
        let joined = parts.join(";");
        // REG_EXPAND_SZ keeps entries like %SystemRoot% working
        let bytes: Vec<u8> = joined
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        key.set_raw_value(
            "Path",
            &winreg::RegValue {
                bytes,
                vtype: REG_EXPAND_SZ,
            },
        )?;
        Ok(())
    }

    fn same_dir(a: &str, b: &Path) -> bool {
        a.trim_end_matches('\\')
            .eq_ignore_ascii_case(b.to_string_lossy().trim_end_matches('\\'))
    }

    /// Adds the install directory to the user PATH in the registry
    pub fn add_to_path(layout: &Layout, ui: &Ui, allowed: bool) -> Result<()> {
        let dir = layout.path_dir();
        let (key, mut parts) = user_path()?;
        if parts.iter().any(|p| same_dir(p, dir)) {
            Ui::ok(format!("{} is already on your PATH", dir.display()));
            return Ok(());
        }
        if !allowed {
            Ui::warn(format!(
                "{} is not on your PATH, add it to run `fonketh` from any terminal",
                dir.display()
            ));
            return Ok(());
        }
        if !ui.confirm(&format!("Add {} to your PATH?", dir.display()), true)? {
            Ui::warn("Skipped");
            return Ok(());
        }
        parts.push(dir.to_string_lossy().into_owned());
        write_user_path(&key, &parts)?;
        Ui::ok("PATH updated (new terminals pick it up)");
        Ok(())
    }

    pub fn remove_from_path(layout: &Layout) -> Result<()> {
        let dir = layout.path_dir();
        let (key, parts) = user_path()?;
        let kept: Vec<String> = parts.into_iter().filter(|p| !same_dir(p, dir)).collect();
        write_user_path(&key, &kept)?;
        Ok(())
    }

    fn shortcut_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(appdata) = dirs::data_dir() {
            paths.push(
                appdata
                    .join("Microsoft")
                    .join("Windows")
                    .join("Start Menu")
                    .join("Programs")
                    .join("Fonketh.lnk"),
            );
        }
        if let Some(desktop) = dirs::desktop_dir() {
            paths.push(desktop.join("Fonketh.lnk"));
        }
        paths
    }

    fn create_shortcut(lnk: &Path, layout: &Layout) -> Result<()> {
        // .lnk files are COM territory; let PowerShell do it
        let script = format!(
            "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{lnk}');\
             $s.TargetPath='{launcher}';$s.Arguments='launch';$s.WorkingDirectory='{home}';\
             $s.IconLocation='{exe},0';$s.Description='Fonketh, the gamified P2P mining pool';$s.Save()",
            lnk = lnk.display(),
            launcher = layout.launcher_bin().display(),
            exe = layout.game_binary().display(),
            home = layout.home.display(),
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()?;
        anyhow::ensure!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(())
    }

    /// Start Menu shortcut always, Desktop shortcut on request
    pub fn create_shortcuts(layout: &Layout, ui: &Ui) -> Result<()> {
        let paths = shortcut_paths();
        if let Some(start) = paths.first() {
            match create_shortcut(start, layout) {
                Ok(()) => Ui::ok("Start Menu shortcut created"),
                Err(e) => Ui::warn(format!("Could not create Start Menu shortcut: {e}")),
            }
        }
        if let Some(desktop) = paths.get(1)
            && ui.confirm("Create a Desktop shortcut?", true)?
        {
            match create_shortcut(desktop, layout) {
                Ok(()) => Ui::ok("Desktop shortcut created"),
                Err(e) => Ui::warn(format!("Could not create Desktop shortcut: {e}")),
            }
        }
        Ok(())
    }

    pub fn remove_shortcuts() {
        for lnk in shortcut_paths() {
            let _ = std::fs::remove_file(lnk);
        }
    }

    pub fn hints() -> Vec<&'static str> {
        vec![
            "Windows may show a SmartScreen warning on first launch (the game is not signed): More info -> Run anyway.",
            "Allow the firewall prompt so peers can reach you on UDP port 7331.",
        ]
    }
}
