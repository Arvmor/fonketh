//! Chat rendering: message log rows and the input field.
//!
//! Keyboard handling lives in `input.rs`; this module only reflects state.

use crate::prelude::*;
use crate::theme::{age_label, font, palette, shorten, span, text};
use bevy::prelude::*;
use std::fmt::Display;

/// Number of messages kept in the visible log
const CHAT_VISIBLE_MESSAGES: usize = 8;

/// Rebuilds the chat rows when messages arrive and once a second for ages
pub fn render_chat_log<W>(
    time: Res<Time>,
    mut commands: Commands,
    world_state: Res<WorldStateResource<W>>,
    mut state: ResMut<ChatLogState>,
    log: Query<Entity, With<ChatLog>>,
) where
    W: WorldState + Sync + Send + 'static,
    W::Id: Display,
{
    let Ok(log) = log.single() else {
        return;
    };

    let messages = world_state.0.get_chat_messages();
    let refresh_due = state.refresh.tick(time.delta()).just_finished();
    if messages.len() == state.rendered && !refresh_due {
        return;
    }
    state.rendered = messages.len();

    let local = world_state.0.identifier().to_string();
    let start = messages.len().saturating_sub(CHAT_VISIBLE_MESSAGES);

    commands
        .entity(log)
        .despawn_related::<Children>()
        .with_children(|log| {
            for message in &messages[start..] {
                // Compare the stable id: the display name may be an ENS name
                let is_local = message.sender_id() == local;
                let sender_color = if is_local {
                    palette::ACCENT
                } else {
                    palette::INFO
                };
                let sender = if is_local {
                    "you".to_string()
                } else {
                    shorten(message.sender())
                };

                log.spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    column_gap: Val::Px(space::SM),
                    ..default()
                })
                .with_children(|line| {
                    line.spawn((
                        Text::default(),
                        TextFont {
                            font_size: font::CHAT,
                            ..default()
                        },
                        TextColor(palette::TEXT),
                        Node {
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            min_width: Val::ZERO,
                            ..default()
                        },
                    ))
                    .with_children(|body| {
                        body.spawn(span(sender, font::CHAT, sender_color));
                        body.spawn(span("  ", font::CHAT, palette::TEXT));
                        body.spawn(span(message.content(), font::CHAT, palette::TEXT));
                    });
                    line.spawn((
                        text(
                            age_label(message.age().as_secs()),
                            font::SMALL,
                            palette::MUTED,
                        ),
                        Node {
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                });
            }
        });
}

/// Reflects the input buffer, placeholder, caret and focus highlight
#[allow(clippy::type_complexity)]
pub fn render_chat_input(
    time: Res<Time>,
    mut chat: ResMut<ChatInputText>,
    mut field: Query<(&mut Text, &mut TextColor), With<ChatInputField>>,
    mut caret: Query<&mut Visibility, With<ChatCaret>>,
    mut input_row: Query<&mut BorderColor, With<ChatInputRow>>,
) {
    if chat.is_active && chat.caret_timer.tick(time.delta()).just_finished() {
        chat.caret_visible = !chat.caret_visible;
    }

    if let Ok((mut text, mut color)) = field.single_mut() {
        if chat.is_active {
            text.0.clone_from(&chat.text);
            color.0 = palette::TEXT;
        } else {
            text.0 = "Press Enter to chat".to_string();
            color.0 = palette::MUTED;
        }
    }

    if let Ok(mut visibility) = caret.single_mut() {
        *visibility = if chat.is_active && chat.caret_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut border) = input_row.single_mut() {
        *border = BorderColor::all(if chat.is_active {
            palette::ACCENT
        } else {
            palette::BORDER
        });
    }
}
