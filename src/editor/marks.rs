//! Mark management for bookmarking cursor positions.

use std::collections::HashMap;

/// Manages local marks (a-z) for bookmarking positions in the document.
///
/// Marks store cursor paths that can be jumped to later. They persist during
/// the editing session but are cleared when the file is closed or a new file
/// is loaded.
#[derive(Debug, Clone)]
pub struct MarkSet {
    /// Map from mark name (a-z) to cursor path
    marks: HashMap<char, Vec<usize>>,
}

impl MarkSet {
    /// Creates a new empty mark set.
    pub fn new() -> Self {
        Self {
            marks: HashMap::new(),
        }
    }

    /// Sets a mark at the given cursor path.
    ///
    /// # Arguments
    ///
    /// * `name` - Mark name (should be a-z, but not validated here)
    /// * `cursor_path` - Path to the marked position
    pub fn set_mark(&mut self, name: char, cursor_path: Vec<usize>) {
        self.marks.insert(name, cursor_path);
    }

    /// Gets the cursor path for a mark.
    ///
    /// Returns None if the mark is not set.
    pub fn get_mark(&self, name: char) -> Option<&Vec<usize>> {
        self.marks.get(&name)
    }

    /// Clears all marks.
    pub fn clear(&mut self) {
        self.marks.clear();
    }

    /// Lists all set marks as (name, path) pairs.
    pub fn list(&self) -> Vec<(char, &Vec<usize>)> {
        let mut result: Vec<_> = self.marks.iter().map(|(&c, p)| (c, p)).collect();
        result.sort_by_key(|(c, _)| *c);
        result
    }
}

impl Default for MarkSet {
    fn default() -> Self {
        Self::new()
    }
}
