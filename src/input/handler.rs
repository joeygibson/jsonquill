//! Input event handler for polling and processing keyboard events.

use super::keys::{map_key_event, InputEvent};
use crate::editor::mode::EditorMode;
use crate::editor::state::EditorState;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, Stdin};
use std::time::Duration;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::input::{Events, TermRead};

/// Event source for reading terminal events.
///
/// This enum wraps the events iterator to maintain its state across
/// multiple calls, preventing character loss during rapid input (paste).
enum EventSource {
    /// Reading from stdin
    Stdin(Events<Stdin>),
    /// Reading from /dev/tty (when stdin was piped)
    Tty(Events<File>),
}

/// Handles terminal input events and updates editor state.
///
/// The InputHandler polls for termion events and converts them to
/// high-level InputEvents, then updates the editor state accordingly.
pub struct InputHandler {
    /// Event source iterator (maintains position in input buffer)
    events: EventSource,
    /// True if waiting for register name after " key
    awaiting_register: bool,
}

impl InputHandler {
    /// Creates a new InputHandler that reads from stdin.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::input::InputHandler;
    ///
    /// let handler = InputHandler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            events: EventSource::Stdin(io::stdin().events()),
            awaiting_register: false,
        }
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
            events: EventSource::Tty(tty_file.events()),
            awaiting_register: false,
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
    /// use jsonquill::input::InputHandler;
    /// use std::time::Duration;
    ///
    /// let mut handler = InputHandler::new();
    /// let event = handler.poll_event(Duration::from_millis(100)).unwrap();
    /// ```
    pub fn poll_event(&mut self, _timeout: Duration) -> Result<Option<Event>> {
        // Use the stored events iterator to maintain position in the input buffer.
        // This prevents character loss during rapid input (paste operations).
        match &mut self.events {
            EventSource::Stdin(events) => {
                if let Some(event_result) = events.next() {
                    return Ok(Some(event_result?));
                }
            }
            EventSource::Tty(events) => {
                if let Some(event_result) = events.next() {
                    return Ok(Some(event_result?));
                }
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
    /// use jsonquill::input::InputHandler;
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use termion::event::{Event, Key};
    ///
    /// let mut handler = InputHandler::new();
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new_with_default_theme(tree);
    /// let event = Event::Key(Key::Char('q'));
    /// let should_quit = handler.handle_event(event, &mut state).unwrap();
    /// assert!(should_quit);
    /// ```
    pub fn handle_event(&mut self, event: Event, state: &mut EditorState) -> Result<bool> {
        // Handle register selection if awaiting register
        if self.awaiting_register {
            if let Event::Key(Key::Char(c)) = event {
                // Check if it's a valid register (a-z, A-Z, 0-9, ")
                if c.is_ascii_alphanumeric() || c == '"' {
                    // Uppercase letters enable append mode
                    if c.is_ascii_uppercase() {
                        state.set_pending_register(c.to_ascii_lowercase(), true);
                    } else {
                        state.set_pending_register(c, false);
                    }
                    self.awaiting_register = false;
                    return Ok(false);
                }
            }
            // Invalid register or non-character key - cancel register selection
            self.awaiting_register = false;
            return Ok(false);
        }

        // Handle mark setting if waiting for mark name after 'm'
        if state.pending_mark_set() {
            state.set_pending_mark_set(false);
            if let Event::Key(Key::Char(c)) = event {
                if c.is_ascii_lowercase() {
                    use crate::editor::state::MessageLevel;
                    state.set_mark(c);
                    state.set_message(format!("Mark {} set", c), MessageLevel::Info);
                    return Ok(false);
                }
            }
            // Invalid mark name or non-character key - cancel mark set
            use crate::editor::state::MessageLevel;
            state.set_message("Invalid mark name".to_string(), MessageLevel::Error);
            return Ok(false);
        }

        // Handle mark jumping if waiting for mark name after '\''
        if state.pending_mark_jump() {
            state.set_pending_mark_jump(false);
            if let Event::Key(Key::Char(c)) = event {
                if c.is_ascii_lowercase() {
                    use crate::editor::state::MessageLevel;

                    // Check if this is a motion-to-mark operation (d'a or y'a)
                    if let Some(cmd) = state.pending_command() {
                        if cmd == 'd' || cmd == 'y' {
                            // Get mark position
                            if let Some(mark_path) = state.marks().get_mark(c).cloned() {
                                let count = state.get_count();

                                // Execute operation on range from cursor to mark
                                let result = if cmd == 'd' {
                                    state.delete_to_mark(&mark_path, count)
                                } else {
                                    state.yank_to_mark(&mark_path, count)
                                };

                                // Clear all pending state after operation
                                state.clear_pending();

                                match result {
                                    Ok(_) => {
                                        let op = if cmd == 'd' { "Deleted" } else { "Yanked" };
                                        state.set_message(
                                            format!("{} to mark '{}'", op, c),
                                            MessageLevel::Info,
                                        );
                                    }
                                    Err(e) => {
                                        state.set_message(
                                            format!("Error: {}", e),
                                            MessageLevel::Error,
                                        );
                                    }
                                }
                            } else {
                                state.clear_pending();
                                state.set_message(
                                    format!("Mark '{}' not set", c),
                                    MessageLevel::Error,
                                );
                            }
                            return Ok(false);
                        }
                    }

                    // Regular mark jump (no pending d/y)
                    if state.jump_to_mark(c) {
                        state.set_message("".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message(format!("Mark {} not set", c), MessageLevel::Error);
                    }
                    return Ok(false);
                }
            }
            // Invalid mark name or non-character key - cancel mark jump
            use crate::editor::state::MessageLevel;
            state.set_message("Invalid mark name".to_string(), MessageLevel::Error);
            return Ok(false);
        }

        // Handle mouse events if mouse is enabled
        if let Event::Mouse(mouse_event) = event {
            if state.enable_mouse() {
                // Check if help is shown - mouse scrolls help overlay
                if state.show_help() {
                    match mouse_event {
                        MouseEvent::Press(MouseButton::WheelUp, _, _) => {
                            // Scroll help up 3 lines
                            for _ in 0..3 {
                                state.scroll_help_up();
                            }
                            return Ok(false);
                        }
                        MouseEvent::Press(MouseButton::WheelDown, _, _) => {
                            // Scroll help down 3 lines
                            for _ in 0..3 {
                                state.scroll_help_down();
                            }
                            return Ok(false);
                        }
                        _ => {
                            // Ignore other mouse events
                            return Ok(false);
                        }
                    }
                } else {
                    // Help not shown - scroll main viewport
                    match mouse_event {
                        MouseEvent::Press(MouseButton::WheelUp, _, _) => {
                            // Scroll up 3 lines
                            for _ in 0..3 {
                                state.move_cursor_up();
                            }
                            return Ok(false);
                        }
                        MouseEvent::Press(MouseButton::WheelDown, _, _) => {
                            // Scroll down 3 lines
                            for _ in 0..3 {
                                state.move_cursor_down();
                            }
                            return Ok(false);
                        }
                        _ => {
                            // Ignore other mouse events (clicks, etc.)
                            return Ok(false);
                        }
                    }
                }
            }
            // Mouse disabled, ignore event
            return Ok(false);
        }

        if let Event::Key(key) = event {
            // Handle insert mode separately for character input
            if *state.mode() == EditorMode::Insert {
                match key {
                    Key::Char('\n') => {
                        // Check if we're in rename mode
                        if state.is_renaming_key() {
                            // Commit rename operation
                            use crate::editor::state::MessageLevel;
                            match state.commit_rename() {
                                Ok(_) => {
                                    state.set_mode(EditorMode::Normal);
                                }
                                Err(e) => {
                                    state.set_message(
                                        format!("Rename failed: {}", e),
                                        MessageLevel::Error,
                                    );
                                    state.cancel_rename();
                                    state.set_mode(EditorMode::Normal);
                                }
                            }
                        } else {
                            // Check if we're in add operation
                            use crate::editor::state::AddModeStage;
                            if matches!(state.add_mode_stage(), &AddModeStage::AwaitingValue) {
                                // Commit add operation
                                match state.commit_add_operation() {
                                    Ok(_) => {
                                        state.set_mode(EditorMode::Normal);
                                    }
                                    Err(e) => {
                                        use crate::editor::state::MessageLevel;
                                        state.set_message(
                                            format!("Add failed: {}", e),
                                            MessageLevel::Error,
                                        );
                                        state.cancel_add_operation();
                                    }
                                }
                            } else {
                                // Normal commit editing
                                use crate::editor::state::MessageLevel;
                                match state.commit_editing() {
                                    Ok(_) => {
                                        state.set_mode(EditorMode::Normal);
                                        state.set_message(
                                            "Value updated".to_string(),
                                            MessageLevel::Info,
                                        );
                                    }
                                    Err(e) => {
                                        state.set_message(
                                            format!("Invalid value: {}", e),
                                            MessageLevel::Error,
                                        );
                                    }
                                }
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
                    Key::Left => {
                        state.edit_cursor_left();
                        return Ok(false);
                    }
                    Key::Right => {
                        state.edit_cursor_right();
                        return Ok(false);
                    }
                    Key::Ctrl('a') => {
                        state.edit_cursor_home();
                        return Ok(false);
                    }
                    Key::Ctrl('e') => {
                        state.edit_cursor_end();
                        return Ok(false);
                    }
                    Key::Ctrl('d') => {
                        state.edit_delete_at_cursor();
                        return Ok(false);
                    }
                    Key::Ctrl('k') => {
                        state.edit_kill_to_end();
                        return Ok(false);
                    }
                    Key::Esc => {
                        // Check if we're in rename mode
                        if state.is_renaming_key() {
                            // Cancel rename operation
                            state.cancel_rename();
                            state.set_mode(EditorMode::Normal);
                            use crate::editor::state::MessageLevel;
                            state.set_message("Rename cancelled".to_string(), MessageLevel::Info);
                        } else {
                            // Check if we're in add operation
                            use crate::editor::state::AddModeStage;
                            if matches!(state.add_mode_stage(), &AddModeStage::AwaitingValue) {
                                // Cancel add operation
                                state.cancel_editing();
                                state.cancel_add_operation();
                            } else {
                                // Normal cancel editing
                                state.cancel_editing();
                            }
                            state.set_mode(EditorMode::Normal);
                            use crate::editor::state::MessageLevel;
                            state.set_message("Edit cancelled".to_string(), MessageLevel::Info);
                        }
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
                    Key::Char('\t') => {
                        // Tab completion
                        state.handle_tab_completion();
                        return Ok(false);
                    }
                    Key::Char(c) => {
                        state.push_to_command_buffer(c);
                        return Ok(false);
                    }
                    Key::Backspace => {
                        state.pop_from_command_buffer();
                        // Exit command mode if buffer is now empty
                        if state.command_buffer().is_empty() {
                            state.set_mode(EditorMode::Normal);
                        }
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
                            state
                                .set_message("No matches found".to_string(), MessageLevel::Warning);
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

            // If theme picker is shown, handle navigation and selection
            if state.show_theme_picker() {
                match key {
                    Key::Up | Key::Char('k') => {
                        state.theme_picker_previous();
                        return Ok(false);
                    }
                    Key::Down | Key::Char('j') => {
                        state.theme_picker_next();
                        return Ok(false);
                    }
                    Key::Char('\n') => {
                        state.theme_picker_apply();
                        return Ok(false);
                    }
                    Key::Esc | Key::Char('q') => {
                        state.theme_picker_cancel();
                        return Ok(false);
                    }
                    _ => {
                        // Ignore other keys when theme picker is shown
                        return Ok(false);
                    }
                }
            }

            // If help is shown, handle scrolling and closing
            if state.show_help() {
                match key {
                    Key::Esc | Key::F(1) => {
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

            // Handle digit input in Normal mode for count prefix
            if *state.mode() == EditorMode::Normal {
                if let Key::Char(c) = key {
                    if c.is_ascii_digit() {
                        let digit = c.to_digit(10).unwrap();
                        // '0' can only be part of count if count already started
                        // '0' by itself would be a command (go to start of line in vim)
                        if digit > 0 || state.pending_count().is_some() {
                            state.push_count_digit(digit);
                            return Ok(false);
                        }
                    }

                    // Handle yank path commands (yp, yb, yq)
                    if state.pending_command() == Some('y') {
                        use crate::editor::state::MessageLevel;
                        match c {
                            'p' => {
                                state.clear_pending();
                                if state.yank_path_dot() {
                                    state.set_message(
                                        "Path yanked (dot notation)".to_string(),
                                        MessageLevel::Info,
                                    );
                                } else {
                                    state.set_message(
                                        "Failed to yank path".to_string(),
                                        MessageLevel::Error,
                                    );
                                }
                                return Ok(false);
                            }
                            'b' => {
                                state.clear_pending();
                                if state.yank_path_bracket() {
                                    state.set_message(
                                        "Path yanked (bracket notation)".to_string(),
                                        MessageLevel::Info,
                                    );
                                } else {
                                    state.set_message(
                                        "Failed to yank path".to_string(),
                                        MessageLevel::Error,
                                    );
                                }
                                return Ok(false);
                            }
                            'q' => {
                                state.clear_pending();
                                if state.yank_path_jq() {
                                    state.set_message(
                                        "Path yanked (jq style)".to_string(),
                                        MessageLevel::Info,
                                    );
                                } else {
                                    state.set_message(
                                        "Failed to yank path".to_string(),
                                        MessageLevel::Error,
                                    );
                                }
                                return Ok(false);
                            }
                            _ => {
                                // Not a path yank command, continue with normal processing
                            }
                        }
                    }

                    // Handle screen positioning commands (zz, zt, zb)
                    if state.pending_command() == Some('z') {
                        match c {
                            'z' => {
                                state.clear_pending();
                                state.center_cursor_on_screen();
                                return Ok(false);
                            }
                            't' => {
                                state.clear_pending();
                                state.cursor_to_top_of_screen();
                                return Ok(false);
                            }
                            'b' => {
                                state.clear_pending();
                                state.cursor_to_bottom_of_screen();
                                return Ok(false);
                            }
                            _ => {
                                // Not a screen positioning command, continue with normal processing
                            }
                        }
                    }
                }

                // Handle key input during AwaitingKey stage (before Insert mode)
                use crate::editor::state::AddModeStage;
                if matches!(state.add_mode_stage(), &AddModeStage::AwaitingKey) {
                    match key {
                        Key::Char('\n') => {
                            // Enter pressed - check if this is a container add or scalar add
                            // Container adds have the node stored in temp_container
                            if state.has_temp_container() {
                                // This is a container add (a/o) - commit directly
                                use crate::editor::state::MessageLevel;
                                match state.commit_container_add() {
                                    Ok(_) => {
                                        // Success message set in commit_container_add
                                    }
                                    Err(e) => {
                                        state.set_message(
                                            format!("Add failed: {}", e),
                                            MessageLevel::Error,
                                        );
                                        state.cancel_add_operation();
                                    }
                                }
                            } else {
                                // This is a scalar add (i) - transition to value stage
                                state.transition_add_to_value();
                            }
                            return Ok(false);
                        }
                        Key::Char(c) if c.is_ascii() && !c.is_control() => {
                            // Regular character - add to key buffer
                            // Clear message on first input to show clean key entry area
                            if state.add_key_buffer().is_empty() {
                                state.clear_message();
                            }
                            state.push_to_add_key_buffer(c);
                            return Ok(false);
                        }
                        Key::Backspace => {
                            // Backspace - remove from key buffer
                            state.pop_from_add_key_buffer();
                            return Ok(false);
                        }
                        Key::Esc => {
                            // Escape - cancel add operation
                            state.cancel_add_operation();
                            state.set_mode(EditorMode::Normal);
                            return Ok(false);
                        }
                        _ => {
                            // Ignore other keys
                            return Ok(false);
                        }
                    }
                }
            }

            let input_event = map_key_event(Event::Key(key), state.mode());

            match input_event {
                InputEvent::Quit => {
                    state.clear_pending();
                    state.clear_search_results();
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
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    state.start_editing();
                    if state.edit_buffer().is_some() {
                        state.set_mode(EditorMode::Insert);
                        state.set_message("-- INSERT --".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message(
                            "Cannot edit this node type".to_string(),
                            MessageLevel::Error,
                        );
                    }
                }
                InputEvent::EnterCommandMode => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.clear_command_buffer();
                    state.set_mode(EditorMode::Command);
                }
                InputEvent::EnterSearchMode => {
                    state.clear_pending();
                    state.clear_search_buffer();
                    state.set_search_forward(true);
                    state.set_mode(EditorMode::Search);
                }
                InputEvent::EnterReverseSearchMode => {
                    state.clear_pending();
                    state.clear_search_buffer();
                    state.set_search_forward(false);
                    state.set_mode(EditorMode::Search);
                }
                InputEvent::NextSearchResult => {
                    state.clear_pending();
                    use crate::editor::state::MessageLevel;
                    state.record_jump();
                    let (success, wrapped) = state.next_search_result();
                    if success {
                        if let Some((current, total)) = state.search_results_info() {
                            let wrap_indicator = if wrapped { "W " } else { "" };
                            state.set_message(
                                format!("{}Match {}/{}", wrap_indicator, current, total),
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
                InputEvent::Help => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.toggle_help();
                }
                InputEvent::ExitMode => {
                    state.clear_pending();
                    state.clear_search_results();
                    // If exiting from visual mode, use exit_visual_mode() to clear selection
                    if state.mode() == &EditorMode::Visual {
                        state.exit_visual_mode();
                    } else {
                        state.set_mode(EditorMode::Normal);
                    }
                }
                InputEvent::MoveDown => {
                    let count = state.get_count();
                    state.clear_pending();
                    state.clear_search_results();
                    for _ in 0..count {
                        state.move_cursor_down();
                    }
                    // Update visual selection if in visual mode
                    if state.mode() == &EditorMode::Visual {
                        state.update_visual_selection();
                    }
                }
                InputEvent::MoveUp => {
                    let count = state.get_count();
                    state.clear_pending();
                    state.clear_search_results();
                    for _ in 0..count {
                        state.move_cursor_up();
                    }
                    // Update visual selection if in visual mode
                    if state.mode() == &EditorMode::Visual {
                        state.update_visual_selection();
                    }
                }
                InputEvent::MoveRight => {
                    let count = state.get_count();
                    state.clear_pending();
                    state.clear_search_results();
                    for _ in 0..count {
                        state.toggle_expand_at_cursor();
                    }
                    // Update visual selection if in visual mode
                    if state.mode() == &EditorMode::Visual {
                        state.update_visual_selection();
                    }
                }
                InputEvent::MoveLeft => {
                    let count = state.get_count();
                    state.clear_pending();
                    state.clear_search_results();
                    for _ in 0..count {
                        state.toggle_expand_at_cursor();
                    }
                    // Update visual selection if in visual mode
                    if state.mode() == &EditorMode::Visual {
                        state.update_visual_selection();
                    }
                }
                InputEvent::ExpandAll => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.expand_all_at_cursor();
                }
                InputEvent::CollapseAll => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.collapse_all_at_cursor();
                }
                InputEvent::Yank => {
                    use crate::editor::state::MessageLevel;
                    // In visual mode, yank selection and exit visual mode
                    if state.mode() == &EditorMode::Visual {
                        state.clear_pending();
                        state.clear_search_results();
                        let count = state.yank_visual_selection();
                        if count > 0 {
                            if count > 1 {
                                state.set_message(
                                    format!("{} nodes yanked", count),
                                    MessageLevel::Info,
                                );
                            } else {
                                state.set_message("Node yanked".to_string(), MessageLevel::Info);
                            }
                        }
                        state.exit_visual_mode();
                    } else if state.pending_command() == Some('y') {
                        // Normal mode: second 'y' press (yy)
                        let count = state.get_count();
                        state.clear_pending();
                        state.clear_search_results();

                        if state.yank_nodes(count) {
                            // Record command for repeat
                            use crate::editor::repeat::RepeatableCommand;
                            state.set_last_command(RepeatableCommand::Yank { count });

                            if count > 1 {
                                state.set_message(
                                    format!("{} nodes yanked", count),
                                    MessageLevel::Info,
                                );
                            } else {
                                state.set_message("Node yanked".to_string(), MessageLevel::Info);
                            }
                        } else {
                            state.set_message("Nothing to yank".to_string(), MessageLevel::Error);
                        }
                    } else {
                        // Normal mode: first 'y' press - set pending
                        state.clear_message();
                        state.set_pending_command('y');
                    }
                }
                InputEvent::Delete => {
                    use crate::editor::state::MessageLevel;
                    // In visual mode, delete selection and exit visual mode
                    if state.mode() == &EditorMode::Visual {
                        state.clear_pending();
                        state.clear_search_results();
                        match state.delete_visual_selection() {
                            Ok(count) => {
                                if count > 0 {
                                    if count > 1 {
                                        state.set_message(
                                            format!("{} nodes deleted (yanked)", count),
                                            MessageLevel::Info,
                                        );
                                    } else {
                                        state.set_message(
                                            "Node deleted (yanked)".to_string(),
                                            MessageLevel::Info,
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                state.set_message(
                                    format!("Delete failed: {}", e),
                                    MessageLevel::Error,
                                );
                            }
                        }
                        state.exit_visual_mode();
                    } else if state.pending_command() == Some('d') {
                        // Normal mode: second 'd' press (dd)
                        let count = state.get_count();
                        state.clear_pending();
                        state.clear_search_results();

                        let mut deleted_count = 0;
                        let mut had_error = false;

                        for _ in 0..count {
                            // delete_node_at_cursor now handles register updates internally
                            match state.delete_node_at_cursor() {
                                Ok(_) => {
                                    deleted_count += 1;
                                    // Don't move cursor - deleting moves us to next node automatically
                                }
                                Err(e) => {
                                    if deleted_count == 0 {
                                        state.set_message(
                                            format!("Delete failed: {}", e),
                                            MessageLevel::Error,
                                        );
                                    }
                                    had_error = true;
                                    break;
                                }
                            }
                        }

                        if !had_error && deleted_count > 0 {
                            // Record command for repeat
                            use crate::editor::repeat::RepeatableCommand;
                            state.set_last_command(RepeatableCommand::Delete { count });

                            if deleted_count > 1 {
                                state.set_message(
                                    format!("{} nodes deleted (yanked)", deleted_count),
                                    MessageLevel::Info,
                                );
                            } else {
                                state.set_message(
                                    "Node deleted (yanked)".to_string(),
                                    MessageLevel::Info,
                                );
                            }
                        }
                    } else {
                        // First 'd' press - set pending
                        state.clear_message();
                        state.set_pending_command('d');
                    }
                }
                InputEvent::Paste => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    // In visual mode, delete selection first, then paste
                    if state.mode() == &EditorMode::Visual {
                        let _ = state.delete_visual_selection();
                        state.exit_visual_mode();
                    }
                    match state.paste_node_at_cursor() {
                        Ok(_) => {
                            // Record command for repeat
                            use crate::editor::repeat::RepeatableCommand;
                            state.set_last_command(RepeatableCommand::Paste { before: false });
                            state.set_message("Node pasted after".to_string(), MessageLevel::Info);
                        }
                        Err(e) => {
                            state.set_message(format!("Paste failed: {}", e), MessageLevel::Error);
                        }
                    }
                }
                InputEvent::PasteBefore => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    // In visual mode, delete selection first, then paste before
                    if state.mode() == &EditorMode::Visual {
                        let _ = state.delete_visual_selection();
                        state.exit_visual_mode();
                    }
                    match state.paste_node_before_cursor() {
                        Ok(_) => {
                            // Record command for repeat
                            use crate::editor::repeat::RepeatableCommand;
                            state.set_last_command(RepeatableCommand::Paste { before: true });
                            state.set_message("Node pasted before".to_string(), MessageLevel::Info);
                        }
                        Err(e) => {
                            state.set_message(format!("Paste failed: {}", e), MessageLevel::Error);
                        }
                    }
                }
                InputEvent::SaveAndQuit => {
                    use crate::editor::state::MessageLevel;
                    // Check if this is the second 'Z' press
                    if state.pending_command() == Some('Z') {
                        state.clear_pending();
                        state.clear_search_results();
                        // Save the file
                        if let Some(filename) = state.filename() {
                            use crate::file::saver::save_json_file;
                            match save_json_file(filename, state.tree(), &state.to_config()) {
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
                            state.set_message(
                                "No filename (use :w <filename>)".to_string(),
                                MessageLevel::Error,
                            );
                        }
                    } else {
                        // First 'Z' press - set pending
                        state.clear_message();
                        state.set_pending_command('Z');
                    }
                }
                InputEvent::JumpToTop => {
                    // If there's a pending count, jump to that line number
                    if state.pending_count().is_some() {
                        let line_num = state.get_count();
                        state.clear_pending();
                        state.clear_search_results();
                        state.record_jump();
                        state.jump_to_line(line_num as usize);
                    } else if state.pending_command() == Some('g') {
                        // Second 'g' press (gg) - jump to top
                        state.clear_pending();
                        state.clear_search_results();
                        state.record_jump();
                        state.jump_to_top();
                    } else {
                        // First 'g' press - set pending
                        state.clear_message();
                        state.set_pending_command('g');
                    }
                }
                InputEvent::JumpToBottom => {
                    // If there's a pending count, jump to that line number (vim: <count>G)
                    if state.pending_count().is_some() {
                        let line_num = state.get_count();
                        state.clear_pending();
                        state.clear_search_results();
                        state.record_jump();
                        state.jump_to_line(line_num as usize);
                    } else {
                        // No count - jump to bottom
                        state.clear_pending();
                        state.clear_search_results();
                        state.record_jump();
                        state.jump_to_bottom();
                    }
                }
                InputEvent::HalfPageDown => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.page_down();
                }
                InputEvent::HalfPageUp => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.page_up();
                }
                InputEvent::FullPageDown => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.full_page_down();
                }
                InputEvent::FullPageUp => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.full_page_up();
                }
                InputEvent::Undo => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    if state.undo() {
                        state.set_message("Undo".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message(
                            "Already at oldest change".to_string(),
                            MessageLevel::Info,
                        );
                    }
                }
                InputEvent::Redo => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    if state.redo() {
                        state.set_message("Redo".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message(
                            "Already at newest change".to_string(),
                            MessageLevel::Info,
                        );
                    }
                }
                InputEvent::Add => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.start_add_operation();
                }
                InputEvent::AddArray => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.start_add_container_operation(false); // false = array
                }
                InputEvent::AddObject => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.start_add_container_operation(true); // true = object
                }
                InputEvent::Rename => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.start_rename_operation();
                }
                InputEvent::NextSibling => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.move_to_next_sibling();
                }
                InputEvent::PreviousSibling => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.move_to_previous_sibling();
                }
                InputEvent::FirstSibling => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.move_to_first_sibling();
                }
                InputEvent::LastSibling => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.move_to_last_sibling();
                }
                InputEvent::SearchKeyForward => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    state.record_jump();
                    if state.execute_key_search(true) {
                        if let Some((current, total)) = state.search_results_info() {
                            state.set_message(
                                format!("Key search: match {}/{}", current, total),
                                MessageLevel::Info,
                            );
                        }
                    }
                }
                InputEvent::SearchKeyBackward => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    state.record_jump();
                    if state.execute_key_search(false) {
                        if let Some((current, total)) = state.search_results_info() {
                            state.set_message(
                                format!("Key search: match {}/{}", current, total),
                                MessageLevel::Info,
                            );
                        }
                    }
                }
                InputEvent::NextAtSameOrShallowerDepth => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.move_to_next_at_same_or_shallower_depth();
                }
                InputEvent::PreviousAtSameOrShallowerDepth => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.move_to_previous_at_same_or_shallower_depth();
                }
                InputEvent::MoveToParent => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.move_to_parent();
                }
                InputEvent::ScreenPosition => {
                    // First 'z' press - set pending
                    state.clear_message();
                    state.clear_search_results();
                    state.set_pending_command('z');
                }
                InputEvent::RegisterSelect => {
                    // " key pressed - wait for register name
                    self.awaiting_register = true;
                    state.clear_message();
                    state.clear_search_results();
                }
                InputEvent::InsertCharacter(_)
                | InputEvent::InsertBackspace
                | InputEvent::InsertEnter => {
                    state.clear_pending();
                    state.clear_search_results();
                    // These are handled earlier in insert mode, should never reach here
                }
                InputEvent::EnterVisualMode => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    state.enter_visual_mode();
                    state.set_message("-- VISUAL --".to_string(), MessageLevel::Info);
                }
                InputEvent::MarkSet => {
                    state.clear_pending();
                    state.clear_search_results();
                    state.set_pending_mark_set(true);
                }
                InputEvent::MarkJump => {
                    // Don't clear pending if it's a motion-to-mark operation (d'a or y'a)
                    let is_motion_to_mark =
                        matches!(state.pending_command(), Some('d') | Some('y'));
                    if !is_motion_to_mark {
                        state.clear_pending();
                        // Record jump before jumping to mark (only for regular jumps)
                        state.record_jump();
                    }
                    state.clear_search_results();
                    state.set_pending_mark_jump(true);
                }
                InputEvent::JumpBackward => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    if state.jump_backward() {
                        state.set_message("".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message("Already at oldest jump".to_string(), MessageLevel::Info);
                    }
                }
                InputEvent::JumpForward => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    if state.jump_forward() {
                        state.set_message("".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message("Already at newest jump".to_string(), MessageLevel::Info);
                    }
                }
                InputEvent::Repeat => {
                    state.clear_pending();
                    state.clear_search_results();
                    use crate::editor::state::MessageLevel;
                    match state.repeat_last_command() {
                        Ok(msg) => {
                            state.set_message(msg, MessageLevel::Info);
                        }
                        Err(msg) => {
                            state.set_message(msg, MessageLevel::Error);
                        }
                    }
                }
                InputEvent::Unknown => {
                    state.clear_pending();
                    state.clear_search_results();
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
            state.open_theme_picker();
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
            if state.relative_line_numbers() {
                settings.push("relativenumber");
            } else {
                settings.push("norelativenumber");
            }
            if state.enable_mouse() {
                settings.push("mouse");
            } else {
                settings.push("nomouse");
            }
            if state.create_backup() {
                settings.push("create_backup");
            } else {
                settings.push("nocreate_backup");
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
                        state.set_message("Settings saved".to_string(), MessageLevel::Info);
                    }
                }
                Err(e) => {
                    state.set_message(format!("Error saving config: {}", e), MessageLevel::Error);
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
                        let value = if state.show_line_numbers() {
                            "on"
                        } else {
                            "off"
                        };
                        state.set_message(format!("number is {}", value), MessageLevel::Info);
                    }
                    "relativenumber" | "rnu" => {
                        let value = if state.relative_line_numbers() {
                            "on"
                        } else {
                            "off"
                        };
                        state.set_message(
                            format!("relativenumber is {}", value),
                            MessageLevel::Info,
                        );
                    }
                    "mouse" => {
                        let value = if state.enable_mouse() { "on" } else { "off" };
                        state.set_message(format!("mouse is {}", value), MessageLevel::Info);
                    }
                    "create_backup" => {
                        let value = if state.create_backup() { "on" } else { "off" };
                        state
                            .set_message(format!("create_backup is {}", value), MessageLevel::Info);
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
                "relativenumber" | "rnu" => {
                    state.set_relative_line_numbers(true);
                    state.set_message(
                        "Relative line numbers enabled".to_string(),
                        MessageLevel::Info,
                    );
                }
                "norelativenumber" | "nornu" => {
                    state.set_relative_line_numbers(false);
                    state.set_message(
                        "Relative line numbers disabled".to_string(),
                        MessageLevel::Info,
                    );
                }
                "mouse" => {
                    state.set_enable_mouse(true);
                    state.set_message("Mouse support enabled".to_string(), MessageLevel::Info);
                }
                "nomouse" => {
                    state.set_enable_mouse(false);
                    state.set_message("Mouse support disabled".to_string(), MessageLevel::Info);
                }
                "create_backup" => {
                    state.set_create_backup(true);
                    state.set_message(
                        "Backup file creation enabled".to_string(),
                        MessageLevel::Info,
                    );
                }
                "nocreate_backup" => {
                    state.set_create_backup(false);
                    state.set_message(
                        "Backup file creation disabled".to_string(),
                        MessageLevel::Info,
                    );
                }
                _ => {
                    state.set_message(format!("Unknown setting: {}", setting), MessageLevel::Error);
                }
            }
            return Ok(false);
        }

        // Handle :path and :jp commands
        if let Some(query) = command.strip_prefix("path ") {
            let query = query.trim();
            if query.is_empty() {
                state.set_message("Usage: :path <jsonpath>".to_string(), MessageLevel::Error);
            } else {
                state.execute_jsonpath_search(query);
            }
            return Ok(false);
        }

        if let Some(query) = command.strip_prefix("jp ") {
            let query = query.trim();
            if query.is_empty() {
                state.set_message("Usage: :jp <jsonpath>".to_string(), MessageLevel::Error);
            } else {
                state.execute_jsonpath_search(query);
            }
            return Ok(false);
        }

        // Handle :find command with query
        if let Some(query) = command.strip_prefix("find ") {
            let query = query.trim();
            if query.is_empty() {
                state.set_mode(EditorMode::Search);
                state.set_search_forward(true);
                state.clear_search_buffer();
            } else {
                // Execute text search immediately
                state.clear_search_buffer();
                for ch in query.chars() {
                    state.push_to_search_buffer(ch);
                }
                state.execute_search();
            }
            return Ok(false);
        }

        // Handle :find with no arguments (enter search mode)
        if command == "find" {
            state.set_mode(EditorMode::Search);
            state.set_search_forward(true);
            state.clear_search_buffer();
            return Ok(false);
        }

        // Handle :format command
        if command == "format" {
            match state.format_document() {
                Ok(_) => {
                    state.set_message("Document reformatted".to_string(), MessageLevel::Info);
                }
                Err(e) => {
                    state.set_message(format!("Format failed: {}", e), MessageLevel::Error);
                }
            }
            return Ok(false);
        }

        match command {
            "e!" => {
                // Reload from disk, discarding changes
                if let Some(filename) = state.filename().map(|s| s.to_string()) {
                    use crate::file::loader::load_json_file;
                    match load_json_file(&filename) {
                        Ok(tree) => {
                            state.reload_tree(tree);
                            state.set_message(
                                format!("\"{}\" reloaded", filename),
                                MessageLevel::Info,
                            );
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Error reloading file: {}", e),
                                MessageLevel::Error,
                            );
                        }
                    }
                } else {
                    state.set_message("No file name".to_string(), MessageLevel::Error);
                }
                Ok(false)
            }
            cmd if cmd.starts_with("e ") => {
                // :e filename - load a different file
                let filename = cmd[2..].trim().to_string();
                if filename.is_empty() {
                    state.set_message("No file name specified".to_string(), MessageLevel::Error);
                    return Ok(false);
                }

                // Check for unsaved changes
                if state.is_dirty() {
                    state.set_message(
                        "No write since last change (add ! to override)".to_string(),
                        MessageLevel::Error,
                    );
                    return Ok(false);
                }

                use crate::file::loader::load_json_file;
                match load_json_file(&filename) {
                    Ok(tree) => {
                        state.reload_tree(tree);
                        state.set_filename(filename.clone());
                        state.set_message(format!("\"{}\" loaded", filename), MessageLevel::Info);
                    }
                    Err(e) => {
                        state
                            .set_message(format!("Error loading file: {}", e), MessageLevel::Error);
                    }
                }
                Ok(false)
            }
            cmd if cmd.starts_with("e! ") => {
                // :e! filename - load a different file, discarding changes
                let filename = cmd[3..].trim().to_string();
                if filename.is_empty() {
                    state.set_message("No file name specified".to_string(), MessageLevel::Error);
                    return Ok(false);
                }

                use crate::file::loader::load_json_file;
                match load_json_file(&filename) {
                    Ok(tree) => {
                        state.reload_tree(tree);
                        state.set_filename(filename.clone());
                        state.set_message(format!("\"{}\" loaded", filename), MessageLevel::Info);
                    }
                    Err(e) => {
                        state
                            .set_message(format!("Error loading file: {}", e), MessageLevel::Error);
                    }
                }
                Ok(false)
            }
            "help" => {
                state.toggle_help();
                Ok(false)
            }
            "q" => {
                if state.is_dirty() {
                    state.set_message(
                        "No write since last change (use :q! to force)".to_string(),
                        MessageLevel::Error,
                    );
                    return Ok(false);
                }
                Ok(true)
            }
            "q!" => Ok(true),
            "undo" => {
                if state.undo() {
                    state.set_message("Undo".to_string(), MessageLevel::Info);
                } else {
                    state.set_message("Already at oldest change".to_string(), MessageLevel::Info);
                }
                Ok(false)
            }
            "redo" => {
                if state.redo() {
                    state.set_message("Redo".to_string(), MessageLevel::Info);
                } else {
                    state.set_message("Already at newest change".to_string(), MessageLevel::Info);
                }
                Ok(false)
            }
            cmd if cmd.starts_with("w ") => {
                // :w filename - save to new file and update internal filename
                let filename = cmd[2..].trim().to_string();
                if filename.is_empty() {
                    state.set_message("No file name specified".to_string(), MessageLevel::Error);
                    return Ok(false);
                }

                match save_json_file(&filename, state.tree(), &state.to_config()) {
                    Ok(_) => {
                        state.set_filename(filename.clone());
                        state.clear_dirty();
                        state.set_message(format!("\"{}\" written", filename), MessageLevel::Info);
                    }
                    Err(e) => {
                        state.set_message(format!("Error saving file: {}", e), MessageLevel::Error);
                    }
                }
                Ok(false)
            }
            "w" => {
                if let Some(filename) = state.filename().map(|s| s.to_string()) {
                    match save_json_file(&filename, state.tree(), &state.to_config()) {
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
                Ok(false)
            }
            cmd if cmd.starts_with("wq ") || cmd.starts_with("x ") => {
                // :wq filename or :x filename - save to new file, update filename, and quit
                let filename = if let Some(stripped) = cmd.strip_prefix("wq ") {
                    stripped.trim().to_string()
                } else if let Some(stripped) = cmd.strip_prefix("x ") {
                    stripped.trim().to_string()
                } else {
                    String::new()
                };

                if filename.is_empty() {
                    state.set_message("No file name specified".to_string(), MessageLevel::Error);
                    return Ok(false);
                }

                match save_json_file(&filename, state.tree(), &state.to_config()) {
                    Ok(_) => {
                        state.set_filename(filename);
                        state.clear_dirty();
                        Ok(true)
                    }
                    Err(e) => {
                        state.set_message(format!("Error saving file: {}", e), MessageLevel::Error);
                        Ok(false)
                    }
                }
            }
            "wq" | "x" => {
                if let Some(filename) = state.filename().map(|s| s.to_string()) {
                    match save_json_file(&filename, state.tree(), &state.to_config()) {
                        Ok(_) => {
                            state.clear_dirty();
                            Ok(true)
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Error saving file: {}", e),
                                MessageLevel::Error,
                            );
                            Ok(false)
                        }
                    }
                } else {
                    state.set_message(
                        "No file name (use :wq <filename>)".to_string(),
                        MessageLevel::Error,
                    );
                    Ok(false)
                }
            }
            "" => {
                // Empty command, do nothing
                Ok(false)
            }
            _ => {
                state.set_message(format!("Unknown command: {}", command), MessageLevel::Error);
                Ok(false)
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
    use crate::document::node::{JsonNode, JsonValue};
    use crate::document::tree::JsonTree;
    use crate::editor::mode::EditorMode;
    use termion::event::Key;

    #[test]
    fn test_handler_creation() {
        let _handler = InputHandler::new();
        // Just verify it constructs without panic
    }

    #[test]
    fn test_quit_event() {
        let mut handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);
        let event = Event::Key(Key::Char('q'));

        let should_quit = handler.handle_event(event, &mut state).unwrap();
        assert!(should_quit);
    }

    #[test]
    fn test_quit_blocked_when_dirty() {
        let mut handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);

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
        let mut handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);
        assert_eq!(*state.mode(), EditorMode::Normal);

        let event = Event::Key(Key::Char('e'));
        let should_quit = handler.handle_event(event, &mut state).unwrap();

        assert!(!should_quit);
        assert_eq!(*state.mode(), EditorMode::Insert);
    }

    #[test]
    fn test_enter_command_mode() {
        let mut handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);

        let event = Event::Key(Key::Char(':'));
        handler.handle_event(event, &mut state).unwrap();

        assert_eq!(*state.mode(), EditorMode::Command);
    }

    #[test]
    fn test_exit_mode() {
        let mut handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);
        state.set_mode(EditorMode::Insert);

        let event = Event::Key(Key::Esc);
        handler.handle_event(event, &mut state).unwrap();

        assert_eq!(*state.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_movement_keys_dont_quit() {
        let mut handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);

        let event = Event::Key(Key::Char('j'));
        let should_quit = handler.handle_event(event, &mut state).unwrap();

        assert!(!should_quit);
    }

    #[test]
    fn test_write_with_new_filename() {
        use std::fs;
        use tempfile::TempDir;

        let mut handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Number(42.0)));
        let mut state = EditorState::new_with_default_theme(tree);

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_output.json");
        let file_path_str = file_path.to_str().unwrap();

        // Initially no filename is set
        assert_eq!(state.filename(), None);

        // Simulate entering command mode and typing `:w <filename>`
        state.set_mode(EditorMode::Command);
        state.set_command_buffer(format!("w {}", file_path_str));

        // Execute the command by simulating Enter key
        let event = Event::Key(Key::Char('\n'));
        let result = handler.handle_event(event, &mut state);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // should_quit = false

        // Verify the file was created
        assert!(file_path.exists());

        // Verify the content
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content.trim(), "42");

        // Verify the internal filename was updated
        assert_eq!(state.filename(), Some(file_path_str));

        // Verify dirty flag was cleared
        assert!(!state.is_dirty());
    }

    #[test]
    fn test_wq_with_new_filename() {
        use std::fs;
        use tempfile::TempDir;

        let mut handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::String("test".to_string())));
        let mut state = EditorState::new_with_default_theme(tree);

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_wq.json");
        let file_path_str = file_path.to_str().unwrap();

        // Simulate entering command mode and typing `:wq <filename>`
        state.set_mode(EditorMode::Command);
        state.set_command_buffer(format!("wq {}", file_path_str));

        // Execute the command - should save and quit
        let event = Event::Key(Key::Char('\n'));
        let result = handler.handle_event(event, &mut state);
        assert!(result.is_ok());
        assert!(result.unwrap()); // should_quit = true

        // Verify the file was created
        assert!(file_path.exists());

        // Verify the content
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content.trim(), "\"test\"");
    }
}
