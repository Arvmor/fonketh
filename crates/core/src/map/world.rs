use crate::movements::key_listener;
use crate::prelude::*;
use game_network::Network;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default)]
pub struct ExitStatus(pub(crate) RwLock<bool>);

impl ExitStatus {
    pub fn exit(&self) {
        *self.0.write().unwrap() = true;
    }

    pub fn is_exit(&self) -> bool {
        *self.0.read().unwrap()
    }
}

pub struct World<I, P> {
    exit_status: Arc<ExitStatus>,
    identifier: I,
    players: P,
}

impl<I, P> World<I, P> {
    pub fn new(identifier: I, players: P) -> Self {
        let exit_status = Arc::new(ExitStatus::default());

        Self {
            exit_status,
            identifier,
            players,
        }
    }

    pub fn initialize(self) -> Result<()> {
        // Initialize terminal
        info!("Initializing world");
        let mut terminal = ratatui::init();

        // Listen for key events
        let status = self.exit_status.clone();
        tokio::spawn(async move {
            if let Err(e) = key_listener() {
                error!("Key listener error: {}", e);
                status.exit();
            }
        });

        // Listen for motion events
        self.connect()?;

        // Main game loop - render once for now
        while !self.exit_status.is_exit() {
            // terminal.draw(|frame| self.draw(frame))?;
        }

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
