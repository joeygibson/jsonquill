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

    // Verify syntax colors are set (not black by default)
    assert_eq!(theme.colors.key, Color::Rgb(224, 108, 117));
    assert_eq!(theme.colors.string, Color::Rgb(152, 195, 121));
    assert_eq!(theme.colors.number, Color::Rgb(209, 154, 102));
    assert_eq!(theme.colors.boolean, Color::Rgb(86, 182, 194));
    assert_eq!(theme.colors.null, Color::Rgb(198, 120, 221));
}

#[test]
fn test_dark_theme_ui_colors() {
    let theme = get_builtin_theme("default-dark").unwrap();

    // Verify UI colors
    assert_eq!(theme.colors.background, Color::Rgb(40, 44, 52));
    assert_eq!(theme.colors.foreground, Color::Rgb(171, 178, 191));
    assert_eq!(theme.colors.cursor, Color::Rgb(82, 139, 255));
    assert_eq!(theme.colors.status_line_bg, Color::Rgb(33, 37, 43));
    assert_eq!(theme.colors.status_line_fg, Color::Rgb(171, 178, 191));
}

#[test]
fn test_dark_theme_semantic_colors() {
    let theme = get_builtin_theme("default-dark").unwrap();

    // Verify semantic colors
    assert_eq!(theme.colors.error, Color::Rgb(224, 108, 117));
    assert_eq!(theme.colors.warning, Color::Rgb(229, 192, 123));
    assert_eq!(theme.colors.info, Color::Rgb(97, 175, 239));
    assert_eq!(theme.colors.search_highlight, Color::Rgb(62, 68, 81));
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

    // Verify it creates a valid color set with dark background
    assert_eq!(colors.background, Color::Rgb(40, 44, 52));
    assert_eq!(colors.foreground, Color::Rgb(171, 178, 191));
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
fn test_both_themes_have_same_cursor_color() {
    let dark = get_builtin_theme("default-dark").unwrap();
    let light = get_builtin_theme("default-light").unwrap();

    // Both themes use the same cursor color for consistency
    assert_eq!(dark.colors.cursor, light.colors.cursor);
    assert_eq!(dark.colors.cursor, Color::Rgb(82, 139, 255));
}
