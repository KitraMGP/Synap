//! Theme system for the desktop application.
//!
//! Provides Light and Dark themes with consistent color palettes.

use iced::Color;

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// Toggle to the other theme
    pub fn toggle(&mut self) {
        *self = match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    }
}

/// Theme colors for the application
#[derive(Debug, Clone)]
pub struct ThemeColors {
    /// Background color
    pub background: Color,
    /// Surface color (cards, panels)
    pub surface: Color,
    /// Primary accent color
    pub primary: Color,
    /// Secondary accent color
    pub secondary: Color,
    /// Text color
    pub text: Color,
    /// Text secondary color
    pub text_secondary: Color,
    /// Border color
    pub border: Color,
    /// Edge/connection line color
    pub edge: Color,
    /// Node background
    pub node_bg: Color,
    /// Node border
    pub node_border: Color,
    /// Node focused border
    pub node_focused: Color,
    /// Error color
    pub error: Color,
    /// Success color
    pub success: Color,
}

impl ThemeColors {
    /// Get colors for light theme
    pub fn light() -> Self {
        Self {
            background: Color::from_rgb(0.95, 0.95, 0.97),
            surface: Color::from_rgb(1.0, 1.0, 1.0),
            primary: Color::from_rgb(0.2, 0.5, 0.9),
            secondary: Color::from_rgb(0.5, 0.7, 0.5),
            text: Color::from_rgb(0.1, 0.1, 0.15),
            text_secondary: Color::from_rgb(0.5, 0.5, 0.55),
            border: Color::from_rgb(0.8, 0.8, 0.85),
            edge: Color::from_rgb(0.6, 0.6, 0.65),
            node_bg: Color::from_rgb(1.0, 1.0, 1.0),
            node_border: Color::from_rgb(0.75, 0.75, 0.8),
            node_focused: Color::from_rgb(0.2, 0.5, 0.9),
            error: Color::from_rgb(0.9, 0.2, 0.2),
            success: Color::from_rgb(0.2, 0.7, 0.3),
        }
    }

    /// Get colors for dark theme
    pub fn dark() -> Self {
        Self {
            background: Color::from_rgb(0.08, 0.08, 0.1),
            surface: Color::from_rgb(0.12, 0.12, 0.15),
            primary: Color::from_rgb(0.3, 0.6, 1.0),
            secondary: Color::from_rgb(0.4, 0.7, 0.4),
            text: Color::from_rgb(0.9, 0.9, 0.92),
            text_secondary: Color::from_rgb(0.6, 0.6, 0.65),
            border: Color::from_rgb(0.3, 0.3, 0.35),
            edge: Color::from_rgb(0.4, 0.4, 0.5),
            node_bg: Color::from_rgb(0.15, 0.15, 0.18),
            node_border: Color::from_rgb(0.35, 0.35, 0.4),
            node_focused: Color::from_rgb(0.3, 0.6, 1.0),
            error: Color::from_rgb(1.0, 0.3, 0.3),
            success: Color::from_rgb(0.3, 0.8, 0.4),
        }
    }
}

impl From<Theme> for ThemeColors {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => ThemeColors::light(),
            Theme::Dark => ThemeColors::dark(),
        }
    }
}

/// Extension trait to easily get theme colors
pub trait ThemeExt {
    fn colors(&self) -> ThemeColors;
}

impl ThemeExt for Theme {
    fn colors(&self) -> ThemeColors {
        ThemeColors::from(*self)
    }
}
