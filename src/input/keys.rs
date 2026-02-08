//! Keyboard event mapping and input event types.

use crate::editor::mode::EditorMode;
use crate::input::backend::{BackendEvent, BackendKey};

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

/// Maps a BackendEvent to an InputEvent based on the current editor mode.
///
/// Different modes interpret keys differently (vim-style modal editing):
/// - Normal mode: hjkl for movement, i for insert, : for command, q for quit
/// - Insert mode: Esc to exit
/// - Command mode: Esc to exit
///
/// # Arguments
///
/// * `event` - The BackendEvent to map
/// * `mode` - The current editor mode
///
/// # Returns
///
/// The corresponding InputEvent, or InputEvent::Unknown if not mapped
///
/// # Example
///
/// ```
/// use jsonquill::input::backend::{BackendEvent, BackendKey};
/// use jsonquill::editor::mode::EditorMode;
/// use jsonquill::input::keys::{map_key_event, InputEvent};
///
/// let event = BackendEvent::Key(BackendKey::Char('j'));
/// let input_event = map_key_event(event, &EditorMode::Normal);
/// assert_eq!(input_event, InputEvent::MoveDown);
/// ```
pub fn map_key_event(event: BackendEvent, mode: &EditorMode) -> InputEvent {
    // We only care about key events
    let key = match event {
        BackendEvent::Key(k) => k,
        _ => return InputEvent::Unknown,
    };

    match mode {
        EditorMode::Normal => match key {
            // Ctrl-modified keys
            BackendKey::Ctrl('d') => InputEvent::HalfPageDown,
            BackendKey::Ctrl('u') => InputEvent::HalfPageUp,
            BackendKey::Ctrl('f') => InputEvent::FullPageDown,
            BackendKey::Ctrl('b') => InputEvent::FullPageUp,
            BackendKey::Ctrl('r') => InputEvent::Redo,
            BackendKey::Ctrl('o') => InputEvent::JumpBackward,
            BackendKey::Ctrl('i') => InputEvent::JumpForward,
            // Regular keys
            BackendKey::Char('q') => InputEvent::Quit,
            BackendKey::Char('j') => InputEvent::MoveDown,
            BackendKey::Char('k') => InputEvent::MoveUp,
            BackendKey::Char('h') => InputEvent::MoveLeft,
            BackendKey::Char('l') => InputEvent::MoveRight,
            BackendKey::Char(':') => InputEvent::EnterCommandMode,
            BackendKey::Char('/') => InputEvent::EnterSearchMode,
            BackendKey::Char('?') => InputEvent::EnterReverseSearchMode,
            BackendKey::Char('n') => InputEvent::NextSearchResult,
            BackendKey::Char('d') => InputEvent::Delete,
            BackendKey::Char('y') => InputEvent::Yank,
            BackendKey::Char('p') => InputEvent::Paste,
            BackendKey::Char('P') => InputEvent::PasteBefore,
            BackendKey::Char('Z') => InputEvent::SaveAndQuit,
            BackendKey::Char('g') => InputEvent::JumpToTop,
            BackendKey::Char('G') => InputEvent::JumpToBottom,
            BackendKey::Char('u') => InputEvent::Undo,
            BackendKey::Char('e') => InputEvent::EnterInsertMode,
            BackendKey::Char('i') => InputEvent::Add,
            BackendKey::Char('a') => InputEvent::AddArray,
            BackendKey::Char('o') => InputEvent::AddObject,
            BackendKey::Char('r') => InputEvent::Rename,
            BackendKey::Char('E') => InputEvent::ExpandAll,
            BackendKey::Char('C') => InputEvent::CollapseAll,
            BackendKey::Char('H') => InputEvent::MoveToParent,
            BackendKey::Char('z') => InputEvent::ScreenPosition,
            BackendKey::Char('}') => InputEvent::NextSibling,
            BackendKey::Char('{') => InputEvent::PreviousSibling,
            BackendKey::Char('0') => InputEvent::FirstSibling,
            BackendKey::Char('^') => InputEvent::FirstSibling,
            BackendKey::Char('$') => InputEvent::LastSibling,
            BackendKey::Char('*') => InputEvent::SearchKeyForward,
            BackendKey::Char('#') => InputEvent::SearchKeyBackward,
            BackendKey::Char('w') => InputEvent::NextAtSameOrShallowerDepth,
            BackendKey::Char('b') => InputEvent::PreviousAtSameOrShallowerDepth,
            BackendKey::Char('"') => InputEvent::RegisterSelect,
            BackendKey::Char('v') | BackendKey::Char('V') => InputEvent::EnterVisualMode,
            BackendKey::Char('m') => InputEvent::MarkSet,
            BackendKey::Char('\'') => InputEvent::MarkJump,
            BackendKey::Char('.') => InputEvent::Repeat,
            BackendKey::Down => InputEvent::MoveDown,
            BackendKey::Up => InputEvent::MoveUp,
            BackendKey::Left => InputEvent::MoveLeft,
            BackendKey::Right => InputEvent::MoveRight,
            BackendKey::PageDown => InputEvent::FullPageDown,
            BackendKey::PageUp => InputEvent::FullPageUp,
            BackendKey::Home => InputEvent::JumpToTop,
            BackendKey::End => InputEvent::JumpToBottom,
            BackendKey::F(1) => InputEvent::Help,
            _ => InputEvent::Unknown,
        },
        EditorMode::Insert => match key {
            BackendKey::Esc => InputEvent::ExitMode,
            BackendKey::Char('\n') => InputEvent::InsertEnter,
            BackendKey::Backspace => InputEvent::InsertBackspace,
            BackendKey::Char(c) => InputEvent::InsertCharacter(c),
            _ => InputEvent::Unknown,
        },
        EditorMode::Command => match key {
            BackendKey::Esc => InputEvent::ExitMode,
            _ => InputEvent::Unknown,
        },
        EditorMode::Search => match key {
            BackendKey::Esc => InputEvent::ExitMode,
            _ => InputEvent::Unknown,
        },
        EditorMode::Visual => match key {
            BackendKey::Esc => InputEvent::ExitMode,
            BackendKey::Char('j') => InputEvent::MoveDown,
            BackendKey::Char('k') => InputEvent::MoveUp,
            BackendKey::Char('h') => InputEvent::MoveLeft,
            BackendKey::Char('l') => InputEvent::MoveRight,
            BackendKey::Char('d') => InputEvent::Delete,
            BackendKey::Char('y') => InputEvent::Yank,
            BackendKey::Char('p') => InputEvent::Paste,
            BackendKey::Char('P') => InputEvent::PasteBefore,
            BackendKey::Down => InputEvent::MoveDown,
            BackendKey::Up => InputEvent::MoveUp,
            BackendKey::Left => InputEvent::MoveLeft,
            BackendKey::Right => InputEvent::MoveRight,
            _ => InputEvent::Unknown,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_quit() {
        let event = BackendEvent::Key(BackendKey::Char('q'));
        assert_eq!(map_key_event(event, &EditorMode::Normal), InputEvent::Quit);
    }

    #[test]
    fn test_normal_mode_movement_vim_keys() {
        assert_eq!(
            map_key_event(
                BackendEvent::Key(BackendKey::Char('j')),
                &EditorMode::Normal
            ),
            InputEvent::MoveDown
        );
        assert_eq!(
            map_key_event(
                BackendEvent::Key(BackendKey::Char('k')),
                &EditorMode::Normal
            ),
            InputEvent::MoveUp
        );
        assert_eq!(
            map_key_event(
                BackendEvent::Key(BackendKey::Char('h')),
                &EditorMode::Normal
            ),
            InputEvent::MoveLeft
        );
        assert_eq!(
            map_key_event(
                BackendEvent::Key(BackendKey::Char('l')),
                &EditorMode::Normal
            ),
            InputEvent::MoveRight
        );
    }

    #[test]
    fn test_normal_mode_movement_arrow_keys() {
        assert_eq!(
            map_key_event(BackendEvent::Key(BackendKey::Down), &EditorMode::Normal),
            InputEvent::MoveDown
        );
        assert_eq!(
            map_key_event(BackendEvent::Key(BackendKey::Up), &EditorMode::Normal),
            InputEvent::MoveUp
        );
    }

    #[test]
    fn test_normal_mode_enter_modes() {
        assert_eq!(
            map_key_event(
                BackendEvent::Key(BackendKey::Char('e')),
                &EditorMode::Normal
            ),
            InputEvent::EnterInsertMode
        );
        assert_eq!(
            map_key_event(
                BackendEvent::Key(BackendKey::Char(':')),
                &EditorMode::Normal
            ),
            InputEvent::EnterCommandMode
        );
    }

    #[test]
    fn test_insert_mode_exit() {
        assert_eq!(
            map_key_event(BackendEvent::Key(BackendKey::Esc), &EditorMode::Insert),
            InputEvent::ExitMode
        );
    }

    #[test]
    fn test_command_mode_exit() {
        assert_eq!(
            map_key_event(BackendEvent::Key(BackendKey::Esc), &EditorMode::Command),
            InputEvent::ExitMode
        );
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(
            map_key_event(
                BackendEvent::Key(BackendKey::Char('x')),
                &EditorMode::Normal
            ),
            InputEvent::Unknown
        );
    }
}
