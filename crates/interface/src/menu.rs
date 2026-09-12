//! Start and pause menus.
//!
//! Both menus are full-screen overlays built from the same card, with
//! keyboard (arrows / WASD + Enter) and pointer navigation.

use crate::prelude::*;
use crate::theme::{
    HAIRLINE, RADIUS, RADIUS_SM, caption, column, font, palette, panel, row, shorten, span, text,
};
use bevy::prelude::*;
use game_primitives::events::GameEvent;
use std::fmt::Display;

/// Width of the menu card
const CARD_WIDTH: f32 = 440.0;
/// Height of a menu button
const BUTTON_HEIGHT: f32 = 46.0;

/// Spawns the start menu
pub fn spawn_main_menu<W>(
    mut commands: Commands,
    world_state: Res<WorldStateResource<W>>,
    mut selection: ResMut<MenuSelection>,
) where
    W: WorldState + Sync + Send + 'static,
    W::Id: Display,
{
    let miner = world_state.0.identifier().to_string();
    selection.reset(2);

    commands
        .spawn((menu_root(), DespawnOnExit(Screen::Menu)))
        .with_children(|root| {
            root.spawn(card()).with_children(|card| {
                // Title block
                card.spawn(Node {
                    align_items: AlignItems::Center,
                    ..column(space::XS)
                })
                .with_children(|title| {
                    title.spawn((
                        text("FONKETH", font::TITLE, palette::ACCENT),
                        TextShadow {
                            offset: Vec2::new(0.0, 3.0),
                            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                        },
                    ));
                    title.spawn(text(
                        "Peer-to-peer mining pool on Base",
                        font::BODY,
                        palette::MUTED,
                    ));
                });

                spawn_identity(card, &miner);

                // Actions
                card.spawn(Node {
                    width: Val::Percent(100.0),
                    ..column(space::SM)
                })
                .with_children(|actions| {
                    spawn_button(actions, 0, MenuAction::Play, "Enter World");
                    spawn_button(actions, 1, MenuAction::Quit, "Quit");
                });

                spawn_controls(card);
                spawn_footer(card);
            });
        });
}

/// Spawns the pause menu on top of the HUD
pub fn spawn_pause_menu(mut commands: Commands, mut selection: ResMut<MenuSelection>) {
    selection.reset(2);

    commands
        .spawn((menu_root(), DespawnOnExit(Screen::Paused)))
        .with_children(|root| {
            root.spawn(card()).with_children(|card| {
                card.spawn(text("PAUSED", font::HEADING, palette::TEXT));

                // Live session stats
                card.spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..row(space::MD)
                })
                .with_children(|stats| {
                    spawn_stat(stats, "Mined", StatValue::SessionTotal, palette::ACCENT);
                    spawn_stat(stats, "Next claim", StatValue::PendingBatch, palette::TEXT);
                    spawn_stat(stats, "Claims", StatValue::Claims, palette::SUCCESS);
                    spawn_stat(stats, "Online", StatValue::PlayersOnline, palette::INFO);
                });

                card.spawn(Node {
                    width: Val::Percent(100.0),
                    ..column(space::SM)
                })
                .with_children(|actions| {
                    spawn_button(actions, 0, MenuAction::Resume, "Resume");
                    spawn_button(actions, 1, MenuAction::Quit, "Quit to Desktop");
                });

                spawn_controls(card);
                card.spawn(text("Esc  resumes", font::SMALL, palette::MUTED));
            });
        });
}

/// Performs a menu action
pub fn activate_menu_action<F, Po>(
    action: MenuAction,
    sender: &KeyEventSender<F, Po>,
    next_screen: &mut NextState<Screen>,
    exit: &mut MessageWriter<AppExit>,
) {
    match action {
        MenuAction::Play | MenuAction::Resume => next_screen.set(Screen::Playing),
        MenuAction::Quit => {
            info!("Quit requested from menu");
            // The core removes the player, tells the network and flips the
            // exit status, which shuts the interface down. If the core is
            // gone already, exit directly.
            if sender.0.send(GameEvent::Quit).is_err() {
                warn!("Core channel closed; exiting interface directly");
                exit.write(AppExit::Success);
            }
        }
    }
}

/// Pointer interaction: hovering selects, pressing activates
pub fn handle_menu_pointer<F, Po>(
    buttons: Query<(&MenuButton, &Interaction), Changed<Interaction>>,
    mut selection: ResMut<MenuSelection>,
    sender: Res<KeyEventSender<F, Po>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut exit: MessageWriter<AppExit>,
) where
    F: Send + Sync + 'static,
    Po: Send + Sync + 'static,
{
    for (button, interaction) in buttons.iter() {
        match interaction {
            Interaction::Hovered => selection.index = button.index,
            Interaction::Pressed => {
                selection.index = button.index;
                activate_menu_action(button.action, &sender, &mut next_screen, &mut exit);
            }
            Interaction::None => {}
        }
    }
}

