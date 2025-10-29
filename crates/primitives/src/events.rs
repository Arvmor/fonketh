use serde::{Deserialize, Serialize};

/// Game event variant
///
/// Movement, found, chat message, quit, etc.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent<F, P> {
    Quit,
    PlayerMovement(P),
    PlayerFound(F),
    ChatMessage(String),
}

/// Signed Game Event
#[derive(Debug, Serialize, Deserialize)]

pub struct SignedEvent<A, S, F, P> {
    pub event: GameEvent<F, P>,
    pub address: A,
    pub signature: S,
}

impl<A, S, F, P> SignedEvent<A, S, F, P> {
    /// Creates a new event
    pub fn new(event: GameEvent<F, P>, address: A, signature: S) -> Self {
        Self {
            event,
            address,
            signature,
        }
    }
}
