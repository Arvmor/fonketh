//! Visual language shared by the menus and the HUD.
//!
//! Every color, type size and spacing value used by the UI lives here so the
//! screens stay consistent and a restyle is a one-file change.

use bevy::prelude::*;

/// Color palette
pub mod palette {
    use bevy::prelude::Color;

    /// Translucent panel background used by HUD widgets
    pub const PANEL: Color = Color::srgba(0.05, 0.06, 0.09, 0.86);
    /// Near-opaque card used by menus
    pub const CARD: Color = Color::srgba(0.09, 0.10, 0.14, 0.97);
    /// Full-screen dim drawn behind menus
    pub const SCRIM: Color = Color::srgba(0.02, 0.03, 0.05, 0.70);
    /// Hairline borders
    pub const BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);
    /// Inset track (progress bar background, input field)
    pub const TRACK: Color = Color::srgba(1.0, 1.0, 1.0, 0.07);
    /// Primary text
    pub const TEXT: Color = Color::srgb(0.94, 0.95, 0.97);
    /// Secondary text
    pub const MUTED: Color = Color::srgb(0.58, 0.62, 0.70);
    /// $FONK gold, used for emphasis and selection
    pub const ACCENT: Color = Color::srgb(1.0, 0.78, 0.25);
    /// Positive feedback (claims, joins)
    pub const SUCCESS: Color = Color::srgb(0.36, 0.88, 0.56);
    /// Neutral feedback (other players)
    pub const INFO: Color = Color::srgb(0.47, 0.72, 1.0);
    /// Destructive actions
    pub const DANGER: Color = Color::srgb(0.95, 0.40, 0.40);
    /// Resting button fill
    pub const BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.04);
    /// Selected / hovered button fill
    pub const BUTTON_SELECTED: Color = Color::srgba(1.0, 0.78, 0.25, 0.14);
    /// Pressed button fill
    pub const BUTTON_PRESSED: Color = Color::srgba(1.0, 0.78, 0.25, 0.28);
}

/// Type scale (logical pixels)
pub mod font {
    pub const TITLE: f32 = 54.0;
    pub const HEADING: f32 = 22.0;
    pub const STAT: f32 = 26.0;
    pub const BODY: f32 = 15.0;
    pub const CHAT: f32 = 14.0;
    pub const SMALL: f32 = 11.0;
}

/// Spacing scale (logical pixels)
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 28.0;
}

/// Corner radius for panels and cards
pub const RADIUS: Val = Val::Px(10.0);
/// Corner radius for inner controls (buttons, fields, bars)
pub const RADIUS_SM: Val = Val::Px(6.0);
/// Border width
pub const HAIRLINE: Val = Val::Px(1.0);

/// A bordered, shadowed container. The caller supplies layout and padding.
pub fn panel(node: Node, background: Color) -> impl Bundle {
    (
        Node {
            border: UiRect::all(HAIRLINE),
            ..node
        },
        BackgroundColor(background),
        BorderColor::all(palette::BORDER),
        BorderRadius::all(RADIUS),
        BoxShadow(vec![ShadowStyle {
            color: Color::srgba(0.0, 0.0, 0.0, 0.40),
            x_offset: Val::ZERO,
            y_offset: Val::Px(6.0),
            spread_radius: Val::ZERO,
            blur_radius: Val::Px(16.0),
        }]),
    )
}

/// A single-style text node
pub fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

/// A styled span, to be spawned as a child of a [`Text`] root
pub fn span(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        TextSpan::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

/// A small, upper-cased, muted caption used to label values
pub fn caption(value: &str) -> impl Bundle {
    text(value.to_uppercase(), font::SMALL, palette::MUTED)
}

/// Vertical flex container
pub fn column(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(gap),
        ..default()
    }
}

/// Horizontal flex container with vertically centered items
pub fn row(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(gap),
        ..default()
    }
}

/// Shortens a hex address (`0x1234...abcd`); other identifiers pass through
pub fn shorten(id: &str) -> String {
    if id.is_ascii() && id.starts_with("0x") && id.len() > 14 {
        format!("{}...{}", &id[..6], &id[id.len() - 4..])
    } else {
        id.to_string()
    }
}

/// Human readable age, e.g. `now`, `12s`, `3m`, `2h`
pub fn age_label(secs: u64) -> String {
    match secs {
        0..=2 => "now".to_string(),
        3..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs / 60),
        _ => format!("{}h", secs / 3600),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortens_addresses_only() {
        assert_eq!(
            shorten("0x063049aB85870f97E483567A06BeeA86227d88F9"),
            "0x0630...88F9"
        );
        assert_eq!(shorten("vitalik.eth"), "vitalik.eth");
        assert_eq!(shorten("0x1234"), "0x1234");
    }

    #[test]
    fn formats_age() {
        assert_eq!(age_label(1), "now");
        assert_eq!(age_label(45), "45s");
        assert_eq!(age_label(125), "2m");
        assert_eq!(age_label(7300), "2h");
    }
}
