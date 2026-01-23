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
/// use jsonquill::theme::colors::ThemeColors;
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
    /// Color for collapsed previews (object/array content when collapsed).
    pub preview: Color,
}

impl ThemeColors {
    /// Returns the default dark color scheme.
    ///
    /// This theme uses ANSI colors that match jless, the command-line JSON viewer.
    /// ANSI colors adapt to the user's terminal color scheme, so the actual RGB
    /// values displayed will depend on their terminal configuration.
    ///
    /// # Color Palette
    ///
    /// Based on jless (https://github.com/PaulJuliusMartinez/jless):
    /// - Keys: Light Blue (ANSI 12)
    /// - Strings: Green (ANSI 2)
    /// - Numbers: Magenta (ANSI 5)
    /// - Booleans: Yellow (ANSI 3)
    /// - Null: Dark Gray (ANSI 8)
    /// - Background: Terminal default (Color::Reset)
    /// - Foreground: Gray (ANSI 7)
    /// - Status bar: White background with black text
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::theme::colors::ThemeColors;
    /// use ratatui::style::Color;
    ///
    /// let colors = ThemeColors::default_dark();
    /// assert_eq!(colors.background, Color::Reset);
    /// assert_eq!(colors.status_line_bg, Color::White);
    /// ```
    pub fn default_dark() -> Self {
        Self {
            key: Color::LightBlue,  // ANSI 12 (jless LIGHT_BLUE)
            string: Color::Green,   // ANSI 2
            number: Color::Magenta, // ANSI 5
            boolean: Color::Yellow, // ANSI 3
            null: Color::DarkGray,  // ANSI 8 (jless LIGHT_BLACK)

            background: Color::Reset, // Use terminal's default background
            foreground: Color::Gray,  // ANSI 7 (terminal default light)
            cursor: Color::LightBlue, // ANSI 12 (match key color)
            status_line_bg: Color::White, // White status bar like jless
            status_line_fg: Color::Black, // Black text on white

            error: Color::Red,               // ANSI 1
            warning: Color::Yellow,          // ANSI 3
            info: Color::LightBlue,          // ANSI 12
            search_highlight: Color::Yellow, // ANSI 3 (jless uses yellow for search)
            preview: Color::Cyan,            // ANSI 6 (for collapsed previews)
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
    /// use jsonquill::theme::colors::ThemeColors;
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
            preview: Color::Rgb(1, 132, 188), // Blue for collapsed previews in light theme
        }
    }
}
