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
pub struct EditorState {
    tree: JsonTree,
    mode: EditorMode,
    cursor: Cursor,
    dirty: bool,
    filename: Option<String>,
    tree_view: TreeViewState,
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
    /// // Initially collapsed - 1 line
    /// assert_eq!(state.tree_view().lines().len(), 1);
    ///
    /// // Toggle to expand
    /// state.toggle_expand_at_cursor();
    /// assert_eq!(state.tree_view().lines().len(), 2);
    ///
    /// // Toggle to collapse
    /// state.toggle_expand_at_cursor();
    /// assert_eq!(state.tree_view().lines().len(), 1);
    /// ```
    pub fn toggle_expand_at_cursor(&mut self) {
        let current_path = self.cursor.path().to_vec();
        self.tree_view.toggle_expand(&current_path);
        self.tree_view.rebuild(&self.tree);
    }
}
