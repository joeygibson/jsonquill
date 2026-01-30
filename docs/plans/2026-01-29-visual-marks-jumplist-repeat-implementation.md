# Visual Mode, Marks, Jump List, and Repeat Command Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add vim-style visual mode, marks, jump list, and repeat command to jsonquill for enhanced navigation and editing efficiency.

**Architecture:** Four independent features that integrate with existing editor state, undo/redo, and input handling. Visual mode adds new EditorMode variant, marks and jump list add new data structures in src/editor/, and repeat command adds command recording system.

**Tech Stack:** Rust, ratatui for UI rendering, termion for input handling, existing editor infrastructure (undo, registers, cursor).

---

## Task 1: Jump List Foundation

**Files:**
- Create: `src/editor/jumplist.rs`
- Modify: `src/editor/mod.rs` (add module declaration)
- Test: `tests/jumplist_tests.rs`

**Step 1: Write failing test for JumpList basics**

```rust
// tests/jumplist_tests.rs
use jsonquill::editor::jumplist::JumpList;

#[test]
fn test_jumplist_creation() {
    let jumplist = JumpList::new(100);
    assert_eq!(jumplist.len(), 0);
    assert_eq!(jumplist.current_position(), 0);
}

#[test]
fn test_record_and_backward() {
    let mut jumplist = JumpList::new(100);
    jumplist.record_jump(vec![0]);
    jumplist.record_jump(vec![1]);
    jumplist.record_jump(vec![2]);

    assert_eq!(jumplist.len(), 3);
    assert_eq!(jumplist.jump_backward(), Some(vec![1]));
    assert_eq!(jumplist.jump_backward(), Some(vec![0]));
    assert_eq!(jumplist.jump_backward(), None); // At oldest
}

#[test]
fn test_forward_navigation() {
    let mut jumplist = JumpList::new(100);
    jumplist.record_jump(vec![0]);
    jumplist.record_jump(vec![1]);
    jumplist.record_jump(vec![2]);

    jumplist.jump_backward();
    jumplist.jump_backward();

    assert_eq!(jumplist.jump_forward(), Some(vec![1]));
    assert_eq!(jumplist.jump_forward(), Some(vec![2]));
    assert_eq!(jumplist.jump_forward(), None); // At newest
}

#[test]
fn test_truncate_on_new_jump() {
    let mut jumplist = JumpList::new(100);
    jumplist.record_jump(vec![0]);
    jumplist.record_jump(vec![1]);
    jumplist.record_jump(vec![2]);

    // Jump back then record new jump
    jumplist.jump_backward();
    jumplist.record_jump(vec![3]);

    // Should truncate vec![2] and add vec![3]
    assert_eq!(jumplist.len(), 3);
    assert_eq!(jumplist.jump_backward(), Some(vec![1]));
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test jumplist --lib
```

Expected: Compilation error - module 'jumplist' not found

**Step 3: Create JumpList implementation**

```rust
// src/editor/jumplist.rs
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
        self.current = self.jumps.len();

        // Enforce max size (ring buffer behavior)
        if self.jumps.len() > self.max_size {
            self.jumps.remove(0);
            self.current = self.jumps.len();
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
    pub fn len(&self) -> usize {
        self.jumps.len()
    }

    /// Returns true if the jump list is empty.
    pub fn is_empty(&self) -> bool {
        self.jumps.is_empty()
    }

    /// Returns the current position in the jump list.
    pub fn current_position(&self) -> usize {
        self.current
    }
}
```

**Step 4: Add module declaration**

```rust
// src/editor/mod.rs
// Add this line with other module declarations:
pub mod jumplist;
```

**Step 5: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test jumplist_tests
```

Expected: All 4 tests pass

**Step 6: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/editor/jumplist.rs src/editor/mod.rs tests/jumplist_tests.rs
git commit -m "feat: add JumpList data structure for Ctrl-o/Ctrl-i navigation

Implements ring buffer for cursor position history with:
- Forward/backward navigation
- Truncation on new jump when in middle of history
- Max size limit (100 jumps)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Marks Foundation

**Files:**
- Create: `src/editor/marks.rs`
- Modify: `src/editor/mod.rs`
- Test: `tests/marks_tests.rs`

**Step 1: Write failing test for MarkSet**

```rust
// tests/marks_tests.rs
use jsonquill::editor::marks::MarkSet;

#[test]
fn test_markset_creation() {
    let marks = MarkSet::new();
    assert_eq!(marks.get_mark('a'), None);
}

#[test]
fn test_set_and_get_mark() {
    let mut marks = MarkSet::new();
    marks.set_mark('a', vec![0, 1, 2]);

    assert_eq!(marks.get_mark('a'), Some(&vec![0, 1, 2]));
    assert_eq!(marks.get_mark('b'), None);
}

#[test]
fn test_overwrite_mark() {
    let mut marks = MarkSet::new();
    marks.set_mark('a', vec![0]);
    marks.set_mark('a', vec![1]);

    assert_eq!(marks.get_mark('a'), Some(&vec![1]));
}

#[test]
fn test_list_marks() {
    let mut marks = MarkSet::new();
    marks.set_mark('a', vec![0]);
    marks.set_mark('c', vec![2]);

    let list = marks.list();
    assert_eq!(list.len(), 2);
    assert!(list.contains(&('a', &vec![0])));
    assert!(list.contains(&('c', &vec![2])));
}

