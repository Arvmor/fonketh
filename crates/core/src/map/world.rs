use crate::movements::{Motion, read_key};
use crate::prelude::*;
use crate::utils::{ExitStatus, Identifier};
use crate::world::{Character, GameEvent};
use game_network::prelude::Keypair;
use game_network::prelude::gossipsub::IdentTopic;
use game_network::{Network, Peer2Peer};
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

impl<I, B> World<I, B>
where
    I: Debug + Eq + Hash + Clone + Send + Sync + 'static,
    B: Clone + Eq + Hash + Send + Sync + 'static,
{
    pub fn new(player: Character<I, B>) -> Self {
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

    pub fn initialize(self) -> Result<()> {
        // Initialize terminal
        info!("Initializing world");
        let mut terminal = ratatui::init();

        // Listen for motion events
        let network = Peer2Peer::build(Keypair::generate_ed25519())?;
        network.connect()?;

        // Listen for key events
        let world = self.clone();
        tokio::spawn(async move {
            while !world.exit_status.is_exit() {
                // Read key event
                let event = match read_key() {
                    Ok(Some(e)) => e,
                    Ok(None) => continue,
                    Err(e) => {
                        info!("Key listener error: {:?}", e);
                        world.exit_status.exit();
                        continue;
                    }
                };

                let topic = IdentTopic::new("game_events");
                let data = serde_json::to_vec(&event).unwrap();
                world.update(&world.identifier, event);
                if let Err(e) = network.send(topic, data) {
                    error!("Network error: {:?}", e);
                }
            }
        });

        // Main game loop - render once for now
        while !self.exit_status.is_exit() {
            terminal.draw(|frame| self.r#move(frame))?;
        }

        // Restore terminal
        ratatui::restore();
        Ok(())
    }

    pub fn update(&self, identifier: &I, event: GameEvent) {
        match event {
            GameEvent::PlayerMovement(p) => {
                self.players.update_player(identifier, |player| {
                    player.position += p;
                });
                debug!("Player {:?} moved by: {:?}", identifier, p);
            }
            GameEvent::Quit => {
                self.exit_status.exit();
                debug!("Player {:?} quit", identifier);
            }
        }
    }
}

impl<I, B> Motion for World<I, B>
where
    I: Eq + Hash + Clone,
    B: Clone,
{
    fn r#move(&self, frame: &mut Frame) {
        let p = self.players.get_player(&self.identifier).position;
        frame.render_widget(
            Canvas::default()
                .block(Block::bordered())
                .marker(Marker::Dot)
                .paint(|ctx| {
                    ctx.draw(&CIRCLE);
                })
                .x_bounds([10.0 - p.x as f64, 210.0 - p.x as f64])
                .y_bounds([10.0 + p.y as f64, 110.0 + p.y as f64]),
            frame.area(),
        );
    }
}
