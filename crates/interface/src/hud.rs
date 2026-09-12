//! In-game HUD: mining stats, identity, chat and a controls hint.
//!
//! Layout (full-screen, non-interactive):
//!
//! ```text
//! [ MINED  12 | NEXT CLAIM ████░░ 4/10 | CLAIMS 1 ]           [ MINER 0x1234...abcd | 3 ONLINE ]
//!
//!
//! [ CHAT                              ]
//! [  0x0630...88F9  gm      12s       ]
//! [  you            hello   now       ]                       [ Arrows / WASD  Move ]
//! [ > type a message_                 ]                       [ Enter  Chat  Esc  Menu ]
//! ```

use crate::prelude::*;
use crate::theme::{
    HAIRLINE, RADIUS_SM, caption, column, font, palette, panel, row, shorten, text,
};
use crate::toast::spawn_toast_layer;
use bevy::prelude::*;
use std::fmt::Display;

/// Width of the chat panel
const CHAT_WIDTH: f32 = 400.0;
/// Height of the scrolling message log
const CHAT_LOG_HEIGHT: f32 = 160.0;
/// Width of the claim progress bar
const PROGRESS_WIDTH: f32 = 120.0;
/// Height of the claim progress bar
const PROGRESS_HEIGHT: f32 = 6.0;

/// Sets up the HUD. It starts hidden and is shown when entering the world.
pub fn setup_hud<W>(mut commands: Commands, world_state: Res<WorldStateResource<W>>)
where
    W: WorldState + Sync + Send + 'static,
    W::Id: Display,
{
    let miner = shorten(&world_state.0.identifier().to_string());

    spawn_toast_layer(&mut commands);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(space::LG)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(1),
            Visibility::Hidden,
            HudRoot,
        ))
        .with_children(|hud| {
            // ---- Top row -------------------------------------------------
            hud.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..row(space::MD)
            })
            .with_children(|top| {
                spawn_mining_panel(top);
                spawn_identity_panel(top, &miner);
            });

            // ---- Bottom row ----------------------------------------------
            hud.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                ..row(space::MD)
            })
            .with_children(|bottom| {
                spawn_chat_panel(bottom);
                spawn_controls_hint(bottom);
            });
        });
}

/// Filter matching the HUD root and the toast layer
type HudLayers = Or<(With<HudRoot>, With<ToastLayer>)>;

/// Shows the HUD and toasts
pub fn show_hud(mut hud: Query<&mut Visibility, HudLayers>) {
    for mut visibility in hud.iter_mut() {
        *visibility = Visibility::Inherited;
    }
}