/// Reflects selection and pointer state in the button visuals
pub fn style_menu_buttons(
    selection: Res<MenuSelection>,
    mut buttons: Query<(
        &MenuButton,
        &Interaction,
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
    )>,
    mut labels: Query<&mut TextColor, With<MenuButtonLabel>>,
) {
    for (button, interaction, mut background, mut border, children) in buttons.iter_mut() {
        let selected = button.index == selection.index;
        let pressed = matches!(interaction, Interaction::Pressed);

        let highlight = match button.action {
            MenuAction::Quit => palette::DANGER,
            _ => palette::ACCENT,
        };

        background.0 = match (pressed, selected) {
            (true, _) => palette::BUTTON_PRESSED,
            (false, true) => palette::BUTTON_SELECTED,
            (false, false) => palette::BUTTON,
        };
        *border = BorderColor::all(if selected { highlight } else { palette::BORDER });

        for child in children.iter() {
            if let Ok(mut color) = labels.get_mut(child) {
                color.0 = if selected { highlight } else { palette::TEXT };
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

/// Full-screen dimmed root that centers its content
fn menu_root() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(space::LG)),
            ..default()
        },
        BackgroundColor(palette::SCRIM),
        GlobalZIndex(10),
    )
}

/// The menu card
fn card() -> impl Bundle {
    panel(
        Node {
            width: Val::Px(CARD_WIDTH),
            max_width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(space::XL)),
            ..column(space::XL)
        },
        palette::CARD,
    )
}

/// Menu button with a label
fn spawn_button(parent: &mut ChildSpawnerCommands, index: usize, action: MenuAction, label: &str) {
    parent
        .spawn((
            Button,
            MenuButton { action, index },
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(BUTTON_HEIGHT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(HAIRLINE),
                ..default()
            },
            BackgroundColor(palette::BUTTON),
            BorderColor::all(palette::BORDER),
            BorderRadius::all(RADIUS_SM),
        ))
        .with_children(|button| {
            button.spawn((
                text(label, font::BODY + 1.0, palette::TEXT),
                MenuButtonLabel,
            ));
        });
}

/// "Miner" identity strip
fn spawn_identity(parent: &mut ChildSpawnerCommands, miner: &str) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(Val::Px(space::MD), Val::Px(space::SM)),
                ..row(space::MD)
            },
            BackgroundColor(palette::TRACK),
            BorderRadius::all(RADIUS_SM),
        ))
        .with_children(|strip| {
            strip.spawn(caption("Miner"));
            strip.spawn(text(shorten(miner), font::BODY, palette::TEXT));
        });
}

/// Compact stat tile used in the pause menu
fn spawn_stat(parent: &mut ChildSpawnerCommands, label: &str, value: StatValue, color: Color) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            ..column(space::XS)
        })
        .with_children(|tile| {
            tile.spawn((text("-", font::STAT, color), value));
            tile.spawn(caption(label));
        });
}

/// Controls reference
fn spawn_controls(parent: &mut ChildSpawnerCommands) {
    const BINDINGS: [(&str, &str); 3] = [
        ("Move", "Arrows / WASD"),
        ("Chat", "Enter"),
        ("Menu", "Esc"),
    ];

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(space::MD)),
                border: UiRect::all(HAIRLINE),
                ..column(space::SM)
            },
            BorderColor::all(palette::BORDER),
            BorderRadius::all(RADIUS),
        ))
        .with_children(|list| {
            list.spawn(caption("Controls"));
            for (action, keys) in BINDINGS {
                list.spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..row(space::MD)
                })
                .with_children(|line| {
                    line.spawn(text(action, font::CHAT, palette::MUTED));
                    line.spawn(text(keys, font::CHAT, palette::TEXT));
                });
            }
        });
}

/// Version and network footer
fn spawn_footer(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Text::default(),
            TextFont {
                font_size: font::SMALL,
                ..default()
            },
            TextColor(palette::MUTED),
        ))
        .with_children(|footer| {
            footer.spawn(span(
                concat!("v", env!("CARGO_PKG_VERSION")),
                font::SMALL,
                palette::MUTED,
            ));
            footer.spawn(span("   Base  /  chain 8453", font::SMALL, palette::MUTED));
        });
}
