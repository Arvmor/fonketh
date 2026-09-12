use bevy::prelude::*;

/// Top-level UI screen
///
/// The world (ground, players) renders on every screen; the screen decides
/// which overlay is shown and where keyboard input is routed.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    /// Start menu shown on launch
    #[default]
    Menu,
    /// In-game, HUD visible, input drives the character and chat
    Playing,
    /// Pause overlay on top of the HUD
    Paused,
}