/// Hides the HUD and toasts (menus keep the HUD visible but hide toasts)
pub fn hide_hud(mut hud: Query<&mut Visibility, HudLayers>) {
    for mut visibility in hud.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

/// Hides toasts while a menu is open so they do not overlap the card
pub fn hide_toasts(mut layer: Query<&mut Visibility, With<ToastLayer>>) {
    for mut visibility in layer.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

// ---------------------------------------------------------------------------
// Panels
// ---------------------------------------------------------------------------

/// Session total, claim progress and claim count
fn spawn_mining_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(panel(
            Node {
                padding: UiRect::axes(Val::Px(space::LG), Val::Px(space::MD)),
                align_items: AlignItems::FlexStart,
                ..row(space::XL)
            },
            palette::PANEL,
        ))
        .with_children(|p| {
            spawn_stat(p, "Mined", StatValue::SessionTotal, palette::ACCENT);
            spawn_divider(p);

            // Claim progress: caption aligned with the neighbours, bar at the baseline
            p.spawn(Node {
                align_self: AlignSelf::Stretch,
                justify_content: JustifyContent::SpaceBetween,
                ..column(space::XS)
            })
            .with_children(|tile| {
                tile.spawn(caption("Next claim"));
                tile.spawn(row(space::SM)).with_children(|line| {
                    line.spawn((
                        Node {
                            width: Val::Px(PROGRESS_WIDTH),
                            height: Val::Px(PROGRESS_HEIGHT),
                            ..default()
                        },
                        BackgroundColor(palette::TRACK),
                        BorderRadius::all(Val::Px(PROGRESS_HEIGHT / 2.0)),
                    ))
                    .with_children(|track| {
                        track.spawn((
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(palette::ACCENT),
                            BorderRadius::all(Val::Px(PROGRESS_HEIGHT / 2.0)),
                            BatchProgressFill,
                        ));
                    });
                    line.spawn((
                        text("-", font::CHAT, palette::TEXT),
                        StatValue::PendingBatch,
                    ));
                });
            });

            spawn_divider(p);
            spawn_stat(p, "Claims", StatValue::Claims, palette::SUCCESS);
        });
}

/// Miner address and online count
fn spawn_identity_panel(parent: &mut ChildSpawnerCommands, miner: &str) {
    parent
        .spawn(panel(
            Node {
                padding: UiRect::axes(Val::Px(space::LG), Val::Px(space::MD)),
                ..row(space::XL)
            },
            palette::PANEL,
        ))
        .with_children(|p| {
            p.spawn(Node {
                align_items: AlignItems::FlexEnd,
                ..column(space::XS)
            })
            .with_children(|tile| {
                tile.spawn(caption("Miner"));
                tile.spawn(text(miner, font::BODY, palette::TEXT));
            });
            spawn_divider(p);
            p.spawn(Node {
                align_items: AlignItems::FlexEnd,
                ..column(space::XS)
            })
            .with_children(|tile| {
                tile.spawn(caption("Online"));
                tile.spawn((
                    text("-", font::BODY, palette::INFO),
                    StatValue::PlayersOnline,
                ));
            });
        });
}

/// Message log and input field
fn spawn_chat_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(panel(
            Node {
                width: Val::Px(CHAT_WIDTH),
                max_width: Val::Percent(60.0),
                padding: UiRect::all(Val::Px(space::MD)),
                ..column(space::SM)
            },
            palette::PANEL,
        ))
        .with_children(|p| {
            p.spawn(caption("Chat"));

            // Log: newest at the bottom, older rows clipped at the top
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(CHAT_LOG_HEIGHT),
                    justify_content: JustifyContent::FlexEnd,
                    overflow: Overflow::clip_y(),
                    ..column(space::XS)
                },
                ChatLog,
            ));

            // Input row
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(space::SM), Val::Px(space::XS + 2.0)),
                    border: UiRect::all(HAIRLINE),
                    ..row(space::XS)
                },
                BackgroundColor(palette::TRACK),
                BorderColor::all(palette::BORDER),
                BorderRadius::all(RADIUS_SM),
                ChatInputRow,
            ))
            .with_children(|input| {
                input.spawn(text(">", font::CHAT, palette::MUTED));
                input.spawn((
                    text("Press Enter to chat", font::CHAT, palette::MUTED),
                    Node {
                        flex_grow: 1.0,
                        flex_shrink: 1.0,
                        min_width: Val::ZERO,
                        ..default()
                    },
                    ChatInputField,
                ));
                input.spawn((
                    text("|", font::CHAT, palette::ACCENT),
                    Visibility::Hidden,
                    ChatCaret,
                ));
            });
        });
}

/// Key bindings reminder
fn spawn_controls_hint(parent: &mut ChildSpawnerCommands) {
    const BINDINGS: [(&str, &str); 3] = [
        ("Arrows / WASD", "Move"),
        ("Enter", "Chat"),
        ("Esc", "Menu"),
    ];

    parent
        .spawn(panel(
            Node {
                padding: UiRect::axes(Val::Px(space::LG), Val::Px(space::MD)),
                align_items: AlignItems::FlexEnd,
                ..column(space::XS)
            },
            palette::PANEL,
        ))
        .with_children(|p| {
            for (keys, action) in BINDINGS {
                p.spawn(row(space::SM)).with_children(|line| {
                    line.spawn(text(keys, font::SMALL, palette::TEXT));
                    line.spawn(text(action, font::SMALL, palette::MUTED));
                });
            }
        });
}

// ---------------------------------------------------------------------------
// Pieces
// ---------------------------------------------------------------------------

/// Caption over a large live value
fn spawn_stat(parent: &mut ChildSpawnerCommands, label: &str, value: StatValue, color: Color) {
    parent.spawn(column(space::XS)).with_children(|tile| {
        tile.spawn(caption(label));
        tile.spawn((text("-", font::STAT, color), value));
    });
}

/// Thin vertical separator
fn spawn_divider(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: HAIRLINE,
            height: Val::Px(font::STAT + space::MD),
            ..default()
        },
        BackgroundColor(palette::BORDER),
    ));
}
