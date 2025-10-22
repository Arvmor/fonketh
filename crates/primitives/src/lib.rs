use std::{
    collections::HashMap,
    hash::Hash,
    sync::atomic::{AtomicBool, Ordering},
};

/// Identifier Trait
///
/// Used to identify a unique entity in the system
pub trait Identifier: Send + Sync + 'static {
    type Id: Clone + Eq + Hash + Send + Sync + 'static;

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
pub trait WorldState: Identifier + Send + Sync + 'static {
    type Player: Player;

    fn exit_status(&self) -> ExitStatus;

    fn get_all_players(&self) -> HashMap<Self::Id, Self::Player>;

    fn get_mining_rewards_count(&self) -> u32;
}

/// Player
///
/// Used to store the state of a player
pub trait Player: Identifier + Position {
    type Position: Position;

    fn position(&self) -> Self::Position;
}

/// Position
///
/// Used to store the position of a player
pub trait Position {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}
