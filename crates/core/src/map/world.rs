use crate::movements::read_key;
use crate::prelude::*;
use crate::utils::{ExitStatus, Identifier};
use crate::world::GameEvent;
use game_network::Network;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Circle};
use ratatui::widgets::{Block, Widget};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct PlayersPool<I, P> {
    players: RwLock<HashMap<I, RwLock<P>>>,
}

impl<I: Eq + Hash, P> PlayersPool<I, P> {
    pub fn new() -> Self {
        let players = Default::default();

        Self { players }
    }

    pub fn add_player(&self, identifier: I, player: P) {
        let mut players = self.players.write().unwrap();
        players.insert(identifier, RwLock::new(player));
    }

    pub fn remove_player(&self, identifier: I) {
        let mut players = self.players.write().unwrap();
        players.remove(&identifier);
    }
}

pub struct World<I, P> {
    exit_status: Arc<ExitStatus>,
    identifier: I,
    players: Arc<PlayersPool<I, P>>,
}

const CIRCLE: Circle = Circle {
    x: 20.0,
    y: 40.0,
    radius: 10.0,
    color: Color::Yellow,
};

impl<I: Eq + Hash + Clone, P: Identifier<Id = I>> World<I, P> {
    pub fn new(player: P) -> Self {
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
        let status = self.exit_status.clone();
        let players = self.players.clone();
        tokio::spawn(async move {
            loop {
                // Read key event
                let event = match read_key() {
                    Ok(Some(event)) => event,
                    Ok(None) => continue,
                    Err(e) => {
                        info!("Key listener error: {:?}", e);
                        break status.exit();
                    }
                };

                // Handle event
                match event {
                    GameEvent::PlayerMovement(p) => {
                        info!("Player moved to: {:?}", p);
                    }
                    GameEvent::Quit => break status.exit(),
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
}

impl<I, P> Network for World<I, P> {
    type Connection = ();

    fn connect(&self) -> Result<Self::Connection> {
        Ok(())
    }
}