#[test]
fn test_clear_marks() {
    let mut marks = MarkSet::new();
    marks.set_mark('a', vec![0]);
    marks.set_mark('b', vec![1]);
    marks.clear();

    assert_eq!(marks.get_mark('a'), None);
    assert_eq!(marks.get_mark('b'), None);
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test marks_tests
```

Expected: Compilation error - module 'marks' not found

**Step 3: Create MarkSet implementation**

```rust
// src/editor/marks.rs
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
```

**Step 4: Add module declaration**

```rust
// src/editor/mod.rs
// Add this line:
pub mod marks;
```

**Step 5: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test marks_tests
```

Expected: All 5 tests pass

**Step 6: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/editor/marks.rs src/editor/mod.rs tests/marks_tests.rs
git commit -m "feat: add MarkSet data structure for bookmark management

Implements mark storage with:
- Set/get marks (a-z)
- Clear all marks
- List all marks
- Session-scoped (not persisted)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Repeat Command Foundation

**Files:**
- Create: `src/editor/repeat.rs`
- Modify: `src/editor/mod.rs`
- Test: `tests/repeat_tests.rs`

**Step 1: Write failing test for RepeatableCommand**

```rust
// tests/repeat_tests.rs
use jsonquill::editor::repeat::RepeatableCommand;
use jsonquill::document::node::JsonValue;

#[test]
fn test_repeatable_command_delete() {
    let cmd = RepeatableCommand::Delete { count: 3 };

    match cmd {
        RepeatableCommand::Delete { count } => assert_eq!(count, 3),
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn test_repeatable_command_yank() {
    let cmd = RepeatableCommand::Yank { count: 5 };

    match cmd {
        RepeatableCommand::Yank { count } => assert_eq!(count, 5),
        _ => panic!("Expected Yank command"),
    }
}

#[test]
fn test_repeatable_command_add() {
    let cmd = RepeatableCommand::Add {
        value: JsonValue::String("test".to_string()),
        key: Some("mykey".to_string()),
    };

    match cmd {
        RepeatableCommand::Add { value, key } => {
            assert_eq!(value, JsonValue::String("test".to_string()));
            assert_eq!(key, Some("mykey".to_string()));
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_repeatable_command_clone() {
    let cmd = RepeatableCommand::Delete { count: 2 };
    let cloned = cmd.clone();

    match cloned {
        RepeatableCommand::Delete { count } => assert_eq!(count, 2),
        _ => panic!("Expected Delete command"),
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test repeat_tests
```

Expected: Compilation error - module 'repeat' not found

**Step 3: Create RepeatableCommand implementation**

```rust
// src/editor/repeat.rs
//! Repeatable command tracking for the '.' key.

use crate::document::node::JsonValue;

/// Represents a command that can be repeated with the '.' key.
///
/// This enum captures editing operations that modify the document
/// along with their parameters, so they can be replayed at a different
/// cursor position.
#[derive(Debug, Clone)]
pub enum RepeatableCommand {
    /// Delete with count (dd, 3dd)
    Delete { count: u32 },

    /// Yank with count (yy, 5yy)
    Yank { count: u32 },

    /// Paste after (p) or before (P)
    Paste { before: bool },

    /// Add scalar value (i)
    /// key is Some for object insertions, None for arrays
    Add {
        value: JsonValue,
        key: Option<String>,
    },

    /// Add empty array (a)
    AddArray,

    /// Add empty object (o)
    AddObject,

    /// Rename key (r)
    Rename { new_key: String },

    /// Change value in insert mode (e)
    ChangeValue { new_value: String },
}
```

**Step 4: Add module declaration**

```rust
// src/editor/mod.rs
// Add this line:
pub mod repeat;
```

**Step 5: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test repeat_tests
```

Expected: All 4 tests pass

**Step 6: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/editor/repeat.rs src/editor/mod.rs tests/repeat_tests.rs
git commit -m "feat: add RepeatableCommand enum for '.' key functionality

Implements command representation with variants for:
- Delete, Yank, Paste
- Add (scalar, array, object)
- Rename, ChangeValue

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Visual Mode - Add to EditorMode

**Files:**
- Modify: `src/editor/mode.rs`
- Test: `tests/editor_tests.rs`

**Step 1: Write failing test for Visual mode**

```rust
// Add to tests/editor_tests.rs
use jsonquill::editor::mode::EditorMode;

#[test]
fn test_visual_mode_display() {
    let mode = EditorMode::Visual;
    assert_eq!(format!("{}", mode), "VISUAL");
}

#[test]
fn test_visual_mode_equality() {
    assert_eq!(EditorMode::Visual, EditorMode::Visual);
    assert_ne!(EditorMode::Visual, EditorMode::Normal);
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_visual_mode_display
```

Expected: Compilation error - no variant Visual in EditorMode

**Step 3: Add Visual mode to enum**

```rust
// src/editor/mode.rs
// Modify the EditorMode enum to add Visual:

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Normal mode for navigation and structural operations.
    Normal,
    /// Insert mode for editing JSON values.
    Insert,
    /// Command mode for executing editor commands.
    Command,
    /// Search mode for finding text in keys and values.
    Search,
    /// Visual mode for selecting multiple nodes.
    Visual,
}
```

**Step 4: Update Display implementation**

```rust
// src/editor/mode.rs
// Update the fmt implementation to include Visual:

impl fmt::Display for EditorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorMode::Normal => write!(f, "NORMAL"),
            EditorMode::Insert => write!(f, "INSERT"),
            EditorMode::Command => write!(f, "COMMAND"),
            EditorMode::Search => write!(f, "SEARCH"),
            EditorMode::Visual => write!(f, "VISUAL"),
        }
    }
}
```

**Step 5: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_visual_mode
```

Expected: Both tests pass

**Step 6: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/editor/mode.rs tests/editor_tests.rs
git commit -m "feat: add Visual mode to EditorMode enum

Adds EditorMode::Visual variant with Display implementation.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Add State Fields to EditorState

**Files:**
- Modify: `src/editor/state.rs`
- Test: `tests/editor_tests.rs`

**Step 1: Write failing test for new state fields**

```rust
// Add to tests/editor_tests.rs
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_jumplist_initialized() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert_eq!(state.jumplist().len(), 0);
}

#[test]
fn test_marks_initialized() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert_eq!(state.marks().get_mark('a'), None);
}

#[test]
fn test_visual_state_initialized() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert_eq!(state.visual_anchor(), None);
    assert_eq!(state.visual_selection().len(), 0);
}

#[test]
fn test_last_command_initialized() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert!(state.last_command().is_none());
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_jumplist_initialized
```

Expected: Compilation error - no method jumplist on EditorState

**Step 3: Add new fields to EditorState struct**

```rust
// src/editor/state.rs
// Add these imports at the top:
use super::jumplist::JumpList;
use super::marks::MarkSet;
use super::repeat::RepeatableCommand;

// Add these fields to the EditorState struct (around line 236):
pub struct EditorState {
    // ... existing fields ...
    completion_candidates: Vec<String>,
    completion_index: usize,
    completion_prefix: String,

    // NEW FIELDS:
    jumplist: JumpList,
    marks: MarkSet,
    pending_mark_set: bool,
    pending_mark_jump: bool,
    visual_anchor: Option<Vec<usize>>,
    visual_selection: Vec<Vec<usize>>,
    last_command: Option<RepeatableCommand>,
}
```

**Step 4: Initialize new fields in constructor**

```rust
// src/editor/state.rs
// In the `new` method (around line 330), add these initializations:

Self {
    // ... existing initializations ...
    completion_candidates: Vec::new(),
    completion_index: 0,
    completion_prefix: String::new(),

    // NEW INITIALIZATIONS:
    jumplist: JumpList::new(100),
    marks: MarkSet::new(),
    pending_mark_set: false,
    pending_mark_jump: false,
    visual_anchor: None,
    visual_selection: Vec::new(),
    last_command: None,
}
```

**Step 5: Add getter methods**

