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
use std::time::SystemTime;

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

/// A node in the undo tree.
///
/// Each node represents a state in the edit history and tracks:
/// - The snapshot of editor state
/// - Parent node (for undo navigation)
/// - Child nodes (for redo navigation with branching)
/// - Timestamp when this state was created
/// - Sequence number for chronological ordering
#[derive(Debug, Clone)]
pub struct UndoNode {
    pub snapshot: EditorSnapshot,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub timestamp: SystemTime,
    pub seq: u64,
}

impl UndoNode {
    /// Creates a new undo node.
    ///
    /// # Arguments
    ///
    /// * `snapshot` - The editor state at this point
    /// * `parent` - Index of parent node (None for root)
    /// * `seq` - Sequence number for chronological ordering
    pub fn new(snapshot: EditorSnapshot, parent: Option<usize>, seq: u64) -> Self {
        Self {
            snapshot,
            parent,
            children: Vec::new(),
            timestamp: SystemTime::now(),
            seq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::{JsonNode, JsonValue};

    #[test]
    fn test_undo_node_creation() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let snapshot = EditorSnapshot {
            tree,
            cursor_path: vec![],
        };

        let node = UndoNode::new(snapshot, None, 0);

        assert_eq!(node.seq, 0);
        assert_eq!(node.parent, None);
        assert_eq!(node.children.len(), 0);
    }
}
