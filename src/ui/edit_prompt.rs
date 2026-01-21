use ratatui::{
    layout::Rect,
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::theme::colors::ThemeColors;

/// Renders the edit prompt showing the current edit buffer content.
pub fn render_edit_prompt(
    f: &mut Frame,
    area: Rect,
    buffer: &str,
    colors: &ThemeColors,
) {
    let prompt_text = format!("Edit: {}", buffer);

    let line = Line::from(vec![
        Span::styled(
            prompt_text,
            Style::default()
                .fg(colors.foreground)
                .bg(colors.background)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "█",
            Style::default()
                .fg(colors.cursor)
                .bg(colors.background),
        ),
    ]);

    let prompt = Paragraph::new(line)
        .style(Style::default().bg(colors.background));

    f.render_widget(prompt, area);
}
