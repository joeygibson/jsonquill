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
//! use jsonquill::editor::state::EditorState;
//! use jsonquill::editor::mode::EditorMode;
//! use jsonquill::document::node::{JsonNode, JsonValue};
//! use jsonquill::document::tree::JsonTree;
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
use crate::document::node::{JsonNode, JsonValue};
use crate::document::tree::JsonTree;
use crate::ui::tree_view::TreeViewState;

/// Parses a string into a JsonValue, detecting type automatically.
///
/// - "true"/"false" → Boolean
/// - "null" → Null
/// - Valid number → Number
/// - Everything else → String
fn parse_scalar_value(input: &str) -> JsonValue {
    let trimmed = input.trim();

    // Try boolean
    if trimmed == "true" {
        return JsonValue::Boolean(true);
    }
    if trimmed == "false" {
        return JsonValue::Boolean(false);
    }

    // Try null
    if trimmed == "null" {
        return JsonValue::Null;
    }

    // Try number
    if let Ok(num) = trimmed.parse::<f64>() {
        return JsonValue::Number(num);
    }

    // Default to string (use original input, not trimmed)
    JsonValue::String(input.to_string())
}

/// Test helper to expose private function
#[doc(hidden)]
pub fn parse_scalar_value_for_test(input: &str) -> JsonValue {
    parse_scalar_value(input)
}

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
/// use jsonquill::editor::state::EditorState;
/// use jsonquill::editor::mode::EditorMode;
/// use jsonquill::document::node::{JsonNode, JsonValue};
/// use jsonquill::document::tree::JsonTree;
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

