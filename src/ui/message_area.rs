//! Message area rendering for displaying messages and command input.

use crate::editor::mode::EditorMode;
use crate::editor::state::{EditorState, MessageLevel};
use crate::theme::colors::ThemeColors;
use crate::ui::edit_prompt::render_edit_prompt;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Renders the message area at the bottom of the screen.
///
/// Displays:
/// - Command mode: `:` prompt with input buffer and cursor
/// - Search mode: `/` or `?` prompt with input buffer and cursor
/// - Messages: errors, warnings, info
/// - Empty when no message
pub fn render_message_area(f: &mut Frame, area: Rect, state: &EditorState, colors: &ThemeColors) {
    match state.mode() {
        EditorMode::Command => {
            render_edit_prompt(
                f,
                area,
                state.command_buffer(),
                state.command_cursor_position(),
                state.cursor_visible(),
                colors,
                ":",
            );
            return;
        }
        EditorMode::Search => {
            let prompt = if state.search_forward() { "/" } else { "?" };
            render_edit_prompt(
                f,
                area,
                state.search_buffer(),
                state.search_cursor_position(),
                state.cursor_visible(),
                colors,
                prompt,
            );
            return;
        }
        _ => {}
    }

    let content = if let Some(message) = state.message() {
        let color = match message.level {
            MessageLevel::Error => colors.error,
            MessageLevel::Warning => colors.warning,
            MessageLevel::Info => colors.info,
        };
        Line::from(vec![Span::styled(
            &message.text,
            Style::default().fg(color),
        )])
    } else {
        Line::from("")
    };

    let paragraph =
        Paragraph::new(content).style(Style::default().bg(colors.background).fg(colors.foreground));

    f.render_widget(paragraph, area);
}
