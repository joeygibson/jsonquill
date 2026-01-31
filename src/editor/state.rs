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
//! let mut state = EditorState::new_with_default_theme(tree);
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
use super::jumplist::JumpList;
use super::marks::MarkSet;
use super::mode::EditorMode;
use super::registers::RegisterSet;
use super::repeat::RepeatableCommand;
use crate::document::node::{JsonNode, JsonValue};
use crate::document::tree::JsonTree;
use crate::ui::tree_view::TreeViewState;

/// Type of active search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchType {
    /// Text-based search (/ or ?)
    Text,
    /// JSONPath structural search (:path or :jp)
    JsonPath(String), // Store the query string for display
}

/// State for the interactive theme picker popup.
#[derive(Debug, Clone)]
pub struct ThemePickerState {
    /// List of available theme names
    pub themes: Vec<String>,
    /// Index of currently selected theme
    pub selected_index: usize,
    /// Theme that was active when picker opened (for cancel)
    pub original_theme: String,
    /// Currently applied theme (for UI label)
    pub current_theme: String,
}

impl ThemePickerState {
    /// Creates a new theme picker state.
    ///
    /// Initializes with the list of available themes and sets the selected
    /// index to the current theme if found, otherwise defaults to 0.
    pub fn new(current_theme: String) -> Self {
        let themes = crate::theme::list_builtin_themes();

        // Find index of current theme
        let selected_index = themes.iter().position(|t| t == &current_theme).unwrap_or(0);

        Self {
            themes,
            selected_index,
            original_theme: current_theme.clone(),
            current_theme,
        }
    }
}

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
/// let mut state = EditorState::new_with_default_theme(tree);
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
    show_theme_picker: bool,
    theme_picker_state: Option<ThemePickerState>,
    // Old clipboard fields - TODO: remove after register migration
    // clipboard: Option<JsonNode>,
    // clipboard_key: Option<String>,
    registers: RegisterSet,
    pending_register: Option<char>,
    append_mode: bool,
    search_buffer: String,
    search_results: Vec<Vec<usize>>,
    search_index: usize,
    search_forward: bool,
    search_type: Option<SearchType>,
    show_line_numbers: bool,
    relative_line_numbers: bool,
    enable_mouse: bool,
    create_backup: bool,
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
    add_key_cursor: usize,
    add_insertion_point: Option<Vec<usize>>,
    temp_container: Option<JsonNode>, // Temporary storage for container during add operation
    is_renaming_key: bool,
    rename_original_key: Option<String>,
    // Tab-completion state
    completion_candidates: Vec<String>,
    completion_index: usize,
    completion_prefix: String,
    // Visual mode, marks, jump list, and repeat command state
    jumplist: JumpList,
    marks: MarkSet,
    pending_mark_set: bool,
    pending_mark_jump: bool,
    visual_anchor: Option<Vec<usize>>,
    visual_selection: Vec<Vec<usize>>,
    last_command: Option<RepeatableCommand>,
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
    /// let state = EditorState::new_with_default_theme(tree);
    ///
    /// assert!(!state.is_dirty());
    /// assert_eq!(state.filename(), None);
    /// ```
    pub fn new(tree: JsonTree, initial_theme_name: String) -> Self {
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
            current_theme: initial_theme_name,
            show_theme_picker: false,
            theme_picker_state: None,
            // Old clipboard init - TODO: remove after register migration
            // clipboard: None,
            // clipboard_key: None,
            registers: RegisterSet::new(),
            pending_register: None,
            append_mode: false,
            search_buffer: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            search_forward: true,
            search_type: None,
            show_line_numbers: true,
            relative_line_numbers: false,
            enable_mouse: true,
            create_backup: false,
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
            add_key_cursor: 0,
            add_insertion_point: None,
            temp_container: None,
            is_renaming_key: false,
            rename_original_key: None,
            completion_candidates: Vec::new(),
            completion_index: 0,
            completion_prefix: String::new(),
            jumplist: JumpList::new(100),
            marks: MarkSet::new(),
            pending_mark_set: false,
            pending_mark_jump: false,
            visual_anchor: None,
            visual_selection: Vec::new(),
            last_command: None,
        }
    }

    /// Creates a new editor state with a default theme (for tests).
    ///
    /// This is a convenience method for tests that don't care about the theme.
    #[doc(hidden)]
    pub fn new_with_default_theme(tree: JsonTree) -> Self {
        Self::new(tree, "default-dark".to_string())
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
    /// let state = EditorState::new_with_default_theme(tree);
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
    /// # let mut state = EditorState::new_with_default_theme(tree);
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

    /// Formats the entire document.
    ///
    /// For regular JSON: Uses jq-style formatting (strict multi-line, 2-space indentation).
    /// For JSONL: Uses compact formatting (jq -c equivalent, one JSON object per line).
    ///
    /// Serializes the tree, then parses it back to create a cleanly formatted tree.
    /// This removes any irregular formatting and applies consistent formatting.
    ///
    /// Marks the document as dirty so the user can save the formatted result.
    pub fn format_document(&mut self) -> anyhow::Result<()> {
        use crate::document::parser::parse_json;
        use crate::file::loader::parse_jsonl_content;
        use crate::file::saver::{serialize_node_compact, serialize_node_jq_style};

        // Check if this is a JSONL document
        let is_jsonl = matches!(self.tree.root().value(), JsonValue::JsonlRoot(_));

        let json_str = if is_jsonl {
            // JSONL: use compact formatting (jq -c equivalent)
            if let JsonValue::JsonlRoot(lines) = self.tree.root().value() {
                let formatted_lines: Vec<String> =
                    lines.iter().map(serialize_node_compact).collect();
                format!("{}\n", formatted_lines.join("\n"))
            } else {
                unreachable!("checked is_jsonl above");
            }
        } else {
            // Regular JSON: use jq-style multi-line formatting
            let indent_size = 2;
            let mut json_str = serialize_node_jq_style(self.tree.root(), indent_size, 0);

            // jq always ensures a trailing newline
            if !json_str.ends_with('\n') {
                json_str.push('\n');
            }
            json_str
        };

        // Parse back to create a clean tree
        let new_tree = if is_jsonl {
            parse_jsonl_content(&json_str)?
        } else {
            parse_json(&json_str)?
        };

        // Reload with the formatted tree
        self.reload_tree(new_tree);

        // Mark as dirty so user can save
        self.mark_dirty();

        Ok(())
    }

    /// Reloads the editor with a new tree, resetting cursor and state.
    ///
    /// This is used when reloading from disk or opening a new file.
    /// It resets to default expansion state: fully expanded for JSON, collapsed for JSONL.
    pub fn reload_tree(&mut self, tree: JsonTree) {
        self.tree = tree;
        self.dirty = false;

        // Reset to default expansion state:
        // - Regular JSON files: fully expanded
        // - JSONL files: fully collapsed
        self.tree_view = TreeViewState::new();
        if !matches!(self.tree.root().value(), JsonValue::JsonlRoot(_)) {
            self.tree_view.expand_all(&self.tree);
        }
        self.tree_view.rebuild(&self.tree);

        // Reset cursor to first visible line
        if let Some(first_line) = self.tree_view.lines().first() {
            self.cursor.set_path(first_line.path.clone());
        } else {
            self.cursor.set_path(vec![]);
        }

        // Reset undo tree with new initial snapshot
        let initial_snapshot = super::undo::EditorSnapshot {
            tree: self.tree.clone(),
            cursor_path: self.cursor.path().to_vec(),
        };
        self.undo_tree = super::undo::UndoTree::new(initial_snapshot, 50);

        self.clear_message();
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
    /// let state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// let state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// let state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// let state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// let state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// Stores the deleted node in register history before deletion.
    /// Adjusts the cursor position after deletion and rebuilds the tree view.
    pub fn delete_node_at_cursor(&mut self) -> anyhow::Result<()> {
        use crate::document::node::JsonValue;
        use crate::editor::registers::RegisterContent;

        let path = self.cursor.path().to_vec();

        // Collect the node to delete
        let node = self
            .tree
            .get_node(&path)
            .ok_or_else(|| anyhow::anyhow!("No node at cursor"))?
            .clone();

        // Store key if deleting from object
        let key = if !path.is_empty() {
            let parent_path = &path[..path.len() - 1];
            let index = path[path.len() - 1];
            if let Some(parent) = self.tree.get_node(parent_path) {
                if let JsonValue::Object(fields) = parent.value() {
                    Some(fields[index].0.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let content = RegisterContent::new(vec![node], vec![key]);

        // Update target register if specified
        if let Some(reg) = self.pending_register {
            if self.append_mode {
                self.registers.append_named(reg, content.clone());
            } else {
                self.registers.set_named(reg, content.clone());
            }
        } else {
            // Update unnamed register
            self.registers.set_unnamed(content.clone());

            // Sync to system clipboard
            let json_value = self.node_to_serde_value(content.nodes[0].value());
            if let Ok(json_str) = serde_json::to_string_pretty(&json_value) {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(json_str);
                }
            }
        }

        // Push to delete history ("1-"9)
        self.registers.push_delete_history(content);

        // Find current line index before deletion
        let lines = self.tree_view.lines();
        let current_idx = lines.iter().position(|l| l.path == path);

        // Delete the node
        self.tree.delete_node(&path)?;
        self.mark_dirty();

        // Update expanded paths to account for shifted indices after deletion
        self.tree_view_mut().update_paths_after_deletion(&path);

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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
    /// let mut state = EditorState::new_with_default_theme(tree);
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
        let is_jsonl_line =
            current_path.len() == 1 && matches!(self.tree.root().value(), JsonValue::JsonlRoot(_));

        let was_expanded = self.tree_view.is_expanded(&current_path);

        if is_jsonl_line && !was_expanded {
            // Expanding a JSONL line - expand entire tree within it
            self.tree_view
                .expand_node_and_descendants(&self.tree, &current_path);
        } else {
            // Normal toggle for non-JSONL or collapsing
            self.tree_view.toggle_expand(&current_path);
        }

        self.tree_view.rebuild(&self.tree);
    }

    /// Fully expands the node at the cursor and all its descendants.
    pub fn expand_all_at_cursor(&mut self) {
        let current_path = self.cursor.path().to_vec();
        self.tree_view
            .expand_node_and_descendants(&self.tree, &current_path);
        self.tree_view.rebuild(&self.tree);
    }

    /// Fully collapses the node at the cursor and all its descendants.
    pub fn collapse_all_at_cursor(&mut self) {
        let current_path = self.cursor.path().to_vec();
        self.tree_view
            .collapse_node_and_descendants(&self.tree, &current_path);
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

    /// Scrolls down by a full page (viewport_height lines).
    ///
    /// Scrolls the viewport down by the entire visible height and moves the cursor
    /// to maintain its relative position on screen (vim Ctrl-f behavior).
    pub fn full_page_down(&mut self) {
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

        // Calculate scroll amount (full viewport height)
        let scroll_amount = self.viewport_height;

        // Scroll the viewport down
        let new_scroll = (self.scroll_offset + scroll_amount)
            .min(lines.len().saturating_sub(self.viewport_height));
        self.scroll_offset = new_scroll;

        // Move cursor down by the same amount to maintain screen position
        let new_cursor_idx = (current_idx + scroll_amount).min(lines.len() - 1);
        self.cursor.set_path(lines[new_cursor_idx].path.clone());
    }

    /// Scrolls up by a full page (viewport_height lines).
    ///
    /// Scrolls the viewport up by the entire visible height and moves the cursor
    /// to maintain its relative position on screen (vim Ctrl-b behavior).
    pub fn full_page_up(&mut self) {
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

        // Calculate scroll amount (full viewport height)
        let scroll_amount = self.viewport_height;

        // Scroll the viewport up
        self.scroll_offset = self.scroll_offset.saturating_sub(scroll_amount);

        // Move cursor up by the same amount to maintain screen position
        let new_cursor_idx = current_idx.saturating_sub(scroll_amount);
        self.cursor.set_path(lines[new_cursor_idx].path.clone());
    }

    /// Centers the current cursor line on the screen (zz command).
    ///
    /// Adjusts the scroll offset so the cursor is in the middle of the viewport.
    /// Does not move the cursor position, only adjusts the viewport.
    pub fn center_cursor_on_screen(&mut self) {
        if self.viewport_height == 0 {
            return;
        }

        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let cursor_idx = lines
            .iter()
            .position(|l| l.path == self.cursor.path())
            .unwrap_or(0);

        // Calculate scroll offset to center cursor
        let half_height = self.viewport_height / 2;
        self.scroll_offset = cursor_idx.saturating_sub(half_height);

        // Ensure we don't scroll past the end
        let max_scroll = lines.len().saturating_sub(self.viewport_height);
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    /// Positions the current cursor line at the top of the screen (zt command).
    ///
    /// Adjusts the scroll offset so the cursor is at the top of the viewport.
    /// Does not move the cursor position, only adjusts the viewport.
    pub fn cursor_to_top_of_screen(&mut self) {
        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let cursor_idx = lines
            .iter()
            .position(|l| l.path == self.cursor.path())
            .unwrap_or(0);

        // Set scroll offset so cursor is at top
        self.scroll_offset = cursor_idx;
    }

    /// Positions the current cursor line at the bottom of the screen (zb command).
    ///
    /// Adjusts the scroll offset so the cursor is at the bottom of the viewport.
    /// Does not move the cursor position, only adjusts the viewport.
    pub fn cursor_to_bottom_of_screen(&mut self) {
        if self.viewport_height == 0 {
            return;
        }

        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let cursor_idx = lines
            .iter()
            .position(|l| l.path == self.cursor.path())
            .unwrap_or(0);

        // Set scroll offset so cursor is at bottom
        self.scroll_offset = cursor_idx.saturating_sub(self.viewport_height - 1);
    }

    /// Moves the cursor to the next sibling node.
    ///
    /// Siblings are nodes that share the same parent (same path except for last index).
    /// This increments the last index in the current path and moves there if valid.
    /// If at the last sibling or at root, does nothing.
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
    ///     ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    /// ])));
    /// let mut state = EditorState::new_with_default_theme(tree);
    ///
    /// // Initially at first sibling [0]
    /// assert_eq!(state.cursor().path(), &[0]);
    ///
    /// // Move to next sibling [1]
    /// state.move_to_next_sibling();
    /// assert_eq!(state.cursor().path(), &[1]);
    ///
    /// // Move to next sibling [2]
    /// state.move_to_next_sibling();
    /// assert_eq!(state.cursor().path(), &[2]);
    ///
    /// // At last sibling, stays at [2]
    /// state.move_to_next_sibling();
    /// assert_eq!(state.cursor().path(), &[2]);
    /// ```
    pub fn move_to_next_sibling(&mut self) {
        let current_path = self.cursor.path();

        // Root has no siblings
        if current_path.is_empty() {
            return;
        }

        // Try to increment the last index
        let mut next_path = current_path.to_vec();
        let last_idx = next_path.len() - 1;
        next_path[last_idx] += 1;

        // Check if this path exists in the tree
        if self.tree.get_node(&next_path).is_some() {
            self.cursor.set_path(next_path);
        }
        // If it doesn't exist, we're at the last sibling - do nothing
    }

    /// Moves the cursor to the first sibling node.
    ///
    /// Siblings are nodes that share the same parent (same path except for last index).
    /// This sets the last index to 0 (first sibling).
    /// If at root, does nothing.
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
    ///     ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    /// ])));
    /// let mut state = EditorState::new_with_default_theme(tree);
    ///
    /// // Move to middle sibling
    /// state.cursor_mut().set_path(vec![1]);
    ///
    /// // Jump to first sibling [0]
    /// state.move_to_first_sibling();
    /// assert_eq!(state.cursor().path(), &[0]);
    /// ```
    pub fn move_to_first_sibling(&mut self) {
        let current_path = self.cursor.path();

        // Root has no siblings
        if current_path.is_empty() {
            return;
        }

        // Set last index to 0
        let mut first_path = current_path.to_vec();
        let last_idx = first_path.len() - 1;
        first_path[last_idx] = 0;

        // Check if this path exists in the tree
        if self.tree.get_node(&first_path).is_some() {
            self.cursor.set_path(first_path);
        }
    }

    /// Moves the cursor to the last sibling node.
    ///
    /// Siblings are nodes that share the same parent (same path except for last index).
    /// This finds and moves to the last valid sibling index.
    /// If at root, does nothing.
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
    ///     ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    /// ])));
    /// let mut state = EditorState::new_with_default_theme(tree);
    ///
    /// // Initially at first sibling [0]
    /// assert_eq!(state.cursor().path(), &[0]);
    ///
    /// // Jump to last sibling [2]
    /// state.move_to_last_sibling();
    /// assert_eq!(state.cursor().path(), &[2]);
    /// ```
    pub fn move_to_last_sibling(&mut self) {
        let current_path = self.cursor.path();

        // Root has no siblings
        if current_path.is_empty() {
            return;
        }

        // Get parent path to determine number of siblings
        let parent_path = &current_path[..current_path.len() - 1];

        let parent = if parent_path.is_empty() {
            self.tree.root()
        } else {
            match self.tree.get_node(parent_path) {
                Some(node) => node,
                None => return,
            }
        };

        use crate::document::node::JsonValue;
        let sibling_count = match parent.value() {
            JsonValue::Object(entries) => entries.len(),
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => elements.len(),
            _ => return, // Parent is not a container
        };

        if sibling_count == 0 {
            return;
        }

        // Set last index to the last sibling (count - 1)
        let mut last_path = current_path.to_vec();
        let last_idx = last_path.len() - 1;
        last_path[last_idx] = sibling_count - 1;

        self.cursor.set_path(last_path);
    }

    /// Moves the cursor to the previous sibling node.
    ///
    /// Siblings are nodes that share the same parent (same path except for last index).
    /// This decrements the last index in the current path and moves there if valid.
    /// If at the first sibling or at root, does nothing.
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
    ///     ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    /// ])));
    /// let mut state = EditorState::new_with_default_theme(tree);
    ///
    /// // Move to last sibling first
    /// state.cursor_mut().set_path(vec![2]);
    ///
    /// // Move to previous sibling [1]
    /// state.move_to_previous_sibling();
    /// assert_eq!(state.cursor().path(), &[1]);
    ///
    /// // Move to previous sibling [0]
    /// state.move_to_previous_sibling();
    /// assert_eq!(state.cursor().path(), &[0]);
    ///
    /// // At first sibling, stays at [0]
    /// state.move_to_previous_sibling();
    /// assert_eq!(state.cursor().path(), &[0]);
    /// ```
    pub fn move_to_previous_sibling(&mut self) {
        let current_path = self.cursor.path();

        // Root has no siblings
        if current_path.is_empty() {
            return;
        }

        // Try to decrement the last index
        let last_idx = current_path.len() - 1;
        let current_index = current_path[last_idx];

        // If already at index 0, we're at the first sibling
        if current_index == 0 {
            return;
        }

        let mut prev_path = current_path.to_vec();
        prev_path[last_idx] = current_index - 1;

        // Check if this path exists in the tree (should always exist if index > 0)
        if self.tree.get_node(&prev_path).is_some() {
            self.cursor.set_path(prev_path);
        }
    }

    /// Moves to the next node at the same depth or shallower (w command).
    ///
    /// This is useful for skipping over deep nested structures and jumping to
    /// the next "top-level" node. Similar to vim's word-forward motion but for
    /// tree depth.
    ///
    /// # Examples
    ///
    /// Given a tree like:
    /// ```text
    /// {
    ///   "users": [
    ///     {
    ///       "name": "Alice",  <- depth 3
    ///       "age": 30
    ///     }
    ///   ],
    ///   "config": { ... }      <- pressing 'w' jumps here (depth 1)
    /// }
    /// ```
    pub fn move_to_next_at_same_or_shallower_depth(&mut self) {
        let current_path = self.cursor.path();

        // Find the current line to get its depth
        let lines = self.tree_view.lines();
        let current_line_idx = lines.iter().position(|line| line.path == *current_path);

        if let Some(idx) = current_line_idx {
            let current_depth = lines[idx].depth;

            // Search forward for a line with depth <= current_depth
            for line in &lines[idx + 1..] {
                if line.depth <= current_depth {
                    self.cursor.set_path(line.path.clone());
                    return;
                }
            }
        }
    }

    /// Moves the cursor to the parent node without collapsing.
    ///
    /// This is useful when you want to navigate to a parent node while keeping
    /// its children visible. Unlike pressing 'h' which collapses, this just
    /// moves the cursor up the tree hierarchy.
    ///
    /// # Examples
    ///
    /// Given a tree like:
    /// ```text
    /// {
    ///   "users": [        <- pressing 'H' moves here
    ///     {
    ///       "name": "Alice",
    ///       "age": 30      <- cursor here
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn move_to_parent(&mut self) {
        let current_path = self.cursor.path();

        // Can't move to parent if already at root
        if current_path.is_empty() {
            return;
        }

        // Parent path is current path minus the last index
        let parent_path = &current_path[..current_path.len() - 1];
        self.cursor.set_path(parent_path.to_vec());
    }

    /// Moves to the previous node at the same depth or shallower (b command).
    ///
    /// This is useful for skipping back over deep nested structures to the
    /// previous "top-level" node. Similar to vim's word-backward motion but for
    /// tree depth.
    ///
    /// # Examples
    ///
    /// Given a tree like:
    /// ```text
    /// {
    ///   "users": [ ... ]       <- pressing 'b' jumps here (depth 1)
    ///   "config": {
    ///     "timeout": 30,
    ///     "retry": true        <- depth 2
    ///   }
    /// }
    /// ```
    pub fn move_to_previous_at_same_or_shallower_depth(&mut self) {
        let current_path = self.cursor.path();

        // Find the current line to get its depth
        let lines = self.tree_view.lines();
        let current_line_idx = lines.iter().position(|line| line.path == *current_path);

        if let Some(idx) = current_line_idx {
            let current_depth = lines[idx].depth;

            // Search backward for a line with depth <= current_depth
            for line in lines[..idx].iter().rev() {
                if line.depth <= current_depth {
                    self.cursor.set_path(line.path.clone());
                    return;
                }
            }
        }
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
        self.reset_completion();
    }

    /// Removes the last character from the command buffer.
    pub fn pop_from_command_buffer(&mut self) {
        self.command_buffer.pop();
        self.reset_completion();
    }

    /// Clears the command buffer.
    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
        self.reset_completion();
    }

    /// Handles tab-completion for command mode.
    ///
    /// Generates completion candidates based on the current command buffer
    /// and cycles through them on subsequent Tab presses.
    pub fn handle_tab_completion(&mut self) {
        // If we don't have candidates yet, generate them
        if self.completion_candidates.is_empty() {
            self.completion_prefix = self.command_buffer.clone();
            self.completion_candidates = self.generate_completions(&self.completion_prefix);
            self.completion_index = 0;
        } else {
            // Cycle to next candidate
            if !self.completion_candidates.is_empty() {
                self.completion_index =
                    (self.completion_index + 1) % self.completion_candidates.len();
            }
        }

        // Apply the current completion
        if !self.completion_candidates.is_empty() {
            self.command_buffer = self.completion_candidates[self.completion_index].clone();
        }
    }

    /// Generates completion candidates for the given command prefix.
    fn generate_completions(&self, prefix: &str) -> Vec<String> {
        // Handle `:theme ` completion
        if let Some(partial) = prefix.strip_prefix("theme ") {
            let themes = crate::theme::list_builtin_themes();
            return themes
                .into_iter()
                .filter(|t| t.starts_with(partial))
                .map(|t| format!("theme {}", t))
                .collect();
        }

        // Handle `:set ` completion
        if let Some(partial) = prefix.strip_prefix("set ") {
            let settings = vec![
                "number",
                "nonumber",
                "relativenumber",
                "norelativenumber",
                "rnu",
                "nornu",
                "mouse",
                "nomouse",
                "create_backup",
                "nocreate_backup",
                "save",
            ];
            return settings
                .into_iter()
                .filter(|s| s.starts_with(partial))
                .map(|s| format!("set {}", s))
                .collect();
        }

        // No completions for other commands
        Vec::new()
    }

    /// Resets tab-completion state.
    pub fn reset_completion(&mut self) {
        self.completion_candidates.clear();
        self.completion_index = 0;
        self.completion_prefix.clear();
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

    /// Returns whether the theme picker is currently visible.
    pub fn show_theme_picker(&self) -> bool {
        self.show_theme_picker
    }

    /// Returns a reference to the theme picker state, if active.
    pub fn theme_picker_state(&self) -> Option<&ThemePickerState> {
        self.theme_picker_state.as_ref()
    }

    /// Opens the interactive theme picker popup.
    ///
    /// Initializes the picker with the current theme and list of available themes.
    /// Clears any existing message to avoid visual clutter.
    pub fn open_theme_picker(&mut self) {
        let current = self.current_theme.clone();
        self.theme_picker_state = Some(ThemePickerState::new(current));
        self.show_theme_picker = true;
        self.clear_message();
    }

    /// Moves the theme picker selection to the previous theme.
    ///
    /// Applies the theme immediately for live preview.
    pub fn theme_picker_previous(&mut self) {
        if let Some(picker) = &mut self.theme_picker_state {
            if picker.selected_index > 0 {
                picker.selected_index -= 1;
                let theme = picker.themes[picker.selected_index].clone();
                self.preview_theme(&theme);
            }
        }
    }

    /// Moves the theme picker selection to the next theme.
    ///
    /// Applies the theme immediately for live preview.
    pub fn theme_picker_next(&mut self) {
        if let Some(picker) = &mut self.theme_picker_state {
            if picker.selected_index < picker.themes.len() - 1 {
                picker.selected_index += 1;
                let theme = picker.themes[picker.selected_index].clone();
                self.preview_theme(&theme);
            }
        }
    }

    /// Applies the currently selected theme in the picker.
    ///
    /// Helper method that previews a theme by requesting a theme change
    /// and updating the picker's current_theme field.
    fn preview_theme(&mut self, theme_name: &str) {
        self.request_theme_change(theme_name.to_string());
        if let Some(picker) = &mut self.theme_picker_state {
            picker.current_theme = theme_name.to_string();
        }
    }

    /// Applies the selected theme and closes the picker.
    ///
    /// The theme has already been applied via live preview, so this just
    /// updates the state and closes the picker.
    pub fn theme_picker_apply(&mut self) {
        if let Some(picker) = &self.theme_picker_state {
            // Theme already applied via preview
            self.current_theme = picker.current_theme.clone();
        }
        self.theme_picker_state = None;
        self.show_theme_picker = false;
    }

    /// Cancels theme selection and reverts to the original theme.
    ///
    /// Closes the picker and restores the theme that was active when
    /// the picker was opened.
    pub fn theme_picker_cancel(&mut self) {
        let original_theme = self
            .theme_picker_state
            .as_ref()
            .map(|p| p.original_theme.clone());

        if let Some(theme) = original_theme {
            self.request_theme_change(theme);
        }

        self.theme_picker_state = None;
        self.show_theme_picker = false;
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

    /// Yank nodes starting at cursor for count iterations.
    /// Updates target register (unnamed if not specified), register "0, and system clipboard (unnamed only).
    pub fn yank_nodes(&mut self, count: u32) -> bool {
        use crate::document::node::JsonValue;
        use crate::editor::registers::RegisterContent;

        let mut nodes = Vec::new();
        let mut keys = Vec::new();
        let original_path = self.cursor.path().to_vec();

        // Collect nodes
        for i in 0..count {
            let path = self.cursor.path();
            if let Some(node) = self.tree.get_node(path) {
                nodes.push(node.clone());

                // Store key if yanking from object
                let key = if !path.is_empty() {
                    let parent_path = &path[..path.len() - 1];
                    let index = path[path.len() - 1];
                    if let Some(parent) = self.tree.get_node(parent_path) {
                        if let JsonValue::Object(fields) = parent.value() {
                            Some(fields[index].0.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                keys.push(key);

                // Move down for next iteration (unless it's the last)
                if i < count - 1 {
                    self.move_cursor_down();
                }
            } else {
                break;
            }
        }

        // Restore cursor position
        self.cursor.set_path(original_path);

        if nodes.is_empty() {
            return false;
        }

        let content = RegisterContent::new(nodes, keys);

        // Determine target register
        let target_register = self.pending_register;

        // Update target register
        if let Some(reg) = target_register {
            if self.append_mode {
                self.registers.append_named(reg, content.clone());
            } else {
                self.registers.set_named(reg, content.clone());
            }
        } else {
            // Unnamed register
            self.registers.set_unnamed(content.clone());

            // Sync to system clipboard
            let clipboard_text = if content.nodes.len() == 1 {
                // Single node: serialize as-is
                let json_value = self.node_to_serde_value(content.nodes[0].value());
                serde_json::to_string_pretty(&json_value).ok()
            } else {
                // Multiple nodes: serialize as JSON array
                let array: Vec<serde_json::Value> = content
                    .nodes
                    .iter()
                    .map(|node| self.node_to_serde_value(node.value()))
                    .collect();
                serde_json::to_string_pretty(&array).ok()
            };

            if let Some(json_str) = clipboard_text {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(json_str);
                }
            }
        }

        // Update "0 (last yank)
        self.registers.update_yank_register(content);

        true
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

    /// Computes the path to the current cursor position.
    /// Returns None if at root (empty path).
    ///
    /// # Arguments
    /// * `format` - "dot" for `.foo[3].bar`, "bracket" for `["foo"][3]["bar"]`, "jq" for jq-style
    pub fn compute_path_string(&self, format: &str) -> Option<String> {
        let path = self.cursor.path();
        if path.is_empty() {
            // At root - different formats handle this differently
            return match format {
                "dot" | "jq" => Some(".".to_string()),
                "bracket" => Some("$".to_string()),
                _ => None,
            };
        }

        let mut result = String::new();
        let mut current = self.tree.root();

        // Start with root prefix based on format
        match format {
            "bracket" => result.push('$'),
            "dot" | "jq" => {}
            _ => {}
        }

        for &index in path.iter() {
            use crate::document::node::JsonValue;
            match current.value() {
                JsonValue::Object(entries) => {
                    if let Some((key, node)) = entries.get(index) {
                        match format {
                            "dot" | "jq" => {
                                result.push('.');
                                result.push_str(key);
                            }
                            "bracket" => {
                                result.push('[');
                                result.push('"');
                                // Escape quotes in key
                                for ch in key.chars() {
                                    if ch == '"' || ch == '\\' {
                                        result.push('\\');
                                    }
                                    result.push(ch);
                                }
                                result.push('"');
                                result.push(']');
                            }
                            _ => {}
                        }
                        current = node;
                    } else {
                        return None;
                    }
                }
                JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                    if let Some(node) = elements.get(index) {
                        result.push('[');
                        result.push_str(&index.to_string());
                        result.push(']');
                        current = node;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(result)
    }

    /// Returns the current cursor path in dot notation (e.g., "users[0].name").
    /// Returns empty string for root node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jsonquill::document::node::{JsonNode, JsonValue};
    /// # use jsonquill::document::tree::JsonTree;
    /// # use jsonquill::editor::state::EditorState;
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
    ///     "key".to_string(),
    ///     JsonNode::new(JsonValue::String("value".to_string())),
    /// )])));
    /// let mut state = EditorState::new_with_default_theme(tree);
    /// // Cursor starts at first visible line ("key")
    /// assert_eq!(state.get_current_path(), "key");
    ///
    /// // Explicitly position at root to get empty path
    /// state.cursor_mut().set_path(vec![]);
    /// assert_eq!(state.get_current_path(), "");
    /// ```
    pub fn get_current_path(&self) -> String {
        if self.cursor.path().is_empty() {
            return String::new();
        }

        let path = self.compute_path_string("dot").unwrap_or_default();
        if let Some(stripped) = path.strip_prefix('.') {
            stripped.to_string()
        } else {
            path
        }
    }

    /// Yanks (copies) the path to the current cursor position in dot notation (`.foo[3].bar`).
    /// Returns true if successful.
    pub fn yank_path_dot(&mut self) -> bool {
        // Path yanks to registers not implemented (YAGNI)
        if self.pending_register.is_some() {
            return false;
        }

        if let Some(path_str) = self.compute_path_string("dot") {
            // Try to copy to system clipboard
            use arboard::Clipboard;
            if let Ok(mut clipboard) = Clipboard::new() {
                if clipboard.set_text(path_str.clone()).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    /// Yanks (copies) the path to the current cursor position in bracket notation (`["foo"][3]["bar"]`).
    /// Returns true if successful.
    pub fn yank_path_bracket(&mut self) -> bool {
        // Path yanks to registers not implemented (YAGNI)
        if self.pending_register.is_some() {
            return false;
        }

        if let Some(path_str) = self.compute_path_string("bracket") {
            // Try to copy to system clipboard
            use arboard::Clipboard;
            if let Ok(mut clipboard) = Clipboard::new() {
                if clipboard.set_text(path_str.clone()).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    /// Yanks (copies) the path to the current cursor position in jq-style notation.
    /// Returns true if successful.
    pub fn yank_path_jq(&mut self) -> bool {
        // Path yanks to registers not implemented (YAGNI)
        if self.pending_register.is_some() {
            return false;
        }

        if let Some(path_str) = self.compute_path_string("jq") {
            // Try to copy to system clipboard
            use arboard::Clipboard;
            if let Ok(mut clipboard) = Clipboard::new() {
                if clipboard.set_text(path_str.clone()).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    /// Returns whether there's something in the clipboard.
    pub fn has_clipboard(&self) -> bool {
        !self.registers.get_unnamed().is_empty()
    }

    /// Returns whether there's a temporary container waiting to be added.
    pub fn has_temp_container(&self) -> bool {
        self.temp_container.is_some()
    }

    /// Pastes nodes at cursor from register (after current position).
    pub fn paste_nodes_at_cursor(&mut self) -> anyhow::Result<()> {
        use anyhow::anyhow;

        // Get content from appropriate register
        let content = if let Some(reg) = self.pending_register {
            self.registers
                .get(reg)
                .ok_or_else(|| anyhow!("Nothing in register '{}'", reg))?
                .clone()
        } else {
            self.registers.get_unnamed().clone()
        };

        if content.is_empty() {
            return Err(anyhow!("Nothing to paste"));
        }

        // Create undo checkpoint
        self.checkpoint();

        // Paste each node
        for (node, key) in content.nodes.iter().zip(content.keys.iter()) {
            self.paste_single_node(node.clone(), key.clone(), true)?;
        }

        self.mark_dirty();
        Ok(())
    }

    /// Deprecated alias for paste_nodes_at_cursor (for backwards compatibility).
    pub fn paste_node_at_cursor(&mut self) -> anyhow::Result<()> {
        self.paste_nodes_at_cursor()
    }

    /// Pastes nodes before cursor from register.
    pub fn paste_nodes_before_cursor(&mut self) -> anyhow::Result<()> {
        use anyhow::anyhow;

        let content = if let Some(reg) = self.pending_register {
            self.registers
                .get(reg)
                .ok_or_else(|| anyhow!("Nothing in register '{}'", reg))?
                .clone()
        } else {
            self.registers.get_unnamed().clone()
        };

        if content.is_empty() {
            return Err(anyhow!("Nothing to paste"));
        }

        self.checkpoint();

        for (node, key) in content.nodes.iter().zip(content.keys.iter()) {
            self.paste_single_node(node.clone(), key.clone(), false)?;
        }

        self.mark_dirty();
        Ok(())
    }

    /// Deprecated alias for paste_nodes_before_cursor (for backwards compatibility).
    pub fn paste_node_before_cursor(&mut self) -> anyhow::Result<()> {
        self.paste_nodes_before_cursor()
    }

    /// Helper to paste a single node.
    fn paste_single_node(
        &mut self,
        node: JsonNode,
        key: Option<String>,
        after: bool,
    ) -> anyhow::Result<()> {
        use crate::document::node::JsonValue;
        use anyhow::anyhow;

        let current_path = self.cursor.path().to_vec();

        // Check if cursor is on an expanded container - if so, paste inside it
        if !current_path.is_empty() && after {
            if let Some(current_node) = self.tree.get_node(&current_path) {
                let is_container = current_node.value().is_container();
                let is_expanded = self.tree_view().is_expanded(&current_path);

                if is_container && is_expanded {
                    // Paste inside the expanded container as first child
                    match current_node.value() {
                        JsonValue::Object(_) => {
                            // For objects, need a key
                            let base_key = key.unwrap_or_else(|| "pasted".to_string());
                            let mut key_name = base_key.clone();
                            let mut counter = 1;

                            // Find unique key
                            loop {
                                let test_key = if counter == 1 {
                                    key_name.clone()
                                } else {
                                    format!("{}{}", base_key, counter)
                                };

                                let key_exists =
                                    if let JsonValue::Object(entries) = current_node.value() {
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

                            // Insert at beginning of container (index 0)
                            let mut insert_path = current_path.clone();
                            insert_path.push(0);

                            self.tree.insert_node_in_object(
                                &insert_path,
                                key_name,
                                node.clone(),
                            )?;

                            self.tree_view_mut()
                                .update_paths_after_insertion(&insert_path);
                            self.rebuild_tree_view();
                            self.cursor.set_path(insert_path);
                            return Ok(());
                        }
                        JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                            // Insert at beginning of array (index 0)
                            let mut insert_path = current_path.clone();
                            insert_path.push(0);

                            self.tree.insert_node_in_array(&insert_path, node.clone())?;

                            self.tree_view_mut()
                                .update_paths_after_insertion(&insert_path);
                            self.rebuild_tree_view();
                            self.cursor.set_path(insert_path);
                            return Ok(());
                        }
                        _ => {} // Not a container, fall through to sibling paste
                    }
                }
            }
        }

        // Handle root-level paste: insert inside root container
        if current_path.is_empty() {
            match self.tree.root().value() {
                JsonValue::Object(_) => {
                    // For objects, need a key
                    let base_key = key.unwrap_or_else(|| "pasted".to_string());
                    let mut key_name = base_key.clone();
                    let mut counter = 1;

                    // Find unique key
                    loop {
                        let test_key = if counter == 1 {
                            key_name.clone()
                        } else {
                            format!("{}{}", base_key, counter)
                        };

                        let key_exists =
                            if let JsonValue::Object(entries) = self.tree.root().value() {
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

                    // At root level:
                    // p (after) = paste as first child (index 0)
                    // P (before) = also paste as first child (index 0, can't go before root)
                    let insert_index = 0;

                    let insert_path = vec![insert_index];
                    self.tree
                        .insert_node_in_object(&insert_path, key_name, node.clone())?;

                    // Expand root if not already expanded
                    if !self.tree_view().is_expanded(&[]) {
                        self.tree_view_mut().toggle_expand(&[]);
                    }

                    self.tree_view_mut()
                        .update_paths_after_insertion(&insert_path);
                    self.rebuild_tree_view();
                    self.cursor.set_path(insert_path);
                    return Ok(());
                }
                JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                    // At root level:
                    // p (after) = paste as first child (index 0)
                    // P (before) = also paste as first child (index 0, can't go before root)
                    let insert_index = 0;

                    let insert_path = vec![insert_index];
                    self.tree.insert_node_in_array(&insert_path, node.clone())?;

                    // Expand root if not already expanded
                    if !self.tree_view().is_expanded(&[]) {
                        self.tree_view_mut().toggle_expand(&[]);
                    }

                    self.tree_view_mut()
                        .update_paths_after_insertion(&insert_path);
                    self.rebuild_tree_view();
                    self.cursor.set_path(insert_path);
                    return Ok(());
                }
                _ => {
                    return Err(anyhow!("Cannot paste - root is not a container"));
                }
            }
        }

        let parent_path = &current_path[..current_path.len() - 1];
        let current_index = current_path[current_path.len() - 1];
        let insert_index = if after {
            current_index + 1
        } else {
            current_index
        };

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
                let base_key = key.unwrap_or_else(|| "pasted".to_string());
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
                    .insert_node_in_object(&insert_path, key_name, node.clone())?;

                // Update tree view and cursor
                self.tree_view_mut()
                    .update_paths_after_insertion(&insert_path);
                self.rebuild_tree_view();
                self.cursor.set_path(insert_path);
            }
            JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree.insert_node_in_array(&insert_path, node.clone())?;

                // Update tree view and cursor
                self.tree_view_mut()
                    .update_paths_after_insertion(&insert_path);
                self.rebuild_tree_view();
                self.cursor.set_path(insert_path);
            }
            _ => {
                return Err(anyhow!("Parent is not a container type"));
            }
        }

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
        self.search_type = None;
    }

    /// Sets the search direction.
    pub fn set_search_forward(&mut self, forward: bool) {
        self.search_forward = forward;
    }

    /// Gets the key name at the current cursor position, if it's an object property.
    /// Returns None if the cursor is not on an object key.
    fn get_current_key_name(&self) -> Option<String> {
        let current_path = self.cursor.path();
        let lines = self.tree_view.lines();

        // Find the line at the current cursor position
        let line = lines.iter().find(|l| l.path == current_path)?;

        // Return the key if it exists
        line.key.clone()
    }

    /// Executes a search for the current object key name.
    /// If the cursor is on an object property, searches for all occurrences of that key name.
    ///
    /// # Arguments
    /// * `forward` - If true, search forward; if false, search backward
    ///
    /// # Returns
    /// Returns true if a search was initiated, false if no key was found
    pub fn execute_key_search(&mut self, forward: bool) -> bool {
        // Get the current key name
        let key_name = match self.get_current_key_name() {
            Some(key) => key,
            None => {
                self.set_message("Not on an object key".to_string(), MessageLevel::Warning);
                return false;
            }
        };

        // Set up the search
        self.search_buffer = key_name.clone();
        self.search_forward = forward;
        self.execute_search();

        true
    }

    /// Executes a search for the current search buffer text.
    /// Uses smart case: case-insensitive search unless the pattern contains uppercase letters.
    pub fn execute_search(&mut self) {
        if self.search_buffer.is_empty() {
            return;
        }

        // Smart case: case-insensitive unless pattern has uppercase
        let case_sensitive = self.search_buffer.chars().any(|c| c.is_uppercase());
        let query = if case_sensitive {
            self.search_buffer.clone()
        } else {
            self.search_buffer.to_lowercase()
        };

        self.search_results.clear();
        self.search_index = 0;
        self.search_type = Some(SearchType::Text);

        // Search through all visible lines
        for line in self.tree_view.lines() {
            let mut matches = false;

            // Check key name
            if let Some(key) = &line.key {
                let key_text = if case_sensitive {
                    key.clone()
                } else {
                    key.to_lowercase()
                };
                if key_text.contains(&query) {
                    matches = true;
                }
            }

            // Check string values
            if let crate::ui::tree_view::ValueType::String = line.value_type {
                let value_text = if case_sensitive {
                    line.value_preview.clone()
                } else {
                    line.value_preview.to_lowercase()
                };
                if value_text.contains(&query) {
                    matches = true;
                }
            }

            if matches {
                self.search_results.push(line.path.clone());
            }
        }

        // Jump to first or last result based on search direction
        if !self.search_results.is_empty() {
            if self.search_forward {
                self.search_index = 0;
                self.cursor.set_path(self.search_results[0].clone());
            } else {
                self.search_index = self.search_results.len() - 1;
                self.cursor
                    .set_path(self.search_results[self.search_index].clone());
            }
        }
    }

    /// Executes a JSONPath query and populates search results.
    pub fn execute_jsonpath_search(&mut self, query: &str) {
        use crate::jsonpath::{Evaluator, Parser};

        self.search_results.clear();
        self.search_index = 0;

        // Parse the JSONPath query
        let path = match Parser::parse(query) {
            Ok(p) => p,
            Err(e) => {
                self.set_message(format!("Invalid JSONPath: {}", e), MessageLevel::Error);
                return;
            }
        };

        // Evaluate against the tree root
        let evaluator = Evaluator::new(self.tree.root());
        self.search_results = evaluator.evaluate_paths(&path.segments);

        // Set search type
        self.search_type = Some(SearchType::JsonPath(query.to_string()));

        // Jump to first result or show message
        if !self.search_results.is_empty() {
            self.cursor.set_path(self.search_results[0].clone());
            self.set_message(
                format!("Found {} matches for {}", self.search_results.len(), query),
                MessageLevel::Info,
            );
        } else {
            self.set_message(format!("No matches for {}", query), MessageLevel::Info);
        }
    }

    /// Jumps to the next search result (respects search direction).
    /// Returns (success, wrapped) where wrapped indicates if the search wrapped around.
    pub fn next_search_result(&mut self) -> (bool, bool) {
        if self.search_results.is_empty() {
            return (false, false);
        }

        let wrapped;
        if self.search_forward {
            let old_index = self.search_index;
            self.search_index = (self.search_index + 1) % self.search_results.len();
            wrapped = self.search_index < old_index; // Wrapped if new index is less than old
        } else {
            wrapped = self.search_index == 0; // Wrapped if we're going back from 0
            self.search_index = if self.search_index == 0 {
                self.search_results.len() - 1
            } else {
                self.search_index - 1
            };
        }
        self.cursor
            .set_path(self.search_results[self.search_index].clone());
        (true, wrapped)
    }

    /// Returns the current search results info.
    pub fn search_results_info(&self) -> Option<(usize, usize)> {
        if self.search_results.is_empty() {
            None
        } else {
            Some((self.search_index + 1, self.search_results.len()))
        }
    }

    /// Returns the current search type, if any.
    pub fn search_type(&self) -> Option<&SearchType> {
        self.search_type.as_ref()
    }

    /// Clears search results but preserves search buffer and type.
    /// This removes search info from the status bar while keeping
    /// the search query available for potential "repeat search" features.
    pub fn clear_search_results(&mut self) {
        self.search_results.clear();
        self.search_index = 0;
    }

    /// Returns whether line numbers should be shown.
    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    /// Sets whether line numbers should be shown.
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Returns whether relative line numbers should be shown.
    pub fn relative_line_numbers(&self) -> bool {
        self.relative_line_numbers
    }

    /// Sets whether relative line numbers should be shown.
    pub fn set_relative_line_numbers(&mut self, show: bool) {
        self.relative_line_numbers = show;
    }

    /// Returns whether mouse support is enabled.
    pub fn enable_mouse(&self) -> bool {
        self.enable_mouse
    }

    /// Sets whether mouse support is enabled.
    pub fn set_enable_mouse(&mut self, enable: bool) {
        self.enable_mouse = enable;
    }

    /// Returns whether backup files should be created before saving.
    pub fn create_backup(&self) -> bool {
        self.create_backup
    }

    /// Sets whether backup files should be created before saving.
    pub fn set_create_backup(&mut self, enable: bool) {
        self.create_backup = enable;
    }

    /// Returns a Config object with the current editor settings.
    pub fn to_config(&self) -> crate::config::Config {
        use crate::config::Config;

        Config {
            theme: self.current_theme.clone(),
            show_line_numbers: self.show_line_numbers,
            relative_line_numbers: self.relative_line_numbers,
            enable_mouse: self.enable_mouse,
            create_backup: self.create_backup,
            ..Config::default()
        }
    }

    /// Saves current settings to the config file.
    pub fn save_config(&self) -> anyhow::Result<()> {
        self.to_config().save()
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
                    // Can't edit containers
                }
                crate::document::node::JsonValue::String(s) => {
                    // Pre-populate with current string value (without JSON quotes)
                    let content = s.clone();
                    self.edit_cursor = content.len();
                    self.edit_buffer = Some(content);
                    self.reset_cursor_blink();
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
                    self.reset_cursor_blink();
                }
                crate::document::node::JsonValue::Boolean(b) => {
                    // Pre-populate with current boolean value
                    let content = b.to_string();
                    self.edit_cursor = content.len();
                    self.edit_buffer = Some(content);
                    self.reset_cursor_blink();
                }
                crate::document::node::JsonValue::Null => {
                    // Pre-populate with "null"
                    self.edit_cursor = 4; // "null".len()
                    self.edit_buffer = Some("null".to_string());
                    self.reset_cursor_blink();
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
            self.edit_cursor += ch.len_utf8(); // Advance by byte length, not 1
            self.reset_cursor_blink();
        }
    }

    /// Removes the character before the cursor (backspace).
    pub fn pop_from_edit_buffer(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            if self.edit_cursor > 0 {
                // Find the start of the character before cursor (handles multi-byte UTF-8)
                let char_start = buffer[..self.edit_cursor]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                buffer.remove(char_start);
                self.edit_cursor = char_start;
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
        if let Some(ref buffer) = self.edit_buffer {
            if self.edit_cursor > 0 {
                // Find the start of the previous character (handles multi-byte UTF-8)
                let char_start = buffer[..self.edit_cursor]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.edit_cursor = char_start;
                self.reset_cursor_blink();
            }
        }
    }

    /// Moves the edit cursor right by one character.
    pub fn edit_cursor_right(&mut self) {
        if let Some(ref buffer) = self.edit_buffer {
            if self.edit_cursor < buffer.len() {
                // Find the start of the next character (handles multi-byte UTF-8)
                if let Some((next_pos, _)) = buffer[self.edit_cursor..].char_indices().nth(1) {
                    self.edit_cursor += next_pos;
                } else {
                    // No next character, move to end
                    self.edit_cursor = buffer.len();
                }
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
        self.pending_register = None;
        self.append_mode = false;
    }

    /// Returns a reference to the unnamed register content.
    pub fn get_unnamed_register(&self) -> &crate::editor::registers::RegisterContent {
        self.registers.get_unnamed()
    }

    /// Returns the pending register character if one is waiting.
    pub fn get_pending_register(&self) -> Option<char> {
        self.pending_register
    }

    /// Sets the pending register and append mode for the next yank/delete operation.
    pub fn set_pending_register(&mut self, register: char, append: bool) {
        self.pending_register = Some(register);
        self.append_mode = append;
    }

    /// Returns whether append mode is active for the pending register.
    pub fn get_append_mode(&self) -> bool {
        self.append_mode
    }

    /// Returns a reference to the register set.
    pub fn registers(&self) -> &RegisterSet {
        &self.registers
    }

    /// Clears the pending register and append mode.
    pub fn clear_register_pending(&mut self) {
        self.pending_register = None;
        self.append_mode = false;
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

    /// Returns the current cursor position in the add key buffer.
    pub fn add_key_cursor_position(&self) -> usize {
        self.add_key_cursor
    }

    /// Pushes a character to the add key buffer at cursor position.
    pub fn push_to_add_key_buffer(&mut self, ch: char) {
        self.add_key_buffer.insert(self.add_key_cursor, ch);
        self.add_key_cursor += ch.len_utf8(); // Advance by byte length, not 1
        self.reset_cursor_blink();
    }

    /// Removes the character before cursor in the add key buffer (backspace).
    pub fn pop_from_add_key_buffer(&mut self) {
        if self.add_key_cursor > 0 {
            // Find the start of the character before cursor (handles multi-byte UTF-8)
            let char_start = self.add_key_buffer[..self.add_key_cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.add_key_buffer.remove(char_start);
            self.add_key_cursor = char_start;
            self.reset_cursor_blink();
        }
    }

    /// Clears the add key buffer and resets cursor.
    pub fn clear_add_key_buffer(&mut self) {
        self.add_key_buffer.clear();
        self.add_key_cursor = 0;
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
                JsonValue::Object(_) | JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                    // Root is container, we can add to it
                    // Determine which type
                    match self.tree.root().value() {
                        JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                            // Array or JSONL: go straight to value input
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
                            self.clear_add_key_buffer(); // Reset buffer and cursor
                            self.add_insertion_point = Some(vec![0]); // Insert at position 0
                            self.reset_cursor_blink();
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
        // If it's an EMPTY container, add inside it
        // If it's a NON-EMPTY container and NOT at root, add as sibling after it (to preserve expansion state)
        // If it's a NON-EMPTY container AT root, add inside it (can't add sibling to root)
        if let Some(current_node) = self.tree.get_node(&current_path) {
            match current_node.value() {
                JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                    // If array/JSONL is empty OR we're at root level, add inside it
                    if elements.is_empty() || current_path.is_empty() {
                        let insert_index = elements.len(); // Get length before mutable borrow

                        // Ensure the container is expanded so the new child will be visible
                        if !self.tree_view().is_expanded(&current_path) {
                            self.tree_view_mut().toggle_expand(&current_path);
                        }

                        self.add_mode_stage = AddModeStage::AwaitingValue;
                        let mut insertion_path = current_path.clone();
                        insertion_path.push(insert_index); // Insert at end
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
                    // Non-empty array/JSONL at non-root: fall through to add sibling after
                }
                JsonValue::Object(entries) => {
                    // If object is empty OR we're at root level, add inside it
                    if entries.is_empty() || current_path.is_empty() {
                        let insert_index = entries.len(); // Get length before mutable borrow

                        // Ensure the container is expanded so the new child will be visible
                        if !self.tree_view().is_expanded(&current_path) {
                            self.tree_view_mut().toggle_expand(&current_path);
                        }

                        self.add_mode_stage = AddModeStage::AwaitingKey;
                        self.clear_add_key_buffer(); // Reset buffer and cursor
                        let mut insertion_path = current_path.clone();
                        insertion_path.push(insert_index); // Insert at end
                        self.add_insertion_point = Some(insertion_path);
                        self.reset_cursor_blink();
                        // Stay in Normal mode, wait for key input
                        return;
                    }
                    // Non-empty object at non-root: fall through to add sibling after
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
            JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                // Adding to array/JSONL: insert after current element
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
                self.clear_add_key_buffer(); // Reset buffer and cursor
                let mut insertion_path = parent_path.to_vec();
                insertion_path.push(current_index + 1);
                self.add_insertion_point = Some(insertion_path);
                self.reset_cursor_blink();
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

        // Special case: if cursor is at root (empty path)
        if current_path.is_empty() {
            // At root - check if root is a container
            match self.tree.root().value() {
                JsonValue::Object(_) => {
                    self.add_mode_stage = AddModeStage::AwaitingKey;
                    self.clear_add_key_buffer(); // Reset buffer and cursor
                    self.add_insertion_point = Some(vec![0]);
                    // For object containers in object root, we need a key
                    // Store the container temporarily and wait for key
                    self.temp_container = Some(container_node);
                    self.reset_cursor_blink();
                    return;
                }
                JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                    // Insert directly into array/JSONL at position 0
                    let insertion_path = vec![0];
                    match self
                        .tree
                        .insert_node_in_array(&insertion_path, container_node)
                    {
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
                    return;
                }
                _ => {
                    self.set_message(
                        "Cannot add sibling to root node".to_string(),
                        MessageLevel::Error,
                    );
                    return;
                }
            }
        }

        // Check if the current node (not parent) is a container
        // If it's empty OR at root level, add inside it
        // If it's non-empty and not at root, add as sibling (to preserve expansion state)
        if let Some(current_node) = self.tree.get_node(&current_path) {
            match current_node.value() {
                JsonValue::Array(elements) | JsonValue::JsonlRoot(elements)
                    if elements.is_empty() || current_path.is_empty() =>
                {
                    let insert_index = elements.len(); // Get length before mutable borrow

                    // Empty array/JSONL or root-level array/JSONL: add inside it
                    // Ensure the container is expanded so the new child will be visible
                    if !self.tree_view().is_expanded(&current_path) {
                        self.tree_view_mut().toggle_expand(&current_path);
                    }

                    let mut insertion_path = current_path.clone();
                    insertion_path.push(insert_index); // Insert at end

                    match self
                        .tree
                        .insert_node_in_array(&insertion_path, container_node)
                    {
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
                    return;
                }
                JsonValue::Object(entries) if entries.is_empty() || current_path.is_empty() => {
                    let insert_index = entries.len(); // Get length before mutable borrow

                    // Empty object or root-level object: add inside it
                    // Ensure the container is expanded so the new child will be visible
                    if !self.tree_view().is_expanded(&current_path) {
                        self.tree_view_mut().toggle_expand(&current_path);
                    }

                    self.add_mode_stage = AddModeStage::AwaitingKey;
                    self.clear_add_key_buffer(); // Reset buffer and cursor
                    let mut insertion_path = current_path.clone();
                    insertion_path.push(insert_index); // Insert at end
                    self.add_insertion_point = Some(insertion_path);
                    // For containers in objects, we need a key
                    // Store the container temporarily and wait for key
                    self.temp_container = Some(container_node);
                    self.reset_cursor_blink();
                    return;
                }
                _ => {
                    // Non-empty container at non-root or scalar: fall through to add sibling
                }
            }
        }

        // Current node is either a scalar or non-empty container
        // Add sibling after it in parent container
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
                self.clear_add_key_buffer(); // Reset buffer and cursor
                self.add_insertion_point = Some(path);
                // For containers in objects, we need a key
                // Store the container temporarily and wait for key
                self.temp_container = Some(container_node);
                self.reset_cursor_blink();
            }
            JsonValue::Array(_) | JsonValue::JsonlRoot(_) => {
                // Insert directly into array/JSONL (no key needed)
                match self.tree.insert_node_in_array(&path, container_node) {
                    Ok(_) => {
                        self.tree_view_mut().update_paths_after_insertion(&path);
                        self.rebuild_tree_view();
                        self.cursor.set_path(path.clone());
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
            _ => {
                self.set_message("Parent is not a container".to_string(), MessageLevel::Error);
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
            self.set_message("Cannot rename root node".to_string(), MessageLevel::Error);
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
        use crate::document::node::JsonValue;
        use anyhow::anyhow;

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

        // Get the container from temporary storage
        let container_node = self
            .temp_container
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

    // Visual mode, marks, jump list, and repeat command accessors

    /// Returns a reference to the jump list.
    pub fn jumplist(&self) -> &JumpList {
        &self.jumplist
    }

    /// Returns a mutable reference to the jump list.
    pub fn jumplist_mut(&mut self) -> &mut JumpList {
        &mut self.jumplist
    }

    /// Records the current cursor position in the jump list.
    ///
    /// This should be called before any "big jump" (gg, G, search, marks)
    /// but not for regular movement commands (j, k, h, l).
    pub fn record_jump(&mut self) {
        let cursor = self.cursor.path().to_vec();
        self.jumplist.record_jump(cursor);
    }

    /// Jumps backward in the jump list.
    ///
    /// Returns true if successful, false if already at oldest jump.
    pub fn jump_backward(&mut self) -> bool {
        if let Some(path) = self.jumplist.jump_backward() {
            self.cursor.set_path(path.clone());
            true
        } else {
            false
        }
    }

    /// Jumps forward in the jump list.
    ///
    /// Returns true if successful, false if already at newest jump.
    pub fn jump_forward(&mut self) -> bool {
        if let Some(path) = self.jumplist.jump_forward() {
            self.cursor.set_path(path.clone());
            true
        } else {
            false
        }
    }

    /// Returns a reference to the mark set.
    pub fn marks(&self) -> &MarkSet {
        &self.marks
    }

    /// Returns a mutable reference to the mark set.
    pub fn marks_mut(&mut self) -> &mut MarkSet {
        &mut self.marks
    }

    /// Returns whether we're waiting for a mark name after 'm'.
    pub fn pending_mark_set(&self) -> bool {
        self.pending_mark_set
    }

    /// Sets the pending mark set state.
    pub fn set_pending_mark_set(&mut self, pending: bool) {
        self.pending_mark_set = pending;
    }

    /// Returns whether we're waiting for a mark name after '\''.
    pub fn pending_mark_jump(&self) -> bool {
        self.pending_mark_jump
    }

    /// Sets the pending mark jump state.
    pub fn set_pending_mark_jump(&mut self, pending: bool) {
        self.pending_mark_jump = pending;
    }

    /// Sets a mark at the current cursor position.
    ///
    /// # Arguments
    ///
    /// * `name` - The mark name (a-z)
    pub fn set_mark(&mut self, name: char) {
        let cursor = self.cursor.path().to_vec();
        self.marks.set_mark(name, cursor);
    }

    /// Jumps to a previously set mark.
    ///
    /// # Arguments
    ///
    /// * `name` - The mark name (a-z)
    ///
    /// # Returns
    ///
    /// Returns true if the mark exists and the jump was successful,
    /// false if the mark doesn't exist.
    pub fn jump_to_mark(&mut self, name: char) -> bool {
        if let Some(path) = self.marks.get_mark(name) {
            self.cursor.set_path(path.clone());
            true
        } else {
            false
        }
    }

    /// Yanks nodes from cursor to mark (motion-to-mark: y'a).
    ///
    /// Calculates the range of visible nodes between cursor and mark,
    /// then yanks all nodes in that range.
    pub fn yank_to_mark(&mut self, mark_path: &[usize], count: u32) -> anyhow::Result<()> {
        let range = self.calculate_range_to_mark(mark_path)?;
        self.yank_nodes_in_range(&range, count)
    }

    /// Deletes nodes from cursor to mark (motion-to-mark: d'a).
    ///
    /// Calculates the range of visible nodes between cursor and mark,
    /// then deletes all nodes in that range.
    pub fn delete_to_mark(&mut self, mark_path: &[usize], count: u32) -> anyhow::Result<()> {
        let range = self.calculate_range_to_mark(mark_path)?;
        self.delete_nodes_in_range(&range, count)
    }

    /// Calculates the range of visible node paths between cursor and mark.
    ///
    /// Returns a vector of paths for all visible nodes in the range (inclusive).
    fn calculate_range_to_mark(&self, mark_path: &[usize]) -> anyhow::Result<Vec<Vec<usize>>> {
        use anyhow::anyhow;

        let cursor_path = self.cursor.path();
        let lines = self.tree_view.lines();

        // Find cursor and mark positions in visible lines
        let cursor_idx = lines
            .iter()
            .position(|l| l.path == cursor_path)
            .ok_or_else(|| anyhow!("Cursor position not found in visible lines"))?;

        let mark_idx = lines
            .iter()
            .position(|l| l.path.as_slice() == mark_path)
            .ok_or_else(|| anyhow!("Mark position not found in visible lines"))?;

        // Calculate range (inclusive, handles both directions)
        let (start_idx, end_idx) = if cursor_idx <= mark_idx {
            (cursor_idx, mark_idx)
        } else {
            (mark_idx, cursor_idx)
        };

        // Collect all paths in range
        let range: Vec<Vec<usize>> = lines[start_idx..=end_idx]
            .iter()
            .map(|l| l.path.clone())
            .collect();

        Ok(range)
    }

    /// Yanks all nodes in the given range.
    fn yank_nodes_in_range(&mut self, range: &[Vec<usize>], _count: u32) -> anyhow::Result<()> {
        use crate::document::node::JsonValue;
        use crate::editor::registers::RegisterContent;
        use anyhow::anyhow;

        if range.is_empty() {
            return Err(anyhow!("No nodes in range to yank"));
        }

        let mut nodes = Vec::new();
        let mut keys = Vec::new();

        // Collect all nodes in range
        for path in range {
            if let Some(node) = self.tree.get_node(path) {
                nodes.push(node.clone());

                // Get the key if this is an object property
                let key = if !path.is_empty() {
                    let parent_path = &path[..path.len() - 1];
                    let index = path[path.len() - 1];
                    if let Some(parent) = self.tree.get_node(parent_path) {
                        if let JsonValue::Object(fields) = parent.value() {
                            Some(fields[index].0.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                keys.push(key);
            }
        }

        if nodes.is_empty() {
            return Err(anyhow!("Failed to collect nodes from range"));
        }

        let content = RegisterContent::new(nodes, keys);

        // Store in register
        if let Some(reg) = self.pending_register {
            if self.append_mode {
                self.registers.append_named(reg, content);
            } else {
                self.registers.set_named(reg, content);
            }
        } else {
            // Update unnamed register and sync to system clipboard
            self.registers.set_unnamed(content.clone());

            // Sync first node to system clipboard
            if !content.nodes.is_empty() {
                let json_value = self.node_to_serde_value(content.nodes[0].value());
                if let Ok(json_str) = serde_json::to_string_pretty(&json_value) {
                    use arboard::Clipboard;
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let _ = clipboard.set_text(json_str);
                    }
                }
            }
        }

        Ok(())
    }

    /// Deletes all nodes in the given range.
    fn delete_nodes_in_range(&mut self, range: &[Vec<usize>], _count: u32) -> anyhow::Result<()> {
        // First yank to register (like dd does)
        self.yank_nodes_in_range(range, 1)?;

        // Delete nodes in reverse order to maintain path validity
        let mut sorted_range = range.to_vec();
        sorted_range.sort_by(|a, b| b.cmp(a)); // Reverse order

        for path in sorted_range {
            self.tree.delete_node(&path)?;
            self.tree_view_mut().update_paths_after_deletion(&path);
        }

        self.mark_dirty();

        // Move cursor to first node that was deleted (or nearest valid)
        self.rebuild_tree_view();

        if let Some(first_path) = range.first() {
            // Try to position cursor at the first deleted position
            let lines = self.tree_view.lines();
            if let Some(line_idx) = lines.iter().position(|l| l.path >= *first_path) {
                self.cursor.set_path(lines[line_idx].path.clone());
            } else if !lines.is_empty() {
                // If nothing after, go to last line
                self.cursor.set_path(lines[lines.len() - 1].path.clone());
            }
        }

        // Create undo checkpoint
        self.checkpoint();

        Ok(())
    }

    /// Returns the visual mode anchor position.
    pub fn visual_anchor(&self) -> Option<&Vec<usize>> {
        self.visual_anchor.as_ref()
    }

    /// Returns the visual selection (all selected node paths).
    pub fn visual_selection(&self) -> &[Vec<usize>] {
        &self.visual_selection
    }

    /// Enters visual mode at the current cursor position.
    pub fn enter_visual_mode(&mut self) {
        self.visual_anchor = Some(self.cursor.path().to_vec());
        self.visual_selection = vec![self.cursor.path().to_vec()];
        self.mode = EditorMode::Visual;
    }

    /// Exits visual mode and returns to normal mode.
    pub fn exit_visual_mode(&mut self) {
        self.visual_anchor = None;
        self.visual_selection.clear();
        self.mode = EditorMode::Normal;
    }

    /// Updates the visual selection based on current cursor position.
    pub fn update_visual_selection(&mut self) {
        if let Some(anchor) = &self.visual_anchor {
            // Calculate selection range based on visible lines
            let lines = self.tree_view.lines();

            // Find indices of anchor and cursor in visible lines
            let anchor_idx = lines.iter().position(|line| &line.path == anchor);
            let cursor_idx = lines
                .iter()
                .position(|line| line.path == *self.cursor.path());

            if let (Some(a_idx), Some(c_idx)) = (anchor_idx, cursor_idx) {
                let (start, end) = if a_idx <= c_idx {
                    (a_idx, c_idx)
                } else {
                    (c_idx, a_idx)
                };

                // Collect all paths in the visual range
                let all_paths: Vec<Vec<usize>> = lines[start..=end]
                    .iter()
                    .map(|line| line.path.clone())
                    .collect();

                // Filter out paths that are children of other paths in the selection
                // A path is a child if it starts with another path in the selection
                let mut top_level_paths = Vec::new();
                for path in &all_paths {
                    let is_child = all_paths.iter().any(|other_path| {
                        other_path != path
                            && path.len() > other_path.len()
                            && path.starts_with(other_path)
                    });
                    if !is_child {
                        top_level_paths.push(path.clone());
                    }
                }

                self.visual_selection = top_level_paths;
            }
        }
    }

    /// Yanks all nodes in the visual selection.
    ///
    /// Returns the number of nodes yanked.
    pub fn yank_visual_selection(&mut self) -> usize {
        use crate::document::node::JsonValue;
        use crate::editor::registers::RegisterContent;

        if self.visual_selection.is_empty() {
            return 0;
        }

        // Collect all nodes in the selection
        let mut nodes = Vec::new();
        let mut keys = Vec::new();

        for path in &self.visual_selection {
            if let Some(node) = self.tree.get_node(path) {
                nodes.push(node.clone());

                // Store key if yanking from object
                let key = if !path.is_empty() {
                    let parent_path = &path[..path.len() - 1];
                    let index = path[path.len() - 1];
                    if let Some(parent) = self.tree.get_node(parent_path) {
                        if let JsonValue::Object(fields) = parent.value() {
                            Some(fields[index].0.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                keys.push(key);
            }
        }

        if nodes.is_empty() {
            return 0;
        }

        let count = nodes.len();
        let content = RegisterContent::new(nodes, keys);

        // Determine target register
        let target_register = self.pending_register;

        // Update target register
        if let Some(reg) = target_register {
            if self.append_mode {
                self.registers.append_named(reg, content.clone());
            } else {
                self.registers.set_named(reg, content.clone());
            }
        } else {
            // Unnamed register
            self.registers.set_unnamed(content.clone());

            // Sync to system clipboard
            let clipboard_text = if content.nodes.len() == 1 {
                // Single node: serialize as-is
                let json_value = self.node_to_serde_value(content.nodes[0].value());
                serde_json::to_string_pretty(&json_value).ok()
            } else {
                // Multiple nodes: serialize as JSON array
                let array: Vec<serde_json::Value> = content
                    .nodes
                    .iter()
                    .map(|node| self.node_to_serde_value(node.value()))
                    .collect();
                serde_json::to_string_pretty(&array).ok()
            };

            if let Some(text) = clipboard_text {
                if let Err(e) = arboard::Clipboard::new().and_then(|mut c| c.set_text(text)) {
                    eprintln!("Failed to sync to system clipboard: {}", e);
                }
            }
        }

        // Also store in register "0 (yank register)
        self.registers.update_yank_register(content);

        // Clear pending register and append mode
        self.pending_register = None;
        self.append_mode = false;

        count
    }

    /// Deletes all nodes in the visual selection.
    ///
    /// Returns the number of nodes deleted.
    pub fn delete_visual_selection(&mut self) -> anyhow::Result<usize> {
        if self.visual_selection.is_empty() {
            return Ok(0);
        }

        // First yank the selection
        let _ = self.yank_visual_selection();

        // Delete nodes in reverse order (from bottom to top) to avoid path invalidation
        let mut paths = self.visual_selection.clone();
        paths.sort_by(|a, b| b.cmp(a)); // Sort in reverse order

        let mut deleted_count = 0;
        for path in paths {
            if self.tree.delete_node(&path).is_ok() {
                deleted_count += 1;
            }
        }

        // After deletion, move cursor to first deleted position or closest valid node
        if let Some(first_path) = self.visual_selection.first() {
            // Try to navigate to the position where the first deleted node was
            self.cursor.set_path(first_path.clone());

            // If that path is no longer valid, move to a valid position
            if self.tree.get_node(self.cursor.path()).is_none() {
                // Move up until we find a valid node
                while !self.cursor.path().is_empty() {
                    let mut parent_path = self.cursor.path().to_vec();
                    parent_path.pop();
                    if self.tree.get_node(&parent_path).is_some() {
                        self.cursor.set_path(parent_path);
                        break;
                    }
                    if parent_path.is_empty() {
                        break;
                    }
                }
            }
        }

        Ok(deleted_count)
    }

    /// Returns the last repeatable command.
    pub fn last_command(&self) -> Option<&RepeatableCommand> {
        self.last_command.as_ref()
    }

    /// Sets the last repeatable command.
    pub fn set_last_command(&mut self, cmd: RepeatableCommand) {
        self.last_command = Some(cmd);
    }

    /// Clears the last repeatable command.
    pub fn clear_last_command(&mut self) {
        self.last_command = None;
    }

    /// Repeats the last command.
    ///
    /// Returns a success message if successful, error message otherwise.
    pub fn repeat_last_command(&mut self) -> Result<String, String> {
        use crate::editor::repeat::RepeatableCommand;

        let cmd = match &self.last_command {
            Some(cmd) => cmd.clone(),
            None => return Err("No command to repeat".to_string()),
        };

        match cmd {
            RepeatableCommand::Delete { count } => {
                let mut deleted_count = 0;
                for _ in 0..count {
                    if self.delete_node_at_cursor().is_ok() {
                        deleted_count += 1;
                    } else {
                        break;
                    }
                }
                if deleted_count > 0 {
                    if deleted_count > 1 {
                        Ok(format!("{} nodes deleted (yanked)", deleted_count))
                    } else {
                        Ok("Node deleted (yanked)".to_string())
                    }
                } else {
                    Err("Delete failed".to_string())
                }
            }
            RepeatableCommand::Yank { count } => {
                if self.yank_nodes(count) {
                    if count > 1 {
                        Ok(format!("{} nodes yanked", count))
                    } else {
                        Ok("Node yanked".to_string())
                    }
                } else {
                    Err("Nothing to yank".to_string())
                }
            }
            RepeatableCommand::Paste { before } => {
                let result = if before {
                    self.paste_node_before_cursor()
                } else {
                    self.paste_node_at_cursor()
                };
                match result {
                    Ok(_) => {
                        if before {
                            Ok("Node pasted before".to_string())
                        } else {
                            Ok("Node pasted after".to_string())
                        }
                    }
                    Err(e) => Err(format!("Paste failed: {}", e)),
                }
            }
            RepeatableCommand::Add { value: _, key: _ } => {
                // This is complex - for now, skip it
                Err("Cannot repeat add operation yet".to_string())
            }
            RepeatableCommand::AddArray => {
                // TODO: implement
                Err("Cannot repeat add array operation yet".to_string())
            }
            RepeatableCommand::AddObject => {
                // TODO: implement
                Err("Cannot repeat add object operation yet".to_string())
            }
            RepeatableCommand::Rename { new_key: _ } => {
                // TODO: implement
                Err("Cannot repeat rename operation yet".to_string())
            }
            RepeatableCommand::ChangeValue { new_value: _ } => {
                // TODO: implement
                Err("Cannot repeat change value operation yet".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::{JsonNode, JsonValue};
    use crate::document::tree::JsonTree;

    #[test]
    fn test_get_current_path_dot_notation() {
        // Create tree: {"users": [{"name": "Alice"}]}
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "users".to_string(),
            JsonNode::new(JsonValue::Array(vec![JsonNode::new(JsonValue::Object(
                vec![(
                    "name".to_string(),
                    JsonNode::new(JsonValue::String("Alice".to_string())),
                )],
            ))])),
        )])));

        let mut state = EditorState::new_with_default_theme(tree);

        // Cursor starts at first visible line (the "users" key)
        assert_eq!(state.get_current_path(), "users");

        // Navigate to array element [0]
        state.move_cursor_down();
        assert_eq!(state.get_current_path(), "users[0]");

        // Navigate to "name" key
        state.move_cursor_down();
        assert_eq!(state.get_current_path(), "users[0].name");

        // Test with explicit root positioning
        state.cursor_mut().set_path(vec![]);
        assert_eq!(state.get_current_path(), "");
    }

    #[test]
    fn test_get_current_path_jsonl() {
        // Create JSONL tree with lines: [{"id": 1}, {"id": 2}]
        let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(vec![
            JsonNode::new(JsonValue::Object(vec![(
                "id".to_string(),
                JsonNode::new(JsonValue::Number(1.0)),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "id".to_string(),
                JsonNode::new(JsonValue::Number(2.0)),
            )])),
        ])));

        let mut state = EditorState::new_with_default_theme(tree);

        // JSONL starts collapsed, cursor at first line
        assert_eq!(state.get_current_path(), "[0]");

        // Expand first line to see its contents
        state.toggle_expand_at_cursor();

        // Navigate to "id" field in first line
        state.move_cursor_down();
        assert_eq!(state.get_current_path(), "[0].id");

        // Move cursor down to second line (collapsed)
        state.move_cursor_down();
        assert_eq!(state.get_current_path(), "[1]");

        // Expand second line
        state.toggle_expand_at_cursor();

        // Navigate to "id" field in second line
        state.move_cursor_down();
        assert_eq!(state.get_current_path(), "[1].id");
    }

    #[test]
    fn test_editor_state_has_registers() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let state = EditorState::new_with_default_theme(tree);

        // Should start with empty registers
        assert!(state.get_unnamed_register().is_empty());
        assert_eq!(state.get_pending_register(), None);
        assert!(!state.get_append_mode());
    }

    #[test]
    fn test_yank_to_named_register() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "key".to_string(),
            JsonNode::new(JsonValue::String("value".to_string())),
        )])));

        let mut state = EditorState::new_with_default_theme(tree);
        // Cursor starts at first visible line (the "key" field at path [0])

        // Yank to register 'a'
        state.set_pending_register('a', false);
        let result = state.yank_nodes(1);

        assert!(result);
        // Should be in register 'a'
        let reg_a = state.registers.get_named('a').unwrap();
        assert_eq!(reg_a.nodes.len(), 1);
    }

    #[test]
    fn test_paste_from_named_register() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
            JsonValue::String("existing".to_string()),
        )])));

        let mut state = EditorState::new_with_default_theme(tree);

        // Expand the array to see its contents
        state.toggle_expand_at_cursor();

        // Move cursor to the first element in the array
        state.move_cursor_down();

        // Manually populate register 'a'
        let node = JsonNode::new(JsonValue::String("test".to_string()));
        let content = crate::editor::registers::RegisterContent::new(vec![node], vec![None]);
        state.registers.set_named('a', content);

        // Paste from register 'a'
        state.set_pending_register('a', false);
        let result = state.paste_nodes_at_cursor();

        assert!(result.is_ok());
    }

    #[test]
    fn test_paste_from_numbered_register() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
            JsonValue::String("existing".to_string()),
        )])));

        let mut state = EditorState::new_with_default_theme(tree);

        // Expand the array to see its contents
        state.toggle_expand_at_cursor();

        // Move cursor to the first element in the array
        state.move_cursor_down();

        // Manually populate register "0
        let node = JsonNode::new(JsonValue::Number(42.0));
        let content = crate::editor::registers::RegisterContent::new(vec![node], vec![None]);
        state.registers.set_numbered(0, content);

        // Paste from register "0
        state.set_pending_register('0', false);
        let result = state.paste_nodes_at_cursor();

        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_pushes_to_history() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
        ])));

        let mut state = EditorState::new_with_default_theme(tree);

        // Expand array to see elements
        state.toggle_expand_at_cursor();

        // Move to first element
        state.move_cursor_down();

        // Delete should push to "1
        let _ = state.delete_node_at_cursor();

        let reg_1 = state.registers.get_numbered(1);
        assert_eq!(reg_1.nodes.len(), 1);
    }
}
