//! Status line widget for displaying editor state information.
//!
//! The status line shows:
//! - Current mode (NORMAL, INSERT, COMMAND)
//! - Filename (or "[No Name]" if unsaved)
//! - Dirty indicator "[+]" for unsaved changes
//! - Cursor position (row/total)
//!
//! Example status line: `NORMAL | data.json [+]                    5/20`

use crate::editor::state::EditorState;
use crate::theme::colors::ThemeColors;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Renders the status line showing mode, filename, and dirty indicator.
///
/// # Arguments
///
/// * `f` - The ratatui frame to render into
/// * `area` - The rectangular area to render the status line in
/// * `state` - The editor state containing mode, filename, and dirty flag
/// * `colors` - Theme colors for styling the status line
///
/// # Example
///
/// ```no_run
/// use ratatui::Frame;
/// use ratatui::layout::Rect;
/// use jsonquill::editor::state::EditorState;
/// use jsonquill::theme;
///
/// # fn example(f: &mut Frame, area: Rect) {
/// let state = EditorState::new_with_default_theme(jsonquill::document::tree::JsonTree::new(
///     jsonquill::document::node::JsonNode::new(
///         jsonquill::document::node::JsonValue::Null
///     )
/// ));
/// let theme = theme::get_builtin_theme("default-dark").unwrap();
/// jsonquill::ui::status_line::render_status_line(f, area, &state, &theme.colors);
/// # }
/// ```
pub fn render_status_line(f: &mut Frame, area: Rect, state: &EditorState, colors: &ThemeColors) {
    let mode_text = format!("{}", state.mode());
    let filename = state.filename().unwrap_or("[No Name]");
    let dirty_indicator = if state.is_dirty() { " [+]" } else { "" };

    // Get current path if not at root
    let cursor_path = state.cursor().path();
    let path_display = if !cursor_path.is_empty() {
        let path = state.get_current_path();
        format!(" {}", path)
    } else {
        String::new()
    };

    // Build left side components
    let mode_and_file = format!("{} | {}", mode_text, filename);

    // Show pending register if any
    let register_info = if let Some(reg) = state.get_pending_register() {
        if state.get_append_mode() {
            format!(" \"{}", reg.to_ascii_uppercase())
        } else {
            format!(" \"{}", reg)
        }
    } else {
        String::new()
    };

    // Add search results info if available
    let search_info = if let Some((current, total)) = state.search_results_info() {
        use crate::editor::state::SearchType;
        match state.search_type() {
            Some(SearchType::Text) => {
                format!(
                    " [Search: \"{}\"] Match {}/{}",
                    state.search_buffer(),
                    current,
                    total
                )
            }
            Some(SearchType::JsonPath(query)) => {
                format!(" [JSONPath: {}] Match {}/{}", query, current, total)
            }
            None => format!(" Match {}/{}", current, total),
        }
    } else {
        String::new()
    };

    // Get cursor position
    let row = state.cursor_position().0;
    let total = state.total_lines();
    let position = format!("{}/{}", row, total);

    // Calculate padding to position right-aligned text
    let total_width = area.width as usize;
    let left_len = mode_and_file.len()
        + path_display.len()
        + dirty_indicator.len()
        + register_info.len()
        + search_info.len();
    let position_len = position.len();

    // Ensure we don't overflow
    let padding = if left_len + position_len + 1 < total_width {
        total_width - left_len - position_len
    } else {
        1
    };

    // Build multi-span line with colored path
    let default_style = Style::default()
        .fg(colors.status_line_fg)
        .bg(colors.status_line_bg);

    let path_style = Style::default().fg(colors.key).bg(colors.status_line_bg);
    let register_style = Style::default()
        .fg(colors.warning)
        .bg(colors.status_line_bg);

    let mut spans = vec![Span::styled(mode_and_file, default_style)];

    if !path_display.is_empty() {
        spans.push(Span::styled(path_display, path_style));
    }

    if !dirty_indicator.is_empty() {
        spans.push(Span::styled(dirty_indicator, default_style));
    }

    if !register_info.is_empty() {
        spans.push(Span::styled(register_info, register_style));
    }

    if !search_info.is_empty() {
        spans.push(Span::styled(search_info, default_style));
    }

    spans.push(Span::styled(" ".repeat(padding), default_style));
    spans.push(Span::styled(position, default_style));

    let line = Line::from(spans);

    let status = Paragraph::new(line);

    f.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::{JsonNode, JsonValue};
    use crate::document::tree::JsonTree;
    use crate::editor::state::EditorState;
    use crate::theme;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_status_line_no_filename() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let state = EditorState::new_with_default_theme(tree);
        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        // Should display [No Name] when no filename is set
        let text: String = content.iter().take(80).map(|c| c.symbol()).collect();
        assert!(
            text.contains("[No Name]"),
            "Status line should show [No Name]: {}",
            text
        );
    }

    #[test]
    fn test_status_line_with_filename() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);
        state.set_filename("test.json".to_string());
        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        let text: String = content.iter().take(80).map(|c| c.symbol()).collect();
        assert!(
            text.contains("test.json"),
            "Status line should show filename: {}",
            text
        );
    }

    #[test]
    fn test_status_line_dirty_indicator() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);
        state.mark_dirty();
        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        let text: String = content.iter().take(80).map(|c| c.symbol()).collect();
        assert!(
            text.contains("[+]"),
            "Status line should show dirty indicator: {}",
            text
        );
    }

    #[test]
    fn test_status_line_clean_file() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);
        state.set_filename("clean.json".to_string());
        // Don't mark as dirty
        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        let text: String = content.iter().take(80).map(|c| c.symbol()).collect();
        assert!(
            !text.contains("[+]"),
            "Clean file should not show dirty indicator: {}",
            text
        );
    }

    #[test]
    fn test_status_line_different_modes() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let theme = theme::get_builtin_theme("default-dark").unwrap();

        // Test NORMAL mode
        let state = EditorState::new_with_default_theme(tree);
        terminal
            .draw(|f| {
                render_status_line(f, f.area(), &state, &theme.colors);
            })
            .unwrap();

        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .take(80)
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("NORMAL"), "Should show NORMAL mode: {}", text);
    }

    #[test]
    fn test_status_line_jsonpath_search_results() {
        let backend = TestBackend::new(120, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        // Create tree: {"users": [{"name": "Alice"}]}
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "users".to_string(),
            JsonNode::new(JsonValue::Array(vec![JsonNode::new(JsonValue::Object(
                vec![(
                    "name".to_string(),
                    JsonNode::new(JsonValue::String("Alice".to_string())),
                )],
            ))])),
        )])));

        let mut state = EditorState::new_with_default_theme(tree);
        state.set_filename("test.json".to_string());

        // Execute JSONPath search
        state.execute_jsonpath_search("$.users[0].name");

        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        let text: String = content.iter().take(120).map(|c| c.symbol()).collect();
        assert!(
            text.contains("JSONPath"),
            "Status line should show JSONPath search type: {}",
            text
        );
        assert!(
            text.contains("Match 1/1"),
            "Status line should show match count: {}",
            text
        );
    }

    #[test]
    fn test_status_line_shows_current_path() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        // Create tree: {"users": [{"name": "Alice"}]}
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "users".to_string(),
            JsonNode::new(JsonValue::Array(vec![JsonNode::new(JsonValue::Object(
                vec![(
                    "name".to_string(),
                    JsonNode::new(JsonValue::String("Alice".to_string())),
                )],
            ))])),
        )])));

        let mut state = EditorState::new_with_default_theme(tree);
        state.set_filename("test.json".to_string());

        // Navigate to users[0].name
        state.move_cursor_down(); // users key
        state.move_cursor_down(); // users[0]
        state.move_cursor_down(); // name key

        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        let text: String = content.iter().take(100).map(|c| c.symbol()).collect();
        assert!(
            text.contains("users[0].name"),
            "Status line should show current path: {}",
            text
        );
    }

    #[test]
    fn test_status_line_no_path_for_empty_object() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        // Create empty object - cursor will be at root since there are no children
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));

        let mut state = EditorState::new_with_default_theme(tree);
        state.set_filename("test.json".to_string());

        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        let text: String = content.iter().take(80).map(|c| c.symbol()).collect();

        // Should show filename but no path for empty object at root
        assert!(text.contains("test.json"), "Should show filename: {}", text);

        // Check that there's no path indicators after filename
        // Format should be: "NORMAL | test.json             1/1"
        // Not: "NORMAL | test.json key             1/1"
        let parts: Vec<&str> = text.split("test.json").collect();
        if parts.len() > 1 {
            let after_filename = parts[1].trim_start();
            // Should start with spaces (padding) or position counter, not a path
            assert!(
                !after_filename.starts_with(char::is_alphabetic),
                "Should not have path for empty object at root: {}",
                text
            );
        }
    }

    #[test]
    fn test_status_line_shows_top_level_path() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        // Create tree: {"users": {"name": "Alice"}}
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "users".to_string(),
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            )])),
        )])));

        let mut state = EditorState::new_with_default_theme(tree);
        state.set_filename("test.json".to_string());

        // Navigate to top-level "users" key (first move_down)
        state.move_cursor_down();

        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        let text: String = content.iter().take(100).map(|c| c.symbol()).collect();
        assert!(
            text.contains("users"),
            "Status line should show top-level key path: {}",
            text
        );
    }

    #[test]
    fn test_status_line_path_uses_key_color() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        // Create tree: {"users": [{"name": "Alice"}]}
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "users".to_string(),
            JsonNode::new(JsonValue::Array(vec![JsonNode::new(JsonValue::Object(
                vec![(
                    "name".to_string(),
                    JsonNode::new(JsonValue::String("Alice".to_string())),
                )],
            ))])),
        )])));

        let mut state = EditorState::new_with_default_theme(tree);
        state.set_filename("test.json".to_string());

        // Navigate to users[0].name (has a path)
        state.move_cursor_down(); // users key
        state.move_cursor_down(); // users[0]
        state.move_cursor_down(); // name key

        let theme = theme::get_builtin_theme("default-dark").unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_line(f, area, &state, &theme.colors);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();

        // Find the path segment in the buffer
        let mut found_path_with_key_color = false;
        let text: String = content.iter().take(100).map(|c| c.symbol()).collect();

        // Verify the path appears in the output
        assert!(
            text.contains("users[0].name"),
            "Status line should contain path: {}",
            text
        );

        // Check that the cells containing the path have the key color
        // The path "users[0].name" should be rendered with colors.key foreground
        for (i, cell) in content.iter().enumerate().take(100) {
            // Check if this cell is part of the path
            if text[..i.min(text.len())].ends_with("test.json") {
                // We're past the filename, check next segment for path
                let remaining = &text[i..];
                if remaining.starts_with(" users") || remaining.starts_with("users") {
                    // This should be the path segment with key color
                    if cell.fg == theme.colors.key {
                        found_path_with_key_color = true;
                        break;
                    }
                }
            }
        }

        assert!(
            found_path_with_key_color,
            "Path segment should use theme's key color"
        );
    }
}
