//! Undo/redo system with branching undo tree.
//!
//! This module implements vim-style undo/redo with a branching tree structure
//! that preserves all edit history. When you undo then make a new edit, the old
//! "future" is preserved as a branch that can still be accessed.
//!
//! # Architecture
//!
//! - `EditorSnapshot`: Captures tree and cursor state at a point in time
//! - `UndoNode`: Tree node containing snapshot, parent, children, and metadata
//! - `UndoTree`: Manages the tree structure and navigation

use crate::document::tree::JsonTree;

/// Snapshot of editor state at a specific point in time.
///
/// Contains only the state needed to restore the editor to this point:
/// - The JSON document tree
/// - The cursor position within the tree
#[derive(Debug, Clone)]
pub struct EditorSnapshot {
    pub tree: JsonTree,
    pub cursor_path: Vec<usize>,
}
