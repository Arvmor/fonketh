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
    pub use tracing::{debug, error, info, trace, warn};
}

/// Bincode Helper
///
/// Used to encode and decode values using bincode
pub struct BincodeHelper;

impl BincodeHelper {
    /// Encode a value using bincode
    pub fn encode<T: serde::Serialize>(value: &T) -> anyhow::Result<Vec<u8>> {
        match bincode::serde::encode_to_vec(value, bincode::config::standard()) {
            Ok(value) => Ok(value),
            Err(e) => {
                tracing::error!("Bincode Encode Failed: {e:?}");
                Err(anyhow::anyhow!("Bincode Encode Failed: {e:?}"))
            }
        }
    }

    /// Decode a value using bincode
    pub fn decode<T: for<'de> serde::Deserialize<'de>>(value: &[u8]) -> anyhow::Result<T> {
        match bincode::serde::decode_from_slice(value, bincode::config::standard()) {
            Ok((result, _)) => Ok(result),
            Err(e) => {
                tracing::error!("Bincode Decode Failed: {e:?}");
                Err(anyhow::anyhow!("Bincode Decode Failed: {e:?}"))
            }
        }
    }
}
