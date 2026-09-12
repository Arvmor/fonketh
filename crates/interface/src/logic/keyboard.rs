use bevy::input::keyboard::KeyCode;
use game_primitives::Position;
use game_primitives::events::GameEvent;

/// Maps a movement key to the game event it produces
///
/// Quitting and chat are handled by the menus and the input router, so only
/// movement keys map to events here. Arrow keys and WASD are equivalent.
pub fn keyboard_events<F, P>(key: KeyCode) -> Option<GameEvent<F, P>>
where
    P: Position<Unit = i32>,
{
    let event = match key {
        KeyCode::ArrowRight | KeyCode::KeyD => GameEvent::PlayerMovement(Position::new(1, 0)),
        KeyCode::ArrowLeft | KeyCode::KeyA => GameEvent::PlayerMovement(Position::new(-1, 0)),
        KeyCode::ArrowUp | KeyCode::KeyW => GameEvent::PlayerMovement(Position::new(0, 1)),
        KeyCode::ArrowDown | KeyCode::KeyS => GameEvent::PlayerMovement(Position::new(0, -1)),
        _ => return None,
    };

    Some(event)
}
