pub mod channels;
pub mod map;
pub mod movements;
pub mod player;

// Crate Internal API
pub mod world {
    pub use crate::map::World;
    pub use crate::movements::Position;
    pub use crate::player::Character;
    pub use game_contract::prelude::B256;
    pub use game_contract::prelude::LocalSigner as Keypair;
    pub use game_primitives::events::GameEvent;
}

// Crate Prelude
pub mod prelude {
    pub type GameEventMessage = GameEvent<(Address, U256), Position>;
    pub use crate::world::Position;
    pub use anyhow::{Result, anyhow};
    pub use game_contract::prelude::{Address, U256};
    pub use game_primitives::events::GameEvent;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::json;
    pub use tracing::{debug, error, info, trace, warn};
}
