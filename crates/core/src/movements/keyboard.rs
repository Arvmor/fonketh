use crate::prelude::*;
use ratatui::crossterm::{
    self,
    event::{Event, KeyCode},
};

fn keyboard_events(event: Event) -> Result<()> {
    // Short Circuit none-key events
    let Event::Key(event) = event else {
        return Ok(());
    };

    // Check for event variants
    match event.code {
        // Quit keys
        KeyCode::Esc | KeyCode::Char('q') => {
            return Ok(());
        }
        // Movement keys
        KeyCode::Right => {
            return Ok(());
        }
        KeyCode::Left => {
            return Ok(());
        }
        KeyCode::Up => {
            return Ok(());
        }
        KeyCode::Down => {
            return Ok(());
        }
        // Confirm keys
        KeyCode::Enter => {
            return Ok(());
        }
        _ => {}
    }

    Ok(())
}

pub fn keyboard_listener() -> Result<()> {
    loop {
        let event = crossterm::event::read()?;
        keyboard_events(event)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_event() {
        let event = Event::Key(KeyCode::Esc.into());
        let result = keyboard_events(event);
        assert!(result.is_ok());
    }
}
