use crate::movements::read_key;
use crate::prelude::*;
use crate::utils::{ExitStatus, Identifier};
use crate::world::{Character, GameEvent};
use game_network::Network;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Circle};
use ratatui::widgets::{Block, Widget};
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

        // Listen for key events
        let world = self.clone();
        tokio::spawn(async move {
            while !world.exit_status.is_exit() {
                // Read key event
                match read_key() {
                    Ok(Some(e)) => world.update(&world.identifier, e),
                    Ok(None) => continue,
                    Err(e) => {
                        info!("Key listener error: {:?}", e);
                        world.exit_status.exit();
                    }
                }
            }
        });

        // Listen for motion events
        self.connect()?;

        // Main game loop - render once for now
        while !self.exit_status.is_exit() {
            terminal.draw(|frame| self.draw(frame))?;
        }

        // Restore terminal
        ratatui::restore();
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let surface = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let horizontal = Layout::horizontal(surface);
        let vertical = Layout::vertical(surface);
        let [_, right] = horizontal.areas(frame.area());
        let [pong, _] = vertical.areas(right);

        frame.render_widget(self.pong_canvas(), pong);
    }

    fn pong_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("Pong"))
            .marker(Marker::Dot)
            .paint(|ctx| {
                ctx.draw(&CIRCLE);
            })
            .x_bounds([10.0, 210.0])
            .y_bounds([10.0, 110.0])
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

impl<I, B> Network for World<I, B> {
    type Connection = ();

    fn connect(&self) -> Result<Self::Connection> {
        Ok(())
    }
}
