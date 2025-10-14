use crate::movements::Position;
use crate::prelude::{Deserialize, Serialize};
use game_contract::prelude::{Address, B256};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent<F = (Address, B256)> {
    Quit,
    PlayerMovement(Position),
    PlayerFound(F),
}