/// Stage of the add operation state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddModeStage {
    /// Not in add mode
    None,
    /// Pressed 'a' in object, waiting for key input
    AwaitingKey,
    /// Key entered or skipped (arrays), waiting for value input
    AwaitingValue,
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
    edit_cursor: usize,
    cursor_visible: bool,
    cursor_blink_ticks: u8,
    pending_command: Option<char>,
    pending_count: Option<u32>,
    scroll_offset: usize,
    viewport_height: usize,
    undo_tree: super::undo::UndoTree,
    add_mode_stage: AddModeStage,
    add_key_buffer: String,
    add_insertion_point: Option<Vec<usize>>,
    is_renaming_key: bool,
    rename_original_key: Option<String>,
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![])));
    /// let state = EditorState::new(tree);
    ///
    /// assert!(!state.is_dirty());
    /// assert_eq!(state.filename(), None);
    /// ```
    pub fn new(tree: JsonTree) -> Self {
        let mut tree_view = TreeViewState::new();
        // Expand all nodes by default for regular JSON files
        // JSONL files start collapsed to show previews
        if !matches!(tree.root().value(), JsonValue::JsonlRoot(_)) {
            tree_view.expand_all(&tree);
        }
        tree_view.rebuild(&tree);

        // Initialize cursor to first visible line if available
        let mut cursor = Cursor::new();
        if let Some(first_line) = tree_view.lines().first() {
            cursor.set_path(first_line.path.clone());
        }

        // Initialize undo tree with initial snapshot
        let undo_limit = 50; // Default from Config
        let initial_snapshot = super::undo::EditorSnapshot {
            tree: tree.clone(),
            cursor_path: cursor.path().to_vec(),
        };
        let undo_tree = super::undo::UndoTree::new(initial_snapshot, undo_limit);

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
            edit_cursor: 0,
            cursor_visible: true,
            cursor_blink_ticks: 0,
            pending_command: None,
            pending_count: None,
            scroll_offset: 0,
            viewport_height: 20,
            undo_tree,
            add_mode_stage: AddModeStage::None,
            add_key_buffer: String::new(),
            add_insertion_point: None,
            is_renaming_key: false,
            rename_original_key: None,
        }
    }

    /// Returns a reference to the JSON tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// # use jsonquill::document::node::{JsonNode, JsonValue};
    /// # use jsonquill::document::tree::JsonTree;
    /// # use jsonquill::editor::state::EditorState;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::editor::mode::EditorMode;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::editor::mode::EditorMode;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::editor::state::EditorState;
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

        self.checkpoint();
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use jsonquill::document::tree::JsonTree;
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

        // Check if we're expanding a JSONL line (direct child of JsonlRoot)
        let is_jsonl_line = current_path.len() == 1
            && matches!(self.tree.root().value(), JsonValue::JsonlRoot(_));

        let was_expanded = self.tree_view.is_expanded(&current_path);

        if is_jsonl_line && !was_expanded {
            // Expanding a JSONL line - expand entire tree within it
            self.tree_view.expand_node_and_descendants(&self.tree, &current_path);
        } else {
            // Normal toggle for non-JSONL or collapsing
            self.tree_view.toggle_expand(&current_path);
        }

        self.tree_view.rebuild(&self.tree);
    }

    /// Returns the current scroll offset (top line of viewport).
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Adjusts scroll offset to ensure the cursor is visible in the viewport.
    ///
    /// # Arguments
    ///
    /// * `viewport_height` - The height of the visible area in lines
    pub fn adjust_scroll_to_cursor(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        // Store viewport height for page up/down
        self.viewport_height = viewport_height;

        let lines = self.tree_view.lines();
        if lines.is_empty() {
            self.scroll_offset = 0;
            return;
        }

        // Find current cursor line index
        let cursor_idx = lines
            .iter()
            .position(|l| l.path == self.cursor.path())
            .unwrap_or(0);

        // Ensure cursor is visible in viewport
        if cursor_idx < self.scroll_offset {
            // Cursor is above viewport, scroll up
            self.scroll_offset = cursor_idx;
        } else if cursor_idx >= self.scroll_offset + viewport_height {
            // Cursor is below viewport, scroll down
            self.scroll_offset = cursor_idx - viewport_height + 1;
        }
    }

    /// Jumps to the first line in the tree.
    pub fn jump_to_top(&mut self) {
        let lines = self.tree_view.lines();
        if let Some(first_line) = lines.first() {
            self.cursor.set_path(first_line.path.clone());
            self.scroll_offset = 0;
        }
    }

    /// Jumps to the last line in the tree.
    pub fn jump_to_bottom(&mut self) {
        let lines = self.tree_view.lines();
        if let Some(last_line) = lines.last() {
            self.cursor.set_path(last_line.path.clone());
        }
    }

    /// Jumps to a specific line number (1-based).
    ///
    /// If the line number is valid, moves the cursor to that line.
    /// If the line number is out of bounds, does nothing.
    pub fn jump_to_line(&mut self, line_num: usize) {
        let lines = self.tree_view.lines();
        if line_num == 0 || line_num > lines.len() {
            return;
        }
        let idx = line_num - 1; // Convert to 0-based index
        if let Some(line) = lines.get(idx) {
            self.cursor.set_path(line.path.clone());
        }
    }

    /// Scrolls down one page (half viewport height).
    ///
    /// This scrolls the viewport down by half its height and moves the cursor
    /// to maintain its relative position on screen (vim Ctrl-d behavior).
    pub fn page_down(&mut self) {
        if self.viewport_height == 0 {
            return;
        }

        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let current_idx = lines
            .iter()
            .position(|l| l.path == self.cursor.path())
            .unwrap_or(0);

        // Calculate scroll amount (half viewport height)
        let scroll_amount = self.viewport_height / 2;

        // Scroll the viewport down
        let new_scroll = (self.scroll_offset + scroll_amount)
            .min(lines.len().saturating_sub(self.viewport_height));
        self.scroll_offset = new_scroll;

        // Move cursor down by the same amount to maintain screen position
        let new_cursor_idx = (current_idx + scroll_amount).min(lines.len() - 1);
        self.cursor.set_path(lines[new_cursor_idx].path.clone());
    }

    /// Scrolls up one page (half viewport height).
    ///
    /// This scrolls the viewport up by half its height and moves the cursor
    /// to maintain its relative position on screen (vim Ctrl-u behavior).
    pub fn page_up(&mut self) {
        if self.viewport_height == 0 {
            return;
        }

        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let current_idx = lines
            .iter()
            .position(|l| l.path == self.cursor.path())
            .unwrap_or(0);

        // Calculate scroll amount (half viewport height)
        let scroll_amount = self.viewport_height / 2;

        // Scroll the viewport up
        self.scroll_offset = self.scroll_offset.saturating_sub(scroll_amount);

        // Move cursor up by the same amount to maintain screen position
        let new_cursor_idx = current_idx.saturating_sub(scroll_amount);
        self.cursor.set_path(lines[new_cursor_idx].path.clone());
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
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                let arr: Vec<serde_json::Value> = elements
                    .iter()
                    .map(|v| self.node_to_serde_value(v.value()))
                    .collect();
                serde_json::Value::Array(arr)
            }
            JsonValue::String(s) => serde_json::Value::String(s.clone()),
            JsonValue::Number(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
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
        use crate::document::node::JsonValue;
        use anyhow::anyhow;

        let clipboard_node = self
            .clipboard
            .clone()
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
            self.tree
                .get_node(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        match parent.value() {
            JsonValue::Object(_) => {
                // Use the original key name if available, otherwise use "pasted"
                let base_key = self
                    .clipboard_key
                    .clone()
                    .unwrap_or_else(|| "pasted".to_string());
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

                self.tree
                    .insert_node_in_object(&insert_path, key_name, clipboard_node)?;
            }
            JsonValue::Array(_) => {
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree
                    .insert_node_in_array(&insert_path, clipboard_node)?;
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

        self.checkpoint();
        Ok(())
    }

    /// Pastes the clipboard node before the current cursor position.
    /// For objects, generates a unique key name. For arrays, inserts before current index.
    pub fn paste_node_before_cursor(&mut self) -> anyhow::Result<()> {
        use crate::document::node::JsonValue;
        use anyhow::anyhow;

        let clipboard_node = self
            .clipboard
            .clone()
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
            self.tree
                .get_node(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        match parent.value() {
            JsonValue::Object(_) => {
                // Use the original key name if available, otherwise use "pasted"
                let base_key = self
                    .clipboard_key
                    .clone()
                    .unwrap_or_else(|| "pasted".to_string());
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

                self.tree
                    .insert_node_in_object(&insert_path, key_name, clipboard_node)?;
            }
            JsonValue::Array(_) => {
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree
                    .insert_node_in_array(&insert_path, clipboard_node)?;
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

        self.checkpoint();
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
        self.cursor
            .set_path(self.search_results[self.search_index].clone());
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
                crate::document::node::JsonValue::Object(_)
                | crate::document::node::JsonValue::Array(_)
                | crate::document::node::JsonValue::JsonlRoot(_) => {
                    return; // Can't edit containers
                }
                crate::document::node::JsonValue::String(s) => {
                    // Pre-populate with current string value (without JSON quotes)
                    let content = s.clone();
                    self.edit_cursor = content.len();
                    self.edit_buffer = Some(content);
                }
                crate::document::node::JsonValue::Number(n) => {
                    // Pre-populate with current number value
                    let num_str = if n.fract() == 0.0 && n.is_finite() {
                        format!("{:.0}", n)
                    } else {
                        n.to_string()
                    };
                    self.edit_cursor = num_str.len();
                    self.edit_buffer = Some(num_str);
                }
                crate::document::node::JsonValue::Boolean(b) => {
                    // Pre-populate with current boolean value
                    let content = b.to_string();
                    self.edit_cursor = content.len();
                    self.edit_buffer = Some(content);
                }
                crate::document::node::JsonValue::Null => {
                    // Pre-populate with "null"
                    self.edit_cursor = 4; // "null".len()
                    self.edit_buffer = Some("null".to_string());
                }
            }
        }
    }

    /// Cancels editing and clears the edit buffer without saving changes.
    pub fn cancel_editing(&mut self) {
        self.edit_buffer = None;
        self.edit_cursor = 0;
    }

    /// Commits the edited value from the buffer to the tree.
    /// Parses the buffer according to the original node's type and updates the tree.
    /// Returns an error if the buffer content is invalid for the node's type.
    pub fn commit_editing(&mut self) -> anyhow::Result<()> {
        use crate::document::node::JsonValue;
        use anyhow::{anyhow, Context};

        let buffer_content = self
            .edit_buffer
            .as_ref()
            .ok_or_else(|| anyhow!("No active edit buffer"))?
            .clone();

        let path = self.cursor.path();
        let node = self
            .tree
            .get_node(path)
            .ok_or_else(|| anyhow!("Node not found at cursor"))?;

        // Special case: "null" always converts to Null regardless of original type
        let new_value = if buffer_content == "null" {
            JsonValue::Null
        } else {
            // Otherwise, determine the new value based on the original node's type
            match node.value() {
                JsonValue::String(_) => JsonValue::String(buffer_content),
                JsonValue::Number(_) => {
                    let num = buffer_content
                        .parse::<f64>()
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
                JsonValue::Object(_) | JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                    return Err(anyhow!("Cannot edit container types"));
                }
            }
        };

        // Update the node in the tree
        let node_mut = self
            .tree
            .get_node_mut(path)
            .ok_or_else(|| anyhow!("Node not found for update"))?;
        *node_mut.value_mut() = new_value;

        // Clear edit buffer and mark dirty
        self.edit_buffer = None;
        self.mark_dirty();
        self.rebuild_tree_view();

        self.checkpoint();
        Ok(())
    }

    /// Inserts a character at the current cursor position in the edit buffer.
    pub fn push_to_edit_buffer(&mut self, ch: char) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.insert(self.edit_cursor, ch);
            self.edit_cursor += 1;
            self.reset_cursor_blink();
        }
    }

    /// Removes the character before the cursor (backspace).
    pub fn pop_from_edit_buffer(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            if self.edit_cursor > 0 {
                buffer.remove(self.edit_cursor - 1);
                self.edit_cursor -= 1;
                self.reset_cursor_blink();
            }
        }
    }

    /// Clears the edit buffer entirely and resets cursor.
    pub fn clear_edit_buffer(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.clear();
            self.edit_cursor = 0;
            self.reset_cursor_blink();
        }
    }

    /// Moves the edit cursor left by one character.
    pub fn edit_cursor_left(&mut self) {
        if self.edit_cursor > 0 {
            self.edit_cursor -= 1;
            self.reset_cursor_blink();
        }
    }

    /// Moves the edit cursor right by one character.
    pub fn edit_cursor_right(&mut self) {
        if let Some(ref buffer) = self.edit_buffer {
            if self.edit_cursor < buffer.len() {
                self.edit_cursor += 1;
                self.reset_cursor_blink();
            }
        }
    }

    /// Moves the edit cursor to the beginning of the buffer (Ctrl-a).
    pub fn edit_cursor_home(&mut self) {
        self.edit_cursor = 0;
        self.reset_cursor_blink();
    }

    /// Moves the edit cursor to the end of the buffer (Ctrl-e).
    pub fn edit_cursor_end(&mut self) {
        if let Some(ref buffer) = self.edit_buffer {
            self.edit_cursor = buffer.len();
            self.reset_cursor_blink();
        }
    }

    /// Deletes the character at the cursor position (Ctrl-d).
    pub fn edit_delete_at_cursor(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            if self.edit_cursor < buffer.len() {
                buffer.remove(self.edit_cursor);
                self.reset_cursor_blink();
            }
        }
    }

    /// Deletes from cursor to end of buffer (Ctrl-k).
    pub fn edit_kill_to_end(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.truncate(self.edit_cursor);
            self.reset_cursor_blink();
        }
    }

    /// Returns the current edit cursor position.
    pub fn edit_cursor_position(&self) -> usize {
        self.edit_cursor
    }

    /// Returns whether the cursor is currently visible (for blinking).
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Updates the cursor blink state. Call this periodically to make cursor blink.
    /// Toggles visibility every ~5 ticks (adjust based on render frequency).
    pub fn update_cursor_blink(&mut self) {
        self.cursor_blink_ticks = self.cursor_blink_ticks.wrapping_add(1);
        if self.cursor_blink_ticks >= 5 {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_ticks = 0;
        }
    }

    /// Resets cursor to visible (called on any edit action to show immediate feedback).
    pub fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_ticks = 0;
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

    /// Returns the current pending count, defaulting to 1 if none.
    pub fn get_count(&self) -> u32 {
        self.pending_count.unwrap_or(1)
    }

    /// Returns the raw pending count (None if no count entered).
    pub fn pending_count(&self) -> Option<u32> {
        self.pending_count
    }

    /// Adds a digit to the pending count.
    /// First digit starts the count, subsequent digits multiply by 10 and add.
    pub fn push_count_digit(&mut self, digit: u32) {
        if let Some(count) = self.pending_count {
            self.pending_count = Some(count.saturating_mul(10).saturating_add(digit));
        } else {
            self.pending_count = Some(digit);
        }
    }

    /// Clears the pending count.
    pub fn clear_pending_count(&mut self) {
        self.pending_count = None;
    }

    /// Clears both pending command and count (used together often).
    pub fn clear_pending(&mut self) {
        self.pending_command = None;
        self.pending_count = None;
    }

    /// Returns the current cursor position as (row, col) where row is 1-based line number.
    ///
    /// Returns (0, 0) if the cursor is not found in the tree view.
    pub fn cursor_position(&self) -> (usize, usize) {
        let lines = self.tree_view.lines();
        let current_path = self.cursor.path();

        if let Some(idx) = lines.iter().position(|l| l.path == current_path) {
            let row = idx + 1; // 1-based line number
            let col = 1; // Tree view doesn't have horizontal position
            (row, col)
        } else {
            (0, 0)
        }
    }

    /// Returns the total number of lines in the tree view.
    pub fn total_lines(&self) -> usize {
        self.tree_view.lines().len()
    }

    /// Captures the current editor state as an undo checkpoint.
    ///
    /// This is called automatically before mutation operations to enable undo/redo.
    /// Checkpoints capture both the tree structure and cursor position.
    fn checkpoint(&mut self) {
        let snapshot = super::undo::EditorSnapshot {
            tree: self.tree.clone(),
            cursor_path: self.cursor.path().to_vec(),
        };
        self.undo_tree.add_checkpoint(snapshot);
    }

    /// Undoes the last operation.
    ///
    /// Restores the editor to the previous checkpoint state, including both
    /// the tree structure and cursor position. Returns true if undo succeeded,
    /// false if already at the root state.
    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_tree.undo() {
            self.tree = snapshot.tree;
            self.cursor.set_path(snapshot.cursor_path);
            self.rebuild_tree_view();
            true
        } else {
            false
        }
    }

    /// Redoes the last undone operation.
    ///
    /// Restores the editor to the next checkpoint state (newest branch if multiple
    /// exist), including both the tree structure and cursor position. Returns true
    /// if redo succeeded, false if no redo history exists.
    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_tree.redo() {
            self.tree = snapshot.tree;
            self.cursor.set_path(snapshot.cursor_path);
            self.rebuild_tree_view();
            true
        } else {
            false
        }
    }

    /// Returns the current add mode stage.
    pub fn add_mode_stage(&self) -> &AddModeStage {
        &self.add_mode_stage
    }

    /// Returns the current add key buffer.
    pub fn add_key_buffer(&self) -> &str {
        &self.add_key_buffer
    }

    /// Pushes a character to the add key buffer.
    pub fn push_to_add_key_buffer(&mut self, ch: char) {
        self.add_key_buffer.push(ch);
        self.reset_cursor_blink();
    }

    /// Removes the last character from the add key buffer.
    pub fn pop_from_add_key_buffer(&mut self) {
        self.add_key_buffer.pop();
        self.reset_cursor_blink();
    }

    /// Clears the add key buffer.
    pub fn clear_add_key_buffer(&mut self) {
        self.add_key_buffer.clear();
    }

    /// Starts an add operation at the current cursor position.
    ///
    /// Determines whether we're adding to an array or object, and sets the
    /// appropriate add_mode_stage. For arrays, immediately enters Insert mode.
    /// For objects, stays in Normal mode and waits for key input.
    pub fn start_add_operation(&mut self) {
        use crate::document::node::JsonValue;

        // Clear any previous messages so the edit area is visible
        self.clear_message();

        let current_path = self.cursor.path().to_vec();

        // Special case: if cursor is at root (empty path)
        if current_path.is_empty() {
            // Check if root is a container
            match self.tree.root().value() {
                JsonValue::Object(_) | JsonValue::Array(_) => {
                    // Root is container, we can add to it
                    // Determine which type
                    match self.tree.root().value() {
                        JsonValue::Array(_) => {
                            // Array: go straight to value input
                            self.add_mode_stage = AddModeStage::AwaitingValue;
                            self.add_insertion_point = Some(vec![0]); // Insert at position 0

                            // Enter Insert mode with empty edit buffer
                            self.edit_buffer = Some(String::new());
                            self.edit_cursor = 0;
                            self.set_mode(EditorMode::Insert);
                            self.reset_cursor_blink();
                            // Set mode indicator message
                            self.set_message("-- INSERT --".to_string(), MessageLevel::Info);
                        }
                        JsonValue::Object(_) => {
                            // Object: need key first
                            self.add_mode_stage = AddModeStage::AwaitingKey;
                            self.add_insertion_point = Some(vec![0]); // Insert at position 0
                        }
                        _ => unreachable!(),
                    }
                }
                _ => {
                    // Root is scalar, can't add sibling
                    self.set_message(
                        "Cannot add sibling to root node".to_string(),
                        MessageLevel::Error,
                    );
                }
            }
            return;
        }

        // Check if the current node (not parent) is a container
        // If so, add INSIDE it rather than after it
        if let Some(current_node) = self.tree.get_node(&current_path) {
            match current_node.value() {
                JsonValue::Array(_) => {
                    // Current node is an array - add first child inside it
                    self.add_mode_stage = AddModeStage::AwaitingValue;
                    let mut insertion_path = current_path.clone();
                    insertion_path.push(0); // Insert at position 0 (first child)
                    self.add_insertion_point = Some(insertion_path);

                    // Enter Insert mode with empty edit buffer
                    self.edit_buffer = Some(String::new());
                    self.edit_cursor = 0;
                    self.set_mode(EditorMode::Insert);
                    self.reset_cursor_blink();
                    // Set mode indicator message
                    self.set_message("-- INSERT --".to_string(), MessageLevel::Info);
                    return;
                }
                JsonValue::Object(_) => {
                    // Current node is an object - add first child inside it
                    self.add_mode_stage = AddModeStage::AwaitingKey;
                    let mut insertion_path = current_path.clone();
                    insertion_path.push(0); // Insert at position 0 (first child)
                    self.add_insertion_point = Some(insertion_path);
                    // Stay in Normal mode, wait for key input
                    return;
                }
                _ => {
                    // Current node is a scalar, fall through to add sibling
                }
            }
        }

        // Current node is a scalar - add sibling after it in parent container
        let parent_path = &current_path[..current_path.len() - 1];
        let current_index = current_path[current_path.len() - 1];

        // Get parent node
        let parent = if parent_path.is_empty() {
            self.tree.root()
        } else {
            match self.tree.get_node(parent_path) {
                Some(node) => node,
                None => {
                    self.set_message("Invalid cursor position".to_string(), MessageLevel::Error);
                    return;
                }
            }
        };

        // Determine parent type and set up add operation
        match parent.value() {
            JsonValue::Array(_) => {
                // Adding to array: insert after current element
                self.add_mode_stage = AddModeStage::AwaitingValue;
                let mut insertion_path = parent_path.to_vec();
                insertion_path.push(current_index + 1);
                self.add_insertion_point = Some(insertion_path);

                // Enter Insert mode with empty edit buffer
                self.edit_buffer = Some(String::new());
                self.edit_cursor = 0;
                self.set_mode(EditorMode::Insert);
                self.reset_cursor_blink();
                // Set mode indicator message
                self.set_message("-- INSERT --".to_string(), MessageLevel::Info);
            }
            JsonValue::Object(_) => {
                // Adding to object: need key first
                self.add_mode_stage = AddModeStage::AwaitingKey;
                let mut insertion_path = parent_path.to_vec();
                insertion_path.push(current_index + 1);
                self.add_insertion_point = Some(insertion_path);
                // Stay in Normal mode, wait for key input
            }
            _ => {
                self.set_message("Parent is not a container".to_string(), MessageLevel::Error);
            }
        }
    }

    /// Commits the add operation by creating and inserting the new node.
    ///
    /// Parses the edit buffer value, creates a JsonNode, inserts it at the
    /// add_insertion_point, creates an undo checkpoint, and moves cursor to
    /// the new node.
    pub fn commit_add_operation(&mut self) -> anyhow::Result<()> {
        use anyhow::anyhow;

        // Verify we're in AwaitingValue stage
        if !matches!(self.add_mode_stage, AddModeStage::AwaitingValue) {
            return Err(anyhow!("Not in AwaitingValue stage"));
        }

        // Get the value from edit buffer
        let value_str = self
            .edit_buffer
            .as_ref()
            .ok_or_else(|| anyhow!("No edit buffer"))?;

        // Parse the value
        let value = parse_scalar_value(value_str);
        let node = JsonNode::new(value);

        // Get insertion point
        let insertion_path = self
            .add_insertion_point
            .as_ref()
            .ok_or_else(|| anyhow!("No insertion point set"))?
            .clone();

        // Determine parent type and insert
        let parent_path = if insertion_path.is_empty() {
            &[]
        } else {
            &insertion_path[..insertion_path.len() - 1]
        };

        let parent = if parent_path.is_empty() {
            self.tree.root()
        } else {
            self.tree
                .get_node(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        match parent.value() {
            JsonValue::Array(_) => {
                self.tree.insert_node_in_array(&insertion_path, node)?;
                self.set_message("Added element".to_string(), MessageLevel::Info);
            }
            JsonValue::Object(_) => {
                let key = self.add_key_buffer.clone();
                self.tree
                    .insert_node_in_object(&insertion_path, key.clone(), node)?;
                self.set_message(format!("Added field '{}'", key), MessageLevel::Info);
            }
            _ => {
                return Err(anyhow!("Parent is not a container"));
            }
        }

        // Update expanded paths to account for shifted indices after insertion
        self.tree_view_mut()
            .update_paths_after_insertion(&insertion_path);

        // Rebuild tree view to show new node
        self.rebuild_tree_view();

        // Move cursor to newly created node
        self.cursor.set_path(insertion_path.clone());

        // Mark dirty and create undo checkpoint
        self.mark_dirty();
        self.checkpoint();

        // Clear add operation state and edit buffer
        self.cancel_add_operation();
        self.cancel_editing();

        Ok(())
    }

    /// Transitions from AwaitingKey to AwaitingValue stage.
    ///
    /// Called when user presses Enter after typing object key.
    pub fn transition_add_to_value(&mut self) {
        if matches!(self.add_mode_stage, AddModeStage::AwaitingKey) {
            // Check for empty key
            if self.add_key_buffer.is_empty() {
                self.set_message("Key cannot be empty".to_string(), MessageLevel::Error);
                return;
            }

            // Transition to value stage
            self.add_mode_stage = AddModeStage::AwaitingValue;

            // Enter Insert mode
            self.edit_buffer = Some(String::new());
            self.edit_cursor = 0;
            self.set_mode(EditorMode::Insert);
            self.reset_cursor_blink();
            // Set mode indicator message
            self.set_message("-- INSERT --".to_string(), MessageLevel::Info);
        }
    }

    /// Cancels the add operation and clears all related state.
    pub fn cancel_add_operation(&mut self) {
        self.add_mode_stage = AddModeStage::None;
        self.add_key_buffer.clear();
        self.add_insertion_point = None;
    }

    /// Starts an add container operation (ao for object, aa for array).
    ///
    /// Immediately adds an empty container {} or [] without going through
    /// the value input stage. For objects, prompts for key name first.
    ///
    /// # Arguments
    ///
    /// * `is_object` - true for object {}, false for array []
    pub fn start_add_container_operation(&mut self, is_object: bool) {
        use crate::document::node::JsonValue;

        // Clear any previous messages so the edit area is visible
        self.clear_message();

        let current_path = self.cursor.path().to_vec();

        // Create the container node
        let container_node = if is_object {
            JsonNode::new(JsonValue::Object(vec![]))
        } else {
            JsonNode::new(JsonValue::Array(vec![]))
        };

        // Determine insertion point and parent type
        let (insertion_path, parent_is_object) = if current_path.is_empty() {
            // At root - check if root is a container
            match self.tree.root().value() {
                JsonValue::Object(_) => {
                    self.add_mode_stage = AddModeStage::AwaitingKey;
                    self.add_insertion_point = Some(vec![0]);
                    // For object containers in object root, we need a key
                    // Store the container temporarily and wait for key
                    self.clipboard = Some(container_node);
                    return;
                }
                JsonValue::Array(_) => (vec![0], false),
                _ => {
                    self.set_message(
                        "Cannot add sibling to root node".to_string(),
                        MessageLevel::Error,
                    );
                    return;
                }
            }
        } else {
            let parent_path = &current_path[..current_path.len() - 1];
            let current_index = current_path[current_path.len() - 1];

            let parent = if parent_path.is_empty() {
                self.tree.root()
            } else {
                match self.tree.get_node(parent_path) {
                    Some(node) => node,
                    None => {
                        self.set_message("Invalid cursor position".to_string(), MessageLevel::Error);
                        return;
                    }
                }
            };

            let mut path = parent_path.to_vec();
            path.push(current_index + 1);

            match parent.value() {
                JsonValue::Object(_) => {
                    self.add_mode_stage = AddModeStage::AwaitingKey;
                    self.add_insertion_point = Some(path);
                    // For containers in objects, we need a key
                    // Store the container temporarily and wait for key
                    self.clipboard = Some(container_node);
                    return;
                }
                JsonValue::Array(_) => (path, false),
                _ => {
                    self.set_message("Parent is not a container".to_string(), MessageLevel::Error);
                    return;
                }
            }
        };

        // Insert directly into array (no key needed)
        if !parent_is_object {
            match self.tree.insert_node_in_array(&insertion_path, container_node) {
                Ok(_) => {
                    self.tree_view_mut()
                        .update_paths_after_insertion(&insertion_path);
                    self.rebuild_tree_view();
                    self.cursor.set_path(insertion_path.clone());
                    self.mark_dirty();
                    self.checkpoint();

                    let msg = if is_object {
                        "Added empty object"
                    } else {
                        "Added empty array"
                    };
                    self.set_message(msg.to_string(), MessageLevel::Info);
                }
                Err(e) => {
                    self.set_message(format!("Add failed: {}", e), MessageLevel::Error);
                }
            }
        }
    }

    /// Starts a rename operation on the current object key.
    ///
    /// Checks if the cursor is on an object key (not array element, not root),
    /// then enters Insert mode with the current key name pre-populated in the
    /// edit buffer.
    pub fn start_rename_operation(&mut self) {
        use crate::document::node::JsonValue;

        // Clear any previous messages so the edit area is visible
        self.clear_message();

        let current_path = self.cursor.path().to_vec();

        // Can't rename root
        if current_path.is_empty() {
            self.set_message(
                "Cannot rename root node".to_string(),
                MessageLevel::Error,
            );
            return;
        }

        // Get parent to check if it's an object
        let parent_path = &current_path[..current_path.len() - 1];
        let current_index = current_path[current_path.len() - 1];

        let parent = if parent_path.is_empty() {
            self.tree.root()
        } else {
            match self.tree.get_node(parent_path) {
                Some(node) => node,
                None => {
                    self.set_message("Invalid cursor position".to_string(), MessageLevel::Error);
                    return;
                }
            }
        };

        // Check if parent is an object
        if let JsonValue::Object(entries) = parent.value() {
            // Get the current key name
            if let Some((key, _)) = entries.get(current_index) {
                let key_name = key.clone();

                // Enter rename mode with key name in edit buffer
                self.is_renaming_key = true;
                self.rename_original_key = Some(key_name.clone());
                self.edit_buffer = Some(key_name.clone());
                self.edit_cursor = key_name.len();
                self.set_mode(EditorMode::Insert);
                self.reset_cursor_blink();
                self.set_message("-- RENAME --".to_string(), MessageLevel::Info);
            } else {
                self.set_message("Invalid object index".to_string(), MessageLevel::Error);
            }
        } else {
            self.set_message(
                "Can only rename object keys, not array elements".to_string(),
                MessageLevel::Error,
            );
        }
    }

    /// Commits the rename operation, updating the key name in the object.
    pub fn commit_rename(&mut self) -> anyhow::Result<()> {
        use anyhow::anyhow;
        use crate::document::node::JsonValue;

        let new_key = self
            .edit_buffer
            .as_ref()
            .ok_or_else(|| anyhow!("No edit buffer"))?
            .clone();

        if new_key.is_empty() {
            return Err(anyhow!("Key cannot be empty"));
        }

        let original_key = self
            .rename_original_key
            .as_ref()
            .ok_or_else(|| anyhow!("No original key stored"))?
            .clone();

        // If key didn't change, just exit
        if new_key == original_key {
            self.cancel_rename();
            return Ok(());
        }

        let current_path = self.cursor.path().to_vec();
        if current_path.is_empty() {
            return Err(anyhow!("Cannot rename root"));
        }

        let parent_path = &current_path[..current_path.len() - 1];
        let current_index = current_path[current_path.len() - 1];

        // Get parent and verify it's still an object
        let parent = if parent_path.is_empty() {
            self.tree.root_mut()
        } else {
            self.tree
                .get_node_mut(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        if let JsonValue::Object(entries) = parent.value_mut() {
            // Check if new key already exists
            if entries.iter().any(|(k, _)| k == &new_key) {
                return Err(anyhow!("Key '{}' already exists", new_key));
            }

            // Update the key at the current index
            if let Some((key, _)) = entries.get_mut(current_index) {
                *key = new_key.clone();

                self.mark_dirty();
                self.rebuild_tree_view();
                self.checkpoint();
                self.set_message(
                    format!("Renamed '{}' to '{}'", original_key, new_key),
                    MessageLevel::Info,
                );
            } else {
                return Err(anyhow!("Invalid object index"));
            }
        } else {
            return Err(anyhow!("Parent is not an object"));
        }

        self.cancel_rename();
        Ok(())
    }

    /// Cancels the rename operation and clears related state.
    pub fn cancel_rename(&mut self) {
        self.is_renaming_key = false;
        self.rename_original_key = None;
        self.edit_buffer = None;
        self.edit_cursor = 0;
    }

    /// Returns whether we're currently in rename mode.
    pub fn is_renaming_key(&self) -> bool {
        self.is_renaming_key
    }

    /// Commits a container add operation after receiving the key name.
    ///
    /// Called when user finishes entering a key name for adding a container
    /// to an object. Retrieves the temporarily stored container from clipboard
    /// and inserts it with the provided key.
    pub fn commit_container_add(&mut self) -> anyhow::Result<()> {
        use anyhow::anyhow;

        // Get the container from temporary storage (clipboard)
        let container_node = self
            .clipboard
            .take()
            .ok_or_else(|| anyhow!("No container to add"))?;

        // Get the key from add_key_buffer
        let key = self.add_key_buffer.clone();
        if key.is_empty() {
            return Err(anyhow!("Key cannot be empty"));
        }

        // Get insertion point
        let insertion_path = self
            .add_insertion_point
            .as_ref()
            .ok_or_else(|| anyhow!("No insertion point set"))?
            .clone();

        // Insert the container with the key
        self.tree
            .insert_node_in_object(&insertion_path, key.clone(), container_node.clone())?;

        self.tree_view_mut()
            .update_paths_after_insertion(&insertion_path);
        self.rebuild_tree_view();
        self.cursor.set_path(insertion_path.clone());
        self.mark_dirty();
        self.checkpoint();

        let container_type = match container_node.value() {
            JsonValue::Object(_) => "object",
            JsonValue::Array(_) => "array",
            _ => "container",
        };
        self.set_message(
            format!("Added empty {} '{}'", container_type, key),
            MessageLevel::Info,
        );

        // Clear add operation state
        self.cancel_add_operation();

        Ok(())
    }
}
