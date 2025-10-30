use crate::prelude::*;
use bevy::prelude::*;

/// HUD text styling constants
const HUD_TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const HUD_BG_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
const CHAT_TEXT_COLOR: Color = Color::srgb(0.3, 1.0, 0.3);
const HUD_PADDING_BOTTOM: Val = Val::Px(5.0);
const HUD_PADDING_ALL: Val = Val::Px(10.0);
const HUD_FONT_SIZE: f32 = 18.0;
const CHAT_FONT_SIZE: f32 = 16.0;

/// Sets up the HUD layout with proper structure and positioning
pub fn setup_hud(mut commands: Commands) {
    // Root HUD container - covers the entire screen
    let root = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    };

    // Top HUD bar - for mining rewards and player count
    let top_bar = Node {
        height: Val::Auto,
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(HUD_PADDING_ALL),
        ..root.clone()
    };

    // Mining rewards display (left side)
    let mine_bar = (
        Text::default(),
        TextFont {
            font_size: HUD_FONT_SIZE,
            ..default()
        },
        TextColor(HUD_TEXT_COLOR),
        StatusBar,
    );

    // Player count display (right side)
    let player_bar = (
        Text::default(),
        TextFont {
            font_size: HUD_FONT_SIZE,
            ..default()
        },
        TextColor(HUD_TEXT_COLOR),
        PlayerCount,
    );

    // Bottom HUD bar - for chat and instructions
    let bottom_bar = Node {
        flex_direction: FlexDirection::Column,
        ..top_bar.clone()
    };

    // Chat input field
    let chat_input = (
        Text::default(),
        TextFont {
            font_size: CHAT_FONT_SIZE,
            ..default()
        },
        TextColor(CHAT_TEXT_COLOR),
        Node {
            margin: UiRect::bottom(HUD_PADDING_BOTTOM),
            ..default()
        },
        ChatInput,
    );

    // Chat box - recent messages
    let chat_box = (
        Text::new("> Press Enter to type"),
        TextFont {
            font_size: CHAT_FONT_SIZE,
            ..default()
        },
        TextColor(HUD_TEXT_COLOR),
        Node {
            margin: UiRect::bottom(HUD_PADDING_BOTTOM),
            ..default()
        },
        ChatBox,
    );

    // Instructions text
    let help_text = (
        Text::new("Arrow Keys: Move | Enter: Chat | Esc: Quit"),
        TextFont {
            font_size: CHAT_FONT_SIZE,
            ..default()
        },
        TextColor(HUD_TEXT_COLOR.with_alpha(0.7)),
        InstructionsText,
    );

    commands
        .spawn((root, HudRoot))
        // Inner HUD structure
        .with_children(|parent| {
            // TOP BAR
            parent
                .spawn((top_bar, BackgroundColor(HUD_BG_COLOR), TopHudBar))
                .with_children(|top_bar| {
                    top_bar.spawn(mine_bar);
                    top_bar.spawn(player_bar);
                });

            // BOTTOM BAR
            parent
                .spawn((bottom_bar, BackgroundColor(HUD_BG_COLOR), BottomHudBar))
                .with_children(|bottom_bar| {
                    bottom_bar.spawn(chat_input);
                    bottom_bar.spawn(chat_box);
                    bottom_bar.spawn(help_text);
                });
        });
}

/// System to update player count display
pub fn update_player_count<W>(
    world_state: Res<WorldStateResource<W>>,
    mut text_query: Query<&mut Text, With<PlayerCount>>,
) where
    W: WorldState + Sync + Send + 'static,
{
    let player_count = world_state.0.get_all_players().len();
    for mut text in text_query.iter_mut() {
        text.0 = format!("Online Players #{player_count}");
    }
}
