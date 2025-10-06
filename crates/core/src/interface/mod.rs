mod app;

pub use app::{Interface, KeyCode, KeyboardInput};

/// Magic speed number
const MAGIC_SPEED: f32 = 24.0;

/// Magic ground speed number
const MAGIC_GROUND_SPEED: f32 = -14.0;

/// Delay before going idle
const IDLE_DURATION: std::time::Duration = std::time::Duration::from_millis(200);

/// FPS
const FPS: u8 = 14;
