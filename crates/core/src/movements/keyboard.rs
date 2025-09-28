use crate::movements::Position;
use crate::prelude::*;
use crate::world::GameEvent;
use ratatui::crossterm::{
    self,
    event::{Event, KeyCode, KeyEventKind},
};
use std::time::Duration;

fn keyboard_events(event: Event) -> Option<GameEvent> {
    // Short Circuit none-key events
    let Event::Key(event) = event else {
        return None;
    };

    // Use only release events to avoid double events
    debug!("Keyboard event: {:?}", event);
    if event.kind == KeyEventKind::Repeat {
        return None;
    }

    // Check for event variants
    let event = match event.code {
        // Quit keys
        KeyCode::Esc | KeyCode::Char('q') => GameEvent::Quit,
        // Movement keys
        KeyCode::Right => GameEvent::PlayerMovement(Position::new(1, 0)),
        KeyCode::Left => GameEvent::PlayerMovement(Position::new(-1, 0)),
        KeyCode::Up => GameEvent::PlayerMovement(Position::new(0, -1)),
        KeyCode::Down => GameEvent::PlayerMovement(Position::new(0, 1)),
        // Confirm keys
        KeyCode::Enter => return None,
        _ => return None,
    };

    Some(event)
}

pub fn read_key() -> Result<Option<GameEvent>> {
    let is_available = crossterm::event::poll(Duration::from_millis(10))?;

    if is_available {
        let key = crossterm::event::read()?;
        let event = keyboard_events(key);
        return Ok(event);
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_event() {
        let event = Event::Key(KeyCode::Esc.into());
        let result = keyboard_events(event).unwrap();
        assert_eq!(result, GameEvent::Quit);

        let event = Event::Key(KeyCode::Char('q').into());
        let result = keyboard_events(event).unwrap();
        assert_eq!(result, GameEvent::Quit);
    }
}
