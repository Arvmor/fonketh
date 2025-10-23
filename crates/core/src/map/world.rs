#[cfg(feature = "interface")]
use crate::movements::keyboard_events;
use crate::prelude::*;
use crate::world::{Character, GameEvent};
use game_contract::Rewarder;
use game_contract::prelude::{Address, U256};
#[cfg(feature = "interface")]
use game_interface::{Interface, KeyboardInput};
use game_network::Peer2Peer;
use game_network::prelude::gossipsub::{IdentTopic, Message};
use game_network::prelude::{Keypair, PeerId};
use game_primitives::{ExitStatus, Identifier, WorldState};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
#[cfg(feature = "interface")]
use std::sync::mpsc;
use std::sync::{Arc, RwLock};

/// Players pool
///
/// Used to store all players in the world
#[derive(Debug)]
pub struct PlayersPool<I, B, T = i32> {
    players: RwLock<HashMap<I, Character<I, B, T>>>,
}

impl<I: Eq + Hash, B, T> PlayersPool<I, B, T> {
    /// Creates a new players pool
    pub fn new() -> Self {
        let players = Default::default();

        Self { players }
    }

    /// Adds a player to the players pool
    pub fn add_player(&self, identifier: I, player: Character<I, B, T>) {
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
        F: FnOnce(&mut Character<I, B, T>) -> R,
    {
        let mut players = self.players.write().unwrap();
        players.get_mut(identifier).map(func)
    }
}

#[derive(Debug, Clone)]
pub struct World<I, B, T = i32> {
    pub exit_status: Arc<ExitStatus>,
    identifier: I,
    players: Arc<PlayersPool<I, B, T>>,
    mining_rewards: Arc<RwLock<u32>>,
    mined: Arc<RwLock<HashSet<(Address, U256)>>>,
}

impl<B> World<PeerId, B, i32>
where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    /// Creates a new world
    ///
    /// Initializes the world with the player
    pub fn new(player: Character<PeerId, B, i32>) -> Self {
        let exit_status = Arc::new(ExitStatus::default());
        let players = Arc::new(PlayersPool::new());
        let identifier = player.identifier();
        let mining_rewards = Arc::new(Default::default());
        let mined = Arc::new(Default::default());

        // Add player to players pool
        players.add_player(player.identifier(), player);

        Self {
            exit_status,
            identifier,
            players,
            mining_rewards,
            mined,
        }
    }

    /// Initializes the world
    ///
    /// Runs Network and Interface
    pub async fn initialize(self, private_key: Vec<u8>) -> Result<()> {
        info!("Initializing world");
        let rpc = "https://mainnet.base.org";
        let chain_id = 8453;
        let client = game_contract::RewarderClient::new(rpc, &private_key, chain_id).await?;

        // Run network loop
        let topic = IdentTopic::new("game_events");
        let keypair = Keypair::ed25519_from_bytes(private_key)?;
        let (tx, rx) = Peer2Peer::build(keypair)?.start(vec![topic.clone()]);

        // Run core loop
        #[cfg(feature = "interface")]
        let (txb, rxb) = mpsc::channel();
        let world = self.clone();
        tokio::spawn(async move {
            #[cfg(feature = "interface")]
            world.runner(topic, rxb, tx, rx, client).await.unwrap();
            #[cfg(not(feature = "interface"))]
            world.runner(topic, tx, rx, client).await.unwrap();
        });

        // Run interface loop
        #[cfg(feature = "interface")]
        Interface::run(txb, self);

        #[cfg(not(feature = "interface"))]
        tokio::signal::ctrl_c().await.unwrap();

        Ok(())
    }

    /// Handles the message passing from input and network
    async fn runner(
        self,
        topic: IdentTopic,
        #[cfg(feature = "interface")] rxb: mpsc::Receiver<KeyboardInput>,
        tx: tokio::sync::mpsc::Sender<(IdentTopic, Vec<u8>)>,
        mut rx: tokio::sync::mpsc::Receiver<Message>,
        mut client: game_contract::RewarderClient,
    ) -> anyhow::Result<()> {
        while !self.exit_status.is_exit() {
            // Listen for key events
            #[cfg(feature = "interface")]
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
                    self.update(&m.source.unwrap(), &event);
                }
            }

            // Mine a new address
            if let Some(mined) = client.miner.run() {
                info!("Mined address: {mined:?}");
                let event = GameEvent::PlayerFound(mined);
                self.update(&self.identifier, &event);

                // Send event to network
                let data = serde_json::to_vec(&event)?;
                if let Err(e) = tx.send((topic.clone(), data)).await {
                    error!("Network error: {:?}", e);
                };
            }

            // If mined enough
            // Spawn Claim Transaction
            if self.get_mined_count() >= 10
                && let Ok(batch) = self.drain_mined_batch().try_into()
            {
                let contract = client.contract.clone();
                tokio::spawn(async move {
                    // Register the transaction
                    let pending_tx = match contract.processMiningArray(batch).send().await {
                        Ok(pending) => pending.register().await,
                        Err(e) => return error!("Claim transaction error: {e}"),
                    };

                    // Wait for the transaction to be mined
                    let tx = match pending_tx {
                        Ok(tx) => tx.await,
                        Err(e) => return error!("Pending Transaction error: {e}"),
                    };

                    match tx {
                        Ok(tx) => info!("Claimed successfully {tx:?}"),
                        Err(e) => error!("Claimed failed error: {e}"),
                    };
                });
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
                    let mut new_player = Character::new(*identifier, Default::default(), (0, 0));
                    new_player.position = *p;
                    self.players.add_player(*identifier, new_player);
                }
            }
            GameEvent::PlayerFound(f) => {
                info!("Player {identifier:?} found: {f:?}");
                // Increment mining rewards counter
                self.mined.write().unwrap().insert(*f);
                *self.mining_rewards.write().unwrap() += 1;
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
    pub fn get_all_players(&self) -> HashMap<PeerId, Character<PeerId, B, i32>>
    where
        B: Clone,
    {
        self.players.players.read().unwrap().clone()
    }

    /// Gets the current mining rewards count
    pub fn get_mining_rewards_count(&self) -> u32 {
        *self.mining_rewards.read().unwrap()
    }

    /// Gets the current mined addresses
    pub fn drain_mined_batch(&self) -> Vec<Rewarder::MinerData> {
        self.mined
            .write()
            .unwrap()
            .drain()
            .map(|(a, n)| Rewarder::MinerData {
                minerAddress: a,
                nonce: n,
            })
            .collect()
    }

    /// Get the mined addresses count
    pub fn get_mined_count(&self) -> usize {
        self.mined.read().unwrap().len()
    }
}

impl<I: Clone, B, T> Identifier for World<I, B, T> {
    type Id = I;

    fn identifier(&self) -> Self::Id {
        self.identifier.clone()
    }
}

impl<I, B, T> WorldState for World<I, B, T>
where
    I: Clone,
    T: Copy + Into<f64>,
{
    type Player = Character<I, B, T>;

    fn exit_status(&self) -> Arc<ExitStatus> {
        self.exit_status.clone()
    }

    fn get_all_players(&self) -> HashMap<Self::Id, Self::Player> {
        todo!()
    }

    fn get_mining_rewards_count(&self) -> u32 {
        todo!()
    }
}
