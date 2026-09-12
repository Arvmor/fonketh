use bevy::prelude::*;
use game_primitives::Identifier;
use std::time::Duration;

// ---------------------------------------------------------------------------
// World entities
// ---------------------------------------------------------------------------

/// Component to identify the right sprite
/// TODO - REMOVE THIS COMPONENT
#[derive(Component)]
pub struct RightSprite;

/// Component to identify player entities
#[derive(Component)]
pub struct PlayerEntity<I: Identifier> {
    pub peer_id: I::Id,
}

/// Component to identify the ground entity
#[derive(Component)]
pub struct Ground;

/// Component to identify the main/local player
#[derive(Component)]
pub struct MainPlayer;

/// Component to identify the animation configuration
#[derive(Component)]
pub struct AnimationConfig {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
}

impl AnimationConfig {
    /// Creates a new animation configuration
    pub fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    /// Creates a new timer from the FPS
    pub fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

// ---------------------------------------------------------------------------
// HUD
// ---------------------------------------------------------------------------

/// Root of the in-game HUD; hidden while the start menu is up
#[derive(Component)]
pub struct HudRoot;

/// Which live value a text node displays
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatValue {
    /// Treasures mined this session (never resets)
    SessionTotal,
    /// Treasures waiting in the current claim batch, as `n/10`
    PendingBatch,
    /// Claim batches submitted on-chain this session
    Claims,
    /// Players currently online
    PlayersOnline,
}

/// Fill bar showing progress towards the next on-chain claim
#[derive(Component)]
pub struct BatchProgressFill;

/// Container the chat rows are spawned into
#[derive(Component)]
pub struct ChatLog;

/// Chat input row (highlighted while typing)
#[derive(Component)]
pub struct ChatInputRow;

/// Text node showing the message being typed, or the placeholder
#[derive(Component)]
pub struct ChatInputField;

/// Blinking caret next to the input text
#[derive(Component)]
pub struct ChatCaret;

/// Container toasts are stacked into
#[derive(Component)]
pub struct ToastLayer;

/// A transient notification
#[derive(Component)]
pub struct Toast {
    pub timer: Timer,
    pub base_background: Color,
    pub base_border: Color,
}

// ---------------------------------------------------------------------------
// Menus
// ---------------------------------------------------------------------------

/// What a menu button does when activated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Leave the start menu and enter the world
    Play,
    /// Close the pause menu
    Resume,
    /// Tell the core to quit; the app exits once the world does
    Quit,
}

/// A keyboard- and pointer-selectable menu button
#[derive(Component, Debug, Clone, Copy)]
pub struct MenuButton {
    pub action: MenuAction,
    /// Position in the menu, used for keyboard navigation
    pub index: usize,
}

/// Label inside a [`MenuButton`], recolored on selection
#[derive(Component)]
pub struct MenuButtonLabel;
