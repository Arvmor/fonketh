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

/// Players pool
///
/// Used to store all players in the world
#[derive(Debug)]
pub struct PlayersPool<I, B> {
    players: RwLock<HashMap<I, Character<I, B>>>,
}

impl<I: Eq + Hash, B> PlayersPool<I, B> {
    /// Creates a new players pool
    pub fn new() -> Self {
        let players = Default::default();

        Self { players }
    }

    /// Adds a player to the players pool
    pub fn add_player(&self, identifier: I, player: Character<I, B>) {
        let mut players = self.players.write().unwrap();
        players.insert(identifier, player);
    }

    /// Removes a player from the players pool
    pub fn remove_player(&self, identifier: &I) {
        let mut players = self.players.write().unwrap();
        players.remove(identifier);
    }

    /// Updates a player in the players pool
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
    pub exit_status: Arc<ExitStatus>,
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

        // Run network loop
        let topic = IdentTopic::new("game_events");
        let (tx, rx) = Peer2Peer::build(keypair)?.start(vec![topic.clone()]);

        // Run core loop
        let (txb, rxb) = mpsc::channel();
        let world = self.clone();
        tokio::spawn(async move {
            world.runner(topic, rxb, tx, rx).await.unwrap();
        });

        // Run interface loop
        Interface::run(txb, self);
        Ok(())
    }

    /// Handles the message passing from input and network
    async fn runner(
        self,
        topic: IdentTopic,
        rxb: mpsc::Receiver<KeyboardInput>,
        tx: tokio::sync::mpsc::Sender<(IdentTopic, Vec<u8>)>,
        mut rx: tokio::sync::mpsc::Receiver<Message>,
    ) -> anyhow::Result<()> {
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
                // Update player position
                info!("Player {identifier:?} moved by: {p:?}");
                let res = self.players.update_player(identifier, |player| {
                    player.position += *p;
                });

                // If new player, add to players pool
                if res.is_none() {
                    let mut new_player = Character::new(*identifier, Default::default());
                    new_player.position = *p;
                    self.players.add_player(*identifier, new_player);
                }
            }
            GameEvent::PlayerFound(f) => {
                info!("Player {identifier:?} found: {f:?}");
            }
            GameEvent::Quit => {
                info!("Player {identifier:?} quit");
                self.players.remove_player(identifier);

                // Quit if the local player quit
                if identifier == &self.identifier {
                    self.exit_status.exit();
                }
            }
        }
    }

    /// Gets all players from the world
    pub fn get_all_players(&self) -> HashMap<PeerId, Character<PeerId, B>>
    where
        B: Clone,
    {
        self.players.players.read().unwrap().clone()
    }
}

impl<B> Identifier for World<PeerId, B> {
    type Id = game_network::prelude::PeerId;

    fn identifier(&self) -> Self::Id {
        self.identifier
    }
}

impl Identifier for Message {
    type Id = game_network::prelude::PeerId;

    fn identifier(&self) -> Self::Id {
        self.source.unwrap()
    }
}
