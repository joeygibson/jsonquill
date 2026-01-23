//! Cursor position tracking for the JSON tree.
//!
//! This module provides the `Cursor` struct that represents the current position
//! in the JSON tree as a path of indices. The cursor is used to navigate through
//! nested JSON structures (objects and arrays) and identify the currently selected
//! node.
//!
//! # Path Representation
//!
//! The cursor stores a path as a vector of indices, where each index represents
//! the position within a parent container:
//!
//! - For JSON objects: index is the position in the key-value pair list
//! - For JSON arrays: index is the array element position
//!
//! An empty path represents the root node.
//!
//! # Example
//!
//! ```
//! use jsonquill::editor::cursor::Cursor;
//!
//! // Start at root (empty path)
//! let mut cursor = Cursor::new();
//! assert_eq!(cursor.path(), &[] as &[usize]);
//!
//! // Navigate into first child, then second child
//! cursor.push(0);
//! cursor.push(1);
//! assert_eq!(cursor.path(), &[0, 1]);
//!
//! // Navigate back up
//! cursor.pop();
//! assert_eq!(cursor.path(), &[0]);
//! ```

/// Path to a node in the JSON tree (indices into objects/arrays).
///
/// The `Cursor` tracks the current position in the JSON tree as a sequence of
/// indices. Each index in the path represents a child position within a parent
/// container node (object or array).
///
/// # Examples
///
/// ```
/// use jsonquill::editor::cursor::Cursor;
///
/// let mut cursor = Cursor::new();
/// assert_eq!(cursor.path(), &[] as &[usize]);
///
/// cursor.push(0);
/// cursor.push(2);
/// assert_eq!(cursor.path(), &[0, 2]);
///
/// cursor.pop();
/// assert_eq!(cursor.path(), &[0]);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cursor {
    path: Vec<usize>,
}

impl Cursor {
    /// Creates a new cursor at the root position (empty path).
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::editor::cursor::Cursor;
    ///
    /// let cursor = Cursor::new();
    /// assert_eq!(cursor.path(), &[] as &[usize]);
    /// ```
    pub fn new() -> Self {
        Self { path: vec![] }
    }

    /// Returns a reference to the current path.
    ///
    /// The path is a slice of indices representing the position in the tree.
    /// An empty slice indicates the root node.
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::editor::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new();
    /// cursor.push(0);
    /// cursor.push(1);
    /// assert_eq!(cursor.path(), &[0, 1]);
    /// ```
    pub fn path(&self) -> &[usize] {
        &self.path
    }

    /// Pushes a new index onto the path, moving the cursor deeper into the tree.
    ///
    /// This is typically used when navigating into a child node of an object or array.
    ///
    /// # Arguments
    ///
    /// * `index` - The child position to navigate to
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::editor::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new();
    /// cursor.push(0);
    /// assert_eq!(cursor.path(), &[0]);
    /// cursor.push(2);
    /// assert_eq!(cursor.path(), &[0, 2]);
    /// ```
    pub fn push(&mut self, index: usize) {
        self.path.push(index);
    }

    /// Removes and returns the last index from the path, moving the cursor up one level.
    ///
    /// Returns `None` if the cursor is already at the root (empty path).
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::editor::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new();
    /// cursor.push(0);
    /// cursor.push(1);
    ///
    /// assert_eq!(cursor.pop(), Some(1));
    /// assert_eq!(cursor.path(), &[0]);
    ///
    /// assert_eq!(cursor.pop(), Some(0));
    /// assert_eq!(cursor.path(), &[] as &[usize]);
    ///
    /// assert_eq!(cursor.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<usize> {
        self.path.pop()
    }

    /// Replaces the entire path with a new one.
    ///
    /// This is useful for jumping to a specific location in the tree without
    /// incrementally pushing/popping indices.
    ///
    /// # Arguments
    ///
    /// * `path` - The new path to set
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::editor::cursor::Cursor;
    ///
    /// let mut cursor = Cursor::new();
    /// cursor.set_path(vec![0, 1, 2]);
    /// assert_eq!(cursor.path(), &[0, 1, 2]);
    /// ```
    pub fn set_path(&mut self, path: Vec<usize>) {
        self.path = path;
    }
}
