//! Editor state management.
//!
//! This module provides the `EditorState` struct that manages all runtime state
//! for the editor, including the JSON document tree, current editing mode, cursor
//! position, dirty flag (unsaved changes), and optional filename.
//!
//! The `EditorState` acts as the central state container that coordinates between
//! the document model, user interface, and editing operations.
//!
//! # State Components
//!
//! - **Tree**: The JSON document structure being edited
//! - **Mode**: Current editing mode (Normal, Insert, or Command)
//! - **Cursor**: Current position in the tree
//! - **Dirty flag**: Whether there are unsaved changes
//! - **Filename**: Optional path to the file being edited
//!
//! # Example
//!
//! ```
//! use jeditor::editor::state::EditorState;
//! use jeditor::editor::mode::EditorMode;
//! use jeditor::document::node::{JsonNode, JsonValue};
//! use jeditor::document::tree::JsonTree;
//!
//! // Create an editor state with an empty object
//! let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
//! let mut state = EditorState::new(tree);
//!
//! // Starts in Normal mode, not dirty
//! assert_eq!(state.mode(), &EditorMode::Normal);
//! assert!(!state.is_dirty());
//!
//! // Make a change and mark dirty
//! state.mark_dirty();
//! assert!(state.is_dirty());
//!
//! // Switch to Insert mode
//! state.set_mode(EditorMode::Insert);
//! assert_eq!(state.mode(), &EditorMode::Insert);
//! ```

use super::cursor::Cursor;
use super::mode::EditorMode;
use crate::document::tree::JsonTree;
use crate::document::node::JsonNode;
use crate::ui::tree_view::TreeViewState;

/// Manages the complete runtime state of the editor.
///
/// `EditorState` is the central state container that holds:
/// - The JSON document tree being edited
/// - The current editing mode (Normal/Insert/Command)
/// - The cursor position in the tree
/// - A dirty flag indicating unsaved changes
/// - An optional filename for the document
///
/// # Examples
///
/// ```
/// use jeditor::editor::state::EditorState;
/// use jeditor::editor::mode::EditorMode;
/// use jeditor::document::node::{JsonNode, JsonValue};
/// use jeditor::document::tree::JsonTree;
///
/// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
/// let mut state = EditorState::new(tree);
///
/// // Check initial state
/// assert_eq!(state.mode(), &EditorMode::Normal);
/// assert!(!state.is_dirty());
/// assert_eq!(state.filename(), None);
///
/// // Modify state
/// state.mark_dirty();
/// state.set_filename("data.json".to_string());
/// assert!(state.is_dirty());
/// assert_eq!(state.filename(), Some("data.json"));
/// ```
/// Represents a message to display to the user.
#[derive(Debug, Clone)]
pub struct Message {
    pub text: String,
    pub level: MessageLevel,
}

/// Message severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

pub struct EditorState {
    tree: JsonTree,
    mode: EditorMode,
    cursor: Cursor,
    dirty: bool,
    filename: Option<String>,
    tree_view: TreeViewState,
    message: Option<Message>,
    command_buffer: String,
    show_help: bool,
    help_scroll: usize,
    pending_theme: Option<String>,
    current_theme: String,
    clipboard: Option<JsonNode>,
    search_buffer: String,
    search_results: Vec<Vec<usize>>,
    search_index: usize,
    show_line_numbers: bool,
}

