//! Keyboard routing.
//!
//! One system reads every key press and dispatches it based on the current
//! screen and whether the chat field has focus, so a key can never reach two
//! consumers (typing `w` in chat no longer walks, `Esc` never quits outright).

use crate::logic::keyboard_events;
use crate::menu::activate_menu_action;
use crate::prelude::*;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use game_primitives::events::GameEvent;

/// Routes keyboard input to the menu, the chat field or the character
#[allow(clippy::too_many_arguments)]
pub fn route_keyboard<F, Po>(
    mut keys: MessageReader<KeyboardInput>,
    screen: Res<State<Screen>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut chat: ResMut<ChatInputText>,
    mut selection: ResMut<MenuSelection>,
    buttons: Query<&MenuButton>,
    sender: Res<KeyEventSender<F, Po>>,
    mut exit: MessageWriter<AppExit>,
) where
    F: Send + Sync + 'static,
    Po: Position<Unit = i32> + Send + Sync + 'static,
{
    for key in keys.read().filter(|k| k.state.is_pressed()) {
        trace!("Keyboard event: {key:?}");

        let handled = match screen.get() {
            Screen::Playing if chat.is_active => type_in_chat(key, &mut chat, &sender),
            Screen::Playing => play(key, &mut chat, &mut next_screen, &sender),
            Screen::Menu | Screen::Paused => navigate_menu(
                key,
                screen.get(),
                &mut selection,
                &buttons,
                &sender,
                &mut next_screen,
                &mut exit,
            ),
        };

        // A screen change takes effect next frame; do not let the remaining
        // presses of this frame leak into the old screen's handlers.
        if handled == Handled::ScreenChanged {
            break;
        }
    }
}

/// Outcome of handling one key press
#[derive(Debug, PartialEq, Eq)]
enum Handled {
    Continue,
    ScreenChanged,
}

/// Chat field has focus: edit the buffer, submit or cancel
fn type_in_chat<F, Po>(
    key: &KeyboardInput,
    chat: &mut ChatInputText,
    sender: &KeyEventSender<F, Po>,
) -> Handled {
    match &key.logical_key {
        Key::Enter => {
            if let Some(message) = chat.submit() {
                debug!("Sending chat message: {message}");
                if let Err(e) = sender.0.send(GameEvent::ChatMessage(message)) {
                    error!("Error sending chat message: {e:?}");
                }
            }
        }
        Key::Escape => chat.cancel(),
        Key::Backspace => {
            chat.text.pop();
        }
        _ => {
            if let Some(text) = &key.text {
                chat.push_str(text);
            }
        }
    }

    Handled::Continue
}

/// In the world: move, open chat or pause
fn play<F, Po>(
    key: &KeyboardInput,
    chat: &mut ChatInputText,
    next_screen: &mut NextState<Screen>,
    sender: &KeyEventSender<F, Po>,
) -> Handled
where
    Po: Position<Unit = i32>,
{
    match &key.logical_key {
        Key::Enter => {
            chat.activate();
            Handled::Continue
        }
        Key::Escape => {
            next_screen.set(Screen::Paused);
            Handled::ScreenChanged
        }
        _ => {
            if let Some(event) = keyboard_events(key.key_code)
                && let Err(e) = sender.0.send(event)
            {
                error!("Error sending keyboard event: {e:?}");
            }
            Handled::Continue
        }
    }
}

/// In a menu: move the selection, activate it, or resume from pause
fn navigate_menu<F, Po>(
    key: &KeyboardInput,
    screen: &Screen,
    selection: &mut MenuSelection,
    buttons: &Query<&MenuButton>,
    sender: &KeyEventSender<F, Po>,
    next_screen: &mut NextState<Screen>,
    exit: &mut MessageWriter<AppExit>,
) -> Handled {
    match &key.logical_key {
        Key::ArrowUp => selection.previous(),
        Key::ArrowDown => selection.next(),
        Key::Character(c) if c.eq_ignore_ascii_case("w") => selection.previous(),
        Key::Character(c) if c.eq_ignore_ascii_case("s") => selection.next(),
        Key::Enter | Key::Space => {
            let action = buttons
                .iter()
                .find(|b| b.index == selection.index)
                .map(|b| b.action);
            if let Some(action) = action {
                activate_menu_action(action, sender, next_screen, exit);
                return Handled::ScreenChanged;
            }
        }
        Key::Escape if *screen == Screen::Paused => {
            next_screen.set(Screen::Playing);
            return Handled::ScreenChanged;
        }
        _ => {}
    }

    Handled::Continue
}
