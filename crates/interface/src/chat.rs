use crate::prelude::*;
use bevy::prelude::*;
use game_primitives::events::GameEvent;

/// System to handle chat input from keyboard
pub fn handle_chat_input<F, Po>(
    keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut chat_input: ResMut<ChatInputText>,
    sender: Res<KeyEventSender<F, Po>>,
) where
    F: Send + Sync + 'static,
    Po: Send + Sync + 'static,
{
    // Toggle chat input with Enter key
    if keyboard_input.just_pressed(KeyCode::Enter) {
        if !chat_input.is_active {
            // Activate chat input
            chat_input.is_active = true;
            chat_input.text.clear();
        } else if !chat_input.text.is_empty() {
            // Send the message when deactivating
            chat_input.is_active = false;
            let message = std::mem::take(&mut chat_input.text);

            // Send to network (this would need to be implemented with the network layer)
            // For now, we'll just add it to local messages
            debug!("Sending chat message: {message}");
            if let Err(e) = sender.0.send(GameEvent::ChatMessage(message)) {
                error!("Error sending chat message: {e:?}");
            }
        } else {
            // Deactivate without sending if empty
            chat_input.is_active = false;
        }
    }

    // Handle escape to cancel chat
    if keyboard_input.just_pressed(KeyCode::Escape) && chat_input.is_active {
        chat_input.is_active = false;
        chat_input.text.clear();
    }

    // Handle text input when chat is active
    if chat_input.is_active {
        // Handle backspace
        if keyboard_input.just_pressed(KeyCode::Backspace) {
            chat_input.text.pop();
        }

        // Handle character input (simplified - in a real implementation you'd need to handle text input events)
        // This is a basic implementation that would need to be enhanced with proper text input handling
        for key in keyboard_input.get_just_pressed() {
            if let Some(c) = key_to_char(key) {
                chat_input.text.push(c)
            }
        }
    }
}

/// Helper function to convert KeyCode to character
fn key_to_char(key: &KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Numpad0 => Some('0'),
        KeyCode::Numpad1 => Some('1'),
        KeyCode::Numpad2 => Some('2'),
        KeyCode::Numpad3 => Some('3'),
        KeyCode::Numpad4 => Some('4'),
        KeyCode::Numpad5 => Some('5'),
        KeyCode::Numpad6 => Some('6'),
        KeyCode::Numpad7 => Some('7'),
        KeyCode::Numpad8 => Some('8'),
        KeyCode::Numpad9 => Some('9'),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

/// System to display chat messages
#[allow(clippy::type_complexity)]
pub fn display_chat_messages<W>(
    world_state: Res<WorldStateResource<W>>,
    chat_input: Res<ChatInputText>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<ChatBox>>,
        Query<&mut Text, With<ChatInput>>,
    )>,
) where
    W: WorldState + Sync + Send + 'static,
{
    // Update chat box with recent messages
    if let Ok(mut chat_box_text) = text_queries.p0().single_mut() {
        let mut recent = Vec::with_capacity(6);

        if !chat_input.is_active {
            recent.push("> Press Enter to type".to_string());
        }

        // Show last 5 messages
        for m in world_state.0.get_chat_messages().into_iter().rev().take(5) {
            recent.push(m);
        }

        chat_box_text.0 = recent.join("\n");
    }

    // Update chat input field
    if let Ok(mut chat_input_text) = text_queries.p1().single_mut() {
        if chat_input.is_active {
            chat_input_text.0 = format!("> {}_", chat_input.text);
        } else {
            chat_input_text.0.clear();
        }
    }
}
