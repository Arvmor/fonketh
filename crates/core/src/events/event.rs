use crate::movements::Position;

#[derive(Debug, PartialEq, Eq)]
pub enum GameEvent {
    Quit,
    PlayerMovement(Position),
}
