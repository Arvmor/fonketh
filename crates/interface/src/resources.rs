use crate::logic::KeyboardInput;
use crate::movements::PlayerStateInfo;
use bevy::prelude::*;
use game_primitives::{Identifier, WorldState};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;

/// Resource that holds the keyboard event sender
#[derive(Resource)]
pub struct KeyEventSender(pub Sender<KeyboardInput>);

/// Resource that holds the world state
#[derive(Resource)]
pub struct WorldStateResource<W: WorldState>(pub W);

/// Resource that holds the players that have been spawned in the UI
#[derive(Resource, Default)]
pub struct SpawnedPlayers<I: Identifier> {
    pub spawned: HashSet<I::Id>,
}

/// Resource that holds the player states and movement times within the interface
#[derive(Resource, Default)]
pub struct PlayerStates<I: Identifier> {
    pub players: HashMap<I::Id, PlayerStateInfo>,
}

/// Resource that holds the mining rewards counter
#[derive(Resource, Default)]
pub struct MiningRewards {
    pub count: u32,
}
