//! Help overlay for displaying keybindings and commands.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::theme::colors::ThemeColors;

/// Renders a centered help overlay showing keybindings and commands.
///
/// Displays:
/// - Navigation keybindings
/// - Editing operations
/// - Command mode commands
/// - Instructions to close (press ? or Esc)
pub fn render_help_overlay(f: &mut Frame, colors: &ThemeColors, scroll: usize) {
    let area = centered_rect(80, 85, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" jeditor Help ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.info))
        .style(Style::default().bg(colors.background));

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(colors.key).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  j/k           ", Style::default().fg(colors.number)),
            Span::raw("Move cursor down/up (prefix with count)"),
        ]),
        Line::from(vec![
            Span::styled("  h/l           ", Style::default().fg(colors.number)),
            Span::raw("Collapse/expand node"),
        ]),
        Line::from(vec![
            Span::styled("  gg            ", Style::default().fg(colors.number)),
            Span::raw("Jump to top of document"),
        ]),
        Line::from(vec![
            Span::styled("  G             ", Style::default().fg(colors.number)),
            Span::raw("Jump to bottom of document"),
        ]),
        Line::from(vec![
            Span::styled("  <count>g      ", Style::default().fg(colors.number)),
            Span::raw("Jump to line <count> (e.g., 5g)"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-d        ", Style::default().fg(colors.number)),
            Span::raw("Page down"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-u        ", Style::default().fg(colors.number)),
            Span::raw("Page up"),
        ]),
        Line::from(vec![
            Span::styled("  Arrow keys    ", Style::default().fg(colors.number)),
            Span::raw("Also work for navigation"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Modes", Style::default().fg(colors.key).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  i             ", Style::default().fg(colors.number)),
            Span::raw("Enter INSERT mode (edit values/keys)"),
        ]),
        Line::from(vec![
            Span::styled("  :             ", Style::default().fg(colors.number)),
            Span::raw("Enter COMMAND mode"),
        ]),
        Line::from(vec![
            Span::styled("  Esc           ", Style::default().fg(colors.number)),
            Span::raw("Return to NORMAL mode"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Editing (NORMAL mode)", Style::default().fg(colors.key).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  dd            ", Style::default().fg(colors.number)),
            Span::raw("Delete current node (prefix with count)"),
        ]),
        Line::from(vec![
            Span::styled("  yy            ", Style::default().fg(colors.number)),
            Span::raw("Yank (copy) current node (prefix with count)"),
        ]),
        Line::from(vec![
            Span::styled("  p/P           ", Style::default().fg(colors.number)),
            Span::raw("Paste after/before cursor"),
        ]),
        Line::from(vec![
            Span::styled("  a             ", Style::default().fg(colors.number)),
            Span::raw("Add new field/element after cursor"),
        ]),
        Line::from(vec![
            Span::styled("  u             ", Style::default().fg(colors.number)),
            Span::raw("Undo last change"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-r        ", Style::default().fg(colors.number)),
            Span::raw("Redo last undone change"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Search", Style::default().fg(colors.key).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  /             ", Style::default().fg(colors.number)),
            Span::raw("Search in keys and values"),
        ]),
        Line::from(vec![
            Span::styled("  n             ", Style::default().fg(colors.number)),
            Span::raw("Jump to next search result"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Commands", Style::default().fg(colors.key).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  :w            ", Style::default().fg(colors.number)),
            Span::raw("Write (save) file"),
        ]),
        Line::from(vec![
            Span::styled("  :q            ", Style::default().fg(colors.number)),
            Span::raw("Quit (warns if unsaved)"),
        ]),
        Line::from(vec![
            Span::styled("  :q!           ", Style::default().fg(colors.number)),
            Span::raw("Quit without saving"),
        ]),
        Line::from(vec![
            Span::styled("  :wq / :x / ZZ ", Style::default().fg(colors.number)),
            Span::raw("Save and quit"),
        ]),
        Line::from(vec![
            Span::styled("  :theme        ", Style::default().fg(colors.number)),
            Span::raw("List/change themes"),
        ]),
        Line::from(vec![
            Span::styled("  :set          ", Style::default().fg(colors.number)),
            Span::raw("Show settings"),
        ]),
        Line::from(vec![
            Span::styled("  :set number   ", Style::default().fg(colors.number)),
            Span::raw("Enable line numbers"),
        ]),
        Line::from(vec![
            Span::styled("  :set nonumber ", Style::default().fg(colors.number)),
            Span::raw("Disable line numbers"),
        ]),
        Line::from(vec![
            Span::styled("  :set save     ", Style::default().fg(colors.number)),
            Span::raw("Save settings to config file"),
        ]),
        Line::from(vec![
            Span::styled("  :undo         ", Style::default().fg(colors.number)),
            Span::raw("Undo last change"),
        ]),
        Line::from(vec![
            Span::styled("  :redo         ", Style::default().fg(colors.number)),
            Span::raw("Redo last undone change"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Other", Style::default().fg(colors.key).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  q             ", Style::default().fg(colors.number)),
            Span::raw("Quit (NORMAL mode only)"),
        ]),
        Line::from(vec![
            Span::styled("  ?             ", Style::default().fg(colors.number)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓ or j/k to scroll • ? or Esc to close", Style::default().fg(colors.info).add_modifier(Modifier::ITALIC)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Helper function to create a centered rect using up certain percentage of the available rect
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
