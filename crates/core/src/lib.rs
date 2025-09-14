pub mod events;
pub mod map;
pub mod movements;
pub mod player;
pub mod utils;

// Crate Internal API
pub mod world {
    pub use crate::events::GameEvent;
    pub use crate::map::World;
    pub use crate::player::Character;
}

// Crate Prelude
pub mod prelude {
    pub use anyhow::{Result, anyhow};
    pub use ratatui::Frame;
    pub use ratatui::layout::{Constraint, Layout};
    pub use ratatui::style::Color;
    pub use ratatui::symbols::Marker;
    pub use ratatui::widgets::canvas::{Canvas, Circle};
    pub use ratatui::widgets::{Block, Widget};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::json;
    pub use tracing::{debug, error, info, trace, warn};
}
