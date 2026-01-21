use jeditor::theme::{get_builtin_theme, colors::ThemeColors};
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

    // jless-inspired color scheme
    assert_eq!(theme.colors.key, Color::Rgb(92, 133, 255));      // Light blue (ANSI 12)
    assert_eq!(theme.colors.string, Color::Rgb(184, 255, 167));  // Bright green
    assert_eq!(theme.colors.number, Color::Rgb(184, 255, 167));  // Bright green
    assert_eq!(theme.colors.boolean, Color::Rgb(255, 255, 85));  // Yellow (ANSI 3)
    assert_eq!(theme.colors.null, Color::Rgb(128, 128, 128));    // Gray (ANSI 8)
}

#[test]
fn test_dark_theme_ui_colors() {
    let theme = get_builtin_theme("default-dark").unwrap();

    // jless-inspired UI colors
    assert_eq!(theme.colors.background, Color::Rgb(0, 0, 0));      // Black
    assert_eq!(theme.colors.foreground, Color::Rgb(184, 255, 167)); // Bright green
    assert_eq!(theme.colors.cursor, Color::Rgb(92, 133, 255));     // Light blue
    assert_eq!(theme.colors.status_line_bg, Color::Rgb(20, 20, 20)); // Very dark gray
    assert_eq!(theme.colors.status_line_fg, Color::Rgb(184, 255, 167));
}

#[test]
fn test_dark_theme_semantic_colors() {
    let theme = get_builtin_theme("default-dark").unwrap();

    // jless-inspired semantic colors
    assert_eq!(theme.colors.error, Color::Rgb(255, 85, 85));      // Bright red
    assert_eq!(theme.colors.warning, Color::Rgb(255, 255, 85));   // Yellow
    assert_eq!(theme.colors.info, Color::Rgb(92, 133, 255));      // Light blue
    assert_eq!(theme.colors.search_highlight, Color::Rgb(255, 255, 85)); // Yellow highlight
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

    // Verify it creates a valid color set with dark background (jless-inspired)
    assert_eq!(colors.background, Color::Rgb(0, 0, 0));
    assert_eq!(colors.foreground, Color::Rgb(184, 255, 167));
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

    // Dark theme uses light blue cursor (jless-inspired)
    assert_eq!(dark.colors.cursor, Color::Rgb(92, 133, 255));
    // Light theme keeps its original cursor color
    assert_eq!(light.colors.cursor, Color::Rgb(82, 139, 255));
}
