//! Input event handler for polling and processing keyboard events.

use super::keys::{map_key_event, InputEvent};
use crate::editor::state::EditorState;
use crate::editor::mode::EditorMode;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self};
use std::time::Duration;
use termion::event::{Event, Key};
use termion::input::TermRead;

/// Handles terminal input events and updates editor state.
///
/// The InputHandler polls for termion events and converts them to
/// high-level InputEvents, then updates the editor state accordingly.
pub struct InputHandler {
    /// File handle for /dev/tty when stdin was piped
    tty_file: Option<File>,
}

impl InputHandler {
    /// Creates a new InputHandler that reads from stdin.
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::input::InputHandler;
    ///
    /// let handler = InputHandler::new();
    /// ```
    pub fn new() -> Self {
        Self { tty_file: None }
    }

    /// Creates a new InputHandler that reads from /dev/tty.
    /// Use this when stdin has been consumed for piped data.
    pub fn new_with_tty() -> Result<Self> {
        let tty_file = File::options()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .context("Failed to open /dev/tty for keyboard input")?;

        Ok(Self {
            tty_file: Some(tty_file),
        })
    }

    /// Polls for a terminal event with a timeout.
    ///
    /// Returns Some(Event) if an event occurred, None if timeout elapsed.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for an event
    ///
    /// # Errors
    ///
    /// Returns an error if the event system fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jeditor::input::InputHandler;
    /// use std::time::Duration;
    ///
    /// let mut handler = InputHandler::new();
    /// let event = handler.poll_event(Duration::from_millis(100)).unwrap();
    /// ```
    pub fn poll_event(&mut self, _timeout: Duration) -> Result<Option<Event>> {
        // Termion's events() iterator blocks until an event is available
        // We'll use a non-blocking read with a timeout simulation
        if let Some(tty_file) = &mut self.tty_file {
            // Read from /dev/tty
            let mut events = tty_file.events();
            if let Some(event_result) = events.next() {
                return Ok(Some(event_result?));
            }
        } else {
            // Read from stdin
            let mut events = io::stdin().events();
            if let Some(event_result) = events.next() {
                return Ok(Some(event_result?));
            }
        }

        Ok(None)
    }

