pub mod events;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Identifier Trait
///
/// Used to identify a unique entity in the system
pub trait Identifier {
    type Id: Clone;

    fn identifier(&self) -> Self::Id;
}

/// Exit status
///
/// Used to signal that the world should exit
#[derive(Debug, Default)]
pub struct ExitStatus(AtomicBool);

impl ExitStatus {
    /// Set the exit status to true
    pub fn exit(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Check if the exit status is true
    pub fn is_exit(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// World state
///
/// Used to store the state of the world
pub trait WorldState: Identifier {
    type Player: Player;
    /// Gets the exit status of the world
    fn exit_status(&self) -> Arc<ExitStatus>;
    /// Gets all players from the world
    fn get_all_players(&self) -> HashMap<Self::Id, Self::Player>;
    /// Gets the current mining rewards count
    fn get_mining_rewards_count(&self) -> u32;
}

/// Player
///
/// Used to store the state of a player
pub trait Player: Identifier {
    type Position: Position + Clone;

    fn position(&self) -> Self::Position;
}

/// Position
///
/// Used to store the position of a player
pub trait Position {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
}
