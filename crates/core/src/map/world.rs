use crate::movements::key_listener;
use crate::prelude::*;

pub struct World<I, P> {
    pub identifier: I,
    pub players: P,
}

impl<I, P> World<I, P> {
    pub fn new(identifier: I, players: P) -> Self {
        Self {
            identifier,
            players,
        }
    }

    pub fn initialize(self) -> Result<()> {
        info!("Initializing world");

        if let Err(e) = key_listener() {
            error!("Key listener error: {}", e);
        }

        Ok(())
    }
}
