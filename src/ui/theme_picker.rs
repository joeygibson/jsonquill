//! Interactive theme picker popup with live preview.

use crate::editor::state::ThemePickerState;
use crate::theme::colors::ThemeColors;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Renders the interactive theme picker popup.
///
/// Displays a centered overlay showing a list of available themes with:
/// - `>` cursor indicator on the selected theme
/// - `(current)` label on the currently active theme
/// - Highlighted background for the selected theme
/// - Footer with keyboard shortcuts
///
/// # Arguments
///
/// * `f` - The ratatui Frame to render into
/// * `state` - The theme picker state containing theme list and selection
/// * `colors` - Theme colors for styling the picker
pub fn render_theme_picker(f: &mut Frame, state: &ThemePickerState, colors: &ThemeColors) {
    // Calculate centered area (40% width, fit content height)
    let area = centered_rect(40, 60, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    // Create border block
    let block = Block::default()
        .title(" Select Theme ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.info))
        .style(Style::default().bg(colors.background));

    // Build theme list lines
    let mut lines = vec![Line::from("")]; // Top padding

    for (idx, theme) in state.themes.iter().enumerate() {
        let is_selected = idx == state.selected_index;
        let is_current = theme == &state.original_theme;

        // Build line components
        let cursor = if is_selected { "> " } else { "  " };
        let label = if is_current { " (current)" } else { "" };
        let text = format!("{}{}{}", cursor, theme, label);

        // Apply styling
        let style = if is_selected {
            Style::default()
                .fg(ratatui::style::Color::White)
                .bg(colors.cursor)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        lines.push(Line::from(Span::styled(text, style)));
    }

    // Bottom padding and footer
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓: Navigate  Enter: Apply  Esc: Cancel",
        Style::default()
            .fg(colors.info)
            .add_modifier(Modifier::ITALIC),
    )));

    // Render the paragraph
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Helper function to create a centered rectangle.
///
/// Creates a rectangle that is centered in the given area with the specified
/// percentage width and height.
///
/// # Arguments
///
/// * `percent_x` - Percentage width (0-100)
/// * `percent_y` - Percentage height (0-100)
/// * `r` - The parent rectangle to center within
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
