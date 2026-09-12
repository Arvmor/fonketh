//! Keeping an installed game current
//!
//! The launcher runs this before starting the game: look up the latest
//! release, and when it differs from `VERSION`, download, verify and swap
//! `bin/` and `assets/` in. Failures never block playing the installed
//! version.

use crate::layout::Layout;
use crate::release::{GitHub, obtain_archive};
use crate::ui::Ui;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Environment variable that disables automatic updates when set to `0`, `false` or `off`
pub const AUTO_UPDATE_ENV: &str = "FONKETH_AUTO_UPDATE";

/// Persistent launcher settings, a tiny `key = value` file in the game's home
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Update silently on launch; otherwise only mention available updates
    pub auto_update: bool,
    /// Where the installer put the `fonketh` command, so uninstall can find it
    pub command: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_update: true,
            command: None,
        }
    }
}

impl Settings {
    pub fn load(layout: &Layout) -> Self {
        fs::read_to_string(layout.settings_file())
            .map(|s| Self::parse(&s))
            .unwrap_or_default()
    }

    pub fn save(&self, layout: &Layout) -> Result<()> {
        fs::create_dir_all(&layout.home)?;
        fs::write(layout.settings_file(), self.render())
            .with_context(|| format!("writing {}", layout.settings_file().display()))
    }

    fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "auto_update" => {
                    settings.auto_update = parse_bool(value).unwrap_or(settings.auto_update);
                }
                "command" if !value.trim().is_empty() => {
                    settings.command = Some(PathBuf::from(value.trim()));
                }
                _ => {}
            }
        }
        settings
    }

    fn render(&self) -> String {
        format!(
            "# Fonketh launcher settings\n\
             # auto_update: update the game before every launch (true) or only mention new versions (false)\n\
             auto_update = {}\n\
             # command: the `fonketh` command the installer created\n\
             command = {}\n",
            self.auto_update,
            self.command
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        )
    }

    /// Effective policy: the environment variable wins over the file
    pub fn auto_update_enabled(&self) -> bool {
        match std::env::var(AUTO_UPDATE_ENV) {
            Ok(value) => parse_bool(&value).unwrap_or(self.auto_update),
            Err(_) => self.auto_update,
        }
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Result of comparing the installed version with GitHub
pub enum Check {
    UpToDate(String),
    Available { installed: String, latest: String },
}

/// Asks GitHub for the latest tag and compares it with `VERSION`
pub fn check(github: &GitHub, layout: &Layout) -> Result<Check> {
    let installed = layout
        .installed_version()
        .context("no VERSION file, is the game installed?")?;
    let latest = github.latest_version()?;
    Ok(if latest == installed {
        Check::UpToDate(installed)
    } else {
        Check::Available { installed, latest }
    })
}

/// Downloads `version` and swaps it in over the installed game
pub fn apply(github: &GitHub, layout: &Layout, version: &str) -> Result<()> {
    let workdir = tempfile::Builder::new()
        .prefix("fonketh-update-")
        .tempdir()
        .context("creating a temporary directory")?;
    let (archive, _) = obtain_archive(github, version, None, workdir.path())?;
    layout.install_files(&archive, version, workdir.path())?;
    Ui::ok(format!("Updated to {version}"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let custom = Settings {
            auto_update: false,
            command: Some(PathBuf::from("/opt/bin/fonketh")),
        };
        assert_eq!(Settings::parse(&custom.render()), custom);
        assert_eq!(Settings::parse(""), Settings::default());
        assert_eq!(
            Settings::parse("auto_update = nonsense"),
            Settings::default()
        );
        assert_eq!(
            Settings::parse("# comment\nauto_update=off\ncommand=\n"),
            Settings {
                auto_update: false,
                command: None
            }
        );
    }
}
