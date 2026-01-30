//! Jump list for tracking cursor position history.

/// Manages cursor position history for Ctrl-o/Ctrl-i navigation.
///
/// The jump list stores cursor paths as a vector with a current position pointer.
/// When jumping backward/forward, the pointer moves through the history.
/// Recording a new jump when not at the end truncates future history.
#[derive(Debug, Clone)]
pub struct JumpList {
    /// Stored cursor paths
    jumps: Vec<Vec<usize>>,
    /// Current position in jump list (0-based index)
    current: usize,
    /// Maximum jumps to store
    max_size: usize,
}

impl JumpList {
    /// Creates a new jump list with a maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            jumps: Vec::new(),
            current: 0,
            max_size,
        }
    }

    /// Records a new jump at the current cursor position.
    ///
    /// If not at the end of the list, truncates all jumps after current position.
    /// If at max capacity, removes oldest jump.
    pub fn record_jump(&mut self, cursor_path: Vec<usize>) {
        // Don't record duplicate of current position
        if let Some(last) = self.jumps.get(self.current) {
            if last == &cursor_path {
                return;
            }
        }

        // Truncate future history if in the middle of the list
        if self.current < self.jumps.len() {
            self.jumps.truncate(self.current + 1);
        }

        // Add new jump
        self.jumps.push(cursor_path);
        self.current = self.jumps.len() - 1;

        // Enforce max size (ring buffer behavior)
        if self.jumps.len() > self.max_size {
            self.jumps.remove(0);
            self.current = self.jumps.len() - 1;
        }
    }

    /// Jump backward in history.
    ///
    /// Returns the cursor path to jump to, or None if at the oldest position.
    pub fn jump_backward(&mut self) -> Option<Vec<usize>> {
        if self.current == 0 || self.jumps.is_empty() {
            return None;
        }

        self.current -= 1;
        Some(self.jumps[self.current].clone())
    }

    /// Jump forward in history.
    ///
    /// Returns the cursor path to jump to, or None if at the newest position.
    pub fn jump_forward(&mut self) -> Option<Vec<usize>> {
        if self.current >= self.jumps.len().saturating_sub(1) || self.jumps.is_empty() {
            return None;
        }

        self.current += 1;
        Some(self.jumps[self.current].clone())
    }

    /// Returns the number of jumps stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.jumps.len()
    }

    /// Returns true if the jump list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.jumps.is_empty()
    }

    /// Returns the current position in the jump list.
    pub fn current_position(&self) -> usize {
        self.current
    }
}
