//! Tree-based navigation for JSON documents.
//!
//! This module provides the `JsonTree` type for navigating JSON structures using
//! path-based indexing. It enables traversal of nested objects and arrays by
//! specifying a sequence of indices that represent the path from the root to a
//! target node.
//!
//! # Example
//!
//! ```
//! use jeditor::document::tree::JsonTree;
//! use jeditor::document::node::{JsonNode, JsonValue};
//!
//! // Create a simple tree
//! let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
//!     ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
//!     ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
//! ])));
//!
//! // Access the root
//! assert!(tree.root().value().is_object());
//!
//! // Navigate to first field
//! let path = vec![0];
//! let child = tree.get_node(&path).unwrap();
//! if let JsonValue::String(s) = child.value() {
//!     assert_eq!(s, "Alice");
//! }
//! ```

use super::node::{JsonNode, JsonValue};

/// A JSON document represented as a navigable tree structure.
///
/// `JsonTree` wraps a root `JsonNode` and provides methods for navigating
/// the tree using path-based indexing. Paths are represented as slices of
/// indices, where each index selects either:
/// - An object field by position in the key-value pair vector
/// - An array element by position in the element vector
#[derive(Debug, Clone)]
pub struct JsonTree {
    root: JsonNode,
}

impl JsonTree {
    /// Creates a new `JsonTree` with the given root node.
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::document::tree::JsonTree;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// assert!(matches!(tree.root().value(), JsonValue::Null));
    /// ```
    pub fn new(root: JsonNode) -> Self {
        Self { root }
    }

    /// Returns an immutable reference to the root node.
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::document::tree::JsonTree;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Boolean(true)));
    /// assert!(matches!(tree.root().value(), JsonValue::Boolean(true)));
    /// ```
    pub fn root(&self) -> &JsonNode {
        &self.root
    }

    /// Returns a mutable reference to the root node.
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::document::tree::JsonTree;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    ///
    /// let mut tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// *tree.root_mut().value_mut() = JsonValue::Boolean(false);
    /// assert!(matches!(tree.root().value(), JsonValue::Boolean(false)));
    /// ```
    pub fn root_mut(&mut self) -> &mut JsonNode {
        &mut self.root
    }

    /// Gets an immutable reference to a node at the specified path.
    ///
    /// The path is a sequence of indices that navigate through the tree:
    /// - For objects: the index selects the nth key-value pair
    /// - For arrays: the index selects the nth element
    /// - For non-container values: any path beyond the current node returns None
    ///
    /// Returns `None` if:
    /// - The path is out of bounds at any level
    /// - The path attempts to traverse a non-container value
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::document::tree::JsonTree;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    ///
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
    ///     ("items".to_string(), JsonNode::new(JsonValue::Array(vec![
    ///         JsonNode::new(JsonValue::Number(1.0)),
    ///         JsonNode::new(JsonValue::Number(2.0)),
    ///     ]))),
    /// ])));
    ///
    /// // Navigate to items[1]
    /// let path = vec![0, 1]; // First object field, second array element
    /// let node = tree.get_node(&path).unwrap();
    /// assert!(matches!(node.value(), JsonValue::Number(2.0)));
    ///
    /// // Invalid path
    /// let invalid_path = vec![0, 99];
    /// assert!(tree.get_node(&invalid_path).is_none());
    /// ```
    pub fn get_node(&self, path: &[usize]) -> Option<&JsonNode> {
        let mut current = &self.root;

        for &index in path {
            match current.value() {
                JsonValue::Object(entries) => {
                    current = &entries.get(index)?.1;
                }
                JsonValue::Array(elements) => {
                    current = elements.get(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Gets a mutable reference to a node at the specified path.
    ///
    /// This method follows the same path resolution rules as `get_node`,
    /// but returns a mutable reference. Note that obtaining a mutable
    /// reference to a node marks it as modified.
    ///
    /// Returns `None` if:
    /// - The path is out of bounds at any level
    /// - The path attempts to traverse a non-container value
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::document::tree::JsonTree;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    ///
    /// let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
    ///     JsonNode::new(JsonValue::String("old".to_string())),
    /// ])));
    ///
    /// // Modify first array element
    /// let path = vec![0];
    /// if let Some(node) = tree.get_node_mut(&path) {
    ///     *node.value_mut() = JsonValue::String("new".to_string());
    /// }
    ///
    /// // Verify the change
    /// let node = tree.get_node(&path).unwrap();
    /// if let JsonValue::String(s) = node.value() {
    ///     assert_eq!(s, "new");
    /// }
    /// ```
    pub fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut JsonNode> {
        let mut current = &mut self.root;

        for &index in path {
            match current.value_mut() {
                JsonValue::Object(entries) => {
                    current = &mut entries.get_mut(index)?.1;
                }
                JsonValue::Array(elements) => {
                    current = elements.get_mut(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Deletes the node at the given path.
    /// Returns an error if the path is empty (cannot delete root) or invalid.
    pub fn delete_node(&mut self, path: &[usize]) -> anyhow::Result<()> {
        use anyhow::{anyhow, Context};

        if path.is_empty() {
            return Err(anyhow!("Cannot delete root node"));
        }

        // Get parent path (all but last index)
        let parent_path = &path[..path.len() - 1];
        let index = path[path.len() - 1];

        // Get mutable reference to parent node
        let parent = self.get_node_mut(parent_path)
            .ok_or_else(|| anyhow!("Parent node not found"))?;

        // Delete from parent based on its type
        match parent.value_mut() {
            JsonValue::Object(entries) => {
                if index >= entries.len() {
                    return Err(anyhow!("Index {} out of bounds for object with {} entries", index, entries.len()));
                }
                entries.remove(index);
            }
            JsonValue::Array(elements) => {
                if index >= elements.len() {
                    return Err(anyhow!("Index {} out of bounds for array with {} elements", index, elements.len()));
                }
                elements.remove(index);
            }
            _ => {
                return Err(anyhow!("Parent is not a container type"));
            }
        }

        Ok(())
    }
}