```rust
// src/editor/state.rs
// Add these public getter methods near the end of the impl block:

/// Returns a reference to the jump list.
pub fn jumplist(&self) -> &JumpList {
    &self.jumplist
}

/// Returns a mutable reference to the jump list.
pub fn jumplist_mut(&mut self) -> &mut JumpList {
    &mut self.jumplist
}

/// Returns a reference to the mark set.
pub fn marks(&self) -> &MarkSet {
    &self.marks
}

/// Returns a mutable reference to the mark set.
pub fn marks_mut(&mut self) -> &mut MarkSet {
    &mut self.marks
}

/// Returns whether we're waiting for a mark name after 'm'.
pub fn pending_mark_set(&self) -> bool {
    self.pending_mark_set
}

/// Sets the pending mark set state.
pub fn set_pending_mark_set(&mut self, pending: bool) {
    self.pending_mark_set = pending;
}

/// Returns whether we're waiting for a mark name after '\''.
pub fn pending_mark_jump(&self) -> bool {
    self.pending_mark_jump
}

/// Sets the pending mark jump state.
pub fn set_pending_mark_jump(&mut self, pending: bool) {
    self.pending_mark_jump = pending;
}

/// Returns the visual mode anchor position.
pub fn visual_anchor(&self) -> Option<&Vec<usize>> {
    self.visual_anchor.as_ref()
}

/// Returns the visual selection (all selected node paths).
pub fn visual_selection(&self) -> &[Vec<usize>] {
    &self.visual_selection
}

/// Enters visual mode at the current cursor position.
pub fn enter_visual_mode(&mut self) {
    self.visual_anchor = Some(self.cursor.path().to_vec());
    self.visual_selection = vec![self.cursor.path().to_vec()];
    self.mode = EditorMode::Visual;
}

/// Exits visual mode and returns to normal mode.
pub fn exit_visual_mode(&mut self) {
    self.visual_anchor = None;
    self.visual_selection.clear();
    self.mode = EditorMode::Normal;
}

/// Updates the visual selection based on current cursor position.
pub fn update_visual_selection(&mut self) {
    if let Some(anchor) = &self.visual_anchor {
        // Calculate selection range based on visible lines
        let lines = self.tree_view.lines();

        // Find indices of anchor and cursor in visible lines
        let anchor_idx = lines.iter().position(|line| &line.path == anchor);
        let cursor_idx = lines.iter().position(|line| &line.path == self.cursor.path());

        if let (Some(a_idx), Some(c_idx)) = (anchor_idx, cursor_idx) {
            let (start, end) = if a_idx <= c_idx {
                (a_idx, c_idx)
            } else {
                (c_idx, a_idx)
            };

            self.visual_selection = lines[start..=end]
                .iter()
                .map(|line| line.path.clone())
                .collect();
        }
    }
}

/// Returns the last repeatable command.
pub fn last_command(&self) -> Option<&RepeatableCommand> {
    self.last_command.as_ref()
}

/// Sets the last repeatable command.
pub fn set_last_command(&mut self, cmd: RepeatableCommand) {
    self.last_command = Some(cmd);
}

/// Clears the last repeatable command.
pub fn clear_last_command(&mut self) {
    self.last_command = None;
}
```

**Step 6: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_jumplist_initialized test_marks_initialized test_visual_state_initialized test_last_command_initialized
```

Expected: All 4 tests pass

**Step 7: Run all tests to ensure no regressions**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test
```

Expected: All existing tests still pass

**Step 8: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/editor/state.rs tests/editor_tests.rs
git commit -m "feat: add state fields for jumplist, marks, visual mode, and repeat

Adds to EditorState:
- JumpList for Ctrl-o/Ctrl-i navigation
- MarkSet for bookmark management
- Visual mode state (anchor, selection)
- RepeatableCommand tracking
- Pending state for mark operations

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Add Input Events for New Features

**Files:**
- Modify: `src/input/keys.rs`
- Test: `tests/input_tests.rs`

**Step 1: Write failing test for new input events**

```rust
// Create new file: tests/input_tests.rs
use jsonquill::input::keys::{map_key_event, InputEvent};
use jsonquill::editor::mode::EditorMode;
use termion::event::{Event, Key};

#[test]
fn test_visual_mode_key() {
    let event = Event::Key(Key::Char('v'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::EnterVisualMode);
}

#[test]
fn test_mark_set_key() {
    let event = Event::Key(Key::Char('m'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::MarkSet);
}

#[test]
fn test_mark_jump_key() {
    let event = Event::Key(Key::Char('\''));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::MarkJump);
}

#[test]
fn test_jump_backward_key() {
    let event = Event::Key(Key::Ctrl('o'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::JumpBackward);
}

#[test]
fn test_jump_forward_key() {
    let event = Event::Key(Key::Ctrl('i'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::JumpForward);
}

#[test]
fn test_repeat_key() {
    let event = Event::Key(Key::Char('.'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::Repeat);
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_visual_mode_key
```

Expected: Compilation error - no variant EnterVisualMode

**Step 3: Add new InputEvent variants**

```rust
// src/input/keys.rs
// Add these variants to the InputEvent enum (around line 96):

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    // ... existing variants ...
    /// Register selection prefix (")
    RegisterSelect,

    // NEW VARIANTS:
    /// Enter visual mode (v)
    EnterVisualMode,
    /// Set mark (m)
    MarkSet,
    /// Jump to mark (')
    MarkJump,
    /// Jump backward in jump list (Ctrl-o)
    JumpBackward,
    /// Jump forward in jump list (Ctrl-i)
    JumpForward,
    /// Repeat last command (.)
    Repeat,

    /// Insert a character in insert mode
    InsertCharacter(char),
    // ... rest of variants ...
}
```

**Step 4: Add key mappings**

```rust
// src/input/keys.rs
// In the map_key_event function, add these mappings in the Normal mode match:

EditorMode::Normal => match key {
    // Ctrl-modified keys
    Key::Ctrl('d') => InputEvent::HalfPageDown,
    Key::Ctrl('u') => InputEvent::HalfPageUp,
    Key::Ctrl('f') => InputEvent::FullPageDown,
    Key::Ctrl('b') => InputEvent::FullPageUp,
    Key::Ctrl('r') => InputEvent::Redo,
    Key::Ctrl('o') => InputEvent::JumpBackward,  // NEW
    Key::Ctrl('i') => InputEvent::JumpForward,   // NEW

    // Regular keys
    Key::Char('q') => InputEvent::Quit,
    // ... existing mappings ...
    Key::Char('b') => InputEvent::PreviousAtSameOrShallowerDepth,
    Key::Char('"') => InputEvent::RegisterSelect,
    Key::Char('v') => InputEvent::EnterVisualMode,  // NEW
    Key::Char('m') => InputEvent::MarkSet,          // NEW
    Key::Char('\'') => InputEvent::MarkJump,        // NEW
    Key::Char('.') => InputEvent::Repeat,           // NEW
    // ... rest of mappings ...
}
```

**Step 5: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test input_tests
```

Expected: All 6 tests pass

**Step 6: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/input/keys.rs tests/input_tests.rs
git commit -m "feat: add input events for visual mode, marks, jumplist, and repeat

Adds InputEvent variants:
- EnterVisualMode (v)
- MarkSet (m), MarkJump (')
- JumpBackward (Ctrl-o), JumpForward (Ctrl-i)
- Repeat (.)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Implement Jump List Navigation

**Files:**
- Modify: `src/input/handler.rs`
- Test: `tests/jumplist_integration_tests.rs`

**Step 1: Write failing integration test**

```rust
// tests/jumplist_integration_tests.rs
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::input::InputHandler;
use termion::event::{Event, Key};

#[test]
fn test_jump_backward_navigation() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    // Navigate to different positions and record jumps
    // Jump to line 2 (simulate 'gg')
    state.cursor_mut().set_path(vec![0]);
    state.jumplist_mut().record_jump(state.cursor().path().to_vec());

    // Jump to line 3 (simulate 'G')
    state.cursor_mut().set_path(vec![2]);
    state.jumplist_mut().record_jump(state.cursor().path().to_vec());

    // Now jump backward
    let event = Event::Key(Key::Ctrl('o'));
    handler.handle_event(event, &mut state).unwrap();

    assert_eq!(state.cursor().path(), &vec![0]);
}

