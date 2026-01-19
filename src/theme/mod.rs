//! Theme system for jeditor.
//!
//! This module provides the theme infrastructure for jeditor, including:
//! - Color definitions ([`colors`] module)
//! - Theme data structure ([`Theme`])
//! - Built-in theme access ([`get_builtin_theme`])
//!
//! # Built-in Themes
//!
//! jeditor includes two built-in themes:
//! - `"default-dark"`: A dark theme optimized for low-light environments
//! - `"default-light"`: A light theme for well-lit environments
//!
//! # Examples
//!
//! ```
//! use jeditor::theme::get_builtin_theme;
//!
//! // Load the default dark theme
//! let theme = get_builtin_theme("default-dark").unwrap();
//! println!("Theme: {}", theme.name);
//!
//! // Access theme colors
//! println!("Background: {:?}", theme.colors.background);
//! ```

pub mod colors;

use colors::ThemeColors;

/// A color theme for the jeditor terminal UI.
///
/// Each theme has a name and a set of colors defined by [`ThemeColors`].
/// Themes can be loaded from the built-in set using [`get_builtin_theme`].
///
/// # Examples
///
/// ```
/// use jeditor::theme::{Theme, get_builtin_theme};
///
/// let theme = get_builtin_theme("default-dark").unwrap();
/// assert_eq!(theme.name, "default-dark");
/// ```
#[derive(Debug, Clone)]
pub struct Theme {
    /// The name of the theme (e.g., "default-dark").
    pub name: String,
    /// The color definitions for this theme.
    pub colors: ThemeColors,
}

/// Returns a built-in theme by name.
///
/// # Arguments
///
/// * `name` - The name of the theme to retrieve. Valid values are:
///   - `"default-dark"`: Dark theme for low-light environments
///   - `"default-light"`: Light theme for well-lit environments
///
/// # Returns
///
/// - `Some(Theme)` if the theme name is recognized
/// - `None` if the theme name is not found
///
/// # Examples
///
/// ```
/// use jeditor::theme::get_builtin_theme;
///
/// // Get a valid theme
/// let dark = get_builtin_theme("default-dark");
/// assert!(dark.is_some());
///
/// // Try an invalid theme name
/// let invalid = get_builtin_theme("nonexistent");
/// assert!(invalid.is_none());
/// ```
pub fn get_builtin_theme(name: &str) -> Option<Theme> {
    match name {
        "default-dark" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::default_dark(),
        }),
        "default-light" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::default_light(),
        }),
        _ => None,
    }
}
