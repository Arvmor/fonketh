use crate::movements::{Motion, read_key};
use crate::prelude::*;
use crate::utils::{ExitStatus, Identifier};
use crate::world::{Character, GameEvent};
use game_network::Peer2Peer;
use game_network::prelude::gossipsub::{IdentTopic, Message};
use game_network::prelude::{Keypair, PeerId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
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

const CIRCLE: Circle = Circle {
    x: 20.0,
    y: 40.0,
    radius: 10.0,
    color: Color::Yellow,
};

impl<B> World<PeerId, B>
where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
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

    pub async fn initialize(self, keypair: Keypair) -> Result<()> {
        // Listen for motion events
        info!("Initializing world");
        let topic = IdentTopic::new("game_events");
        let (tx, mut rx) = Peer2Peer::build(keypair)?.start(vec![topic.clone()]);

        // Main game loop - render once for now
        while !self.exit_status.is_exit() {
            // Listen for key events
            match read_key() {
                Ok(None) => {}
                Ok(Some(e)) => {
                    self.update(&self.identifier, &e);

                    // Send event to network
                    let data = serde_json::to_vec(&e)?;
                    if let Err(e) = tx.send((topic.clone(), data)).await {
                        error!("Network error: {:?}", e);
                    };
                }
                Err(e) => {
                    info!("Key listener error: {:?}", e);
                    self.exit_status.exit();
                }
            };

            // Listen for network events
            if let Ok(m) = rx.try_recv() {
                let event = serde_json::from_slice(&m.data).unwrap();
                info!("Received Network event: {:#?}", event);

                self.update(&m.identifier(), &event);
            }
        }

        Ok(())
    }

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

impl<I, B> Motion for World<I, B>
where
    I: Eq + Hash + Clone,
    B: Clone,
{
    fn r#move(&self, frame: &mut Frame) {
        let circles = self
            .players
            .players
            .read()
            .unwrap()
            .values()
            .map(|player| {
                let mut c = CIRCLE;
                c.x += player.position.x as f64;
                c.y -= player.position.y as f64;
                c
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Canvas::default()
                .block(Block::bordered())
                .marker(Marker::Dot)
                .paint(|ctx| {
                    for c in &circles {
                        ctx.draw(c);
                    }
                })
                .x_bounds([10.0, 210.0])
                .y_bounds([10.0, 110.0]),
            frame.area(),
        );
    }
}