#[test]
fn test_jump_forward_navigation() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    // Record jumps
    state.cursor_mut().set_path(vec![0]);
    state.jumplist_mut().record_jump(state.cursor().path().to_vec());

    state.cursor_mut().set_path(vec![2]);
    state.jumplist_mut().record_jump(state.cursor().path().to_vec());

    // Jump backward
    let event = Event::Key(Key::Ctrl('o'));
    handler.handle_event(event, &mut state).unwrap();

    // Jump forward
    let event = Event::Key(Key::Ctrl('i'));
    handler.handle_event(event, &mut state).unwrap();

    assert_eq!(state.cursor().path(), &vec![2]);
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_jump_backward_navigation
```

Expected: Test fails - jumps not handled

**Step 3: Add jump handling to input handler**

```rust
// src/input/handler.rs
// In the handle_event method, add handling for jump events.
// Find the match on input_event and add these cases (around line 200):

let input_event = map_key_event(event, state.mode());

match input_event {
    // ... existing cases ...

    InputEvent::JumpBackward => {
        if let Some(path) = state.jumplist_mut().jump_backward() {
            // Find closest valid ancestor if path is invalid
            let valid_path = find_valid_path(&state.tree(), &path);
            state.cursor_mut().set_path(valid_path.clone());
            state.rebuild_tree_view();
            state.ensure_cursor_visible();

            let pos = state.jumplist().current_position();
            let total = state.jumplist().len();
            state.set_message_info(format!("Jump {}/{}", pos + 1, total));
        } else {
            state.set_message_info("At oldest jump".to_string());
        }
    }

    InputEvent::JumpForward => {
        if let Some(path) = state.jumplist_mut().jump_forward() {
            let valid_path = find_valid_path(&state.tree(), &path);
            state.cursor_mut().set_path(valid_path.clone());
            state.rebuild_tree_view();
            state.ensure_cursor_visible();

            let pos = state.jumplist().current_position();
            let total = state.jumplist().len();
            state.set_message_info(format!("Jump {}/{}", pos + 1, total));
        } else {
            state.set_message_info("At newest jump".to_string());
        }
    }

    // ... rest of cases ...
}

// Add this helper function at the end of the file:

/// Finds the closest valid ancestor path for a potentially invalid path.
///
/// If the path points to a deleted node, walks up the tree until
/// a valid node is found.
fn find_valid_path(tree: &jsonquill::document::tree::JsonTree, path: &[usize]) -> Vec<usize> {
    let mut current_path = path.to_vec();

    while !current_path.is_empty() {
        if tree.get_node(&current_path).is_some() {
            return current_path;
        }
        current_path.pop();
    }

    // Return root if nothing else is valid
    vec![]
}
```

**Step 4: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test jumplist_integration_tests
```

Expected: Both tests pass

**Step 5: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/input/handler.rs tests/jumplist_integration_tests.rs
git commit -m "feat: implement Ctrl-o/Ctrl-i jump list navigation

Adds input handling for:
- Ctrl-o: jump backward in history
- Ctrl-i: jump forward in history
- Shows position feedback (Jump 3/10)
- Handles deleted nodes by finding closest ancestor

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Record Jumps on Navigation Commands

**Files:**
- Modify: `src/input/handler.rs`
- Test: `tests/jumplist_integration_tests.rs`

**Step 1: Write failing test for jump recording**

```rust
// Add to tests/jumplist_integration_tests.rs

#[test]
fn test_jumps_recorded_on_goto_top() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    // Start at line 1
    state.cursor_mut().set_path(vec![1]);

    assert_eq!(state.jumplist().len(), 0);

    // Jump to top (gg)
    let event = Event::Key(Key::Char('g'));
    handler.handle_event(event, &mut state).unwrap();
    let event = Event::Key(Key::Char('g'));
    handler.handle_event(event, &mut state).unwrap();

    // Should have recorded the jump
    assert_eq!(state.jumplist().len(), 1);
}

#[test]
fn test_jumps_recorded_on_search() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Start search
    state.start_search("Alice".to_string(), true);

    let initial_len = state.jumplist().len();

    // Execute search (simulating 'n')
    if !state.search_results().is_empty() {
        state.jumplist_mut().record_jump(state.cursor().path().to_vec());
        let next_result = state.search_results()[0].clone();
        state.cursor_mut().set_path(next_result);
    }

    assert_eq!(state.jumplist().len(), initial_len + 1);
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_jumps_recorded_on_goto_top
```

Expected: Test fails - no jump recorded

**Step 3: Add jump recording before big navigation**

```rust
// src/input/handler.rs
// Modify the handling for JumpToTop, JumpToBottom, and search navigation
// to record jumps BEFORE moving cursor.

// Find InputEvent::JumpToTop handler and modify it:

InputEvent::JumpToTop => {
    if let Some(count) = state.consume_pending_count() {
        // Record jump before moving
        state.jumplist_mut().record_jump(state.cursor().path().to_vec());

        // Jump to specific line (count)
        if let Some(path) = state.tree_view().path_at_line(count.saturating_sub(1) as usize) {
            state.cursor_mut().set_path(path.to_vec());
            state.ensure_cursor_visible();
        }
    } else {
        // Check for 'gg' command
        if state.pending_command() == Some('g') {
            // Record jump before moving
            state.jumplist_mut().record_jump(state.cursor().path().to_vec());

            state.clear_pending_command();
            if let Some(first_path) = state.tree_view().lines().first().map(|l| l.path.clone()) {
                state.cursor_mut().set_path(first_path);
                state.ensure_cursor_visible();
            }
        } else {
            state.set_pending_command('g');
        }
    }
}

// Find InputEvent::JumpToBottom handler and modify it:

InputEvent::JumpToBottom => {
    if let Some(count) = state.consume_pending_count() {
        // Record jump before moving
        state.jumplist_mut().record_jump(state.cursor().path().to_vec());

        // Jump to specific line
        if let Some(path) = state.tree_view().path_at_line(count.saturating_sub(1) as usize) {
            state.cursor_mut().set_path(path.to_vec());
            state.ensure_cursor_visible();
        }
    } else {
        // Record jump before moving
        state.jumplist_mut().record_jump(state.cursor().path().to_vec());

        if let Some(last_path) = state.tree_view().lines().last().map(|l| l.path.clone()) {
            state.cursor_mut().set_path(last_path);
            state.ensure_cursor_visible();
        }
    }
}

// Find InputEvent::NextSearchResult handler and modify it:

InputEvent::NextSearchResult => {
    if !state.search_results().is_empty() {
        // Record jump before moving
        state.jumplist_mut().record_jump(state.cursor().path().to_vec());

        // ... existing search navigation code ...
    }
}

// Also update SearchKeyForward and SearchKeyBackward similarly:

InputEvent::SearchKeyForward => {
    // Record jump before search
    state.jumplist_mut().record_jump(state.cursor().path().to_vec());
    // ... existing code ...
}

InputEvent::SearchKeyBackward => {
    // Record jump before search
    state.jumplist_mut().record_jump(state.cursor().path().to_vec());
    // ... existing code ...
}
```

