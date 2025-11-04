mod chat;
mod coins;
mod components;
mod hud;
mod logic;
mod minings;
mod movements;
mod resources;

/// Prelude for common components and resources
mod prelude {
    /// Magic speed number
    pub const MAGIC_SPEED: f32 = 24.0;
    /// Delay before going idle
    pub const IDLE_DURATION: std::time::Duration = std::time::Duration::from_millis(200);
    /// FPS
    pub const FPS: u8 = 14;
    /// Camera boundary - horizontal distance from center before camera starts following
    pub const CAMERA_BOUNDARY_X: f32 = 150.0;
    /// Camera boundary - vertical distance from center before camera starts following
    pub const CAMERA_BOUNDARY_Y: f32 = 150.0;

    pub use crate::chat::*;
    pub use crate::coins::*;
    pub use crate::components::*;
    pub use crate::hud::*;
    pub use crate::resources::*;
    pub use game_primitives::*;
}

pub use logic::{Interface, keyboard_events};
