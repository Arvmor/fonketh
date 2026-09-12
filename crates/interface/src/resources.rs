use crate::movements::PlayerStateInfo;
use bevy::prelude::*;
use game_primitives::events::GameEvent;
use game_primitives::{Identifier, WorldState};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;

/// Resource that holds the keyboard event sender
#[derive(Resource)]
pub struct KeyEventSender<F, Po>(pub Sender<GameEvent<F, Po>>);

/// Resource that holds the world state
#[derive(Resource)]
pub struct WorldStateResource<W: WorldState>(pub W);

/// Resource that holds the players that have been spawned in the UI
#[derive(Resource)]
pub struct SpawnedPlayers<I: Identifier> {
    pub spawned: HashSet<I::Id>,
}

impl<I: Identifier> Default for SpawnedPlayers<I> {
    fn default() -> Self {
        let spawned = Default::default();
        Self { spawned }
    }
}

/// Resource that holds the player states and movement times within the interface
#[derive(Resource)]
pub struct PlayerStates<I: Identifier> {
    pub players: HashMap<I::Id, PlayerStateInfo>,
}

impl<I: Identifier> Default for PlayerStates<I> {
    fn default() -> Self {
        let players = Default::default();
        Self { players }
    }
}

/// Number of mined treasures the core batches into one on-chain claim
pub const CLAIM_BATCH_SIZE: u32 = 10;

/// Mining statistics derived from the world's pending-batch counter
///
/// The world only exposes the size of the batch waiting to be claimed, which
/// drops back to zero after every claim. This resource turns those changes
/// into a running session total and a claim count.
#[derive(Resource, Default, Debug)]
pub struct MiningStats {
    /// Treasures currently waiting in the claim batch
    pub pending: u32,
    /// Treasures mined since the app started
    pub session_total: u32,
    /// Claim batches submitted since the app started
    pub claims: u32,
}

/// Player population as last observed by the interface
#[derive(Resource, Default, Debug)]
pub struct Population {
    /// `None` until the first observation, so startup does not toast
    pub online: Option<usize>,
}

/// Maximum characters accepted in a single chat message
pub const CHAT_MAX_LEN: usize = 240;

/// Resource that holds the current chat input text
#[derive(Resource)]
pub struct ChatInputText {
    pub text: String,
    pub is_active: bool,
    /// Drives the caret blink
    pub caret_timer: Timer,
    pub caret_visible: bool,
}

impl Default for ChatInputText {
    fn default() -> Self {
        Self {
            text: String::new(),
            is_active: false,
            caret_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            caret_visible: true,
        }
    }
}

impl ChatInputText {
    /// Opens the input field
    pub fn activate(&mut self) {
        self.is_active = true;
        self.caret_visible = true;
        self.caret_timer.reset();
        self.text.clear();
    }

    /// Closes the input field and returns the message, if any
    pub fn submit(&mut self) -> Option<String> {
        self.is_active = false;
        let message = std::mem::take(&mut self.text);
        let trimmed = message.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    /// Closes the input field, discarding the text
    pub fn cancel(&mut self) {
        self.is_active = false;
        self.text.clear();
    }

    /// Appends printable characters up to the length limit
    pub fn push_str(&mut self, input: &str) {
        for c in input.chars().filter(|c| !c.is_control()) {
            if self.text.chars().count() >= CHAT_MAX_LEN {
                break;
            }
            self.text.push(c);
        }
    }
}

/// Bookkeeping for the rendered chat log
#[derive(Resource)]
pub struct ChatLogState {
    /// Number of messages currently rendered
    pub rendered: usize,
    /// Periodic refresh so relative timestamps stay current
    pub refresh: Timer,
}

impl Default for ChatLogState {
    fn default() -> Self {
        Self {
            rendered: 0,
            refresh: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

/// Keyboard selection within the active menu
#[derive(Resource, Default, Debug)]
pub struct MenuSelection {
    pub index: usize,
    pub count: usize,
}

impl MenuSelection {
    /// Resets the selection for a menu with `count` buttons
    pub fn reset(&mut self, count: usize) {
        self.index = 0;
        self.count = count;
    }

    /// Moves the selection down, wrapping around
    pub fn next(&mut self) {
        if self.count > 0 {
            self.index = (self.index + 1) % self.count;
        }
    }

    /// Moves the selection up, wrapping around
    pub fn previous(&mut self) {
        if self.count > 0 {
            self.index = (self.index + self.count - 1) % self.count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_input_trims_and_limits() {
        let mut input = ChatInputText::default();
        input.activate();
        input.push_str("  hello\nworld ");
        assert_eq!(input.text, "  helloworld ");
        assert_eq!(input.submit(), Some("helloworld".to_string()));
        assert!(!input.is_active);

        input.activate();
        input.push_str(&"a".repeat(CHAT_MAX_LEN + 50));
        assert_eq!(input.text.chars().count(), CHAT_MAX_LEN);

        input.activate();
        input.push_str("   ");
        assert_eq!(input.submit(), None);
    }

    #[test]
    fn menu_selection_wraps() {
        let mut sel = MenuSelection::default();
        sel.reset(3);
        sel.previous();
        assert_eq!(sel.index, 2);
        sel.next();
        assert_eq!(sel.index, 0);
    }
}
