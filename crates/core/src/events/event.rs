use crate::movements::Position;
use crate::prelude::{Deserialize, Serialize};
use alloy_primitives::{Address, B256};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent<F = (Address, B256)> {
    Quit,
    PlayerMovement(Position),
    PlayerFound(F),
}