**Step 4: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_jumps_recorded
```

Expected: Both tests pass

**Step 5: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/input/handler.rs tests/jumplist_integration_tests.rs
git commit -m "feat: record jumps on navigation commands

Records jump before:
- gg, G, <count>G (line jumps)
- n (search navigation)
- *, # (key search)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Implement Mark Set and Jump

**Files:**
- Modify: `src/input/handler.rs`
- Test: `tests/marks_integration_tests.rs`

**Step 1: Write failing integration test**

```rust
// tests/marks_integration_tests.rs
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::input::InputHandler;
use termion::event::{Event, Key};

#[test]
fn test_set_and_jump_to_mark() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    // Move to position [1]
    state.cursor_mut().set_path(vec![1]);

    // Set mark 'a' (press 'm', then 'a')
    let event = Event::Key(Key::Char('m'));
    handler.handle_event(event, &mut state).unwrap();
    assert!(state.pending_mark_set());

    let event = Event::Key(Key::Char('a'));
    handler.handle_event(event, &mut state).unwrap();
    assert!(!state.pending_mark_set());

    // Move to different position
    state.cursor_mut().set_path(vec![2]);

    // Jump to mark 'a' (press '\'', then 'a')
    let event = Event::Key(Key::Char('\''));
    handler.handle_event(event, &mut state).unwrap();
    assert!(state.pending_mark_jump());

    let event = Event::Key(Key::Char('a'));
    handler.handle_event(event, &mut state).unwrap();
    assert!(!state.pending_mark_jump());

    // Should be back at position [1]
    assert_eq!(state.cursor().path(), &vec![1]);
}

#[test]
fn test_jump_to_unset_mark() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    // Try to jump to mark 'z' (not set)
    let event = Event::Key(Key::Char('\''));
    handler.handle_event(event, &mut state).unwrap();

    let event = Event::Key(Key::Char('z'));
    handler.handle_event(event, &mut state).unwrap();

    // Should show error message
    assert!(state.message().is_some());
    let msg = state.message().unwrap();
    assert!(msg.text.contains("not set"));
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_set_and_jump_to_mark
```

Expected: Test fails - marks not handled

**Step 3: Add mark handling to input handler**

```rust
// src/input/handler.rs
// In the handle_event method, add handling for mark events.

// First, add handling at the TOP of handle_event for pending mark state
// (similar to how awaiting_register is handled):

pub fn handle_event(&mut self, event: Event, state: &mut EditorState) -> Result<bool> {
    // Handle mark operations if awaiting mark name
    if state.pending_mark_set() {
        if let Event::Key(Key::Char(c)) = event {
            if c >= 'a' && c <= 'z' {
                // Set mark at current cursor position
                state.marks_mut().set_mark(c, state.cursor().path().to_vec());
                state.set_pending_mark_set(false);
                state.set_message_info(format!("Mark '{}' set", c));
                return Ok(false);
            }
        }
        // Invalid key or Esc cancels
        state.set_pending_mark_set(false);
        state.set_message_info("Cancelled".to_string());
        return Ok(false);
    }

    if state.pending_mark_jump() {
        if let Event::Key(Key::Char(c)) = event {
            if c >= 'a' && c <= 'z' {
                if let Some(path) = state.marks().get_mark(c) {
                    // Record current position in jump list
                    state.jumplist_mut().record_jump(state.cursor().path().to_vec());

                    // Jump to mark (handle deleted nodes)
                    let valid_path = find_valid_path(&state.tree(), path);
                    state.cursor_mut().set_path(valid_path.clone());
                    state.rebuild_tree_view();
                    state.ensure_cursor_visible();

                    state.set_pending_mark_jump(false);

                    if &valid_path != path {
                        state.set_message_warning(format!(
                            "Mark '{}' position no longer exists, jumped to nearest node", c
                        ));
                    } else {
                        state.set_message_info(format!("Jumped to mark '{}'", c));
                    }
                } else {
                    state.set_message_error(format!("Mark '{}' not set", c));
                    state.set_pending_mark_jump(false);
                }
                return Ok(false);
            }
        }
        // Invalid key or Esc cancels
        state.set_pending_mark_jump(false);
        state.set_message_info("Cancelled".to_string());
        return Ok(false);
    }

    // ... existing event handling code ...

    // Then add cases for MarkSet and MarkJump in the match on input_event:

    InputEvent::MarkSet => {
        state.set_pending_mark_set(true);
        state.set_message_info("Mark: ".to_string());
    }

    InputEvent::MarkJump => {
        state.set_pending_mark_jump(true);
        state.set_message_info("Jump to mark: ".to_string());
    }

    // ... rest of cases ...
}
```

**Step 4: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test marks_integration_tests
```

Expected: Both tests pass

**Step 5: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/input/handler.rs tests/marks_integration_tests.rs
git commit -m "feat: implement mark set (m) and jump (') commands

Adds two-key sequences:
- m{a-z}: set mark at current position
- '{a-z}: jump to mark position
- Shows feedback messages
- Integrates with jump list
- Handles deleted nodes

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Implement Visual Mode Entry and Exit

**Files:**
- Modify: `src/input/handler.rs`
- Test: `tests/visual_mode_tests.rs`

**Step 1: Write failing test**

```rust
// tests/visual_mode_tests.rs
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::editor::mode::EditorMode;
use jsonquill::input::InputHandler;
use termion::event::{Event, Key};

#[test]
fn test_enter_visual_mode() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    state.cursor_mut().set_path(vec![0]);

    // Press 'v' to enter visual mode
    let event = Event::Key(Key::Char('v'));
    handler.handle_event(event, &mut state).unwrap();

    assert_eq!(state.mode(), &EditorMode::Visual);
    assert_eq!(state.visual_anchor(), Some(&vec![0]));
    assert_eq!(state.visual_selection().len(), 1);
}

#[test]
fn test_exit_visual_mode_with_escape() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Number(1.0)));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    // Enter visual mode
    state.enter_visual_mode();
    assert_eq!(state.mode(), &EditorMode::Visual);

    // Press Esc to exit
    let event = Event::Key(Key::Esc);
    handler.handle_event(event, &mut state).unwrap();

    assert_eq!(state.mode(), &EditorMode::Normal);
    assert_eq!(state.visual_anchor(), None);
    assert_eq!(state.visual_selection().len(), 0);
}

#[test]
fn test_visual_selection_extends_on_movement() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    state.cursor_mut().set_path(vec![0]);

    // Enter visual mode
    let event = Event::Key(Key::Char('v'));
    handler.handle_event(event, &mut state).unwrap();

    // Move down twice
    let event = Event::Key(Key::Char('j'));
    handler.handle_event(event, &mut state).unwrap();
    let event = Event::Key(Key::Char('j'));
    handler.handle_event(event, &mut state).unwrap();

    // Selection should include 3 nodes
    assert_eq!(state.visual_selection().len(), 3);
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_enter_visual_mode
```

Expected: Test fails - visual mode not entered

**Step 3: Add visual mode handling**

