//! Input history for command and search modes.
//!
//! Provides vim-style history browsing with Up/Down arrows, supporting
//! both command mode (`:`) and search mode (`/`/`?`).

/// A reusable history buffer with Up/Down browsing support.
///
/// Stores entries oldest-first and supports browsing through them
/// with `browse_up` (older) and `browse_down` (newer). The current
/// input buffer is saved as a "draft" on first browse so it can be
/// restored when browsing past the newest entry.
pub struct InputHistory {
    entries: Vec<String>,
    max_size: usize,
    browse_index: Option<usize>,
    draft: String,
}

impl InputHistory {
    /// Creates a new empty history with the given maximum capacity.
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
            browse_index: None,
            draft: String::new(),
        }
    }

    /// Pushes an entry to the history.
    ///
    /// Skips empty strings and consecutive duplicates. Enforces max_size
    /// by removing the oldest entry when full. Resets browse state.
    pub fn push(&mut self, entry: String) {
        if entry.is_empty() {
            return;
        }
        // Skip consecutive duplicates
        if self.entries.last() == Some(&entry) {
            self.reset_browse();
            return;
        }
        self.entries.push(entry);
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
        }
        self.reset_browse();
    }

    /// Browses up (older) in history.
    ///
    /// On first call, saves `current_buffer` as the draft and returns
    /// the newest entry. Subsequent calls return progressively older
    /// entries. Returns `None` if already at the oldest entry or history
    /// is empty.
    pub fn browse_up(&mut self, current_buffer: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        match self.browse_index {
            None => {
                // First browse: save draft, go to newest
                self.draft = current_buffer.to_string();
                let idx = self.entries.len() - 1;
                self.browse_index = Some(idx);
                Some(&self.entries[idx])
            }
            Some(idx) => {
                if idx > 0 {
                    let new_idx = idx - 1;
                    self.browse_index = Some(new_idx);
                    Some(&self.entries[new_idx])
                } else {
                    // Already at oldest
                    None
                }
            }
        }
    }

    /// Browses down (newer) in history.
    ///
    /// Moves toward newer entries. When moving past the newest entry,
    /// returns the saved draft. Returns `None` if not currently browsing.
    pub fn browse_down(&mut self) -> Option<&str> {
        match self.browse_index {
            None => None,
            Some(idx) => {
                if idx + 1 < self.entries.len() {
                    let new_idx = idx + 1;
                    self.browse_index = Some(new_idx);
                    Some(&self.entries[new_idx])
                } else {
                    // Past newest entry, return draft
                    self.browse_index = None;
                    Some(&self.draft)
                }
            }
        }
    }

    /// Resets browse state (index and draft).
    pub fn reset_browse(&mut self) {
        self.browse_index = None;
        self.draft.clear();
    }
}

impl Default for InputHistory {
    fn default() -> Self {
        Self::new(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_is_empty() {
        let history = InputHistory::new(10);
        assert_eq!(history.entries.len(), 0);
        assert_eq!(history.browse_index, None);
    }

    #[test]
    fn test_push_adds_entries() {
        let mut history = InputHistory::new(10);
        history.push("first".to_string());
        history.push("second".to_string());
        assert_eq!(history.entries, vec!["first", "second"]);
    }

    #[test]
    fn test_push_skips_empty() {
        let mut history = InputHistory::new(10);
        history.push("".to_string());
        assert_eq!(history.entries.len(), 0);
    }

    #[test]
    fn test_push_skips_consecutive_duplicates() {
        let mut history = InputHistory::new(10);
        history.push("cmd".to_string());
        history.push("cmd".to_string());
        assert_eq!(history.entries.len(), 1);
    }

    #[test]
    fn test_push_allows_non_consecutive_duplicates() {
        let mut history = InputHistory::new(10);
        history.push("a".to_string());
        history.push("b".to_string());
        history.push("a".to_string());
        assert_eq!(history.entries, vec!["a", "b", "a"]);
    }

    #[test]
    fn test_push_enforces_max_size() {
        let mut history = InputHistory::new(3);
        history.push("a".to_string());
        history.push("b".to_string());
        history.push("c".to_string());
        history.push("d".to_string());
        assert_eq!(history.entries, vec!["b", "c", "d"]);
    }

    #[test]
    fn test_browse_up_empty_history() {
        let mut history = InputHistory::new(10);
        assert_eq!(history.browse_up("draft"), None);
    }

    #[test]
    fn test_browse_up_returns_newest_first() {
        let mut history = InputHistory::new(10);
        history.push("first".to_string());
        history.push("second".to_string());
        assert_eq!(history.browse_up("current"), Some("second"));
    }

    #[test]
    fn test_browse_up_saves_draft() {
        let mut history = InputHistory::new(10);
        history.push("entry".to_string());
        history.browse_up("my draft");
        assert_eq!(history.draft, "my draft");
    }

    #[test]
    fn test_browse_up_walks_through_history() {
        let mut history = InputHistory::new(10);
        history.push("first".to_string());
        history.push("second".to_string());
        history.push("third".to_string());
        assert_eq!(history.browse_up(""), Some("third"));
        assert_eq!(history.browse_up(""), Some("second"));
        assert_eq!(history.browse_up(""), Some("first"));
        assert_eq!(history.browse_up(""), None); // at oldest
    }

    #[test]
    fn test_browse_down_returns_none_when_not_browsing() {
        let mut history = InputHistory::new(10);
        history.push("entry".to_string());
        assert_eq!(history.browse_down(), None);
    }

    #[test]
    fn test_browse_down_walks_forward() {
        let mut history = InputHistory::new(10);
        history.push("first".to_string());
        history.push("second".to_string());
        history.push("third".to_string());

        // Browse up to oldest
        history.browse_up("");
        history.browse_up("");
        history.browse_up("");

        // Browse back down
        assert_eq!(history.browse_down(), Some("second"));
        assert_eq!(history.browse_down(), Some("third"));
    }

    #[test]
    fn test_browse_down_past_newest_returns_draft() {
        let mut history = InputHistory::new(10);
        history.push("entry".to_string());

        history.browse_up("my draft");
        assert_eq!(history.browse_down(), Some("my draft"));
        // Now browse_index should be None
        assert_eq!(history.browse_index, None);
    }

    #[test]
    fn test_browse_up_then_down_round_trip() {
        let mut history = InputHistory::new(10);
        history.push("a".to_string());
        history.push("b".to_string());

        assert_eq!(history.browse_up("draft"), Some("b"));
        assert_eq!(history.browse_up("draft"), Some("a"));
        assert_eq!(history.browse_down(), Some("b"));
        assert_eq!(history.browse_down(), Some("draft"));
    }

    #[test]
    fn test_reset_browse() {
        let mut history = InputHistory::new(10);
        history.push("entry".to_string());
        history.browse_up("draft");
        assert!(history.browse_index.is_some());

        history.reset_browse();
        assert_eq!(history.browse_index, None);
        assert_eq!(history.draft, "");
    }

    #[test]
    fn test_push_resets_browse() {
        let mut history = InputHistory::new(10);
        history.push("first".to_string());
        history.browse_up("draft");
        assert!(history.browse_index.is_some());

        history.push("second".to_string());
        assert_eq!(history.browse_index, None);
    }

    #[test]
    fn test_default() {
        let history = InputHistory::default();
        assert_eq!(history.max_size, 50);
        assert_eq!(history.entries.len(), 0);
    }
}
