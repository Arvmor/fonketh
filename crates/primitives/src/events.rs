use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent<F, P> {
    Quit,
    PlayerMovement(P),
    PlayerFound(F),
    ChatMessage(String),
}

#[derive(Debug, Serialize, Deserialize)]

pub struct Event<A, S, F, P> {
    pub event: GameEvent<F, P>,
    pub address: A,
    pub signature: S,
}

impl<A, S, F, P> Event<A, S, F, P> {
    /// Creates a new event
    pub fn new(event: GameEvent<F, P>, address: A, signature: S) -> Self {
        Self {
            event,
            address,
            signature,
        }
    }
}

impl<A, S, F, P> TryInto<Vec<u8>> for &Event<A, S, F, P>
where
    A: Serialize,
    F: Serialize,
    P: Serialize,
{
    type Error = serde_json::Error;
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let data = (&self.event, &self.address);
        serde_json::to_vec(&data)
    }
}

impl<'de, A, S, F, P> TryFrom<&'de [u8]> for Event<A, S, F, P>
where
    A: Deserialize<'de>,
    F: Deserialize<'de>,
    P: Deserialize<'de>,
    S: Deserialize<'de>,
{
    type Error = serde_json::Error;
    fn try_from(value: &'de [u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(value)
    }
}
