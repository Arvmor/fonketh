mod components;
mod logic;
mod minings;
mod movements;
mod resources;

/// Prelude for common components and resources
mod prelude {
    /// Magic speed number
    pub const MAGIC_SPEED: f32 = 24.0;
    /// Magic ground speed number
    pub const MAGIC_GROUND_SPEED: f32 = -14.0;
    /// Delay before going idle
    pub const IDLE_DURATION: std::time::Duration = std::time::Duration::from_millis(200);
    /// FPS
    pub const FPS: u8 = 14;

    pub use crate::components::*;
    pub use crate::resources::*;
    pub use game_primitives::*;
}

pub use logic::{Interface, KeyCode, KeyboardInput};
