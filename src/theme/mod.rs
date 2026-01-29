//! Theme system for jsonquill.
//!
//! This module provides the theme infrastructure for jsonquill, including:
//! - Color definitions ([`colors`] module)
//! - Theme data structure ([`Theme`])
//! - Built-in theme access ([`get_builtin_theme`])
//!
//! # Built-in Themes
//!
//! jsonquill includes multiple built-in themes:
//! - `"default-dark"`: A dark theme optimized for low-light environments
//! - `"default-light"`: A light theme for well-lit environments
//! - `"gruvbox-dark"`: Retro groove color scheme with warm, earthy tones
//! - `"nord"`: Arctic, north-bluish color palette
//! - `"dracula"`: Dark theme with vibrant purples and pinks
//! - `"solarized-dark"`: Precision color scheme for machines and people
//! - `"monokai"`: Popular color scheme inspired by Monokai Pro
//! - `"one-dark"`: The default dark theme from Atom editor
//!
//! # Examples
//!
//! ```
//! use jsonquill::theme::get_builtin_theme;
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

/// A color theme for the jsonquill terminal UI.
///
/// Each theme has a name and a set of colors defined by [`ThemeColors`].
/// Themes can be loaded from the built-in set using [`get_builtin_theme`].
///
/// # Examples
///
/// ```
/// use jsonquill::theme::{Theme, get_builtin_theme};
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
///   - `"gruvbox-dark"`: Retro groove color scheme
///   - `"nord"`: Arctic, north-bluish palette
///   - `"dracula"`: Vibrant purples and pinks
///   - `"solarized-dark"`: Precision color scheme
///   - `"monokai"`: Monokai Pro inspired
///   - `"one-dark"`: Atom's default dark theme
///
/// # Returns
///
/// - `Some(Theme)` if the theme name is recognized
/// - `None` if the theme name is not found
///
/// # Examples
///
/// ```
/// use jsonquill::theme::get_builtin_theme;
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
        "gruvbox-dark" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::gruvbox_dark(),
        }),
        "nord" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::nord(),
        }),
        "dracula" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::dracula(),
        }),
        "solarized-dark" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::solarized_dark(),
        }),
        "monokai" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::monokai(),
        }),
        "one-dark" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::one_dark(),
        }),
        "gruvbox-light" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::gruvbox_light(),
        }),
        "solarized-light" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::solarized_light(),
        }),
        "tokyo-night" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::tokyo_night(),
        }),
        "catppuccin-mocha" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::catppuccin_mocha(),
        }),
        "catppuccin-latte" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::catppuccin_latte(),
        }),
        "github-dark" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::github_dark(),
        }),
        "github-light" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::github_light(),
        }),
        _ => None,
    }
}

/// Returns a list of all available built-in theme names.
///
/// # Returns
///
/// A vector of theme names that can be used with `get_builtin_theme`.
///
/// # Examples
///
/// ```
/// use jsonquill::theme::list_builtin_themes;
///
/// let themes = list_builtin_themes();
/// assert!(themes.contains(&"default-dark".to_string()));
/// assert!(themes.contains(&"default-light".to_string()));
/// ```
pub fn list_builtin_themes() -> Vec<String> {
    let mut themes = vec![
        "catppuccin-latte".to_string(),
        "catppuccin-mocha".to_string(),
        "default-dark".to_string(),
        "default-light".to_string(),
        "dracula".to_string(),
        "github-dark".to_string(),
        "github-light".to_string(),
        "gruvbox-dark".to_string(),
        "gruvbox-light".to_string(),
        "monokai".to_string(),
        "nord".to_string(),
        "one-dark".to_string(),
        "solarized-dark".to_string(),
        "solarized-light".to_string(),
        "tokyo-night".to_string(),
    ];
    themes.sort();
    themes
}
