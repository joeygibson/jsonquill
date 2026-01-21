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
    clipboard_key: Option<String>,
    search_buffer: String,
    search_results: Vec<Vec<usize>>,
    search_index: usize,
    show_line_numbers: bool,
    edit_buffer: Option<String>,
    pending_command: Option<char>,
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
            clipboard_key: None,
            search_buffer: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            show_line_numbers: true,
            edit_buffer: None,
            pending_command: None,
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

    /// Deletes the node at the current cursor position.
    /// Adjusts the cursor position after deletion and rebuilds the tree view.
    pub fn delete_node_at_cursor(&mut self) -> anyhow::Result<()> {
        let path = self.cursor.path().to_vec();

        // Find current line index before deletion
        let lines = self.tree_view.lines();
        let current_idx = lines.iter().position(|l| l.path == path);

        // Delete the node
        self.tree.delete_node(&path)?;
        self.mark_dirty();
        self.rebuild_tree_view();

        // Adjust cursor position
        let new_lines = self.tree_view.lines();
        if new_lines.is_empty() {
            // No lines left, cursor stays at root
            self.cursor.set_path(vec![]);
        } else if let Some(idx) = current_idx {
            // Try to keep cursor at same visual position
            let new_idx = idx.min(new_lines.len() - 1);
            self.cursor.set_path(new_lines[new_idx].path.clone());
        } else if !new_lines.is_empty() {
            // Cursor wasn't found, move to first line
            self.cursor.set_path(new_lines[0].path.clone());
        }

        Ok(())
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

            // Store the key name if yanking from an object
            self.clipboard_key = None;
            if !path.is_empty() {
                let parent_path = &path[..path.len() - 1];
                let index = path[path.len() - 1];

                let parent = if parent_path.is_empty() {
                    Some(self.tree.root())
                } else {
                    self.tree.get_node(parent_path)
                };

                if let Some(parent_node) = parent {
                    use crate::document::node::JsonValue;
                    if let JsonValue::Object(entries) = parent_node.value() {
                        if let Some((key, _)) = entries.get(index) {
                            self.clipboard_key = Some(key.clone());
                        }
                    }
                }
            }

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

    /// Pastes the clipboard node after the current cursor position.
    /// For objects, generates a unique key name. For arrays, inserts after current index.
    pub fn paste_node_at_cursor(&mut self) -> anyhow::Result<()> {
        use anyhow::anyhow;
        use crate::document::node::JsonValue;

        let clipboard_node = self.clipboard.clone()
            .ok_or_else(|| anyhow!("Nothing to paste"))?;

        let current_path = self.cursor.path().to_vec();

        // Determine parent and insert position
        if current_path.is_empty() {
            return Err(anyhow!("Cannot paste at root level"));
        }

        let parent_path = &current_path[..current_path.len() - 1];
        let current_index = current_path[current_path.len() - 1];
        let insert_index = current_index + 1;

        // Get parent node to determine type
        let parent = if parent_path.is_empty() {
            self.tree.root()
        } else {
            self.tree.get_node(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        match parent.value() {
            JsonValue::Object(_) => {
                // Use the original key name if available, otherwise use "pasted"
                let base_key = self.clipboard_key.clone().unwrap_or_else(|| "pasted".to_string());
                let mut key_name = base_key.clone();
                let mut counter = 1;

                // Keep trying until we find a unique key
                loop {
                    let test_key = if counter == 1 {
                        key_name.clone()
                    } else {
                        format!("{}{}", base_key, counter)
                    };

                    // Check if key exists
                    let parent_ref = if parent_path.is_empty() {
                        self.tree.root()
                    } else {
                        self.tree.get_node(parent_path).unwrap()
                    };

                    let key_exists = if let JsonValue::Object(entries) = parent_ref.value() {
                        entries.iter().any(|(k, _)| k == &test_key)
                    } else {
                        false
                    };

                    if !key_exists {
                        key_name = test_key;
                        break;
                    }

                    counter += 1;
                }

                // Build the full path for insertion
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree.insert_node_in_object(&insert_path, key_name, clipboard_node)?;
            }
            JsonValue::Array(_) => {
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree.insert_node_in_array(&insert_path, clipboard_node)?;
            }
            _ => {
                return Err(anyhow!("Parent is not a container type"));
            }
        }

        self.mark_dirty();
        self.rebuild_tree_view();

        // Move cursor to newly pasted node
        let mut new_cursor_path = parent_path.to_vec();
        new_cursor_path.push(insert_index);
        self.cursor.set_path(new_cursor_path);

        Ok(())
    }

    /// Pastes the clipboard node before the current cursor position.
    /// For objects, generates a unique key name. For arrays, inserts before current index.
    pub fn paste_node_before_cursor(&mut self) -> anyhow::Result<()> {
        use anyhow::anyhow;
        use crate::document::node::JsonValue;

        let clipboard_node = self.clipboard.clone()
            .ok_or_else(|| anyhow!("Nothing to paste"))?;

        let current_path = self.cursor.path().to_vec();

        // Determine parent and insert position
        if current_path.is_empty() {
            return Err(anyhow!("Cannot paste at root level"));
        }

        let parent_path = &current_path[..current_path.len() - 1];
        let current_index = current_path[current_path.len() - 1];
        let insert_index = current_index; // Insert BEFORE current (at current position)

        // Get parent node to determine type
        let parent = if parent_path.is_empty() {
            self.tree.root()
        } else {
            self.tree.get_node(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        match parent.value() {
            JsonValue::Object(_) => {
                // Use the original key name if available, otherwise use "pasted"
                let base_key = self.clipboard_key.clone().unwrap_or_else(|| "pasted".to_string());
                let mut key_name = base_key.clone();
                let mut counter = 1;

                // Keep trying until we find a unique key
                loop {
                    let test_key = if counter == 1 {
                        key_name.clone()
                    } else {
                        format!("{}{}", base_key, counter)
                    };

                    // Check if key exists
                    let parent_ref = if parent_path.is_empty() {
                        self.tree.root()
                    } else {
                        self.tree.get_node(parent_path).unwrap()
                    };

                    let key_exists = if let JsonValue::Object(entries) = parent_ref.value() {
                        entries.iter().any(|(k, _)| k == &test_key)
                    } else {
                        false
                    };

                    if !key_exists {
                        key_name = test_key;
                        break;
                    }

                    counter += 1;
                }

                // Build the full path for insertion
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree.insert_node_in_object(&insert_path, key_name, clipboard_node)?;
            }
            JsonValue::Array(_) => {
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree.insert_node_in_array(&insert_path, clipboard_node)?;
            }
            _ => {
                return Err(anyhow!("Parent is not a container type"));
            }
        }

        self.mark_dirty();
        self.rebuild_tree_view();

        // Move cursor to newly pasted node
        let mut new_cursor_path = parent_path.to_vec();
        new_cursor_path.push(insert_index);
        self.cursor.set_path(new_cursor_path);

        Ok(())
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

    /// Returns the current edit buffer content, if editing.
    pub fn edit_buffer(&self) -> Option<&str> {
        self.edit_buffer.as_deref()
    }

    /// Starts editing the node at the current cursor position.
    /// Starts with an empty buffer for typing a new value.
    pub fn start_editing(&mut self) {
        let path = self.cursor.path();
        if let Some(node) = self.tree.get_node(path) {
            // Check if node is editable (not a container)
            match node.value() {
                crate::document::node::JsonValue::Object(_) | crate::document::node::JsonValue::Array(_) => {
                    return; // Can't edit containers
                }
                _ => {
                    // Start with empty buffer for inserting new value
                    self.edit_buffer = Some(String::new());
                }
            }
        }
    }

    /// Cancels editing and clears the edit buffer without saving changes.
    pub fn cancel_editing(&mut self) {
        self.edit_buffer = None;
    }

    /// Commits the edited value from the buffer to the tree.
    /// Parses the buffer according to the original node's type and updates the tree.
    /// Returns an error if the buffer content is invalid for the node's type.
    pub fn commit_editing(&mut self) -> anyhow::Result<()> {
        use crate::document::node::JsonValue;
        use anyhow::{anyhow, Context};

        let buffer_content = self.edit_buffer.as_ref()
            .ok_or_else(|| anyhow!("No active edit buffer"))?
            .clone();

        let path = self.cursor.path();
        let node = self.tree.get_node(path)
            .ok_or_else(|| anyhow!("Node not found at cursor"))?;

        // Special case: "null" always converts to Null regardless of original type
        let new_value = if buffer_content == "null" {
            JsonValue::Null
        } else {
            // Otherwise, determine the new value based on the original node's type
            match node.value() {
                JsonValue::String(_) => JsonValue::String(buffer_content),
                JsonValue::Number(_) => {
                    let num = buffer_content.parse::<f64>()
                        .context("Invalid number format")?;
                    JsonValue::Number(num)
                }
                JsonValue::Boolean(_) => {
                    let bool_val = match buffer_content.as_str() {
                        "true" => true,
                        "false" => false,
                        _ => return Err(anyhow!("Boolean value must be true or false")),
                    };
                    JsonValue::Boolean(bool_val)
                }
                JsonValue::Null => {
                    // This shouldn't happen since we checked for "null" above
                    JsonValue::Null
                }
                JsonValue::Object(_) | JsonValue::Array(_) => {
                    return Err(anyhow!("Cannot edit container types"));
                }
            }
        };

        // Update the node in the tree
        let node_mut = self.tree.get_node_mut(path)
            .ok_or_else(|| anyhow!("Node not found for update"))?;
        *node_mut.value_mut() = new_value;

        // Clear edit buffer and mark dirty
        self.edit_buffer = None;
        self.mark_dirty();
        self.rebuild_tree_view();

        Ok(())
    }

    /// Appends a character to the edit buffer.
    pub fn push_to_edit_buffer(&mut self, ch: char) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.push(ch);
        }
    }

    /// Removes the last character from the edit buffer.
    pub fn pop_from_edit_buffer(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.pop();
        }
    }

    /// Clears the edit buffer entirely.
    pub fn clear_edit_buffer(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.clear();
        }
    }

    /// Returns the current pending command character, if any.
    pub fn pending_command(&self) -> Option<char> {
        self.pending_command
    }

    /// Sets the pending command character.
    pub fn set_pending_command(&mut self, ch: char) {
        self.pending_command = Some(ch);
    }

    /// Clears the pending command.
    pub fn clear_pending_command(&mut self) {
        self.pending_command = None;
    }
}
