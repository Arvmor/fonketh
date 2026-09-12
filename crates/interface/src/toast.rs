//! Transient notifications stacked at the top of the screen.

use crate::prelude::*;
use crate::theme::{HAIRLINE, RADIUS_SM, font, palette, row, space};
use bevy::prelude::*;

/// How long a toast stays fully visible before fading
const TOAST_LIFETIME_SECS: f32 = 3.0;
/// Fade-out duration at the end of the lifetime
const TOAST_FADE_SECS: f32 = 0.6;
/// Maximum toasts on screen; the oldest is dropped beyond this
const TOAST_MAX_VISIBLE: usize = 4;

/// Request to show a toast
#[derive(Message, Debug, Clone)]
pub struct ToastRequest {
    pub title: String,
    pub detail: Option<String>,
    pub accent: Color,
}

impl ToastRequest {
    pub fn new(title: impl Into<String>, accent: Color) -> Self {
        Self {
            title: title.into(),
            detail: None,
            accent,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Spawns the (initially empty) container toasts are stacked into
pub fn spawn_toast_layer(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(space::XL * 3.0),
            left: Val::ZERO,
            right: Val::ZERO,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(space::SM),
            ..default()
        },
        // Do not intercept pointer events meant for the world or menus
        Pickable::IGNORE,
        GlobalZIndex(5),
        // Shown together with the HUD once the player enters the world
        Visibility::Hidden,
        ToastLayer,
    ));
}

/// Materializes toast requests into UI nodes
pub fn show_toasts(
    mut commands: Commands,
    mut requests: MessageReader<ToastRequest>,
    layer: Query<(Entity, Option<&Children>), With<ToastLayer>>,
    toasts: Query<Entity, With<Toast>>,
) {
    let Ok((layer, children)) = layer.single() else {
        return;
    };

    for request in requests.read() {
        // Drop the oldest toast if the stack is full
        let visible = children.map_or(0, |c| c.iter().filter(|c| toasts.contains(*c)).count());
        if visible >= TOAST_MAX_VISIBLE
            && let Some(oldest) = children.and_then(|c| c.iter().find(|c| toasts.contains(*c)))
        {
            commands.entity(oldest).despawn();
        }

        let base_background = palette::CARD;
        let base_border = request.accent.with_alpha(0.55);

        commands.entity(layer).with_children(|layer| {
            layer
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(space::LG), Val::Px(space::SM)),
                        border: UiRect::all(HAIRLINE),
                        ..row(space::MD)
                    },
                    BackgroundColor(base_background),
                    BorderColor::all(base_border),
                    BorderRadius::all(RADIUS_SM),
                    Toast {
                        timer: Timer::from_seconds(TOAST_LIFETIME_SECS, TimerMode::Once),
                        base_background,
                        base_border,
                    },
                ))
                .with_children(|toast| {
                    // Accent bar
                    toast.spawn((
                        Node {
                            width: Val::Px(3.0),
                            height: Val::Px(font::BODY + 2.0),
                            ..default()
                        },
                        BackgroundColor(request.accent),
                        BorderRadius::all(Val::Px(2.0)),
                    ));
                    toast.spawn(crate::theme::text(
                        &request.title,
                        font::BODY,
                        palette::TEXT,
                    ));
                    if let Some(detail) = &request.detail {
                        toast.spawn(crate::theme::text(detail, font::SMALL, palette::MUTED));
                    }
                });
        });
    }
}

/// Ages toasts, fades them out and despawns them when done
pub fn tick_toasts(
    time: Res<Time>,
    mut commands: Commands,
    mut toasts: Query<(
        Entity,
        &mut Toast,
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
    )>,
    mut colors: Query<(Option<&mut TextColor>, Option<&mut BackgroundColor>), Without<Toast>>,
) {
    for (entity, mut toast, mut background, mut border, children) in toasts.iter_mut() {
        toast.timer.tick(time.delta());

        if toast.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Opacity multiplier: 1.0 until the fade window, then linear to 0.0
        let remaining = toast.timer.remaining_secs();
        let alpha = (remaining / TOAST_FADE_SECS).clamp(0.0, 1.0);
        if alpha >= 1.0 {
            continue;
        }

        background.0 = toast
            .base_background
            .with_alpha(toast.base_background.alpha() * alpha);
        *border = BorderColor::all(
            toast
                .base_border
                .with_alpha(toast.base_border.alpha() * alpha),
        );

        for child in children.iter() {
            if let Ok((text_color, child_background)) = colors.get_mut(child) {
                if let Some(mut text_color) = text_color {
                    let base = text_color.0;
                    text_color.0 = base.with_alpha(alpha);
                }
                if let Some(mut child_background) = child_background {
                    let base = child_background.0;
                    child_background.0 = base.with_alpha(alpha);
                }
            }
        }
    }
}
