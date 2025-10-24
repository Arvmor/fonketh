use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent<F, P> {
    Quit,
    PlayerMovement(P),
    PlayerFound(F),
    ChatMessage(String),
}
