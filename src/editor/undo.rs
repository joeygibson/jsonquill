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

/// Branching undo tree for managing edit history.
///
/// The undo tree stores all editor states as a tree structure where:
/// - Root node is the initial state when file was opened
/// - Each child represents a modification
/// - Branching occurs when you undo then make a new edit
/// - Current pointer tracks where we are in history
///
/// # Example
///
/// ```text
///     0 (initial)
///     |
///     1 (edit A)
///    / \
///   2   3 (branching: undo, then two different edits)
///   |
///   4
/// ```
#[derive(Debug)]
pub struct UndoTree {
    pub nodes: Vec<UndoNode>,
    current: usize,
    next_seq: u64,
    limit: usize,
}

impl UndoTree {
    /// Creates a new undo tree with an initial snapshot.
    ///
    /// # Arguments
    ///
    /// * `initial_snapshot` - The starting state (root node)
    /// * `limit` - Maximum number of nodes to keep
    pub fn new(initial_snapshot: EditorSnapshot, limit: usize) -> Self {
        let root = UndoNode::new(initial_snapshot, None, 0);
        Self {
            nodes: vec![root],
            current: 0,
            next_seq: 1,
            limit,
        }
    }

    /// Returns the current node index.
    pub fn current(&self) -> usize {
        self.current
    }

    /// Returns the number of nodes in the tree.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the node limit.
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Adds a new checkpoint to the undo tree.
    ///
    /// Creates a new node as a child of the current node. If the current node
    /// already has children (from previous redos), this creates a branch.
    ///
    /// # Arguments
    ///
    /// * `snapshot` - The new state to checkpoint
    pub fn add_checkpoint(&mut self, snapshot: EditorSnapshot) {
        let seq = self.next_seq;
        self.next_seq += 1;

        let new_node = UndoNode::new(snapshot, Some(self.current), seq);
        let new_index = self.nodes.len();

        // Add new node as child of current
        self.nodes[self.current].children.push(new_index);
        self.nodes.push(new_node);

        // Move current pointer to new node
        self.current = new_index;

        // TODO: Implement pruning when limit exceeded
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

    #[test]
    fn test_undo_tree_initialization() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let snapshot = EditorSnapshot {
            tree,
            cursor_path: vec![],
        };

        let undo_tree = UndoTree::new(snapshot, 50);

        assert_eq!(undo_tree.current(), 0);
        assert_eq!(undo_tree.len(), 1);
        assert_eq!(undo_tree.limit(), 50);
    }

    #[test]
    fn test_add_checkpoint() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let snapshot1 = EditorSnapshot {
            tree: tree.clone(),
            cursor_path: vec![],
        };

        let mut undo_tree = UndoTree::new(snapshot1, 50);

        let tree2 = JsonTree::new(JsonNode::new(JsonValue::Boolean(true)));
        let snapshot2 = EditorSnapshot {
            tree: tree2,
            cursor_path: vec![0],
        };

        undo_tree.add_checkpoint(snapshot2);

        assert_eq!(undo_tree.current(), 1);
        assert_eq!(undo_tree.len(), 2);

        // Verify parent-child relationship
        assert_eq!(undo_tree.nodes[1].parent, Some(0));
        assert_eq!(undo_tree.nodes[0].children, vec![1]);
    }
}
