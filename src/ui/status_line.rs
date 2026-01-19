//! Status line widget for displaying editor state information.
//!
//! The status line shows:
//! - Current mode (NORMAL, INSERT, COMMAND)
//! - Filename (or "[No Name]" if unsaved)
//! - Dirty indicator "[+]" for unsaved changes
//!
//! Example status line: `NORMAL | data.json [+]`

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::editor::state::EditorState;
use crate::theme::colors::ThemeColors;

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
/// use jeditor::editor::state::EditorState;
/// use jeditor::theme;
///
/// # fn example(f: &mut Frame, area: Rect) {
/// let state = EditorState::new(jeditor::document::tree::JsonTree::new(
///     jeditor::document::node::JsonNode::new(
///         jeditor::document::node::JsonValue::Null
///     )
/// ));
/// let theme = theme::get_builtin_theme("default-dark").unwrap();
/// jeditor::ui::status_line::render_status_line(f, area, &state, &theme.colors);
/// # }
/// ```
pub fn render_status_line(
    f: &mut Frame,
    area: Rect,
    state: &EditorState,
    colors: &ThemeColors,
) {
    let mode_text = format!("{}", state.mode());
    let filename = state.filename().unwrap_or("[No Name]");
    let dirty_indicator = if state.is_dirty() { " [+]" } else { "" };

    let left = format!("{} | {}{}", mode_text, filename, dirty_indicator);

    let line = Line::from(vec![
        Span::styled(
            left,
            Style::default()
                .fg(colors.status_line_fg)
                .bg(colors.status_line_bg),
        ),
    ]);

    let status = Paragraph::new(line)
        .style(Style::default().bg(colors.status_line_bg));

    f.render_widget(status, area);
}