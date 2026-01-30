//! Keyboard event mapping and input event types.

use crate::editor::mode::EditorMode;
use termion::event::{Event, Key};

/// High-level input events abstracted from raw keyboard input.
///
/// These events represent user intentions (quit, move cursor, enter mode)
/// rather than specific key presses, allowing for mode-specific keybindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// User wants to quit the editor
    Quit,
    /// Move cursor down
    MoveDown,
    /// Move cursor up
    MoveUp,
    /// Move cursor left
    MoveLeft,
    /// Move cursor right
    MoveRight,
    /// Enter insert mode (from normal mode)
    EnterInsertMode,
    /// Enter command mode (from normal mode)
    EnterCommandMode,
    /// Enter search mode (from normal mode)
    EnterSearchMode,
    /// Enter reverse search mode (from normal mode)
    EnterReverseSearchMode,
    /// Exit current mode back to normal mode
    ExitMode,
    /// Delete current node
    Delete,
    /// Yank (copy) current node
    Yank,
    /// Paste from clipboard after cursor
    Paste,
    /// Paste from clipboard before cursor
    PasteBefore,
    /// Save and quit (ZZ)
    SaveAndQuit,
    /// Jump to next search result
    NextSearchResult,
    /// Toggle help overlay
    Help,
    /// Jump to top of document (gg)
    JumpToTop,
    /// Jump to bottom of document (G)
    JumpToBottom,
    /// Half-page down (Ctrl-d)
    HalfPageDown,
    /// Half-page up (Ctrl-u)
    HalfPageUp,
    /// Full-page down (Ctrl-f, PageDown key)
    FullPageDown,
    /// Full-page up (Ctrl-b, PageUp key)
    FullPageUp,
    /// Undo last change
    Undo,
    /// Redo last undone change
    Redo,
    /// Start add scalar operation
    Add,
    /// Add empty array
    AddArray,
    /// Add empty object
    AddObject,
    /// Rename object key
    Rename,
    /// Fully expand current subtree
    ExpandAll,
    /// Fully collapse current subtree
    CollapseAll,
    /// Jump to next sibling
    NextSibling,
    /// Jump to previous sibling
    PreviousSibling,
    /// Jump to first sibling
    FirstSibling,
    /// Jump to last sibling
    LastSibling,
    /// Search for current object key forward
    SearchKeyForward,
    /// Search for current object key backward
    SearchKeyBackward,
    /// Screen positioning command (z prefix)
    ScreenPosition,
    /// Move to next node at same or shallower depth (w)
    NextAtSameOrShallowerDepth,
    /// Move to previous node at same or shallower depth (b)
    PreviousAtSameOrShallowerDepth,
    /// Move to parent node without collapsing (H)
    MoveToParent,
    /// Register selection prefix (")
    RegisterSelect,
    /// Enter visual mode (v)
    EnterVisualMode,
    /// Set mark (m)
    MarkSet,
    /// Jump to mark (')
    MarkJump,
    /// Jump backward in jump list (Ctrl-o)
    JumpBackward,
    /// Jump forward in jump list (Ctrl-i)
    JumpForward,
    /// Repeat last command (.)
    Repeat,
    /// Insert a character in insert mode
    InsertCharacter(char),
    /// Backspace in insert mode
    InsertBackspace,
    /// Enter in insert mode
    InsertEnter,
    /// Unknown or unmapped key
    Unknown,
}

