//! Message area rendering for displaying messages and command input.

use crate::editor::mode::EditorMode;
use crate::editor::state::{EditorState, MessageLevel};
use crate::theme::colors::ThemeColors;
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
/// - Command mode: `:` prompt with input buffer
/// - Messages: errors, warnings, info
/// - Empty when no message
pub fn render_message_area(f: &mut Frame, area: Rect, state: &EditorState, colors: &ThemeColors) {
    let content = match state.mode() {
        EditorMode::Command => {
            // Show command prompt with buffer
            let text = format!(":{}", state.command_buffer());
            Line::from(vec![Span::styled(
                text,
                Style::default().fg(colors.foreground),
            )])
        }
        EditorMode::Search => {
            // Show search prompt with buffer and results
            let mut text = format!("/{}", state.search_buffer());
            if let Some((current, total)) = state.search_results_info() {
                text.push_str(&format!(" ({}/{})", current, total));
            }
            Line::from(vec![Span::styled(text, Style::default().fg(colors.info))])
        }
        _ => {
            // Show message if present
            if let Some(message) = state.message() {
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
            }
        }
    };

    let paragraph =
        Paragraph::new(content).style(Style::default().bg(colors.background).fg(colors.foreground));

    f.render_widget(paragraph, area);
}
