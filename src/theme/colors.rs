//! Color definitions for jsonquill themes.
//!
//! This module defines the [`ThemeColors`] struct which contains all color
//! values used in the jsonquill terminal UI. Colors are organized into three
//! categories: syntax highlighting, UI elements, and semantic colors.

use ratatui::style::Color;

/// Defines all colors used in a jsonquill theme.
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
    /// Background color for visual mode selection.
    pub visual_selection_bg: Color,
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

            error: Color::Red,                    // ANSI 1
            warning: Color::Yellow,               // ANSI 3
            info: Color::LightBlue,               // ANSI 12
            search_highlight: Color::Yellow,      // ANSI 3 (jless uses yellow for search)
            preview: Color::DarkGray,             // ANSI 8 for collapsed previews
            visual_selection_bg: Color::DarkGray, // ANSI 8 for visual mode selection
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
            preview: Color::DarkGray, // ANSI 8 for collapsed previews
            visual_selection_bg: Color::Rgb(220, 220, 220), // Light gray for visual selection
        }
    }

    /// Returns the Gruvbox Dark color scheme.
    ///
    /// A retro groove color scheme with warm, earthy tones.
    /// Based on the popular Gruvbox theme by morhetz.
    pub fn gruvbox_dark() -> Self {
        Self {
            key: Color::Rgb(251, 184, 108),    // orange
            string: Color::Rgb(184, 187, 38),  // green
            number: Color::Rgb(211, 134, 155), // purple
            boolean: Color::Rgb(254, 128, 25), // bright orange
            null: Color::Rgb(146, 131, 116),   // gray

            background: Color::Rgb(40, 40, 40),        // dark bg
            foreground: Color::Rgb(235, 219, 178),     // light fg
            cursor: Color::Rgb(251, 184, 108),         // orange
            status_line_bg: Color::Rgb(60, 56, 54),    // darker bg
            status_line_fg: Color::Rgb(235, 219, 178), // light fg

            error: Color::Rgb(251, 73, 52),              // red
            warning: Color::Rgb(250, 189, 47),           // yellow
            info: Color::Rgb(131, 165, 152),             // aqua
            search_highlight: Color::Rgb(215, 153, 33),  // yellow highlight
            preview: Color::Rgb(146, 131, 116),          // gray
            visual_selection_bg: Color::Rgb(60, 56, 54), // darker gray for selection
        }
    }

    /// Returns the Nord color scheme.
    ///
    /// An arctic, north-bluish color palette.
    /// Based on the Nord theme by Arctic Ice Studio.
    pub fn nord() -> Self {
        Self {
            key: Color::Rgb(136, 192, 208),     // frost cyan
            string: Color::Rgb(163, 190, 140),  // aurora green
            number: Color::Rgb(180, 142, 173),  // aurora purple
            boolean: Color::Rgb(235, 203, 139), // aurora yellow
            null: Color::Rgb(76, 86, 106),      // polar night gray

            background: Color::Rgb(46, 52, 64), // polar night darkest
            foreground: Color::Rgb(216, 222, 233), // snow storm lightest
            cursor: Color::Rgb(136, 192, 208),  // frost cyan
            status_line_bg: Color::Rgb(59, 66, 82), // polar night
            status_line_fg: Color::Rgb(216, 222, 233), // snow storm

            error: Color::Rgb(191, 97, 106),             // aurora red
            warning: Color::Rgb(235, 203, 139),          // aurora yellow
            info: Color::Rgb(136, 192, 208),             // frost cyan
            search_highlight: Color::Rgb(235, 203, 139), // aurora yellow
            preview: Color::Rgb(76, 86, 106),            // polar night gray
            visual_selection_bg: Color::Rgb(59, 66, 82), // polar night for selection
        }
    }

    /// Returns the Dracula color scheme.
    ///
    /// A dark theme with vibrant purples and pinks.
    /// Based on the Dracula theme by Zeno Rocha.
    pub fn dracula() -> Self {
        Self {
            key: Color::Rgb(139, 233, 253),     // cyan
            string: Color::Rgb(241, 250, 140),  // yellow
            number: Color::Rgb(189, 147, 249),  // purple
            boolean: Color::Rgb(255, 121, 198), // pink
            null: Color::Rgb(98, 114, 164),     // comment

            background: Color::Rgb(40, 42, 54),     // background
            foreground: Color::Rgb(248, 248, 242),  // foreground
            cursor: Color::Rgb(189, 147, 249),      // purple
            status_line_bg: Color::Rgb(68, 71, 90), // current line
            status_line_fg: Color::Rgb(248, 248, 242), // foreground

            error: Color::Rgb(255, 85, 85),              // red
            warning: Color::Rgb(255, 184, 108),          // orange
            info: Color::Rgb(139, 233, 253),             // cyan
            search_highlight: Color::Rgb(255, 121, 198), // pink
            preview: Color::Rgb(98, 114, 164),           // comment
            visual_selection_bg: Color::Rgb(68, 71, 90), // current line for selection
        }
    }

    /// Returns the Solarized Dark color scheme.
    ///
    /// A precision color scheme for machines and people.
    /// Based on the Solarized theme by Ethan Schoonover.
    pub fn solarized_dark() -> Self {
        Self {
            key: Color::Rgb(38, 139, 210),    // blue
            string: Color::Rgb(133, 153, 0),  // green
            number: Color::Rgb(211, 54, 130), // magenta
            boolean: Color::Rgb(181, 137, 0), // yellow
            null: Color::Rgb(88, 110, 117),   // base01

            background: Color::Rgb(0, 43, 54),         // base03
            foreground: Color::Rgb(131, 148, 150),     // base0
            cursor: Color::Rgb(38, 139, 210),          // blue
            status_line_bg: Color::Rgb(7, 54, 66),     // base02
            status_line_fg: Color::Rgb(147, 161, 161), // base1

            error: Color::Rgb(220, 50, 47),             // red
            warning: Color::Rgb(203, 75, 22),           // orange
            info: Color::Rgb(42, 161, 152),             // cyan
            search_highlight: Color::Rgb(181, 137, 0),  // yellow
            preview: Color::Rgb(88, 110, 117),          // base01
            visual_selection_bg: Color::Rgb(7, 54, 66), // base02 for selection
        }
    }

    /// Returns the Monokai color scheme.
    ///
    /// A popular color scheme inspired by the Monokai Pro theme.
    pub fn monokai() -> Self {
        Self {
            key: Color::Rgb(102, 217, 239),    // cyan
            string: Color::Rgb(230, 219, 116), // yellow
            number: Color::Rgb(174, 129, 255), // purple
            boolean: Color::Rgb(255, 97, 136), // pink
            null: Color::Rgb(117, 113, 94),    // comment

            background: Color::Rgb(39, 40, 34),     // background
            foreground: Color::Rgb(248, 248, 240),  // foreground
            cursor: Color::Rgb(102, 217, 239),      // cyan
            status_line_bg: Color::Rgb(73, 72, 62), // line highlight
            status_line_fg: Color::Rgb(248, 248, 240), // foreground

            error: Color::Rgb(249, 38, 114),             // pink/red
            warning: Color::Rgb(253, 151, 31),           // orange
            info: Color::Rgb(102, 217, 239),             // cyan
            search_highlight: Color::Rgb(230, 219, 116), // yellow
            preview: Color::Rgb(117, 113, 94),           // comment
            visual_selection_bg: Color::Rgb(73, 72, 62), // line highlight for selection
        }
    }

    /// Returns the One Dark color scheme.
    ///
    /// The default dark theme from Atom editor.
    /// Based on the One Dark theme by Atom.
    pub fn one_dark() -> Self {
        Self {
            key: Color::Rgb(224, 108, 117),     // red
            string: Color::Rgb(152, 195, 121),  // green
            number: Color::Rgb(209, 154, 102),  // orange
            boolean: Color::Rgb(198, 120, 221), // purple
            null: Color::Rgb(92, 99, 112),      // comment

            background: Color::Rgb(40, 44, 52),     // background
            foreground: Color::Rgb(171, 178, 191),  // foreground
            cursor: Color::Rgb(97, 175, 239),       // blue
            status_line_bg: Color::Rgb(33, 37, 43), // gutter bg
            status_line_fg: Color::Rgb(171, 178, 191), // foreground

            error: Color::Rgb(224, 108, 117),            // red
            warning: Color::Rgb(229, 192, 123),          // yellow
            info: Color::Rgb(97, 175, 239),              // blue
            search_highlight: Color::Rgb(229, 192, 123), // yellow
            preview: Color::Rgb(92, 99, 112),            // comment
            visual_selection_bg: Color::Rgb(33, 37, 43), // gutter bg for selection
        }
    }

    /// Returns the Gruvbox Light color scheme.
    ///
    /// A retro groove color scheme with warm tones on light background.
    /// Light variant of the Gruvbox theme by morhetz.
    pub fn gruvbox_light() -> Self {
        Self {
            key: Color::Rgb(175, 58, 3),      // dark orange
            string: Color::Rgb(121, 116, 14), // dark green
            number: Color::Rgb(143, 63, 113), // dark purple
            boolean: Color::Rgb(214, 93, 14), // orange
            null: Color::Rgb(102, 92, 84),    // gray

            background: Color::Rgb(251, 241, 199), // light bg
            foreground: Color::Rgb(60, 56, 54),    // dark fg
            cursor: Color::Rgb(175, 58, 3),        // dark orange
            status_line_bg: Color::Rgb(235, 219, 178), // lighter bg
            status_line_fg: Color::Rgb(60, 56, 54), // dark fg

            error: Color::Rgb(204, 36, 29),                 // red
            warning: Color::Rgb(215, 153, 33),              // yellow
            info: Color::Rgb(69, 133, 136),                 // aqua
            search_highlight: Color::Rgb(250, 189, 47),     // bright yellow
            preview: Color::Rgb(102, 92, 84),               // gray
            visual_selection_bg: Color::Rgb(235, 219, 178), // lighter bg for selection
        }
    }

    /// Returns the Solarized Light color scheme.
    ///
    /// Precision colors for machines and people (light variant).
    /// Based on the Solarized theme by Ethan Schoonover.
    pub fn solarized_light() -> Self {
        Self {
            key: Color::Rgb(38, 139, 210),    // blue
            string: Color::Rgb(133, 153, 0),  // green
            number: Color::Rgb(211, 54, 130), // magenta
            boolean: Color::Rgb(203, 75, 22), // orange
            null: Color::Rgb(147, 161, 161),  // base1

            background: Color::Rgb(253, 246, 227),     // base3
            foreground: Color::Rgb(101, 123, 131),     // base00
            cursor: Color::Rgb(38, 139, 210),          // blue
            status_line_bg: Color::Rgb(238, 232, 213), // base2
            status_line_fg: Color::Rgb(88, 110, 117),  // base01

            error: Color::Rgb(220, 50, 47),                 // red
            warning: Color::Rgb(181, 137, 0),               // yellow
            info: Color::Rgb(42, 161, 152),                 // cyan
            search_highlight: Color::Rgb(181, 137, 0),      // yellow
            preview: Color::Rgb(147, 161, 161),             // base1
            visual_selection_bg: Color::Rgb(238, 232, 213), // base2 for selection
        }
    }

    /// Returns the Tokyo Night color scheme.
    ///
    /// A clean, dark theme with vibrant purples, blues, and teals.
    /// Inspired by the colors of Tokyo at night.
    pub fn tokyo_night() -> Self {
        Self {
            key: Color::Rgb(125, 207, 255),     // cyan
            string: Color::Rgb(158, 206, 106),  // green
            number: Color::Rgb(255, 158, 100),  // orange
            boolean: Color::Rgb(187, 154, 247), // purple
            null: Color::Rgb(86, 95, 137),      // gray

            background: Color::Rgb(26, 27, 38),        // dark bg
            foreground: Color::Rgb(192, 202, 245),     // light fg
            cursor: Color::Rgb(125, 207, 255),         // cyan
            status_line_bg: Color::Rgb(36, 40, 59),    // darker bg
            status_line_fg: Color::Rgb(192, 202, 245), // light fg

            error: Color::Rgb(247, 118, 142),            // red
            warning: Color::Rgb(224, 175, 104),          // yellow
            info: Color::Rgb(125, 207, 255),             // cyan
            search_highlight: Color::Rgb(224, 175, 104), // yellow
            preview: Color::Rgb(86, 95, 137),            // gray
            visual_selection_bg: Color::Rgb(36, 40, 59), // darker bg for selection
        }
    }

    /// Returns the Catppuccin Mocha color scheme.
    ///
    /// A soothing pastel theme with warm, muted colors (dark variant).
    /// Based on the Catppuccin theme by Catppuccin.
    pub fn catppuccin_mocha() -> Self {
        Self {
            key: Color::Rgb(137, 180, 250),     // blue
            string: Color::Rgb(166, 227, 161),  // green
            number: Color::Rgb(250, 179, 135),  // peach
            boolean: Color::Rgb(203, 166, 247), // mauve
            null: Color::Rgb(108, 112, 134),    // surface2

            background: Color::Rgb(30, 30, 46),        // base
            foreground: Color::Rgb(205, 214, 244),     // text
            cursor: Color::Rgb(137, 180, 250),         // blue
            status_line_bg: Color::Rgb(49, 50, 68),    // mantle
            status_line_fg: Color::Rgb(205, 214, 244), // text

            error: Color::Rgb(243, 139, 168),            // red
            warning: Color::Rgb(249, 226, 175),          // yellow
            info: Color::Rgb(137, 220, 235),             // sky
            search_highlight: Color::Rgb(249, 226, 175), // yellow
            preview: Color::Rgb(108, 112, 134),          // surface2
            visual_selection_bg: Color::Rgb(49, 50, 68), // mantle for selection
        }
    }

    /// Returns the Catppuccin Latte color scheme.
    ///
    /// A soothing pastel theme with warm, muted colors (light variant).
    /// Based on the Catppuccin theme by Catppuccin.
    pub fn catppuccin_latte() -> Self {
        Self {
            key: Color::Rgb(30, 102, 245),     // blue
            string: Color::Rgb(64, 160, 43),   // green
            number: Color::Rgb(254, 100, 11),  // peach
            boolean: Color::Rgb(136, 57, 239), // mauve
            null: Color::Rgb(156, 160, 176),   // surface2

            background: Color::Rgb(239, 241, 245),     // base
            foreground: Color::Rgb(76, 79, 105),       // text
            cursor: Color::Rgb(30, 102, 245),          // blue
            status_line_bg: Color::Rgb(230, 233, 239), // mantle
            status_line_fg: Color::Rgb(76, 79, 105),   // text

            error: Color::Rgb(210, 15, 57),                 // red
            warning: Color::Rgb(223, 142, 29),              // yellow
            info: Color::Rgb(4, 165, 229),                  // sky
            search_highlight: Color::Rgb(223, 142, 29),     // yellow
            preview: Color::Rgb(156, 160, 176),             // surface2
            visual_selection_bg: Color::Rgb(230, 233, 239), // mantle for selection
        }
    }

    /// Returns the GitHub Dark color scheme.
    ///
    /// Clean dark theme matching GitHub's interface.
    /// Based on GitHub's Primer color system.
    pub fn github_dark() -> Self {
        Self {
            key: Color::Rgb(121, 192, 255),     // blue
            string: Color::Rgb(127, 219, 202),  // cyan
            number: Color::Rgb(255, 184, 108),  // orange
            boolean: Color::Rgb(255, 122, 135), // red
            null: Color::Rgb(110, 118, 129),    // gray

            background: Color::Rgb(13, 17, 23), // canvas default
            foreground: Color::Rgb(201, 209, 217), // fg default
            cursor: Color::Rgb(121, 192, 255),  // blue
            status_line_bg: Color::Rgb(22, 27, 34), // canvas subtle
            status_line_fg: Color::Rgb(201, 209, 217), // fg default

            error: Color::Rgb(248, 81, 73),              // danger fg
            warning: Color::Rgb(224, 155, 90),           // severe fg
            info: Color::Rgb(121, 192, 255),             // accent fg
            search_highlight: Color::Rgb(224, 155, 90),  // severe fg
            preview: Color::Rgb(110, 118, 129),          // gray
            visual_selection_bg: Color::Rgb(22, 27, 34), // canvas subtle for selection
        }
    }

    /// Returns the GitHub Light color scheme.
    ///
    /// Clean light theme matching GitHub's interface.
    /// Based on GitHub's Primer color system.
    pub fn github_light() -> Self {
        Self {
            key: Color::Rgb(9, 105, 218),     // blue
            string: Color::Rgb(26, 127, 100), // green
            number: Color::Rgb(207, 74, 34),  // orange
            boolean: Color::Rgb(207, 34, 46), // red
            null: Color::Rgb(87, 96, 106),    // gray

            background: Color::Rgb(255, 255, 255), // canvas default
            foreground: Color::Rgb(36, 41, 47),    // fg default
            cursor: Color::Rgb(9, 105, 218),       // blue
            status_line_bg: Color::Rgb(246, 248, 250), // canvas subtle
            status_line_fg: Color::Rgb(36, 41, 47), // fg default

            error: Color::Rgb(207, 34, 46),           // danger fg
            warning: Color::Rgb(191, 87, 0),          // severe fg
            info: Color::Rgb(9, 105, 218),            // accent fg
            search_highlight: Color::Rgb(191, 87, 0), // severe fg
            preview: Color::Rgb(87, 96, 106),         // gray
            visual_selection_bg: Color::Rgb(246, 248, 250), // canvas subtle for selection
        }
    }
}