    /// Handles a terminal event and updates editor state.
    ///
    /// Processes keyboard events and updates the editor state accordingly.
    /// Returns true if the application should quit.
    ///
    /// # Arguments
    ///
    /// * `event` - The termion Event to handle
    /// * `state` - The editor state to update
    ///
    /// # Returns
    ///
    /// Ok(true) if the application should quit, Ok(false) otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if state update fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jeditor::input::InputHandler;
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::tree::JsonTree;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use termion::event::{Event, Key};
    ///
    /// let handler = InputHandler::new();
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new(tree);
    /// let event = Event::Key(Key::Char('q'));
    /// let should_quit = handler.handle_event(event, &mut state).unwrap();
    /// assert!(should_quit);
    /// ```
    pub fn handle_event(&self, event: Event, state: &mut EditorState) -> Result<bool> {
        if let Event::Key(key) = event {
            // Handle insert mode separately for character input
            if *state.mode() == EditorMode::Insert {
                match key {
                    Key::Char('\n') => {
                        // Commit the edit
                        use crate::editor::state::MessageLevel;
                        match state.commit_editing() {
                            Ok(_) => {
                                state.set_mode(EditorMode::Normal);
                                state.set_message("Value updated".to_string(), MessageLevel::Info);
                            }
                            Err(e) => {
                                state.set_message(
                                    format!("Invalid value: {}", e),
                                    MessageLevel::Error,
                                );
                            }
                        }
                        return Ok(false);
                    }
                    Key::Char(c) => {
                        state.push_to_edit_buffer(c);
                        return Ok(false);
                    }
                    Key::Backspace => {
                        state.pop_from_edit_buffer();
                        return Ok(false);
                    }
                    Key::Esc => {
                        state.cancel_editing();
                        state.set_mode(EditorMode::Normal);
                        use crate::editor::state::MessageLevel;
                        state.set_message("Edit cancelled".to_string(), MessageLevel::Info);
                        return Ok(false);
                    }
                    _ => return Ok(false),
                }
            }

            // Handle command mode separately for character input
            if *state.mode() == EditorMode::Command {
                match key {
                    Key::Char('\n') => {
                        // Execute command and return to normal mode
                        let command = state.command_buffer().to_string();
                        state.clear_command_buffer();
                        state.set_mode(EditorMode::Normal);
                        return self.execute_command(&command, state);
                    }
                    Key::Char(c) => {
                        state.push_to_command_buffer(c);
                        return Ok(false);
                    }
                    Key::Backspace => {
                        state.pop_from_command_buffer();
                        return Ok(false);
                    }
                    Key::Esc => {
                        state.clear_command_buffer();
                        state.set_mode(EditorMode::Normal);
                        return Ok(false);
                    }
                    _ => return Ok(false),
                }
            }

            // Handle search mode separately for character input
            if *state.mode() == EditorMode::Search {
                match key {
                    Key::Char('\n') => {
                        // Exit search mode
                        state.set_mode(EditorMode::Normal);
                        use crate::editor::state::MessageLevel;
                        if let Some((_current, total)) = state.search_results_info() {
                            state.set_message(
                                format!("Found {} matches", total),
                                MessageLevel::Info,
                            );
                        } else {
                            state.set_message(
                                "No matches found".to_string(),
                                MessageLevel::Warning,
                            );
                        }
                        return Ok(false);
                    }
                    Key::Char(c) => {
                        state.push_to_search_buffer(c);
                        state.execute_search();
                        return Ok(false);
                    }
                    Key::Backspace => {
                        state.pop_from_search_buffer();
                        state.execute_search();
                        return Ok(false);
                    }
                    Key::Esc => {
                        state.clear_search_buffer();
                        state.set_mode(EditorMode::Normal);
                        return Ok(false);
                    }
                    _ => return Ok(false),
                }
            }

            // Handle help toggle in all modes
            if let Key::Char('?') = key {
                if *state.mode() == EditorMode::Normal {
                    state.toggle_help();
                    return Ok(false);
                }
            }

            // If help is shown, handle scrolling and closing
            if state.show_help() {
                match key {
                    Key::Esc | Key::Char('?') => {
                        state.toggle_help();
                        return Ok(false);
                    }
                    Key::Down | Key::Char('j') => {
                        state.scroll_help_down();
                        return Ok(false);
                    }
                    Key::Up | Key::Char('k') => {
                        state.scroll_help_up();
                        return Ok(false);
                    }
                    _ => {
                        // Ignore other keys when help is shown
                        return Ok(false);
                    }
                }
            }

            let input_event = map_key_event(Event::Key(key), state.mode());

            match input_event {
                InputEvent::Quit => {
                    state.clear_pending_command();
                    if state.is_dirty() {
                        use crate::editor::state::MessageLevel;
                        state.set_message(
                            "No write since last change (use :q! to force)".to_string(),
                            MessageLevel::Error,
                        );
                        return Ok(false);
                    }
                    return Ok(true);
                }
                InputEvent::EnterInsertMode => {
                    state.clear_pending_command();
                    use crate::editor::state::MessageLevel;
                    state.start_editing();
                    if state.edit_buffer().is_some() {
                        state.set_mode(EditorMode::Insert);
                        state.set_message("-- INSERT --".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message("Cannot edit this node type".to_string(), MessageLevel::Error);
                    }
                }
                InputEvent::EnterCommandMode => {
                    state.clear_pending_command();
                    state.clear_command_buffer();
                    state.set_mode(EditorMode::Command);
                }
                InputEvent::EnterSearchMode => {
                    state.clear_pending_command();
                    state.clear_search_buffer();
                    state.set_mode(EditorMode::Search);
                }
                InputEvent::NextSearchResult => {
                    state.clear_pending_command();
                    use crate::editor::state::MessageLevel;
                    if state.next_search_result() {
                        if let Some((current, total)) = state.search_results_info() {
                            state.set_message(
                                format!("Match {}/{}", current, total),
                                MessageLevel::Info,
                            );
                        }
                    } else {
                        state.set_message(
                            "No search results (use / to search)".to_string(),
                            MessageLevel::Warning,
                        );
                    }
                }
                InputEvent::ExitMode => {
                    state.clear_pending_command();
                    state.set_mode(EditorMode::Normal);
                }
                InputEvent::MoveDown => {
                    state.clear_pending_command();
                    state.move_cursor_down();
                }
                InputEvent::MoveUp => {
                    state.clear_pending_command();
                    state.move_cursor_up();
                }
                InputEvent::MoveRight => {
                    state.clear_pending_command();
                    state.toggle_expand_at_cursor();
                }
                InputEvent::MoveLeft => {
                    state.clear_pending_command();
                    state.toggle_expand_at_cursor();
                }
                InputEvent::Yank => {
                    use crate::editor::state::MessageLevel;
                    // Check if this is the second 'y' press
                    if state.pending_command() == Some('y') {
                        state.clear_pending_command();
                        if state.yank_node() {
                            state.set_message("Node yanked".to_string(), MessageLevel::Info);
                        } else {
                            state.set_message("Nothing to yank".to_string(), MessageLevel::Error);
                        }
                    } else {
                        // First 'y' press - set pending
                        state.set_pending_command('y');
                    }
                }
                InputEvent::Delete => {
                    use crate::editor::state::MessageLevel;
                    // Check if this is the second 'd' press
                    if state.pending_command() == Some('d') {
                        state.clear_pending_command();
                        // Yank before deleting (like vim's dd)
                        state.yank_node();
                        match state.delete_node_at_cursor() {
                            Ok(_) => {
                                state.set_message("Node deleted (yanked)".to_string(), MessageLevel::Info);
                            }
                            Err(e) => {
                                state.set_message(
                                    format!("Delete failed: {}", e),
                                    MessageLevel::Error,
                                );
                            }
                        }
                    } else {
                        // First 'd' press - set pending
                        state.set_pending_command('d');
                    }
                }
                InputEvent::Paste => {
                    state.clear_pending_command();
                    use crate::editor::state::MessageLevel;
                    match state.paste_node_at_cursor() {
                        Ok(_) => {
                            state.set_message("Node pasted after".to_string(), MessageLevel::Info);
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Paste failed: {}", e),
                                MessageLevel::Error,
                            );
                        }
                    }
                }
                InputEvent::PasteBefore => {
                    state.clear_pending_command();
                    use crate::editor::state::MessageLevel;
                    match state.paste_node_before_cursor() {
                        Ok(_) => {
                            state.set_message("Node pasted before".to_string(), MessageLevel::Info);
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Paste failed: {}", e),
                                MessageLevel::Error,
                            );
                        }
                    }
                }
                InputEvent::SaveAndQuit => {
                    use crate::editor::state::MessageLevel;
                    // Check if this is the second 'Z' press
                    if state.pending_command() == Some('Z') {
                        state.clear_pending_command();
                        // Save the file
                        if let Some(filename) = state.filename() {
                            use crate::file::saver::save_json_file;
                            match save_json_file(filename, state.tree(), 2, false) {
                                Ok(_) => {
                                    state.clear_dirty();
                                    return Ok(true); // Quit after saving
                                }
                                Err(e) => {
                                    state.set_message(
                                        format!("Save failed: {}", e),
                                        MessageLevel::Error,
                                    );
                                }
                            }
                        } else {
                            state.set_message("No filename (use :w <filename>)".to_string(), MessageLevel::Error);
                        }
                    } else {
                        // First 'Z' press - set pending
                        state.set_pending_command('Z');
                    }
                }
                InputEvent::JumpToTop => {
                    // Check if this is the second 'g' press (gg)
                    if state.pending_command() == Some('g') {
                        state.clear_pending_command();
                        state.jump_to_top();
                    } else {
                        // First 'g' press - set pending
                        state.set_pending_command('g');
                    }
                }
                InputEvent::JumpToBottom => {
                    state.clear_pending_command();
                    state.jump_to_bottom();
                }
                InputEvent::PageDown => {
                    state.clear_pending_command();
                    state.page_down();
                }
                InputEvent::PageUp => {
                    state.clear_pending_command();
                    state.page_up();
                }
                InputEvent::Undo => {
                    state.clear_pending_command();
                    use crate::editor::state::MessageLevel;
                    if state.undo() {
                        state.set_message("Undo".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message("Already at oldest change".to_string(), MessageLevel::Info);
                    }
                }
                InputEvent::Redo => {
                    state.clear_pending_command();
                    use crate::editor::state::MessageLevel;
                    if state.redo() {
                        state.set_message("Redo".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message("Already at newest change".to_string(), MessageLevel::Info);
                    }
                }
                InputEvent::InsertCharacter(_) | InputEvent::InsertBackspace | InputEvent::InsertEnter => {
                    state.clear_pending_command();
                    // These are handled earlier in insert mode, should never reach here
                }
                InputEvent::Unknown => {
                    state.clear_pending_command();
                    // Ignore unknown keys
                }
            }
        }

        Ok(false)
    }

    fn execute_command(&self, command: &str, state: &mut EditorState) -> Result<bool> {
        use crate::editor::state::MessageLevel;
        use crate::file::saver::save_json_file;

        let command = command.trim();

        // Handle :theme command
        if command == "theme" {
            use crate::theme::list_builtin_themes;
            let themes = list_builtin_themes();
            let theme_list = themes.join(", ");
            state.set_message(
                format!("Available themes: {}", theme_list),
                MessageLevel::Info,
            );
            return Ok(false);
        }

        if let Some(theme_name) = command.strip_prefix("theme ") {
            use crate::theme::get_builtin_theme;
            let theme_name = theme_name.trim();
            if get_builtin_theme(theme_name).is_some() {
                state.request_theme_change(theme_name.to_string());
                state.set_message(
                    format!("Switched to theme: {}", theme_name),
                    MessageLevel::Info,
                );
            } else {
                state.set_message(
                    format!("Unknown theme: {} (use :theme to list)", theme_name),
                    MessageLevel::Error,
                );
            }
            return Ok(false);
        }

        // Handle :set commands
        if command == "set" {
            // Show all modified settings
            let mut settings = Vec::new();
            if state.show_line_numbers() {
                settings.push("number");
            } else {
                settings.push("nonumber");
            }
            state.set_message(
                format!("Settings: {}", settings.join(", ")),
                MessageLevel::Info,
            );
            return Ok(false);
        }

        if command == "set save" {
            // Save current settings to config file
            match state.save_config() {
                Ok(_) => {
                    use crate::config::Config;
                    if let Some(path) = Config::config_path() {
                        state.set_message(
                            format!("Settings saved to {}", path.display()),
                            MessageLevel::Info,
                        );
                    } else {
                        state.set_message(
                            "Settings saved".to_string(),
                            MessageLevel::Info,
                        );
                    }
                }
                Err(e) => {
                    state.set_message(
                        format!("Error saving config: {}", e),
                        MessageLevel::Error,
                    );
                }
            }
            return Ok(false);
        }

        if let Some(setting) = command.strip_prefix("set ") {
            let setting = setting.trim();

            // Query setting value
            if let Some(setting_name) = setting.strip_suffix('?') {
                match setting_name {
                    "number" => {
                        let value = if state.show_line_numbers() { "on" } else { "off" };
                        state.set_message(
                            format!("number is {}", value),
                            MessageLevel::Info,
                        );
                    }
                    _ => {
                        state.set_message(
                            format!("Unknown setting: {}", setting_name),
                            MessageLevel::Error,
                        );
                    }
                }
                return Ok(false);
            }

            // Set setting value
            match setting {
                "number" | "nu" => {
                    state.set_show_line_numbers(true);
                    state.set_message("Line numbers enabled".to_string(), MessageLevel::Info);
                }
                "nonumber" | "nonu" => {
                    state.set_show_line_numbers(false);
                    state.set_message("Line numbers disabled".to_string(), MessageLevel::Info);
                }
                _ => {
                    state.set_message(
                        format!("Unknown setting: {}", setting),
                        MessageLevel::Error,
                    );
                }
            }
            return Ok(false);
        }

        match command {
            "q" => {
                if state.is_dirty() {
                    state.set_message(
                        "No write since last change (use :q! to force)".to_string(),
                        MessageLevel::Error,
                    );
                    return Ok(false);
                }
                return Ok(true);
            }
            "q!" => {
                return Ok(true);
            }
            "undo" => {
                if state.undo() {
                    state.set_message("Undo".to_string(), MessageLevel::Info);
                } else {
                    state.set_message("Already at oldest change".to_string(), MessageLevel::Info);
                }
                return Ok(false);
            }
            "redo" => {
                if state.redo() {
                    state.set_message("Redo".to_string(), MessageLevel::Info);
                } else {
                    state.set_message("Already at newest change".to_string(), MessageLevel::Info);
                }
                return Ok(false);
            }
            "w" => {
                if let Some(filename) = state.filename().map(|s| s.to_string()) {
                    match save_json_file(&filename, state.tree(), 2, false) {
                        Ok(_) => {
                            state.clear_dirty();
                            state.set_message(
                                format!("\"{}\" written", filename),
                                MessageLevel::Info,
                            );
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Error saving file: {}", e),
                                MessageLevel::Error,
                            );
                        }
                    }
                } else {
                    state.set_message(
                        "No file name (use :w <filename>)".to_string(),
                        MessageLevel::Error,
                    );
                }
                return Ok(false);
            }
            "wq" | "x" => {
                if let Some(filename) = state.filename().map(|s| s.to_string()) {
                    match save_json_file(&filename, state.tree(), 2, false) {
                        Ok(_) => {
                            state.clear_dirty();
                            return Ok(true);
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Error saving file: {}", e),
                                MessageLevel::Error,
                            );
                            return Ok(false);
                        }
                    }
                } else {
                    state.set_message(
                        "No file name (use :wq <filename>)".to_string(),
                        MessageLevel::Error,
                    );
                    return Ok(false);
                }
            }
            "" => {
                // Empty command, do nothing
                return Ok(false);
            }
            _ => {
                state.set_message(
                    format!("Unknown command: {}", command),
                    MessageLevel::Error,
                );
                return Ok(false);
            }
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termion::event::Key;
    use crate::editor::mode::EditorMode;
    use crate::document::node::{JsonNode, JsonValue};
    use crate::document::tree::JsonTree;

    #[test]
    fn test_handler_creation() {
        let _handler = InputHandler::new();
        // Just verify it constructs without panic
    }

    #[test]
    fn test_quit_event() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);
        let event = Event::Key(Key::Char('q'));

