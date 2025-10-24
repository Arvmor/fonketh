use crate::prelude::*;
use bevy::input::keyboard::KeyCode;
use game_primitives::events::GameEvent;

pub fn keyboard_events<F, P>(key: KeyCode) -> Option<GameEvent<F, P>>
where
    P: Position<Unit = i32>,
{
    // Check for event variants
    let event = match key {
        // Quit keys
        KeyCode::Escape | KeyCode::KeyQ => GameEvent::Quit,
        // Movement keys
        KeyCode::ArrowRight => GameEvent::PlayerMovement(Position::new(1, 0)),
        KeyCode::ArrowLeft => GameEvent::PlayerMovement(Position::new(-1, 0)),
        KeyCode::ArrowUp => GameEvent::PlayerMovement(Position::new(0, 1)),
        KeyCode::ArrowDown => GameEvent::PlayerMovement(Position::new(0, -1)),
        // Confirm keys
        KeyCode::Enter => return None,
        _ => return None,
    };

    Some(event)
}
