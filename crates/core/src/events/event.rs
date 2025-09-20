use crate::movements::Position;
use crate::prelude::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    Quit,
    PlayerMovement(Position),
}
