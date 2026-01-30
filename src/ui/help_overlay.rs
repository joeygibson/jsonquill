//! Help overlay for displaying keybindings and commands.

use crate::theme::colors::ThemeColors;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

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
        .title(" JSONQuill Help ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.info))
        .style(Style::default().bg(colors.background));

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  j/k           ", Style::default().fg(colors.number)),
            Span::raw("Move cursor down/up (prefix with count)"),
        ]),
        Line::from(vec![
            Span::styled("  h/l           ", Style::default().fg(colors.number)),
            Span::raw("Collapse/expand node"),
        ]),
        Line::from(vec![
            Span::styled("  E             ", Style::default().fg(colors.number)),
            Span::raw("Fully expand current subtree"),
        ]),
        Line::from(vec![
            Span::styled("  C             ", Style::default().fg(colors.number)),
            Span::raw("Fully collapse current subtree"),
        ]),
        Line::from(vec![
            Span::styled("  H             ", Style::default().fg(colors.number)),
            Span::raw("Move to parent node (without collapsing)"),
        ]),
        Line::from(vec![
            Span::styled("  gg / Home     ", Style::default().fg(colors.number)),
            Span::raw("Jump to top of document"),
        ]),
        Line::from(vec![
            Span::styled("  G / End       ", Style::default().fg(colors.number)),
            Span::raw("Jump to bottom of document"),
        ]),
        Line::from(vec![
            Span::styled("  <count>G/gg   ", Style::default().fg(colors.number)),
            Span::raw("Jump to line <count> (e.g., 5G or 5gg)"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-d        ", Style::default().fg(colors.number)),
            Span::raw("Half-page down"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-u        ", Style::default().fg(colors.number)),
            Span::raw("Half-page up"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-f / PgDn ", Style::default().fg(colors.number)),
            Span::raw("Full-page down"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-b / PgUp ", Style::default().fg(colors.number)),
            Span::raw("Full-page up"),
        ]),
        Line::from(vec![
            Span::styled("  zz            ", Style::default().fg(colors.number)),
            Span::raw("Center cursor on screen"),
        ]),
        Line::from(vec![
            Span::styled("  zt            ", Style::default().fg(colors.number)),
            Span::raw("Move cursor to top of screen"),
        ]),
        Line::from(vec![
            Span::styled("  zb            ", Style::default().fg(colors.number)),
            Span::raw("Move cursor to bottom of screen"),
        ]),
        Line::from(vec![
            Span::styled("  }             ", Style::default().fg(colors.number)),
            Span::raw("Jump to next sibling"),
        ]),
        Line::from(vec![
            Span::styled("  {             ", Style::default().fg(colors.number)),
            Span::raw("Jump to previous sibling"),
        ]),
        Line::from(vec![
            Span::styled("  0 or ^        ", Style::default().fg(colors.number)),
            Span::raw("Jump to first sibling"),
        ]),
        Line::from(vec![
            Span::styled("  $             ", Style::default().fg(colors.number)),
            Span::raw("Jump to last sibling"),
        ]),
        Line::from(vec![
            Span::styled("  w             ", Style::default().fg(colors.number)),
            Span::raw("Next node at same or shallower depth"),
        ]),
        Line::from(vec![
            Span::styled("  b             ", Style::default().fg(colors.number)),
            Span::raw("Previous node at same or shallower depth"),
        ]),
        Line::from(vec![
            Span::styled("  Arrow keys    ", Style::default().fg(colors.number)),
            Span::raw("Also work for navigation"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Mouse",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Scroll wheel  ", Style::default().fg(colors.number)),
            Span::raw("Scroll viewport (3 lines per tick)"),
        ]),
        Line::from(vec![
            Span::styled("  Trackpad      ", Style::default().fg(colors.number)),
            Span::raw("Scroll viewport smoothly"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Modes",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  e             ", Style::default().fg(colors.number)),
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
        Line::from(vec![Span::styled(
            "Editing (NORMAL mode)",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  dd            ", Style::default().fg(colors.number)),
            Span::raw("Delete current node (prefix with count)"),
        ]),
        Line::from(vec![
            Span::styled("  yy            ", Style::default().fg(colors.number)),
            Span::raw("Yank (copy) current node (prefix with count)"),
        ]),
        Line::from(vec![
            Span::styled("  yp            ", Style::default().fg(colors.number)),
            Span::raw("Yank path in dot notation (.foo[3].bar)"),
        ]),
        Line::from(vec![
            Span::styled("  yb            ", Style::default().fg(colors.number)),
            Span::raw("Yank path in bracket notation ([\"foo\"][3][\"bar\"])"),
        ]),
        Line::from(vec![
            Span::styled("  yq            ", Style::default().fg(colors.number)),
            Span::raw("Yank path in jq style"),
        ]),
        Line::from(vec![
            Span::styled("  p/P           ", Style::default().fg(colors.number)),
            Span::raw("Paste after/before cursor"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Registers",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  \"a            ", Style::default().fg(colors.number)),
            Span::raw("Select register 'a' for next yank/paste/delete"),
        ]),
        Line::from(vec![
            Span::styled("  \"A            ", Style::default().fg(colors.number)),
            Span::raw("Select register 'a' (append mode)"),
        ]),
        Line::from(vec![
            Span::styled("  \"5            ", Style::default().fg(colors.number)),
            Span::raw("Select numbered register 5"),
        ]),
        Line::from(vec![
            Span::styled("  \"ayy          ", Style::default().fg(colors.number)),
            Span::raw("Yank to register 'a'"),
        ]),
        Line::from(vec![
            Span::styled("  \"ap           ", Style::default().fg(colors.number)),
            Span::raw("Paste from register 'a'"),
        ]),
        Line::from(vec![
            Span::styled("  \"0p           ", Style::default().fg(colors.number)),
            Span::raw("Paste from last yank"),
        ]),
        Line::from(vec![
            Span::styled("  \"1p           ", Style::default().fg(colors.number)),
            Span::raw("Paste from last delete"),
        ]),
        Line::from(vec![
            Span::styled("  yy / dd       ", Style::default().fg(colors.number)),
            Span::raw("Yank/delete to unnamed register (syncs clipboard)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Adding Nodes",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  i             ", Style::default().fg(colors.number)),
            Span::raw("Add scalar (inside containers, after scalars)"),
        ]),
        Line::from(vec![
            Span::styled("  a             ", Style::default().fg(colors.number)),
            Span::raw("Add empty array [] after cursor"),
        ]),
        Line::from(vec![
            Span::styled("  o             ", Style::default().fg(colors.number)),
            Span::raw("Add empty object {} after cursor"),
        ]),
        Line::from(vec![
            Span::styled("  r             ", Style::default().fg(colors.number)),
            Span::raw("Rename object key (objects only)"),
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
        Line::from(vec![Span::styled(
            "Visual Mode",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  v             ", Style::default().fg(colors.number)),
            Span::raw("Enter visual mode (select multiple nodes)"),
        ]),
        Line::from(vec![
            Span::styled("  j/k/h/l       ", Style::default().fg(colors.number)),
            Span::raw("Expand/shrink selection (in visual mode)"),
        ]),
        Line::from(vec![
            Span::styled("  d             ", Style::default().fg(colors.number)),
            Span::raw("Delete selection (in visual mode)"),
        ]),
        Line::from(vec![
            Span::styled("  y             ", Style::default().fg(colors.number)),
            Span::raw("Yank (copy) selection (in visual mode)"),
        ]),
        Line::from(vec![
            Span::styled("  p/P           ", Style::default().fg(colors.number)),
            Span::raw("Replace selection with clipboard (in visual mode)"),
        ]),
        Line::from(vec![
            Span::styled("  Esc           ", Style::default().fg(colors.number)),
            Span::raw("Exit visual mode"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Marks & Jump List",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  m{a-z}        ", Style::default().fg(colors.number)),
            Span::raw("Set mark at current position"),
        ]),
        Line::from(vec![
            Span::styled("  '{a-z}        ", Style::default().fg(colors.number)),
            Span::raw("Jump to mark"),
        ]),
        Line::from(vec![
            Span::styled("  y'{a-z}       ", Style::default().fg(colors.number)),
            Span::raw("Yank from cursor to mark"),
        ]),
        Line::from(vec![
            Span::styled("  d'{a-z}       ", Style::default().fg(colors.number)),
            Span::raw("Delete from cursor to mark"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-o        ", Style::default().fg(colors.number)),
            Span::raw("Jump backward in jump list"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-i        ", Style::default().fg(colors.number)),
            Span::raw("Jump forward in jump list"),
        ]),
        Line::from(vec![Span::raw(
            "  Jump list records: gg, G, line jumps, search, marks",
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Repeat Command",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  .             ", Style::default().fg(colors.number)),
            Span::raw("Repeat last edit (dd, yy, p, P)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Search",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  /             ", Style::default().fg(colors.number)),
            Span::raw("Search forward (smart case, shows W when wrapping)"),
        ]),
        Line::from(vec![
            Span::styled("  ?             ", Style::default().fg(colors.number)),
            Span::raw("Search backward in keys and values"),
        ]),
        Line::from(vec![
            Span::styled("  n             ", Style::default().fg(colors.number)),
            Span::raw("Jump to next match (shows current/total)"),
        ]),
        Line::from(vec![
            Span::styled("  *             ", Style::default().fg(colors.number)),
            Span::raw("Search forward for current object key"),
        ]),
        Line::from(vec![
            Span::styled("  #             ", Style::default().fg(colors.number)),
            Span::raw("Search backward for current object key"),
        ]),
        Line::from(vec![
            Span::styled("  :find         ", Style::default().fg(colors.number)),
            Span::raw("Enter text search mode (same as /)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "JSONPath Search (Structural)",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  :path <query> ", Style::default().fg(colors.number)),
            Span::raw("JSONPath search (e.g., :path $.users[*].name)"),
        ]),
        Line::from(vec![
            Span::styled("  :jp <query>   ", Style::default().fg(colors.number)),
            Span::raw("Short alias for :path"),
        ]),
        Line::from(vec![Span::raw(
            "  Supported: $, .prop, [index], [*], .., [start:end]",
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Commands",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "  :w                    ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Write (save) file"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :q                    ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Quit (warns if unsaved)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :q!                   ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Quit without saving"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :wq / :x / ZZ         ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Save and quit"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :e <file>             ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Load a different file (warns if dirty)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :e!                   ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Reload current file, discarding changes"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :e! <file>            ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Load a different file, discarding changes"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :theme                ",
                Style::default().fg(colors.number),
            ),
            Span::raw("List/change themes"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set                  ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Show settings"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set number           ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Enable line numbers"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set nonumber         ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Disable line numbers"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set rnu              ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Enable relative line numbers"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set nornu            ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Disable relative line numbers"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set mouse            ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Enable mouse scrolling"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set nomouse          ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Disable mouse scrolling"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set create_backup    ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Enable backup file creation (.bak)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set nocreate_backup  ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Disable backup file creation"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :set save             ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Save settings to config file"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :undo                 ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Undo last change"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :redo                 ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Redo last undone change"),
        ]),
        Line::from(vec![
            Span::styled(
                "  :format               ",
                Style::default().fg(colors.number),
            ),
            Span::raw("Reformat document with jq-style indentation"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Other",
            Style::default().fg(colors.key).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  q             ", Style::default().fg(colors.number)),
            Span::raw("Quit (NORMAL mode only)"),
        ]),
        Line::from(vec![
            Span::styled("  F1 / :help    ", Style::default().fg(colors.number)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "↑/↓ or j/k or mouse wheel to scroll • F1 or Esc to close",
            Style::default()
                .fg(colors.info)
                .add_modifier(Modifier::ITALIC),
        )]),
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
