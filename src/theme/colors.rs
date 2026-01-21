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
    /// This is a dark theme inspired by jless, the command-line JSON viewer.
    /// Features a true black background with bright, high-contrast colors
    /// optimized for terminal use.
    ///
    /// # Color Palette
    ///
    /// - Background: Black (#000000)
    /// - Foreground: Bright green (#b8ffa7)
    /// - Keys: Light blue (ANSI 12)
    /// - Booleans: Yellow (ANSI 3)
    /// - Null: Gray (ANSI 8)
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::theme::colors::ThemeColors;
    /// use ratatui::style::Color;
    ///
    /// let colors = ThemeColors::default_dark();
    /// assert_eq!(colors.background, Color::Rgb(0, 0, 0));
    /// ```
    pub fn default_dark() -> Self {
        Self {
            key: Color::Rgb(92, 133, 255),       // Light blue (ANSI 12)
            string: Color::Rgb(184, 255, 167),   // Bright green #b8ffa7
            number: Color::Rgb(184, 255, 167),   // Bright green #b8ffa7
            boolean: Color::Rgb(255, 255, 85),   // Yellow (ANSI 3)
            null: Color::Rgb(128, 128, 128),     // Gray (ANSI 8)

            background: Color::Rgb(0, 0, 0),     // Black
            foreground: Color::Rgb(184, 255, 167), // Bright green #b8ffa7
            cursor: Color::Rgb(92, 133, 255),    // Light blue
            status_line_bg: Color::Rgb(20, 20, 20), // Very dark gray
            status_line_fg: Color::Rgb(184, 255, 167),

            error: Color::Rgb(255, 85, 85),      // Bright red
            warning: Color::Rgb(255, 255, 85),   // Yellow
            info: Color::Rgb(92, 133, 255),      // Light blue
            search_highlight: Color::Rgb(255, 255, 85), // Yellow highlight
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