```rust
// src/input/handler.rs
// Add handling for EnterVisualMode and update ExitMode:

match input_event {
    // ... existing cases ...

    InputEvent::EnterVisualMode => {
        if state.mode() == &EditorMode::Normal {
            state.enter_visual_mode();
        }
    }

    InputEvent::ExitMode => {
        match state.mode() {
            EditorMode::Insert => {
                state.cancel_editing();
                state.set_mode(EditorMode::Normal);
            }
            EditorMode::Command => {
                state.clear_command_buffer();
                state.set_mode(EditorMode::Normal);
            }
            EditorMode::Search => {
                state.clear_search_buffer();
                state.set_mode(EditorMode::Normal);
            }
            EditorMode::Visual => {
                state.exit_visual_mode();
            }
            EditorMode::Normal => {}
        }
    }

    // Update movement handlers to update visual selection when in Visual mode
    InputEvent::MoveDown => {
        // ... existing movement code ...

        // Update visual selection if in visual mode
        if state.mode() == &EditorMode::Visual {
            state.update_visual_selection();
        }
    }

    InputEvent::MoveUp => {
        // ... existing movement code ...

        if state.mode() == &EditorMode::Visual {
            state.update_visual_selection();
        }
    }

    // ... repeat for other movement commands (MoveLeft, MoveRight, etc.) ...
}
```

**Step 4: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test visual_mode_tests
```

Expected: All 3 tests pass

**Step 5: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/input/handler.rs tests/visual_mode_tests.rs
git commit -m "feat: implement visual mode entry, exit, and selection

Adds:
- Enter visual mode with 'v' key
- Exit with Esc
- Selection extends on cursor movement
- Visual selection state tracking

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Implement Visual Mode Operations

**Files:**
- Modify: `src/input/handler.rs`
- Modify: `src/editor/state.rs`
- Test: `tests/visual_mode_tests.rs`

**Step 1: Write failing test**

```rust
// Add to tests/visual_mode_tests.rs

