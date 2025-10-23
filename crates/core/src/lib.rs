pub mod events;
pub mod map;
pub mod movements;
pub mod player;

// Crate Internal API
pub mod world {
    pub use crate::events::GameEvent;
    pub use crate::map::World;
    pub use crate::player::Character;
    pub use game_contract::prelude::B256;
    pub use game_network::prelude::Keypair;
}

// Crate Prelude
pub mod prelude {
    pub use anyhow::{Result, anyhow};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::json;
    pub use tracing::{debug, error, info, trace, warn};
}
