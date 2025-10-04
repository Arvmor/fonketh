use crate::interface::{Interface, KeyboardInput};
use crate::prelude::*;
use crate::utils::{ExitStatus, Identifier};
use crate::world::{Character, GameEvent, keyboard_events};
use game_network::Peer2Peer;
use game_network::prelude::gossipsub::{IdentTopic, Message};
use game_network::prelude::{Keypair, PeerId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct PlayersPool<I, B> {
    players: RwLock<HashMap<I, Character<I, B>>>,
}

impl<I: Eq + Hash, B> PlayersPool<I, B> {
    pub fn new() -> Self {
        let players = Default::default();

        Self { players }
    }

    pub fn add_player(&self, identifier: I, player: Character<I, B>) {
        let mut players = self.players.write().unwrap();
        players.insert(identifier, player);
    }

    pub fn remove_player(&self, identifier: &I) {
        let mut players = self.players.write().unwrap();
        players.remove(identifier);
    }

    pub fn get_player(&self, identifier: &I) -> Character<I, B>
    where
        I: Clone,
        B: Clone,
    {
        let players = self.players.read().unwrap();
        players.get(identifier).unwrap().clone()
    }

    pub fn update_player<F, R>(&self, identifier: &I, func: F) -> Option<R>
    where
        F: FnOnce(&mut Character<I, B>) -> R,
    {
        let mut players = self.players.write().unwrap();
        players.get_mut(identifier).map(func)
    }
}

#[derive(Debug, Clone)]
pub struct World<I, B> {
    exit_status: Arc<ExitStatus>,
    identifier: I,
    players: Arc<PlayersPool<I, B>>,
}

impl<B> World<PeerId, B>
where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    /// Creates a new world
    ///
    /// Initializes the world with the player
    pub fn new(player: Character<PeerId, B>) -> Self {
        let exit_status = Arc::new(ExitStatus::default());
        let players = Arc::new(PlayersPool::new());
        let identifier = player.identifier();

        // Add player to players pool
        players.add_player(player.identifier(), player);

        Self {
            exit_status,
            identifier,
            players,
        }
    }

    /// Initializes the world
    ///
    /// Runs Network and Interface
    pub async fn initialize(self, keypair: Keypair) -> Result<()> {
        info!("Initializing world");

        // Main game loop - render once for now
        let (txb, rxb) = mpsc::channel();
        tokio::spawn(async move {
            Interface::run(txb);
        });

        self.runner(keypair, rxb).await?;
        Ok(())
    }

    /// Handles the message passing from input and network
    async fn runner(
        self,
        keypair: Keypair,
        rxb: mpsc::Receiver<KeyboardInput>,
    ) -> anyhow::Result<()> {
        let topic = IdentTopic::new("game_events");
        let (tx, mut rx) = Peer2Peer::build(keypair)?.start(vec![topic.clone()]);

        while !self.exit_status.is_exit() {
            // Listen for key events
            if let Ok(Some(e)) = rxb.try_recv().map(|e| keyboard_events(e.key_code)) {
                info!("Received Keyboard event: {e:?}");
                self.update(&self.identifier, &e);

                // Send event to network
                let data = serde_json::to_vec(&e)?;
                if let Err(e) = tx.send((topic.clone(), data)).await {
                    error!("Network error: {:?}", e);
                };
            }

            // Listen for network events
            if let Ok(m) = rx.try_recv() {
                let event = serde_json::from_slice(&m.data);
                info!("Received Network message: {m:?} => {event:?}");

                if let Ok(event) = event {
                    self.update(&m.identifier(), &event);
                }
            }
        }

        Ok(())
    }

    /// Updates the world
    ///
    /// Based on the Events received
    pub fn update(&self, identifier: &PeerId, event: &GameEvent) {
        match event {
            GameEvent::PlayerMovement(p) => {
                let res = self.players.update_player(identifier, |player| {
                    player.position += *p;
                });
                if res.is_none() {
                    self.players
                        .add_player(*identifier, Character::new(*identifier, Default::default()));
                }
                debug!("Player {:?} moved by: {:?}", identifier, p);
            }
            GameEvent::Quit => {
                self.exit_status.exit();
                debug!("Player {:?} quit", identifier);
            }
        }
    }
}

impl Identifier for Message {
    type Id = game_network::prelude::PeerId;

    fn identifier(&self) -> Self::Id {
        self.source.unwrap()
    }
}
