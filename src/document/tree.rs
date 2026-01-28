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
//! use jsonquill::document::tree::JsonTree;
//! use jsonquill::document::node::{JsonNode, JsonValue};
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

/// A complete JSON document tree.
///
/// `JsonTree` represents a parsed JSON document with a root node and optional
/// original source text for format preservation.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonTree {
    root: JsonNode,
    /// The original JSON string (preserved for unmodified nodes)
    original_source: Option<String>,
}

impl JsonTree {
    /// Creates a new JSON tree with the given root node.
    ///
    /// The tree has no original source, so format preservation is not available.
    /// New nodes created via `JsonNode::new()` are marked as modified by default.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let root = JsonNode::new(JsonValue::Null);
    /// let tree = JsonTree::new(root);
    /// ```
    pub fn new(root: JsonNode) -> Self {
        Self {
            root,
            original_source: None,
        }
    }

    /// Creates a new JSON tree with the given root node and original source.
    ///
    /// The original source enables format preservation for unmodified nodes.
    pub fn with_source(root: JsonNode, original_source: Option<String>) -> Self {
        Self {
            root,
            original_source,
        }
    }

    /// Returns a reference to the original JSON source, if available.
    pub fn original_source(&self) -> Option<&str> {
        self.original_source.as_deref()
    }

    /// Returns a reference to the root node of the tree.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let root = JsonNode::new(JsonValue::Boolean(true));
    /// let tree = JsonTree::new(root);
    ///
    /// assert!(matches!(tree.root().value(), JsonValue::Boolean(true)));
    /// ```
    pub fn root(&self) -> &JsonNode {
        &self.root
    }

    /// Returns a mutable reference to the root node of the tree.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let root = JsonNode::new(JsonValue::Null);
    /// let mut tree = JsonTree::new(root);
    ///
    /// *tree.root_mut().value_mut() = JsonValue::Boolean(false);
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
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
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
                JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
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
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
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
                JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
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
        use anyhow::anyhow;

        if path.is_empty() {
            return Err(anyhow!("Cannot delete root node"));
        }

        // Get parent path (all but last index)
        let parent_path = &path[..path.len() - 1];
        let index = path[path.len() - 1];

        // Get mutable reference to parent node
        let parent = self
            .get_node_mut(parent_path)
            .ok_or_else(|| anyhow!("Parent node not found"))?;

        // Delete from parent based on its type
        match parent.value_mut() {
            JsonValue::Object(entries) => {
                if index >= entries.len() {
                    return Err(anyhow!(
                        "Index {} out of bounds for object with {} entries",
                        index,
                        entries.len()
                    ));
                }
                entries.remove(index);
            }
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                if index >= elements.len() {
                    return Err(anyhow!(
                        "Index {} out of bounds for array with {} elements",
                        index,
                        elements.len()
                    ));
                }
                elements.remove(index);
            }
            _ => {
                return Err(anyhow!("Parent is not a container type"));
            }
        }

        Ok(())
    }

    /// Inserts a node into an object at the specified path and index.
    /// The path must point to the object, and index specifies where to insert.
    pub fn insert_node_in_object(
        &mut self,
        path: &[usize],
        key: String,
        node: JsonNode,
    ) -> anyhow::Result<()> {
        use anyhow::anyhow;

        // Get parent path (all but last index)
        let parent_path = if path.is_empty() {
            &[]
        } else {
            &path[..path.len() - 1]
        };
        let index = if path.is_empty() {
            0
        } else {
            path[path.len() - 1]
        };

        // Get mutable reference to parent (or root if path is empty)
        let target = if parent_path.is_empty() {
            self.root_mut()
        } else {
            self.get_node_mut(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        // Insert into object
        match target.value_mut() {
            JsonValue::Object(entries) => {
                if index > entries.len() {
                    return Err(anyhow!(
                        "Index {} out of bounds for object with {} entries",
                        index,
                        entries.len()
                    ));
                }
                entries.insert(index, (key, node));
            }
            _ => {
                return Err(anyhow!("Target is not an object"));
            }
        }

        Ok(())
    }

    /// Inserts a node into an array at the specified path and index.
    pub fn insert_node_in_array(&mut self, path: &[usize], node: JsonNode) -> anyhow::Result<()> {
        use anyhow::anyhow;

        // Get parent path (all but last index)
        let parent_path = if path.is_empty() {
            &[]
        } else {
            &path[..path.len() - 1]
        };
        let index = if path.is_empty() {
            0
        } else {
            path[path.len() - 1]
        };

        // Get mutable reference to parent (or root if path is empty)
        let target = if parent_path.is_empty() {
            self.root_mut()
        } else {
            self.get_node_mut(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        // Insert into array
        match target.value_mut() {
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                if index > elements.len() {
                    return Err(anyhow!(
                        "Index {} out of bounds for array with {} elements",
                        index,
                        elements.len()
                    ));
                }
                elements.insert(index, node);
            }
            _ => {
                return Err(anyhow!("Target is not an array"));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_with_original_source() {
        let root = JsonNode::new(JsonValue::String("test".to_string()));
        let tree = JsonTree::with_source(root.clone(), Some("\"test\"".to_string()));

        assert_eq!(tree.original_source(), Some("\"test\""));
    }

    #[test]
    fn test_tree_without_original_source() {
        let root = JsonNode::new(JsonValue::Null);
        let tree = JsonTree::new(root);

        assert_eq!(tree.original_source(), None);
    }
}
