pub mod map;
pub mod movements;
pub mod player;

// Crate Internal API
pub mod world {
    pub use crate::map::World;
    pub use crate::player::Character;
}

// Crate Prelude
pub mod prelude {
    pub use anyhow::{Result, anyhow};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::json;
}