/// Maps a termion Event to an InputEvent based on the current editor mode.
///
/// Different modes interpret keys differently (vim-style modal editing):
/// - Normal mode: hjkl for movement, i for insert, : for command, q for quit
/// - Insert mode: Esc to exit
/// - Command mode: Esc to exit
///
/// # Arguments
///
/// * `event` - The termion Event to map
/// * `mode` - The current editor mode
///
/// # Returns
///
/// The corresponding InputEvent, or InputEvent::Unknown if not mapped
///
/// # Example
///
/// ```
/// use termion::event::{Event, Key};
/// use jsonquill::editor::mode::EditorMode;
/// use jsonquill::input::keys::{map_key_event, InputEvent};
///
/// let event = Event::Key(Key::Char('j'));
/// let input_event = map_key_event(event, &EditorMode::Normal);
/// assert_eq!(input_event, InputEvent::MoveDown);
/// ```
pub fn map_key_event(event: Event, mode: &EditorMode) -> InputEvent {
    // We only care about key events
    let key = match event {
        Event::Key(k) => k,
        _ => return InputEvent::Unknown,
    };

    match mode {
        EditorMode::Normal => match key {
            // Ctrl-modified keys
            Key::Ctrl('d') => InputEvent::HalfPageDown,
            Key::Ctrl('u') => InputEvent::HalfPageUp,
            Key::Ctrl('f') => InputEvent::FullPageDown,
            Key::Ctrl('b') => InputEvent::FullPageUp,
            Key::Ctrl('r') => InputEvent::Redo,
            Key::Ctrl('o') => InputEvent::JumpBackward,
            Key::Ctrl('i') => InputEvent::JumpForward,
            // Regular keys
            Key::Char('q') => InputEvent::Quit,
            Key::Char('j') => InputEvent::MoveDown,
            Key::Char('k') => InputEvent::MoveUp,
            Key::Char('h') => InputEvent::MoveLeft,
            Key::Char('l') => InputEvent::MoveRight,
            Key::Char(':') => InputEvent::EnterCommandMode,
            Key::Char('/') => InputEvent::EnterSearchMode,
            Key::Char('?') => InputEvent::EnterReverseSearchMode,
            Key::Char('n') => InputEvent::NextSearchResult,
            Key::Char('d') => InputEvent::Delete,
            Key::Char('y') => InputEvent::Yank,
            Key::Char('p') => InputEvent::Paste,
            Key::Char('P') => InputEvent::PasteBefore,
            Key::Char('Z') => InputEvent::SaveAndQuit,
            Key::Char('g') => InputEvent::JumpToTop,
            Key::Char('G') => InputEvent::JumpToBottom,
            Key::Char('u') => InputEvent::Undo,
            Key::Char('e') => InputEvent::EnterInsertMode,
            Key::Char('i') => InputEvent::Add,
            Key::Char('a') => InputEvent::AddArray,
            Key::Char('o') => InputEvent::AddObject,
            Key::Char('r') => InputEvent::Rename,
            Key::Char('E') => InputEvent::ExpandAll,
            Key::Char('C') => InputEvent::CollapseAll,
            Key::Char('H') => InputEvent::MoveToParent,
            Key::Char('z') => InputEvent::ScreenPosition,
            Key::Char('}') => InputEvent::NextSibling,
            Key::Char('{') => InputEvent::PreviousSibling,
            Key::Char('0') => InputEvent::FirstSibling,
            Key::Char('^') => InputEvent::FirstSibling,
            Key::Char('$') => InputEvent::LastSibling,
            Key::Char('*') => InputEvent::SearchKeyForward,
            Key::Char('#') => InputEvent::SearchKeyBackward,
            Key::Char('w') => InputEvent::NextAtSameOrShallowerDepth,
            Key::Char('b') => InputEvent::PreviousAtSameOrShallowerDepth,
            Key::Char('"') => InputEvent::RegisterSelect,
            Key::Char('v') | Key::Char('V') => InputEvent::EnterVisualMode,
            Key::Char('m') => InputEvent::MarkSet,
            Key::Char('\'') => InputEvent::MarkJump,
            Key::Char('.') => InputEvent::Repeat,
            Key::Down => InputEvent::MoveDown,
            Key::Up => InputEvent::MoveUp,
            Key::Left => InputEvent::MoveLeft,
            Key::Right => InputEvent::MoveRight,
            Key::PageDown => InputEvent::FullPageDown,
            Key::PageUp => InputEvent::FullPageUp,
            Key::Home => InputEvent::JumpToTop,
            Key::End => InputEvent::JumpToBottom,
            Key::F(1) => InputEvent::Help,
            _ => InputEvent::Unknown,
        },
        EditorMode::Insert => match key {
            Key::Esc => InputEvent::ExitMode,
            Key::Char('\n') => InputEvent::InsertEnter,
            Key::Backspace => InputEvent::InsertBackspace,
            Key::Char(c) => InputEvent::InsertCharacter(c),
            _ => InputEvent::Unknown,
        },
        EditorMode::Command => match key {
            Key::Esc => InputEvent::ExitMode,
            _ => InputEvent::Unknown,
        },
        EditorMode::Search => match key {
            Key::Esc => InputEvent::ExitMode,
            _ => InputEvent::Unknown,
        },
        EditorMode::Visual => match key {
            Key::Esc => InputEvent::ExitMode,
            Key::Char('j') => InputEvent::MoveDown,
            Key::Char('k') => InputEvent::MoveUp,
            Key::Char('h') => InputEvent::MoveLeft,
            Key::Char('l') => InputEvent::MoveRight,
            Key::Char('d') => InputEvent::Delete,
            Key::Char('y') => InputEvent::Yank,
            Key::Char('p') => InputEvent::Paste,
            Key::Char('P') => InputEvent::PasteBefore,
            Key::Down => InputEvent::MoveDown,
            Key::Up => InputEvent::MoveUp,
            Key::Left => InputEvent::MoveLeft,
            Key::Right => InputEvent::MoveRight,
            _ => InputEvent::Unknown,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_quit() {
        let event = Event::Key(Key::Char('q'));
        assert_eq!(map_key_event(event, &EditorMode::Normal), InputEvent::Quit);
    }

    #[test]
    fn test_normal_mode_movement_vim_keys() {
        assert_eq!(
            map_key_event(Event::Key(Key::Char('j')), &EditorMode::Normal),
            InputEvent::MoveDown
        );
        assert_eq!(
            map_key_event(Event::Key(Key::Char('k')), &EditorMode::Normal),
            InputEvent::MoveUp
        );
        assert_eq!(
            map_key_event(Event::Key(Key::Char('h')), &EditorMode::Normal),
            InputEvent::MoveLeft
        );
        assert_eq!(
            map_key_event(Event::Key(Key::Char('l')), &EditorMode::Normal),
            InputEvent::MoveRight
        );
    }

    #[test]
    fn test_normal_mode_movement_arrow_keys() {
        assert_eq!(
            map_key_event(Event::Key(Key::Down), &EditorMode::Normal),
            InputEvent::MoveDown
        );
        assert_eq!(
            map_key_event(Event::Key(Key::Up), &EditorMode::Normal),
            InputEvent::MoveUp
        );
    }

    #[test]
    fn test_normal_mode_enter_modes() {
        assert_eq!(
            map_key_event(Event::Key(Key::Char('e')), &EditorMode::Normal),
            InputEvent::EnterInsertMode
        );
        assert_eq!(
            map_key_event(Event::Key(Key::Char(':')), &EditorMode::Normal),
            InputEvent::EnterCommandMode
        );
    }

    #[test]
    fn test_insert_mode_exit() {
        assert_eq!(
            map_key_event(Event::Key(Key::Esc), &EditorMode::Insert),
            InputEvent::ExitMode
        );
    }

    #[test]
    fn test_command_mode_exit() {
        assert_eq!(
            map_key_event(Event::Key(Key::Esc), &EditorMode::Command),
            InputEvent::ExitMode
        );
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(
            map_key_event(Event::Key(Key::Char('x')), &EditorMode::Normal),
            InputEvent::Unknown
        );
    }
}