#[test]
fn test_visual_delete() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    state.cursor_mut().set_path(vec![0]);

    // Enter visual mode and select 2 nodes
    let event = Event::Key(Key::Char('v'));
    handler.handle_event(event, &mut state).unwrap();
    let event = Event::Key(Key::Char('j'));
    handler.handle_event(event, &mut state).unwrap();

    // Delete selection
    let event = Event::Key(Key::Char('d'));
    handler.handle_event(event, &mut state).unwrap();

    // Should be back in normal mode
    assert_eq!(state.mode(), &EditorMode::Normal);

    // Should have deleted 2 nodes, leaving 1
    if let JsonValue::Array(ref arr) = state.tree().root().value() {
        assert_eq!(arr.len(), 1);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_visual_yank() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    state.cursor_mut().set_path(vec![0]);

    // Enter visual mode and select both nodes
    let event = Event::Key(Key::Char('v'));
    handler.handle_event(event, &mut state).unwrap();
    let event = Event::Key(Key::Char('j'));
    handler.handle_event(event, &mut state).unwrap();

    // Yank selection
    let event = Event::Key(Key::Char('y'));
    handler.handle_event(event, &mut state).unwrap();

    // Should be back in normal mode
    assert_eq!(state.mode(), &EditorMode::Normal);

    // Register should contain both nodes
    let yanked = state.registers().get_unnamed();
    assert!(yanked.is_some());
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_visual_delete
```

Expected: Test fails - visual delete not implemented

**Step 3: Add helper methods to EditorState**

```rust
// src/editor/state.rs
// Add these methods to handle visual operations:

/// Deletes all nodes in the visual selection.
///
/// Returns Ok(()) on success, Err if deletion fails.
pub fn delete_visual_selection(&mut self) -> anyhow::Result<()> {
    if self.visual_selection.is_empty() {
        return Ok(());
    }

    // Sort paths by depth (deepest first) to avoid invalidating paths
    let mut paths = self.visual_selection.clone();
    paths.sort_by(|a, b| b.len().cmp(&a.len()).then(b.cmp(a)));

    // Delete each node
    for path in &paths {
        self.tree.delete_node(path)?;
    }

    self.mark_dirty();
    self.save_undo_snapshot();
    self.rebuild_tree_view();

    // Move cursor to first deleted position or parent
    if let Some(first_path) = paths.first() {
        let valid_path = if first_path.is_empty() {
            vec![]
        } else {
            let mut parent_path = first_path[..first_path.len() - 1].to_vec();
            while !parent_path.is_empty() && self.tree.get_node(&parent_path).is_none() {
                parent_path.pop();
            }
            parent_path
        };
        self.cursor.set_path(valid_path);
    }

    self.exit_visual_mode();
    Ok(())
}

/// Yanks all nodes in the visual selection to a register.
pub fn yank_visual_selection(&mut self, register: Option<char>, append: bool) {
    if self.visual_selection.is_empty() {
        return;
    }

    // Collect all nodes in selection
    let mut nodes = Vec::new();
    for path in &self.visual_selection {
        if let Some(node) = self.tree.get_node(path) {
            nodes.push(node.clone());
        }
    }

    // Store as array in register
    if !nodes.is_empty() {
        let reg = register.unwrap_or('"');
        if append && reg != '"' {
            // Append mode: get existing content and extend
            if let Some(existing) = self.registers.get(reg) {
                if let JsonValue::Array(mut arr) = existing.value().clone() {
                    arr.extend(nodes);
                    let combined = JsonNode::new(JsonValue::Array(arr));
                    self.registers.set(reg, combined, None);
                } else {
                    // Existing is not array, replace with array
                    let combined = JsonNode::new(JsonValue::Array(nodes));
                    self.registers.set(reg, combined, None);
                }
            } else {
                let combined = JsonNode::new(JsonValue::Array(nodes));
                self.registers.set(reg, combined, None);
            }
        } else {
            let combined = JsonNode::new(JsonValue::Array(nodes));
            self.registers.set(reg, combined, None);
        }

        self.set_message_info(format!("Yanked {} nodes", self.visual_selection.len()));
    }

    self.exit_visual_mode();
}
```

**Step 4: Add visual operation handling to input handler**

```rust
// src/input/handler.rs
// Modify Delete and Yank handlers to check for Visual mode:

InputEvent::Delete => {
    if state.mode() == &EditorMode::Visual {
        // Visual mode delete
        state.delete_visual_selection()?;
    } else {
        // Normal mode delete (existing code)
        let count = state.consume_pending_count().unwrap_or(1);
        // ... existing delete code ...

        // Record command for repeat
        state.set_last_command(RepeatableCommand::Delete { count });
    }
}

InputEvent::Yank => {
    if state.mode() == &EditorMode::Visual {
        // Visual mode yank
        let register = state.consume_pending_register();
        let append = state.consume_append_mode();
        state.yank_visual_selection(register, append);
    } else {
        // Normal mode yank (existing code)
        let count = state.consume_pending_count().unwrap_or(1);
        // ... existing yank code ...

        // Record command for repeat
        state.set_last_command(RepeatableCommand::Yank { count });
    }
}
```

**Step 5: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_visual_delete test_visual_yank
```

Expected: Both tests pass

**Step 6: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/input/handler.rs src/editor/state.rs tests/visual_mode_tests.rs
git commit -m "feat: implement visual mode delete and yank operations

Adds:
- Visual delete (d): deletes all selected nodes
- Visual yank (y): yanks selection to register
- Operations exit visual mode after completion
- Integrates with register system

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Implement Repeat Command

**Files:**
- Modify: `src/input/handler.rs`
- Modify: `src/editor/state.rs`
- Test: `tests/repeat_tests.rs`

**Step 1: Write failing integration test**

```rust
// Update tests/repeat_tests.rs with integration tests:

use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::input::InputHandler;
use termion::event::{Event, Key};

#[test]
fn test_repeat_delete() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    state.cursor_mut().set_path(vec![0]);

    // Delete first node (dd)
    let event = Event::Key(Key::Char('d'));
    handler.handle_event(event, &mut state).unwrap();
    let event = Event::Key(Key::Char('d'));
    handler.handle_event(event, &mut state).unwrap();

    // Repeat delete (.)
    let event = Event::Key(Key::Char('.'));
    handler.handle_event(event, &mut state).unwrap();

    // Should have deleted 2 nodes total, leaving 1
    if let JsonValue::Array(ref arr) = state.tree().root().value() {
        assert_eq!(arr.len(), 1);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_repeat_with_count() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
        JsonNode::new(JsonValue::Number(4.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    state.cursor_mut().set_path(vec![0]);

    // Delete first node
    let event = Event::Key(Key::Char('d'));
    handler.handle_event(event, &mut state).unwrap();
    let event = Event::Key(Key::Char('d'));
    handler.handle_event(event, &mut state).unwrap();

    // Repeat 2 times (2.)
    let event = Event::Key(Key::Char('2'));
    handler.handle_event(event, &mut state).unwrap();
    let event = Event::Key(Key::Char('.'));
    handler.handle_event(event, &mut state).unwrap();

    // Should have deleted 3 nodes total (1 + 2*1), leaving 1
    if let JsonValue::Array(ref arr) = state.tree().root().value() {
        assert_eq!(arr.len(), 1);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_no_command_to_repeat() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);
    let mut handler = InputHandler::new();

    // Try to repeat with no last command
    let event = Event::Key(Key::Char('.'));
    handler.handle_event(event, &mut state).unwrap();

    // Should show message
    assert!(state.message().is_some());
    let msg = state.message().unwrap();
    assert!(msg.text.contains("No command to repeat"));
}
```

**Step 2: Run test to verify it fails**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test test_repeat_delete
```

Expected: Test fails - repeat not implemented

**Step 3: Add repeat command method to EditorState**

```rust
// src/editor/state.rs
// Add this method:

/// Repeats the last repeatable command.
///
/// Returns Ok(()) on success, Err if no command to repeat or repeat fails.
pub fn repeat_last_command(&mut self, count: u32) -> anyhow::Result<()> {
    let cmd = self.last_command.clone();

    let Some(cmd) = cmd else {
        self.set_message_info("No command to repeat".to_string());
        return Ok(());
    };

    // Execute the command 'count' times
    for _ in 0..count {
        match &cmd {
            RepeatableCommand::Delete { count: delete_count } => {
                for _ in 0..*delete_count {
                    self.delete_current_node()?;
                }
            }
            RepeatableCommand::Yank { count: yank_count } => {
                for _ in 0..*yank_count {
                    self.yank_current_node(None, false)?;
                }
            }
            RepeatableCommand::Paste { before } => {
                self.paste_from_register(None, *before)?;
            }
            RepeatableCommand::Add { value, key } => {
                self.add_node_after_cursor(value.clone(), key.clone())?;
            }
            RepeatableCommand::AddArray => {
                self.add_array_after_cursor()?;
            }
            RepeatableCommand::AddObject => {
                self.add_object_after_cursor()?;
            }
            RepeatableCommand::Rename { new_key } => {
                self.rename_current_key(new_key.clone())?;
            }
            RepeatableCommand::ChangeValue { new_value } => {
                self.change_current_value(new_value.clone())?;
            }
        }
    }

    // The repeated command becomes the new last command
    self.last_command = Some(cmd);
    self.set_message_info("Repeated".to_string());

    Ok(())
}
```

**Step 4: Add repeat handling to input handler**

```rust
// src/input/handler.rs
// Add case for Repeat event:

InputEvent::Repeat => {
    let count = state.consume_pending_count().unwrap_or(1);
    state.repeat_last_command(count)?;
}
```

**Step 5: Update operation handlers to record commands**

```rust
// src/input/handler.rs
// Make sure Delete, Yank, Paste, Add operations record last_command.
// For example, in the Delete handler (normal mode):

InputEvent::Delete => {
    if state.mode() == &EditorMode::Visual {
        state.delete_visual_selection()?;
    } else {
        let count = state.consume_pending_count().unwrap_or(1);

        if state.pending_command() == Some('d') {
            state.clear_pending_command();

            for _ in 0..count {
                state.delete_current_node()?;
            }

            // Record for repeat
            state.set_last_command(RepeatableCommand::Delete { count });
        } else {
            state.set_pending_command('d');
        }
    }
}

// Similar updates for Yank, Paste, Add operations...
```

**Step 6: Run tests to verify they pass**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test repeat_tests
```

Expected: All tests pass (including original enum tests and new integration tests)

**Step 7: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/input/handler.rs src/editor/state.rs tests/repeat_tests.rs
git commit -m "feat: implement repeat command (.) functionality

Adds:
- Repeat last editing command with '.'
- Count prefix support (3. repeats 3 times)
- Records delete, yank, paste, add operations
- Shows 'No command to repeat' when appropriate

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 13: Add Visual Selection Rendering

**Files:**
- Modify: `src/theme/colors.rs`
- Modify: `src/ui/tree_view.rs`
- Test: Manual testing

**Step 1: Add visual_selection_bg to theme**

```rust
// src/theme/colors.rs
// Add field to ThemeColors struct:

pub struct ThemeColors {
    // ... existing fields ...
    pub search_match_bg: Color,

    /// Background color for visual mode selection
    pub visual_selection_bg: Color,
}

// Update default_dark() method:
pub fn default_dark() -> Self {
    Self {
        // ... existing colors ...
        search_match_bg: Color::Rgb(60, 60, 0),
        visual_selection_bg: Color::Rgb(40, 40, 80),  // Dark blue
    }
}

// Update default_light() method:
pub fn default_light() -> Self {
    Self {
        // ... existing colors ...
        search_match_bg: Color::Rgb(255, 255, 150),
        visual_selection_bg: Color::Rgb(200, 200, 255),  // Light blue
    }
}

// Update all other theme implementations (gruvbox, nord, etc.) similarly
```

**Step 2: Update tree view rendering**

```rust
// src/ui/tree_view.rs
// In the render_tree_view function, add visual selection highlighting.
// Find where lines are rendered and add check for visual selection:

for (idx, line) in visible_lines.iter().enumerate() {
    let y_offset = idx as u16;
    let is_cursor = cursor_visible_idx == Some(idx);
    let is_selected = state.visual_selection().contains(&line.path);

    // Determine background color
    let bg_color = if is_selected && state.mode() == &EditorMode::Visual {
        theme.colors.visual_selection_bg
    } else if is_cursor {
        theme.colors.cursor_bg
    } else {
        theme.colors.bg
    };

    // ... rest of rendering with bg_color ...
}
```

**Step 3: Update status line to show selection count**

```rust
// src/ui/status_line.rs
// In the render_status_line function, add visual mode info:

if state.mode() == &EditorMode::Visual {
    let count = state.visual_selection().len();
    let info = format!(" {} nodes selected", count);
    // Render this info in the status line
}
```

**Step 4: Manual testing**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo run -- tests/fixtures/sample.json
```

Test:
1. Press `v` to enter visual mode
2. Move cursor with `j`/`k`
3. Verify selected nodes are highlighted
4. Verify status line shows "VISUAL" and node count
5. Press `Esc` to exit, verify highlighting clears

**Step 5: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/theme/colors.rs src/ui/tree_view.rs src/ui/status_line.rs
git commit -m "feat: add visual selection rendering and UI feedback

Adds:
- visual_selection_bg to all themes
- Background highlighting for selected nodes
- Selection count in status line
- Updates all built-in themes

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 14: Update Help Overlay

**Files:**
- Modify: `src/ui/help_overlay.rs`

**Step 1: Add documentation for new features**

```rust
// src/ui/help_overlay.rs
// Add sections for the new features in the help text:

const HELP_TEXT: &str = "
# jsonquill Help

## Visual Mode
v           - Enter visual mode (select nodes)
j/k         - Extend selection up/down
d           - Delete selected nodes
y           - Yank selected nodes
p/P         - Paste after/before selection
Esc         - Exit visual mode

## Marks
m{a-z}      - Set mark at current position
'{a-z}      - Jump to mark

## Jump List
Ctrl-o      - Jump backward in history
Ctrl-i      - Jump forward in history

## Repeat
.           - Repeat last edit command
3.          - Repeat last command 3 times

## Navigation (NORMAL mode)
... (existing navigation help) ...
";
```

**Step 2: Test help overlay**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo run
# Press ? or F1 to open help
# Verify new sections are visible
```

**Step 3: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add src/ui/help_overlay.rs
git commit -m "docs: update help overlay with new features

Adds documentation for:
- Visual mode commands
- Mark set/jump
- Jump list navigation
- Repeat command

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 15: Update CLAUDE.md Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update usage section**

```markdown
<!-- In CLAUDE.md, update the "Current Status" and "Usage" sections -->

## Current Status

**Working Features:**
✅ Visual mode (v) for multi-node selection and operations
✅ Marks (m/') for bookmarking positions
✅ Jump list (Ctrl-o/Ctrl-i) for navigation history
✅ Repeat command (.) for replaying edits
... (existing features) ...

## Usage

# Visual Mode
v           - Enter visual mode at cursor
j/k/h/l     - Extend selection with movement
d           - Delete all selected nodes
y           - Yank all selected nodes to register
p/P         - Paste after/before selection
Esc         - Exit visual mode

# Marks and Jumps
m{a-z}      - Set mark (a-z) at current position
'{a-z}      - Jump to mark position
Ctrl-o      - Jump backward in jump history
Ctrl-i      - Jump forward in jump history

# Repeat
.           - Repeat last editing command
3.          - Repeat last command 3 times

... (rest of existing usage docs) ...
```

**Step 2: Commit**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with new features

Documents visual mode, marks, jump list, and repeat command
in usage section and feature checklist.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 16: Run Full Test Suite and Format

**Files:**
- All source files

**Step 1: Run all tests**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo test
```

Expected: All tests pass

**Step 2: Run cargo fmt**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo fmt
```

**Step 3: Run cargo clippy**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cargo clippy -- -D warnings
```

Expected: No warnings

**Step 4: Fix any issues**

If clippy reports warnings, fix them and re-run.

**Step 5: Commit formatting**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
git add -A
git commit -m "chore: run cargo fmt and fix clippy warnings

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 17: Final Integration Testing

**Files:**
- Manual testing

**Step 1: Create test JSON file**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
cat > /tmp/test_features.json << 'EOF'
{
  "users": [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25},
    {"name": "Charlie", "age": 35}
  ],
  "settings": {
    "theme": "dark",
    "notifications": true
  }
}
EOF
```

**Step 2: Test visual mode**

```bash
cargo run -- /tmp/test_features.json
```

Manual test checklist:
- [ ] Press `v` at users[0], move down 2 lines, press `d` - deletes 2 users
- [ ] Press `u` to undo
- [ ] Press `v`, select 2 nodes, press `y`, move elsewhere, press `p` - pastes
- [ ] Visual selection highlights correctly

**Step 3: Test marks**

- [ ] Press `m` then `a` at "users" - sets mark
- [ ] Navigate to "settings"
- [ ] Press `'` then `a` - jumps back to "users"
- [ ] Message shows "Jumped to mark 'a'"

**Step 4: Test jump list**

- [ ] Press `gg` to jump to top
- [ ] Press `G` to jump to bottom
- [ ] Press `Ctrl-o` - jumps back to top
- [ ] Press `Ctrl-i` - jumps forward to bottom
- [ ] Messages show "Jump 1/2" etc.

**Step 5: Test repeat**

- [ ] Press `dd` to delete a node
- [ ] Press `.` - deletes another node
- [ ] Press `u` twice to undo
- [ ] Press `3dd` to delete 3 nodes
- [ ] Press `.` - deletes another 3 nodes

**Step 6: Document test results**

If all tests pass, proceed. If any fail, fix bugs and re-test.

**Step 7: Commit test results note**

```bash
cd ~/.config/superpowers/worktrees/jsonquill/visual-marks-jumplist-repeat
# Create a test results file
cat > docs/test-results-visual-marks-jumplist-repeat.md << 'EOF'
# Integration Test Results

Date: 2026-01-29

## Visual Mode
- ✅ Enter/exit visual mode
- ✅ Selection extends on movement
- ✅ Delete selection
- ✅ Yank selection
- ✅ Paste operations
- ✅ Visual highlighting

## Marks
- ✅ Set marks (m{a-z})
- ✅ Jump to marks ('{a-z})
- ✅ Error on unset mark
- ✅ Deleted node handling

## Jump List
- ✅ Ctrl-o backward navigation
- ✅ Ctrl-i forward navigation
- ✅ Recording on big jumps
- ✅ Position feedback messages

## Repeat Command
- ✅ Repeat delete
- ✅ Repeat yank
- ✅ Repeat with count prefix
- ✅ No command error handling

All features working as designed.
EOF

git add docs/test-results-visual-marks-jumplist-repeat.md
git commit -m "docs: add integration test results for new features

All features tested and working:
- Visual mode operations
- Mark set/jump
- Jump list navigation
- Repeat command

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Implementation Complete

**Summary:**

You have successfully implemented:
1. ✅ Jump list with Ctrl-o/Ctrl-i navigation
2. ✅ Marks with m{a-z} set and '{a-z} jump
3. ✅ Visual mode with v entry and selection operations
4. ✅ Repeat command (.) for editing operations

**Total commits:** 17 well-scoped commits following TDD
**Total tests:** ~30+ new tests across all features
**Code quality:** All tests passing, formatted, no clippy warnings

**Next steps:**
- Use @superpowers:finishing-a-development-branch to merge or create PR
- Consider future enhancements from design doc
