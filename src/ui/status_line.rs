//! Status line widget for displaying editor state information.
//!
//! The status line shows:
//! - Current mode (NORMAL, INSERT, COMMAND)
//! - Filename (or "[No Name]" if unsaved)
//! - Dirty indicator "[+]" for unsaved changes
//!
//! Example status line: `NORMAL | data.json [+]`

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
/// let state = EditorState::new(jsonquill::document::tree::JsonTree::new(
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

    let left = format!("{} | {}{}", mode_text, filename, dirty_indicator);

    // Get cursor position
    let (row, col) = state.cursor_position();
    let total = state.total_lines();
    let right = format!("{},{} {}/{}", row, col, row, total);

    // Calculate padding to position right-aligned text
    let total_width = area.width as usize;
    let right_len = right.len();
    let left_len = left.len();

    // Ensure we don't overflow
    let padding = if left_len + right_len + 1 < total_width {
        total_width - left_len - right_len
    } else {
        1
    };

    let content = format!("{}{}{}", left, " ".repeat(padding), right);

    let line = Line::from(Span::styled(
        content,
        Style::default()
            .fg(colors.status_line_fg)
            .bg(colors.status_line_bg),
    ));

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
        let state = EditorState::new(tree);
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
        let mut state = EditorState::new(tree);
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
        let mut state = EditorState::new(tree);
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
        let mut state = EditorState::new(tree);
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
        let state = EditorState::new(tree);
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
}
