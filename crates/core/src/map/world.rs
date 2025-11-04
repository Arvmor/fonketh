use crate::channels::{SignedMessage, SignedReceiver, SignedSender};
use crate::prelude::*;
use crate::world::Character;
use game_contract::RewarderClient;
use game_contract::miner::Rewarder;
use game_network::Peer2Peer;
use game_network::prelude::Keypair;
use game_network::prelude::gossipsub::Message;
use game_primitives::message::ChatMessage;
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
    messages: Arc<RwLock<Vec<ChatMessage>>>,
}

impl<B> World<Address, B, i32>
where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    /// Creates a new world
    ///
    /// Initializes the world with the player
    pub fn new(player: Character<Address, B, i32>) -> Self {
        let exit_status = Arc::new(ExitStatus::default());
        let players = Arc::new(PlayersPool::new());
        let identifier = player.identifier();
        let mining_rewards = Arc::new(Default::default());
        let mined = Arc::new(Default::default());
        let messages = Arc::new(Default::default());

        // Add player to players pool
        players.add_player(player.identifier(), player);

        Self {
            exit_status,
            identifier,
            players,
            mining_rewards,
            mined,
            messages,
        }
    }

    /// Initializes the world
    ///
    /// Runs Network and Interface
    pub async fn initialize(self, private_key: Vec<u8>) -> Result<()> {
        info!("Initializing world");
        let rpc = "https://mainnet.base.org";
        let chain_id = 8453;
        let client = RewarderClient::new(rpc, &private_key, chain_id).await?;

        // Run network loop
        let keypair = Keypair::ed25519_from_bytes(private_key)?;
        let (tx, rx) = Peer2Peer::build(keypair)?.start();

        // Run core loop
        #[cfg(feature = "interface")]
        let (txb, rxb) = mpsc::channel();
        tokio::spawn(self.clone().runner(
            #[cfg(feature = "interface")]
            rxb,
            tx,
            rx,
            client,
        ));

        // Run interface loop
        #[cfg(feature = "interface")]
        game_interface::Interface::run(txb, self);

        #[cfg(not(feature = "interface"))]
        tokio::signal::ctrl_c().await?;

        Ok(())
    }

    /// Handles the message passing from input and network
    async fn runner(
        self,
        #[cfg(feature = "interface")] rxb: mpsc::Receiver<GameEventMessage>,
        tx: tokio::sync::mpsc::Sender<SignedMessage<GameEventMessage>>,
        mut rx: tokio::sync::mpsc::Receiver<Message>,
        mut client: RewarderClient,
    ) -> anyhow::Result<()> {
        while !self.exit_status.is_exit() {
            // Listen for key events
            #[cfg(feature = "interface")]
            if let Ok(e) = rxb.try_recv() {
                info!("Received Keyboard event: {e:?}");
                self.update(&self.identifier, &e, &client).await;

                // Send event to network
                let message = SignedMessage::new(e, client.wallet.address());
                if let Err(e) = tx.send_signed(message, &client.wallet).await {
                    error!("Network error: {:?}", e);
                };
            }

            // Listen for network events
            if let Ok(Some(m)) = rx.receive_signed()
                && let Ok(signed) = serde_json::from_slice::<SignedMessage<_>>(&m.data)
            {
                info!("Received Network message: {m:?} => {signed:?}");
                self.update(&signed.address, &signed.data, &client).await;
            }

            // Mine a new address
            if let Some(mined) = client.miner.run() {
                info!("Mined address: {mined:?}");
                let event = GameEvent::PlayerFound(mined);
                self.update(&self.identifier, &event, &client).await;

                // Send event to network
                let message = SignedMessage::new(event, client.wallet.address());
                if let Err(e) = tx.send_signed(message, &client.wallet).await {
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
    pub async fn update(
        &self,
        identifier: &Address,
        event: &GameEventMessage,
        client: &RewarderClient,
    ) {
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
            GameEvent::ChatMessage(message) => {
                // Get ENS name
                let identifier = match client.ens.nameForAddr(*identifier).call().await {
                    Ok(n) if n.is_empty() => identifier.to_string(),
                    Ok(n) => n,
                    Err(e) => {
                        error!("Failed to get ENS name for address {identifier:?}: {e}");
                        identifier.to_string()
                    }
                };

                info!("Player {identifier:?} sent chat message: {message}");
                self.add_chat_message(identifier, message.clone());
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

    /// Adds a chat message to the messages pool
    pub fn add_chat_message(&self, identifier: String, message: String) {
        let mut messages = self.messages.write().unwrap();
        messages.push(ChatMessage::new(identifier, message));
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
    B: Clone,
    T: Copy + Clone + Into<f64>,
{
    type Player = Character<I, B, T>;
    type Message = ChatMessage;

    fn exit_status(&self) -> Arc<ExitStatus> {
        self.exit_status.clone()
    }

    fn get_all_players(&self) -> HashMap<Self::Id, Self::Player> {
        self.players.players.read().unwrap().clone()
    }

    fn get_mining_rewards_count(&self) -> u32 {
        *self.mining_rewards.read().unwrap()
    }

    fn get_chat_messages(&self) -> Vec<Self::Message> {
        self.messages.read().unwrap().clone()
    }
}