        let should_quit = handler.handle_event(event, &mut state).unwrap();
        assert!(should_quit);
    }

    #[test]
    fn test_quit_blocked_when_dirty() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);

        // Mark the file as dirty
        state.mark_dirty();

        let event = Event::Key(Key::Char('q'));
        let should_quit = handler.handle_event(event, &mut state).unwrap();

        // Should NOT quit when file is dirty
        assert!(!should_quit);

        // Should show error message
        if let Some(msg) = state.message() {
            assert!(msg.text.contains("No write since last change"));
        } else {
            panic!("Expected error message when trying to quit with unsaved changes");
        }
    }

    #[test]
    fn test_enter_insert_mode() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);
        assert_eq!(*state.mode(), EditorMode::Normal);

        let event = Event::Key(Key::Char('i'));
        let should_quit = handler.handle_event(event, &mut state).unwrap();

        assert!(!should_quit);
        assert_eq!(*state.mode(), EditorMode::Insert);
    }

    #[test]
    fn test_enter_command_mode() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);

        let event = Event::Key(Key::Char(':'));
        handler.handle_event(event, &mut state).unwrap();

        assert_eq!(*state.mode(), EditorMode::Command);
    }

    #[test]
    fn test_exit_mode() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);
        state.set_mode(EditorMode::Insert);

        let event = Event::Key(Key::Esc);
        handler.handle_event(event, &mut state).unwrap();

        assert_eq!(*state.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_movement_keys_dont_quit() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);

        let event = Event::Key(Key::Char('j'));
        let should_quit = handler.handle_event(event, &mut state).unwrap();

        assert!(!should_quit);
    }
}
