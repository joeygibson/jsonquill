//! Input event handler for polling and processing keyboard events.

use super::keys::{map_key_event, InputEvent};
use crate::editor::state::EditorState;
use crate::editor::mode::EditorMode;
use crossterm::event::{self, Event};
use anyhow::Result;
use std::time::Duration;

/// Handles terminal input events and updates editor state.
///
/// The InputHandler polls for crossterm events and converts them to
/// high-level InputEvents, then updates the editor state accordingly.
pub struct InputHandler;

impl InputHandler {
    /// Creates a new InputHandler.
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::input::InputHandler;
    ///
    /// let handler = InputHandler::new();
    /// ```
    pub fn new() -> Self {
        Self
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
    /// let handler = InputHandler::new();
    /// let event = handler.poll_event(Duration::from_millis(100)).unwrap();
    /// ```
    pub fn poll_event(&self, timeout: Duration) -> Result<Option<Event>> {
        if event::poll(timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    /// Handles a terminal event and updates editor state.
    ///
    /// Processes keyboard events and updates the editor state accordingly.
    /// Returns true if the application should quit.
    ///
    /// # Arguments
    ///
    /// * `event` - The crossterm Event to handle
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
    /// use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    ///
    /// let handler = InputHandler::new();
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new(tree);
    /// let event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    /// let should_quit = handler.handle_event(event, &mut state).unwrap();
    /// assert!(should_quit);
    /// ```
    pub fn handle_event(&self, event: Event, state: &mut EditorState) -> Result<bool> {
        if let Event::Key(key) = event {
            let input_event = map_key_event(key, state.mode());

            match input_event {
                InputEvent::Quit => return Ok(true),
                InputEvent::EnterInsertMode => {
                    state.set_mode(EditorMode::Insert);
                }
                InputEvent::EnterCommandMode => {
                    state.set_mode(EditorMode::Command);
                }
                InputEvent::ExitMode => {
                    state.set_mode(EditorMode::Normal);
                }
                InputEvent::MoveDown | InputEvent::MoveUp
                | InputEvent::MoveLeft | InputEvent::MoveRight => {
                    // TODO: implement cursor movement in future tasks
                }
                InputEvent::Unknown => {
                    // Ignore unknown keys
                }
            }
        }

        Ok(false)
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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
        let event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));

        let should_quit = handler.handle_event(event, &mut state).unwrap();
        assert!(should_quit);
    }

    #[test]
    fn test_enter_insert_mode() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);
        assert_eq!(*state.mode(), EditorMode::Normal);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        let should_quit = handler.handle_event(event, &mut state).unwrap();

        assert!(!should_quit);
        assert_eq!(*state.mode(), EditorMode::Insert);
    }

    #[test]
    fn test_enter_command_mode() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);

        let event = Event::Key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
        handler.handle_event(event, &mut state).unwrap();

        assert_eq!(*state.mode(), EditorMode::Command);
    }

    #[test]
    fn test_exit_mode() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);
        state.set_mode(EditorMode::Insert);

        let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        handler.handle_event(event, &mut state).unwrap();

        assert_eq!(*state.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_movement_keys_dont_quit() {
        let handler = InputHandler::new();
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new(tree);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        let should_quit = handler.handle_event(event, &mut state).unwrap();

        assert!(!should_quit);
    }
}
