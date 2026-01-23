use jsonquill::theme::{colors::ThemeColors, get_builtin_theme};
use ratatui::style::Color;

// Tests for get_builtin_theme function

#[test]
fn test_default_dark_theme_exists() {
    let theme = get_builtin_theme("default-dark");
    assert!(theme.is_some());
}

#[test]
fn test_default_light_theme_exists() {
    let theme = get_builtin_theme("default-light");
    assert!(theme.is_some());
}

#[test]
fn test_invalid_theme_returns_none() {
    let theme = get_builtin_theme("nonexistent");
    assert!(theme.is_none());
}

#[test]
fn test_theme_name_is_preserved() {
    let dark = get_builtin_theme("default-dark").unwrap();
    assert_eq!(dark.name, "default-dark");

    let light = get_builtin_theme("default-light").unwrap();
    assert_eq!(light.name, "default-light");
}

// Tests for default-dark theme colors

#[test]
fn test_dark_theme_syntax_colors() {
    let theme = get_builtin_theme("default-dark").unwrap();

    // jless uses ANSI colors that adapt to terminal theme
    assert_eq!(theme.colors.key, Color::LightBlue); // ANSI 12
    assert_eq!(theme.colors.string, Color::Green); // ANSI 2
    assert_eq!(theme.colors.number, Color::Magenta); // ANSI 5
    assert_eq!(theme.colors.boolean, Color::Yellow); // ANSI 3
    assert_eq!(theme.colors.null, Color::DarkGray); // ANSI 8
}

#[test]
fn test_dark_theme_ui_colors() {
    let theme = get_builtin_theme("default-dark").unwrap();

    // jless uses terminal defaults with white status bar
    assert_eq!(theme.colors.background, Color::Reset); // Terminal default background
    assert_eq!(theme.colors.foreground, Color::Gray); // Terminal default light
    assert_eq!(theme.colors.cursor, Color::LightBlue); // Match key color
    assert_eq!(theme.colors.status_line_bg, Color::White); // White status bar like jless
    assert_eq!(theme.colors.status_line_fg, Color::Black); // Black text on white
}

#[test]
fn test_dark_theme_semantic_colors() {
    let theme = get_builtin_theme("default-dark").unwrap();

    // Use ANSI colors for semantic meaning
    assert_eq!(theme.colors.error, Color::Red);
    assert_eq!(theme.colors.warning, Color::Yellow);
    assert_eq!(theme.colors.info, Color::LightBlue);
    assert_eq!(theme.colors.search_highlight, Color::Yellow);
}

// Tests for default-light theme colors

#[test]
fn test_light_theme_syntax_colors() {
    let theme = get_builtin_theme("default-light").unwrap();

    // Verify syntax colors are set
    assert_eq!(theme.colors.key, Color::Rgb(166, 38, 164));
    assert_eq!(theme.colors.string, Color::Rgb(80, 161, 79));
    assert_eq!(theme.colors.number, Color::Rgb(152, 104, 1));
    assert_eq!(theme.colors.boolean, Color::Rgb(1, 132, 188));
    assert_eq!(theme.colors.null, Color::Rgb(160, 30, 170));
}

#[test]
fn test_light_theme_ui_colors() {
    let theme = get_builtin_theme("default-light").unwrap();

    // Verify UI colors
    assert_eq!(theme.colors.background, Color::Rgb(250, 250, 250));
    assert_eq!(theme.colors.foreground, Color::Rgb(56, 58, 66));
    assert_eq!(theme.colors.cursor, Color::Rgb(82, 139, 255));
    assert_eq!(theme.colors.status_line_bg, Color::Rgb(238, 238, 238));
    assert_eq!(theme.colors.status_line_fg, Color::Rgb(56, 58, 66));
}

#[test]
fn test_light_theme_semantic_colors() {
    let theme = get_builtin_theme("default-light").unwrap();

    // Verify semantic colors
    assert_eq!(theme.colors.error, Color::Rgb(202, 18, 67));
    assert_eq!(theme.colors.warning, Color::Rgb(152, 104, 1));
    assert_eq!(theme.colors.info, Color::Rgb(1, 132, 188));
    assert_eq!(theme.colors.search_highlight, Color::Rgb(220, 220, 220));
}

// Tests for ThemeColors constructors

#[test]
fn test_theme_colors_default_dark() {
    let colors = ThemeColors::default_dark();

    // Verify it uses terminal default background like jless
    assert_eq!(colors.background, Color::Reset);
    assert_eq!(colors.foreground, Color::Gray);
}

#[test]
fn test_theme_colors_default_light() {
    let colors = ThemeColors::default_light();

    // Verify it creates a valid color set with light background
    assert_eq!(colors.background, Color::Rgb(250, 250, 250));
    assert_eq!(colors.foreground, Color::Rgb(56, 58, 66));
}

// Tests for theme cloning

#[test]
fn test_theme_can_be_cloned() {
    let theme1 = get_builtin_theme("default-dark").unwrap();
    let theme2 = theme1.clone();

    assert_eq!(theme1.name, theme2.name);
    assert_eq!(theme1.colors.background, theme2.colors.background);
}

#[test]
fn test_theme_colors_can_be_cloned() {
    let colors1 = ThemeColors::default_dark();
    let colors2 = colors1.clone();

    assert_eq!(colors1.background, colors2.background);
    assert_eq!(colors1.key, colors2.key);
}

// Tests for theme contrast (dark vs light)

#[test]
fn test_dark_and_light_themes_have_different_backgrounds() {
    let dark = get_builtin_theme("default-dark").unwrap();
    let light = get_builtin_theme("default-light").unwrap();

    assert_ne!(dark.colors.background, light.colors.background);
    assert_ne!(dark.colors.foreground, light.colors.foreground);
}

#[test]
fn test_both_themes_have_different_cursor_colors() {
    let dark = get_builtin_theme("default-dark").unwrap();
    let light = get_builtin_theme("default-light").unwrap();

    // Dark theme uses ANSI light blue, light theme uses RGB
    assert_eq!(dark.colors.cursor, Color::LightBlue);
    assert_eq!(light.colors.cursor, Color::Rgb(82, 139, 255));
}
