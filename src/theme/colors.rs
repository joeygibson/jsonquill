//! Color definitions for jeditor themes.
//!
//! This module defines the [`ThemeColors`] struct which contains all color
//! values used in the jeditor terminal UI. Colors are organized into three
//! categories: syntax highlighting, UI elements, and semantic colors.

use ratatui::style::Color;

/// Defines all colors used in a jeditor theme.
///
/// Colors are organized into three main categories:
/// - **Syntax colors**: Used for JSON syntax highlighting (keys, strings, numbers, etc.)
/// - **UI colors**: Used for interface elements (background, foreground, cursor, status line)
/// - **Semantic colors**: Used for messages and highlights (errors, warnings, info, search)
///
/// # Examples
///
/// ```
/// use jeditor::theme::colors::ThemeColors;
///
/// // Get the default dark theme colors
/// let dark = ThemeColors::default_dark();
/// println!("Background: {:?}", dark.background);
///
/// // Get the default light theme colors
/// let light = ThemeColors::default_light();
/// println!("Background: {:?}", light.background);
/// ```
#[derive(Debug, Clone)]
pub struct ThemeColors {
    // Syntax colors
    /// Color for JSON object keys.
    pub key: Color,
    /// Color for JSON string values.
    pub string: Color,
    /// Color for JSON number values.
    pub number: Color,
    /// Color for JSON boolean values (true/false).
    pub boolean: Color,
    /// Color for JSON null values.
    pub null: Color,

    // UI colors
    /// Main background color for the editor.
    pub background: Color,
    /// Main foreground/text color for the editor.
    pub foreground: Color,
    /// Color for the cursor position indicator.
    pub cursor: Color,
    /// Background color for the status line.
    pub status_line_bg: Color,
    /// Foreground/text color for the status line.
    pub status_line_fg: Color,

    // Semantic colors
    /// Color for error messages and indicators.
    pub error: Color,
    /// Color for warning messages and indicators.
    pub warning: Color,
    /// Color for informational messages and indicators.
    pub info: Color,
    /// Background color for search result highlights.
    pub search_highlight: Color,
}

impl ThemeColors {
    /// Returns the default dark color scheme.
    ///
    /// This is a dark theme inspired by the One Dark color palette,
    /// optimized for comfortable extended use in low-light environments.
    ///
    /// # Color Palette
    ///
    /// - Background: Dark grey (#282c34)
    /// - Foreground: Light grey (#abb2bf)
    /// - Syntax: Warm, vibrant colors for good readability
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::theme::colors::ThemeColors;
    /// use ratatui::style::Color;
    ///
    /// let colors = ThemeColors::default_dark();
    /// assert_eq!(colors.background, Color::Rgb(40, 44, 52));
    /// ```
    pub fn default_dark() -> Self {
        Self {
            key: Color::Rgb(224, 108, 117),      // #e06c75
            string: Color::Rgb(152, 195, 121),   // #98c379
            number: Color::Rgb(209, 154, 102),   // #d19a66
            boolean: Color::Rgb(86, 182, 194),   // #56b6c2
            null: Color::Rgb(198, 120, 221),     // #c678dd

            background: Color::Rgb(40, 44, 52),  // #282c34
            foreground: Color::Rgb(171, 178, 191), // #abb2bf
            cursor: Color::Rgb(82, 139, 255),    // #528bff
            status_line_bg: Color::Rgb(33, 37, 43), // #21252b
            status_line_fg: Color::Rgb(171, 178, 191),

            error: Color::Rgb(224, 108, 117),
            warning: Color::Rgb(229, 192, 123),  // #e5c07b
            info: Color::Rgb(97, 175, 239),      // #61afef
            search_highlight: Color::Rgb(62, 68, 81), // #3e4451
        }
    }

    /// Returns the default light color scheme.
    ///
    /// This is a light theme with high contrast, designed for use in
    /// well-lit environments and for users who prefer light backgrounds.
    ///
    /// # Color Palette
    ///
    /// - Background: Off-white (#fafafa)
    /// - Foreground: Dark grey (#383a42)
    /// - Syntax: Rich, saturated colors for clarity
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::theme::colors::ThemeColors;
    /// use ratatui::style::Color;
    ///
    /// let colors = ThemeColors::default_light();
    /// assert_eq!(colors.background, Color::Rgb(250, 250, 250));
    /// ```
    pub fn default_light() -> Self {
        Self {
            key: Color::Rgb(166, 38, 164),
            string: Color::Rgb(80, 161, 79),
            number: Color::Rgb(152, 104, 1),
            boolean: Color::Rgb(1, 132, 188),
            null: Color::Rgb(160, 30, 170),

            background: Color::Rgb(250, 250, 250),
            foreground: Color::Rgb(56, 58, 66),
            cursor: Color::Rgb(82, 139, 255),
            status_line_bg: Color::Rgb(238, 238, 238),
            status_line_fg: Color::Rgb(56, 58, 66),

            error: Color::Rgb(202, 18, 67),
            warning: Color::Rgb(152, 104, 1),
            info: Color::Rgb(1, 132, 188),
            search_highlight: Color::Rgb(220, 220, 220),
        }
    }
}
