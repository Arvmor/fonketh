use crate::movements::key_listener;
use crate::prelude::*;
use game_network::Network;

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
        // Initialize terminal
        info!("Initializing world");
        let mut terminal = ratatui::init();

        // Listen for key events
        tokio::spawn(async move {
            if let Err(e) = key_listener() {
                error!("Key listener error: {}", e);
            }
        });

        // Listen for motion events
        let connect = self.connect()?;

        // Restore terminal
        ratatui::restore();
        Ok(())
    }
}

impl<I, P> Network for World<I, P> {
    type Connection = ();

    fn connect(&self) -> Result<Self::Connection> {
        Ok(())
    }
}
