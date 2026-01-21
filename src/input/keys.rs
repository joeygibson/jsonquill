//! Keyboard event mapping and input event types.

use crossterm::event::{KeyCode, KeyEvent};
use crate::editor::mode::EditorMode;

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
    /// Jump to top of document (gg)
    JumpToTop,
    /// Jump to bottom of document (G)
    JumpToBottom,
    /// Page down (Ctrl-d)
    PageDown,
    /// Page up (Ctrl-u)
    PageUp,
    /// Insert a character in insert mode
    InsertCharacter(char),
    /// Backspace in insert mode
    InsertBackspace,
    /// Enter in insert mode
    InsertEnter,
    /// Unknown or unmapped key
    Unknown,
}

/// Maps a crossterm KeyEvent to an InputEvent based on the current editor mode.
///
/// Different modes interpret keys differently (vim-style modal editing):
/// - Normal mode: hjkl for movement, i for insert, : for command, q for quit
/// - Insert mode: Esc to exit
/// - Command mode: Esc to exit
///
/// # Arguments
///
/// * `key` - The crossterm KeyEvent to map
/// * `mode` - The current editor mode
///
/// # Returns
///
/// The corresponding InputEvent, or InputEvent::Unknown if not mapped
///
/// # Example
///
/// ```
/// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
/// use jeditor::editor::mode::EditorMode;
/// use jeditor::input::keys::{map_key_event, InputEvent};
///
/// let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
/// let event = map_key_event(key, &EditorMode::Normal);
/// assert_eq!(event, InputEvent::MoveDown);
/// ```
pub fn map_key_event(key: KeyEvent, mode: &EditorMode) -> InputEvent {
    match mode {
        EditorMode::Normal => {
            use crossterm::event::KeyModifiers;

            // Check for Ctrl-modified keys first
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('d') => return InputEvent::PageDown,
                    KeyCode::Char('u') => return InputEvent::PageUp,
                    _ => {}
                }
            }

            // Then check for regular keys
            match key.code {
                KeyCode::Char('q') => InputEvent::Quit,
                KeyCode::Char('j') => InputEvent::MoveDown,
                KeyCode::Char('k') => InputEvent::MoveUp,
                KeyCode::Char('h') => InputEvent::MoveLeft,
                KeyCode::Char('l') => InputEvent::MoveRight,
                KeyCode::Char('i') => InputEvent::EnterInsertMode,
                KeyCode::Char(':') => InputEvent::EnterCommandMode,
                KeyCode::Char('/') => InputEvent::EnterSearchMode,
                KeyCode::Char('n') => InputEvent::NextSearchResult,
                KeyCode::Char('d') => InputEvent::Delete,
                KeyCode::Char('y') => InputEvent::Yank,
                KeyCode::Char('p') => InputEvent::Paste,
                KeyCode::Char('P') => InputEvent::PasteBefore,
                KeyCode::Char('Z') => InputEvent::SaveAndQuit,
                KeyCode::Char('g') => InputEvent::JumpToTop,
                KeyCode::Char('G') => InputEvent::JumpToBottom,
                KeyCode::Down => InputEvent::MoveDown,
                KeyCode::Up => InputEvent::MoveUp,
                KeyCode::Left => InputEvent::MoveLeft,
                KeyCode::Right => InputEvent::MoveRight,
                _ => InputEvent::Unknown,
            }
        },
        EditorMode::Insert => match key.code {
            KeyCode::Esc => InputEvent::ExitMode,
            KeyCode::Char(c) => InputEvent::InsertCharacter(c),
            KeyCode::Backspace => InputEvent::InsertBackspace,
            KeyCode::Enter => InputEvent::InsertEnter,
            _ => InputEvent::Unknown,
        },
        EditorMode::Command => match key.code {
            KeyCode::Esc => InputEvent::ExitMode,
            _ => InputEvent::Unknown,
        },
        EditorMode::Search => match key.code {
            KeyCode::Esc => InputEvent::ExitMode,
            _ => InputEvent::Unknown,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_normal_mode_quit() {
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key, &EditorMode::Normal), InputEvent::Quit);
    }

    #[test]
    fn test_normal_mode_movement_vim_keys() {
        let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key_j, &EditorMode::Normal), InputEvent::MoveDown);

        let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key_k, &EditorMode::Normal), InputEvent::MoveUp);

        let key_h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key_h, &EditorMode::Normal), InputEvent::MoveLeft);

        let key_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key_l, &EditorMode::Normal), InputEvent::MoveRight);
    }

    #[test]
    fn test_normal_mode_movement_arrow_keys() {
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(map_key_event(key, &EditorMode::Normal), InputEvent::MoveDown);

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(map_key_event(key, &EditorMode::Normal), InputEvent::MoveUp);
    }

    #[test]
    fn test_normal_mode_enter_modes() {
        let key_i = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key_i, &EditorMode::Normal), InputEvent::EnterInsertMode);

        let key_colon = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key_colon, &EditorMode::Normal), InputEvent::EnterCommandMode);
    }

    #[test]
    fn test_insert_mode_exit() {
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(map_key_event(key, &EditorMode::Insert), InputEvent::ExitMode);
    }

    #[test]
    fn test_command_mode_exit() {
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(map_key_event(key, &EditorMode::Command), InputEvent::ExitMode);
    }

    #[test]
    fn test_unknown_key() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(map_key_event(key, &EditorMode::Normal), InputEvent::Unknown);
    }
}
