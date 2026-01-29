use crate::theme::colors::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Renders the edit prompt showing the current edit buffer content with cursor.
pub fn render_edit_prompt(
    f: &mut Frame,
    area: Rect,
    buffer: &str,
    cursor_pos: usize,
    cursor_visible: bool,
    colors: &ThemeColors,
    prompt: &str,
) {
    let cursor_pos = cursor_pos.min(buffer.len());

    // Split buffer into: text before cursor, char at cursor, text after cursor
    let chars: Vec<char> = buffer.chars().collect();
    let before: String = chars.iter().take(cursor_pos).collect();
    let after: String = chars.iter().skip(cursor_pos + 1).collect();

    // Get character at cursor position (or space if at end)
    let char_at_cursor = chars.get(cursor_pos).copied().unwrap_or(' ');

    // Build the line with cursor highlighting the character at cursor position
    let mut spans = vec![
        Span::styled(
            prompt,
            Style::default()
                .fg(colors.foreground)
                .bg(colors.background)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            before,
            Style::default()
                .fg(colors.foreground)
                .bg(colors.background)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    // Add character at cursor position with inverted colors (block cursor effect)
    if cursor_visible {
        spans.push(Span::styled(
            char_at_cursor.to_string(),
            Style::default()
                .fg(colors.background)
                .bg(colors.cursor)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        // When cursor is not visible, show the character normally
        spans.push(Span::styled(
            char_at_cursor.to_string(),
            Style::default()
                .fg(colors.foreground)
                .bg(colors.background)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Add text after cursor
    if !after.is_empty() {
        spans.push(Span::styled(
            after,
            Style::default()
                .fg(colors.foreground)
                .bg(colors.background)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let line = Line::from(spans);
    let prompt = Paragraph::new(line).style(Style::default().bg(colors.background));

    f.render_widget(prompt, area);
}
