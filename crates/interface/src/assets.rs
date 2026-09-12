//! Locates the game's `assets/` directory.
//!
//! The game is run from three very different places and the textures have to
//! be found in all of them:
//!
//! 1. `cargo run` from a checkout: Cargo sets `CARGO_MANIFEST_DIR` to the crate
//!    being run (`crates/app`, or `crates/interface` for the preview example),
//!    and the assets live two levels up at the workspace root.
//! 2. An installed release: the installer lays out `bin/fonketh` next to an
//!    `assets/` folder, so the root is the executable's grandparent.
//! 3. A standalone binary dropped next to an `assets/` folder, or run from a
//!    directory that contains one.
//!
//! `FONKETH_ASSETS` overrides everything and must point at the directory that
//! *contains* `assets/`.

use bevy::prelude::*;
use std::env;
use std::path::{Path, PathBuf};

/// Environment variable overriding the assets root
pub const ASSETS_ROOT_ENV: &str = "FONKETH_ASSETS";
/// Folder name holding the textures, relative to the root
pub const ASSETS_DIR: &str = "assets";

/// Directory containing the `assets/` folder
#[derive(Resource, Debug, Clone)]
pub struct AssetsRoot(pub PathBuf);

impl AssetsRoot {
    /// Resolves the assets root, see the module docs for the search order
    pub fn locate() -> Self {
        Self(locate_root())
    }

    /// Path to the `assets/textures` directory
    pub fn textures(&self) -> PathBuf {
        self.0.join(ASSETS_DIR).join("textures")
    }

    /// Turns an on-disk path under the root into a path Bevy's asset server
    /// understands (relative to the root, forward slashes)
    ///
    /// Paths outside the root are returned untouched.
    pub fn asset_path(&self, path: &Path) -> String {
        let relative = path.strip_prefix(&self.0).unwrap_or(path);
        let parts: Vec<_> = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        parts.join("/")
    }
}

/// Candidate roots in priority order, first one holding `assets/` wins
fn locate_root() -> PathBuf {
    if let Some(root) = env::var_os(ASSETS_ROOT_ENV).map(PathBuf::from) {
        return root;
    }

    let mut candidates = Vec::new();
    if let Some(manifest) = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from) {
        candidates.push(manifest.join("..").join(".."));
        candidates.push(manifest);
    }
    if let Some(bin) = env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        candidates.push(bin.clone());
        candidates.push(bin.join(".."));
        candidates.push(bin.join("..").join(".."));
    }
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd);
    }

    candidates
        .iter()
        .find(|root| root.join(ASSETS_DIR).is_dir())
        .map(|root| root.canonicalize().unwrap_or_else(|_| root.clone()))
        .unwrap_or_else(|| {
            warn!(
                "Could not find an `{ASSETS_DIR}/` folder, set {ASSETS_ROOT_ENV} to the directory holding it"
            );
            PathBuf::from(".")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_path_is_relative_to_root() {
        let root = AssetsRoot(PathBuf::from("/opt/fonketh"));
        let path = root.textures().join("characters").join("mod_x.png");
        assert_eq!(
            root.asset_path(&path),
            "assets/textures/characters/mod_x.png"
        );
    }

    #[test]
    fn asset_path_outside_root_is_untouched() {
        let root = AssetsRoot(PathBuf::from("/opt/fonketh"));
        assert_eq!(
            root.asset_path(Path::new("assets/textures/x.png")),
            "assets/textures/x.png"
        );
    }

    #[test]
    fn locates_workspace_root_from_manifest_dir() {
        // Tests run with CARGO_MANIFEST_DIR = crates/interface
        let root = AssetsRoot::locate();
        assert!(root.textures().join("characters").is_dir(), "{root:?}");
    }
}
