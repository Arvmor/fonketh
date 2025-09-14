pub mod player;

// Crate Public API
pub use player::Character;

// Crate Prelude
pub mod prelude {
    pub use anyhow::{Result, anyhow};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::json;
}
