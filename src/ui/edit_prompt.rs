use ratatui::{
    layout::Rect,
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::theme::colors::ThemeColors;

/// Renders the edit prompt showing the current edit buffer content with cursor.
pub fn render_edit_prompt(
    f: &mut Frame,
    area: Rect,
    buffer: &str,
    cursor_pos: usize,
    cursor_visible: bool,
    colors: &ThemeColors,
) {
    // Split buffer at cursor position
    let cursor_pos = cursor_pos.min(buffer.len());
    let (before, after) = buffer.split_at(cursor_pos);

    // Build the line with cursor in the correct position
    let mut spans = vec![
        Span::styled(
            "Edit: ",
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

    // Add cursor (blinking)
    if cursor_visible {
        spans.push(Span::styled(
            "█",
            Style::default()
                .fg(colors.cursor)
                .bg(colors.background),
        ));
    } else {
        spans.push(Span::styled(
            " ",
            Style::default()
                .fg(colors.cursor)
                .bg(colors.background),
        ));
    }

    // Add text after cursor
    spans.push(Span::styled(
        after,
        Style::default()
            .fg(colors.foreground)
            .bg(colors.background)
            .add_modifier(Modifier::BOLD),
    ));

    let line = Line::from(spans);
    let prompt = Paragraph::new(line)
        .style(Style::default().bg(colors.background));

    f.render_widget(prompt, area);
}
