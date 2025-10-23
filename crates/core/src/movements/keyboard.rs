use crate::movements::Position;
use crate::world::GameEvent;
use game_interface::KeyCode;

pub fn keyboard_events(key: KeyCode) -> Option<GameEvent> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_event() {
        let event = KeyCode::Escape;
        let result = keyboard_events(event).unwrap();
        assert_eq!(result, GameEvent::Quit);

        let event = KeyCode::KeyQ;
        let result = keyboard_events(event).unwrap();
        assert_eq!(result, GameEvent::Quit);
    }
}
