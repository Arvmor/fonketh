use bevy::prelude::*;
use game_primitives::Identifier;
use std::time::Duration;

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

/// Component to identify the HUD root container
#[derive(Component)]
pub struct HudRoot;

/// Component to identify the top HUD bar container
#[derive(Component)]
pub struct TopHudBar;

/// Component to identify the bottom HUD bar container
#[derive(Component)]
pub struct BottomHudBar;

/// Component to identify the status bar entity
#[derive(Component)]
pub struct StatusBar;

/// Component to identify the chat box entity
#[derive(Component)]
pub struct ChatBox;

/// Component to identify the chat input field entity
#[derive(Component)]
pub struct ChatInput;

/// Component to identify player count display
#[derive(Component)]
pub struct PlayerCount;

/// Component to identify instructions display
#[derive(Component)]
pub struct InstructionsText;

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
