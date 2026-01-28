//! JSON node representation with metadata tracking.
//!
//! This module provides the core data structures for representing JSON documents
//! in jsonquill. Each JSON value is wrapped in a `JsonNode` that tracks metadata
//! such as modification status and original formatting, enabling format-preserving
//! edits and efficient change tracking.
//!
//! # Example
//!
//! ```
//! use jsonquill::document::node::{JsonNode, JsonValue};
//!
//! // Create a simple string node
//! let mut node = JsonNode::new(JsonValue::String("hello".to_string()));
//! assert!(node.is_modified()); // New nodes are marked as modified
//!
//! // Create a complex nested structure
//! let object = JsonNode::new(JsonValue::Object(vec![
//!     ("name".to_string(), JsonNode::new(JsonValue::String("jsonquill".to_string()))),
//!     ("version".to_string(), JsonNode::new(JsonValue::Number(1.0))),
//! ]));
//!
//! // Modify a value
//! if let JsonValue::Object(ref mut fields) = node.value_mut() {
//!     fields.push(("key".to_string(), JsonNode::new(JsonValue::Null)));
//! }
//! ```

/// A byte range in the original JSON source.
///
/// TextSpan tracks the position of a node's text in the original JSON string,
/// enabling exact format preservation for unmodified nodes.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct TextSpan {
    /// Start byte offset in original JSON
    pub start: usize,
    /// End byte offset in original JSON (exclusive)
    pub end: usize,
}

/// A JSON value without metadata.
///
/// This enum represents the core JSON types: objects, arrays, strings, numbers,
/// booleans, and null. Objects and arrays contain `JsonNode` instances to preserve
/// metadata throughout the tree structure.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// A JSON object containing key-value pairs
    Object(Vec<(String, JsonNode)>),
    /// A JSON array containing ordered values
    Array(Vec<JsonNode>),
    /// A JSON string
    String(String),
    /// A JSON number (represented as f64)
    Number(f64),
    /// A JSON boolean
    Boolean(bool),
    /// A JSON null value
    Null,
    /// A JSONL document root containing lines (each line is a JsonNode)
    JsonlRoot(Vec<JsonNode>),
}

/// A JSON value wrapped with metadata for tracking changes and formatting.
///
/// `JsonNode` is the primary type used throughout jsonquill to represent JSON data.
/// It wraps a `JsonValue` with `NodeMetadata` to track whether the node has been
/// modified and preserve original formatting information for format-preserving edits.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonNode {
    pub(crate) value: JsonValue,
    pub(crate) metadata: NodeMetadata,
}

/// Metadata associated with a JSON node.
///
/// This structure tracks information about a node beyond its value, including
/// whether it has been modified since loading and its byte position in the
/// original source for format preservation.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeMetadata {
    /// Byte range in the original JSON string (for unmodified nodes)
    pub text_span: Option<TextSpan>,
    /// Whether this node has been modified
    pub modified: bool,
}

impl JsonValue {
    /// Returns true if this value is an object.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::node::JsonValue;
    ///
    /// let obj = JsonValue::Object(vec![]);
    /// assert!(obj.is_object());
    ///
    /// let num = JsonValue::Number(42.0);
    /// assert!(!num.is_object());
    /// ```
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    /// Returns true if this value is an array.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::node::JsonValue;
    ///
    /// let arr = JsonValue::Array(vec![]);
    /// assert!(arr.is_array());
    ///
    /// let num = JsonValue::Number(42.0);
    /// assert!(!num.is_array());
    /// ```
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    /// Returns true if this value is a container (object, array, or JSONL root).
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::node::JsonValue;
    ///
    /// let obj = JsonValue::Object(vec![]);
    /// assert!(obj.is_container());
    ///
    /// let arr = JsonValue::Array(vec![]);
    /// assert!(arr.is_container());
    ///
    /// let num = JsonValue::Number(42.0);
    /// assert!(!num.is_container());
    /// ```
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            JsonValue::Object(_) | JsonValue::Array(_) | JsonValue::JsonlRoot(_)
        )
    }
}

impl JsonNode {
    /// Creates a new `JsonNode` with the given value.
    ///
    /// The node is marked as modified by default since it's newly created.
    /// The text_span field is set to None.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let node = JsonNode::new(JsonValue::Number(42.0));
    /// assert!(node.is_modified());
    /// ```
    pub fn new(value: JsonValue) -> Self {
        Self {
            value,
            metadata: NodeMetadata {
                text_span: None,
                modified: true,
            },
        }
    }

    /// Returns an immutable reference to the node's value.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let node = JsonNode::new(JsonValue::Boolean(true));
    /// assert!(matches!(node.value(), JsonValue::Boolean(true)));
    /// ```
    pub fn value(&self) -> &JsonValue {
        &self.value
    }

    /// Returns a mutable reference to the node's value.
    ///
    /// Calling this method automatically marks the node as modified,
    /// even if the value is not actually changed.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let mut node = JsonNode::new(JsonValue::String("old".to_string()));
    /// *node.value_mut() = JsonValue::String("new".to_string());
    /// assert!(node.is_modified());
    /// ```
    pub fn value_mut(&mut self) -> &mut JsonValue {
        self.metadata.modified = true;
        &mut self.value
    }

    /// Returns whether this node has been modified.
    ///
    /// A node is considered modified if it was newly created or if
    /// `value_mut()` has been called on it.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let node = JsonNode::new(JsonValue::Null);
    /// assert!(node.is_modified());
    /// ```
    pub fn is_modified(&self) -> bool {
        self.metadata.modified
    }
}

#[cfg(test)]
mod text_span_tests {
    use super::*;

    #[test]
    fn test_text_span_creation() {
        let span = TextSpan { start: 10, end: 25 };
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 25);
    }

    #[test]
    fn test_text_span_equality() {
        let span1 = TextSpan { start: 5, end: 10 };
        let span2 = TextSpan { start: 5, end: 10 };
        let span3 = TextSpan { start: 5, end: 11 };

        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }

    #[test]
    fn test_text_span_clone() {
        let span1 = TextSpan { start: 0, end: 100 };
        let span2 = span1;

        assert_eq!(span1, span2);
    }

    #[test]
    fn test_node_metadata_with_text_span() {
        let metadata = NodeMetadata {
            text_span: Some(TextSpan { start: 0, end: 10 }),
            modified: false,
        };

        assert!(metadata.text_span.is_some());
        assert_eq!(metadata.text_span.unwrap().start, 0);
        assert_eq!(metadata.text_span.unwrap().end, 10);
        assert!(!metadata.modified);
    }

    #[test]
    fn test_node_metadata_without_text_span() {
        let metadata = NodeMetadata {
            text_span: None,
            modified: true,
        };

        assert!(metadata.text_span.is_none());
        assert!(metadata.modified);
    }
}