impl EditorState {
    /// Creates a new editor state with the given JSON tree.
    ///
    /// The editor starts in Normal mode with the cursor at the root,
    /// no unsaved changes, and no filename set.
    ///
    /// # Arguments
    ///
    /// * `tree` - The JSON document tree to edit
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![])));
    /// let state = EditorState::new(tree);
    ///
    /// assert!(!state.is_dirty());
    /// assert_eq!(state.filename(), None);
    /// ```
    pub fn new(tree: JsonTree) -> Self {
        let mut tree_view = TreeViewState::new();
        // Expand all nodes by default for single JSON files
        tree_view.expand_all(&tree);
        tree_view.rebuild(&tree);

        // Initialize cursor to first visible line if available
        let mut cursor = Cursor::new();
        if let Some(first_line) = tree_view.lines().first() {
            cursor.set_path(first_line.path.clone());
        }

        Self {
            tree,
            mode: EditorMode::Normal,
            cursor,
            dirty: false,
            filename: None,
            tree_view,
            message: None,
            command_buffer: String::new(),
            show_help: false,
            help_scroll: 0,
            pending_theme: None,
            current_theme: "default-dark".to_string(),
            clipboard: None,
            search_buffer: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            show_line_numbers: true,
        }
    }

    /// Returns a reference to the JSON tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let state = EditorState::new(tree);
    ///
    /// let tree_ref = state.tree();
    /// // Use tree_ref for read-only operations
    /// ```
    pub fn tree(&self) -> &JsonTree {
        &self.tree
    }

    /// Returns a mutable reference to the JSON tree.
    ///
    /// **IMPORTANT:** After modifying the tree, you MUST call `rebuild_tree_view()`
    /// to update the tree view display, or the UI will show stale data.
    ///
    /// # Example
    ///
    /// ```
    /// # use jeditor::document::node::{JsonNode, JsonValue};
    /// # use jeditor::document::tree::JsonTree;
    /// # use jeditor::editor::state::EditorState;
    /// # let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    /// # let mut state = EditorState::new(tree);
    /// // Modify the tree
    /// let tree = state.tree_mut();
    /// // ... make modifications ...
    ///
    /// // REQUIRED: Rebuild tree view after modifications
    /// state.rebuild_tree_view();
    /// ```
    pub fn tree_mut(&mut self) -> &mut JsonTree {
        &mut self.tree
    }

    /// Returns a reference to the current editing mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::editor::mode::EditorMode;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let state = EditorState::new(tree);
    ///
    /// assert_eq!(state.mode(), &EditorMode::Normal);
    /// ```
    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    /// Sets the editing mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The new editing mode
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::editor::mode::EditorMode;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new(tree);
    ///
    /// state.set_mode(EditorMode::Insert);
    /// assert_eq!(state.mode(), &EditorMode::Insert);
    ///
    /// state.set_mode(EditorMode::Command);
    /// assert_eq!(state.mode(), &EditorMode::Command);
    /// ```
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    /// Returns a reference to the cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let state = EditorState::new(tree);
    ///
    /// let cursor = state.cursor();
    /// assert_eq!(cursor.path(), &[] as &[usize]);
    /// ```
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Returns a mutable reference to the cursor.
    ///
    /// This allows modification of the cursor position in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new(tree);
    ///
    /// state.cursor_mut().push(0);
    /// assert_eq!(state.cursor().path(), &[0]);
    /// ```
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    /// Returns whether the document has unsaved changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let state = EditorState::new(tree);
    ///
    /// assert!(!state.is_dirty());
    /// ```
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the document as having unsaved changes.
    ///
    /// This should be called after any modification to the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new(tree);
    ///
    /// state.mark_dirty();
    /// assert!(state.is_dirty());
    /// ```
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clears the dirty flag, indicating all changes have been saved.
    ///
    /// This should be called after successfully saving the document.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new(tree);
    ///
    /// state.mark_dirty();
    /// assert!(state.is_dirty());
    ///
    /// state.clear_dirty();
    /// assert!(!state.is_dirty());
    /// ```
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Returns the filename of the document being edited, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let state = EditorState::new(tree);
    ///
    /// assert_eq!(state.filename(), None);
    /// ```
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    /// Sets the filename for the document.
    ///
    /// # Arguments
    ///
    /// * `filename` - The path to the file
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new(tree);
    ///
    /// state.set_filename("config.json".to_string());
    /// assert_eq!(state.filename(), Some("config.json"));
    /// ```
    pub fn set_filename(&mut self, filename: String) {
        self.filename = Some(filename);
    }

    /// Returns a reference to the tree view state.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let state = EditorState::new(tree);
    ///
    /// let tree_view = state.tree_view();
    /// assert_eq!(tree_view.lines().len(), 0);
    /// ```
    pub fn tree_view(&self) -> &TreeViewState {
        &self.tree_view
    }

    /// Returns a mutable reference to the tree view state.
    ///
    /// This allows modification of the tree view state, such as toggling
    /// expand/collapse of nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
    ///     ("key".to_string(), JsonNode::new(JsonValue::Null)),
    /// ])));
    /// let mut state = EditorState::new(tree);
    ///
    /// state.tree_view_mut().toggle_expand(&[0]);
    /// assert!(state.tree_view().is_expanded(&[0]));
    /// ```
    pub fn tree_view_mut(&mut self) -> &mut TreeViewState {
        &mut self.tree_view
    }

    /// Rebuilds the tree view after the JSON tree has been modified.
    ///
    /// IMPORTANT: This must be called after any modifications to the tree
    /// (obtained via `tree_mut()`) to keep the tree view display in sync.
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    /// use jeditor::editor::state::EditorState;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    /// let mut state = EditorState::new(tree);
    ///
    /// // After modifying the tree:
    /// // let tree = state.tree_mut();
    /// // ... modify tree ...
    /// state.rebuild_tree_view();
    /// ```
    pub fn rebuild_tree_view(&mut self) {
        self.tree_view.rebuild(&self.tree);
    }

    /// Moves the cursor down to the next visible line in the tree view.
    ///
    /// If the cursor is at the last line or the tree is empty, this does nothing.
    /// If the cursor is not found in the tree, it moves to the first line.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
    ///     ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
    ///     ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    /// ])));
    /// let mut state = EditorState::new(tree);
    ///
    /// // Initially at first line [0]
    /// assert_eq!(state.cursor().path(), &[0]);
    ///
    /// // Move down to [1]
    /// state.move_cursor_down();
    /// assert_eq!(state.cursor().path(), &[1]);
    ///
    /// // At last line, stays at [1]
    /// state.move_cursor_down();
    /// assert_eq!(state.cursor().path(), &[1]);
    /// ```
    pub fn move_cursor_down(&mut self) {
        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let current_path = self.cursor.path();

        // Find current line index
        if let Some(current_idx) = lines.iter().position(|l| l.path == current_path) {
            if current_idx + 1 < lines.len() {
                let next_path = lines[current_idx + 1].path.clone();
                self.cursor.set_path(next_path);
            }
        } else if !lines.is_empty() {
            // If cursor not found, go to first line
            self.cursor.set_path(lines[0].path.clone());
        }
    }

    /// Moves the cursor up to the previous visible line in the tree view.
    ///
    /// If the cursor is at the first line or the tree is empty, this does nothing.
    /// If the cursor is not found in the tree, it moves to the first line.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
    ///     ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
    ///     ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    /// ])));
    /// let mut state = EditorState::new(tree);
    ///
    /// // Move to second line
    /// state.move_cursor_down();
    /// assert_eq!(state.cursor().path(), &[1]);
    ///
    /// // Move back up to first line
    /// state.move_cursor_up();
    /// assert_eq!(state.cursor().path(), &[0]);
    ///
    /// // At first line, stays at [0]
    /// state.move_cursor_up();
    /// assert_eq!(state.cursor().path(), &[0]);
    /// ```
    pub fn move_cursor_up(&mut self) {
        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let current_path = self.cursor.path();

        if let Some(current_idx) = lines.iter().position(|l| l.path == current_path) {
            if current_idx > 0 {
                let prev_path = lines[current_idx - 1].path.clone();
                self.cursor.set_path(prev_path);
            }
        } else if !lines.is_empty() {
            self.cursor.set_path(lines[0].path.clone());
        }
    }

    /// Toggles expand/collapse at the current cursor position and rebuilds the tree view.
    ///
    /// If the node at the cursor is expandable (object/array), this toggles its
    /// expanded state and rebuilds the tree view to show/hide children.
    ///
    /// # Examples
    ///
    /// ```
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use jeditor::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
    ///     ("user".to_string(), JsonNode::new(JsonValue::Object(vec![
    ///         ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ///     ]))),
    /// ])));
    /// let mut state = EditorState::new(tree);
    ///
    /// // Initially expanded (auto-expansion is default) - 2 lines
    /// assert_eq!(state.tree_view().lines().len(), 2);
    ///
    /// // Toggle to collapse
    /// state.toggle_expand_at_cursor();
    /// assert_eq!(state.tree_view().lines().len(), 1);
    ///
    /// // Toggle to expand again
    /// state.toggle_expand_at_cursor();
    /// assert_eq!(state.tree_view().lines().len(), 2);
    /// ```
    pub fn toggle_expand_at_cursor(&mut self) {
        let current_path = self.cursor.path().to_vec();
        self.tree_view.toggle_expand(&current_path);
        self.tree_view.rebuild(&self.tree);
    }

    /// Returns the current message, if any.
    pub fn message(&self) -> Option<&Message> {
        self.message.as_ref()
    }

    /// Sets a message to display to the user.
    pub fn set_message(&mut self, text: String, level: MessageLevel) {
        self.message = Some(Message { text, level });
    }

    /// Clears the current message.
    pub fn clear_message(&mut self) {
        self.message = None;
    }

    /// Returns the current command buffer.
    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Sets the command buffer.
    pub fn set_command_buffer(&mut self, buffer: String) {
        self.command_buffer = buffer;
    }

    /// Appends a character to the command buffer.
    pub fn push_to_command_buffer(&mut self, ch: char) {
        self.command_buffer.push(ch);
    }

    /// Removes the last character from the command buffer.
    pub fn pop_from_command_buffer(&mut self) {
        self.command_buffer.pop();
    }

    /// Clears the command buffer.
    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }

    /// Returns whether the help overlay is shown.
    pub fn show_help(&self) -> bool {
        self.show_help
    }

    /// Toggles the help overlay visibility.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.help_scroll = 0; // Reset scroll when opening
        }
    }

    /// Returns the current help scroll position.
    pub fn help_scroll(&self) -> usize {
        self.help_scroll
    }

    /// Scrolls the help overlay down.
    pub fn scroll_help_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(1);
    }

    /// Scrolls the help overlay up.
    pub fn scroll_help_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    /// Returns the pending theme name if there is one, consuming it.
    pub fn take_pending_theme(&mut self) -> Option<String> {
        self.pending_theme.take()
    }

    /// Requests a theme change.
    pub fn request_theme_change(&mut self, theme_name: String) {
        self.current_theme = theme_name.clone();
        self.pending_theme = Some(theme_name);
    }

    /// Sets the current theme name (called when theme is applied).
    pub fn set_current_theme(&mut self, theme_name: String) {
        self.current_theme = theme_name;
    }

    /// Yanks (copies) the node at the current cursor position to the clipboard.
    pub fn yank_node(&mut self) -> bool {
        let path = self.cursor.path();
        if let Some(node) = self.tree.get_node(path) {
            self.clipboard = Some(node.clone());

            // Try to copy to system clipboard as formatted JSON
            use arboard::Clipboard;
            if let Ok(mut clipboard) = Clipboard::new() {
                // Convert the JsonValue to serde_json::Value for pretty printing
                let json_value = self.node_to_serde_value(node.value());
                if let Ok(json_str) = serde_json::to_string_pretty(&json_value) {
                    let _ = clipboard.set_text(json_str);
                }
            }

            true
        } else {
            false
        }
    }

    fn node_to_serde_value(&self, value: &crate::document::node::JsonValue) -> serde_json::Value {
        use crate::document::node::JsonValue;
        match value {
            JsonValue::Object(entries) => {
                let map: serde_json::Map<String, serde_json::Value> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), self.node_to_serde_value(v.value())))
                    .collect();
                serde_json::Value::Object(map)
            }
            JsonValue::Array(elements) => {
                let arr: Vec<serde_json::Value> = elements
                    .iter()
                    .map(|v| self.node_to_serde_value(v.value()))
                    .collect();
                serde_json::Value::Array(arr)
            }
            JsonValue::String(s) => serde_json::Value::String(s.clone()),
            JsonValue::Number(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0))
            ),
            JsonValue::Boolean(b) => serde_json::Value::Bool(*b),
            JsonValue::Null => serde_json::Value::Null,
        }
    }

    /// Returns whether there's something in the clipboard.
    pub fn has_clipboard(&self) -> bool {
        self.clipboard.is_some()
    }

    /// Returns the current search buffer.
    pub fn search_buffer(&self) -> &str {
        &self.search_buffer
    }

    /// Appends a character to the search buffer.
    pub fn push_to_search_buffer(&mut self, ch: char) {
        self.search_buffer.push(ch);
    }

    /// Removes the last character from the search buffer.
    pub fn pop_from_search_buffer(&mut self) {
        self.search_buffer.pop();
    }

    /// Clears the search buffer.
    pub fn clear_search_buffer(&mut self) {
        self.search_buffer.clear();
    }

    /// Executes a search for the current search buffer text.
    pub fn execute_search(&mut self) {
        if self.search_buffer.is_empty() {
            return;
        }

        let query = self.search_buffer.to_lowercase();
        self.search_results.clear();
        self.search_index = 0;

        // Search through all visible lines
        for line in self.tree_view.lines() {
            let mut matches = false;

            // Check key name
            if let Some(key) = &line.key {
                if key.to_lowercase().contains(&query) {
                    matches = true;
                }
            }

            // Check string values
            if let crate::ui::tree_view::ValueType::String = line.value_type {
                if line.value_preview.to_lowercase().contains(&query) {
                    matches = true;
                }
            }

            if matches {
                self.search_results.push(line.path.clone());
            }
        }

        // Jump to first result
        if !self.search_results.is_empty() {
            self.cursor.set_path(self.search_results[0].clone());
        }
    }

    /// Jumps to the next search result.
    pub fn next_search_result(&mut self) -> bool {
        if self.search_results.is_empty() {
            return false;
        }

        self.search_index = (self.search_index + 1) % self.search_results.len();
        self.cursor.set_path(self.search_results[self.search_index].clone());
        true
    }

    /// Returns the current search results info.
    pub fn search_results_info(&self) -> Option<(usize, usize)> {
        if self.search_results.is_empty() {
            None
        } else {
            Some((self.search_index + 1, self.search_results.len()))
        }
    }

    /// Returns whether line numbers should be shown.
    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    /// Sets whether line numbers should be shown.
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Saves current settings to the config file.
    pub fn save_config(&self) -> anyhow::Result<()> {
        use crate::config::Config;

        let config = Config {
            theme: self.current_theme.clone(),
            show_line_numbers: self.show_line_numbers,
            ..Config::default()
        };

        config.save()
    }
}
