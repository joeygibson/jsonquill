# Editing Operations Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement functional insert mode for editing JSON values, delete operation for removing nodes, and paste operation for duplicating yanked content.

**Architecture:** Add edit buffer to EditorState for insert mode, implement character input handling in insert mode, add tree modification operations (delete node, insert node), handle different value types (string, number, boolean, null), ensure proper dirty flag and tree view updates.

**Tech Stack:** Rust, existing jsonquill architecture (ratatui, crossterm, serde_json)

---

## Phase 1: Insert Mode - Edit Buffer & State

### Task 1: Add Edit Buffer to EditorState

**Files:**
- Modify: `src/editor/state.rs`
- Modify: `tests/editor_tests.rs`

**Step 1: Write test for edit buffer initialization**

```rust
// tests/editor_tests.rs (add to end of file)

#[test]
fn test_edit_buffer_starts_empty() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("test".to_string())));
    let state = EditorState::new(tree);

    assert_eq!(state.edit_buffer(), None);
}

#[test]
fn test_start_editing_string_value() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ])));
    let mut state = EditorState::new(tree);

    // Move cursor to first element
    state.cursor_mut().set_path(vec![0]);

    // Start editing
    state.start_editing();

    assert!(state.edit_buffer().is_some());
    assert_eq!(state.edit_buffer().unwrap(), "Alice");
}

#[test]
fn test_start_editing_number_value() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("count".to_string(), JsonNode::new(JsonValue::Number(42.0))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    assert_eq!(state.edit_buffer().unwrap(), "42");
}

#[test]
fn test_start_editing_boolean_value() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("active".to_string(), JsonNode::new(JsonValue::Boolean(true))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    assert_eq!(state.edit_buffer().unwrap(), "true");
}

#[test]
fn test_start_editing_null_value() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("data".to_string(), JsonNode::new(JsonValue::Null)),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    assert_eq!(state.edit_buffer().unwrap(), "null");
}

#[test]
fn test_cancel_editing() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();
    assert!(state.edit_buffer().is_some());

    state.cancel_editing();
    assert_eq!(state.edit_buffer(), None);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_edit_buffer`
Expected: FAIL with "method not found"

**Step 3: Add edit buffer field to EditorState**

```rust
// src/editor/state.rs (add to struct fields after line 112)
pub struct EditorState {
    tree: JsonTree,
    mode: EditorMode,
    cursor: Cursor,
    dirty: bool,
    filename: Option<String>,
    tree_view: TreeViewState,
    message: Option<Message>,
    command_buffer: String,
    show_help: bool,
    help_scroll: usize,
    pending_theme: Option<String>,
    current_theme: String,
    clipboard: Option<JsonNode>,
    search_buffer: String,
    search_results: Vec<Vec<usize>>,
    search_index: usize,
    show_line_numbers: bool,
    edit_buffer: Option<String>,  // Add this field
}
```

**Step 4: Update EditorState::new to initialize edit_buffer**

```rust
// src/editor/state.rs (update new() method around line 167)
pub fn new(tree: JsonTree) -> Self {
    let mut tree_view = TreeViewState::new();
    tree_view.expand_all(&tree);
    tree_view.rebuild(&tree);

    let mut cursor = Cursor::new();
    if let Some(first_line) = tree_view.lines().first() {
        cursor.set_path(first_line.path.clone());
    }

    Self {
        tree,
        mode: EditorMode::Normal,
        cursor,
        dirty: false,
        filename: None,
        tree_view,
        message: None,
        command_buffer: String::new(),
        show_help: false,
        help_scroll: 0,
        pending_theme: None,
        current_theme: "default-dark".to_string(),
        clipboard: None,
        search_buffer: String::new(),
        search_results: Vec::new(),
        search_index: 0,
        show_line_numbers: true,
        edit_buffer: None,  // Add this field
    }
}
```

**Step 5: Implement edit buffer methods**

```rust
// src/editor/state.rs (add methods before the closing brace of impl EditorState, around line 841)

    /// Returns the current edit buffer content, if editing.
    pub fn edit_buffer(&self) -> Option<&str> {
        self.edit_buffer.as_deref()
    }

    /// Starts editing the node at the current cursor position.
    /// Loads the node's value into the edit buffer as a string.
    pub fn start_editing(&mut self) {
        let path = self.cursor.path();
        if let Some(node) = self.tree.get_node(path) {
            let value_str = match node.value() {
                crate::document::node::JsonValue::String(s) => s.clone(),
                crate::document::node::JsonValue::Number(n) => {
                    // Format number without unnecessary trailing zeros
                    if n.fract() == 0.0 {
                        format!("{:.0}", n)
                    } else {
                        n.to_string()
                    }
                }
                crate::document::node::JsonValue::Boolean(b) => b.to_string(),
                crate::document::node::JsonValue::Null => "null".to_string(),
                crate::document::node::JsonValue::Object(_) => return, // Can't edit containers
                crate::document::node::JsonValue::Array(_) => return,  // Can't edit containers
            };
            self.edit_buffer = Some(value_str);
        }
    }

    /// Cancels editing and clears the edit buffer without saving changes.
    pub fn cancel_editing(&mut self) {
        self.edit_buffer = None;
    }

    /// Appends a character to the edit buffer.
    pub fn push_to_edit_buffer(&mut self, ch: char) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.push(ch);
        }
    }

    /// Removes the last character from the edit buffer.
    pub fn pop_from_edit_buffer(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.pop();
        }
    }

    /// Clears the edit buffer entirely.
    pub fn clear_edit_buffer(&mut self) {
        if let Some(ref mut buffer) = self.edit_buffer {
            buffer.clear();
        }
    }
```

**Step 6: Run tests to verify they pass**

Run: `cargo test test_edit_buffer`
Expected: All 6 tests PASS

**Step 7: Commit**

```bash
git add src/editor/state.rs tests/editor_tests.rs
git commit -m "feat: add edit buffer to EditorState for insert mode

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Commit Edited Values to Tree

**Files:**
- Modify: `src/editor/state.rs`
- Modify: `tests/editor_tests.rs`

**Step 1: Write test for committing edited values**

```rust
// tests/editor_tests.rs (add)

#[test]
fn test_commit_editing_string() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    // Modify the buffer
    state.clear_edit_buffer();
    state.push_to_edit_buffer('B');
    state.push_to_edit_buffer('o');
    state.push_to_edit_buffer('b');

    // Commit the change
    let result = state.commit_editing();
    assert!(result.is_ok());
    assert!(state.is_dirty());
    assert_eq!(state.edit_buffer(), None);

    // Verify the tree was updated
    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "Bob"),
        _ => panic!("Expected string value"),
    }
}

#[test]
fn test_commit_editing_number() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("count".to_string(), JsonNode::new(JsonValue::Number(42.0))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    state.clear_edit_buffer();
    for ch in "123.45".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_ok());

    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::Number(n) => assert_eq!(*n, 123.45),
        _ => panic!("Expected number value"),
    }
}

#[test]
fn test_commit_editing_boolean() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("active".to_string(), JsonNode::new(JsonValue::Boolean(true))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    state.clear_edit_buffer();
    for ch in "false".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_ok());

    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::Boolean(b) => assert_eq!(*b, false),
        _ => panic!("Expected boolean value"),
    }
}

#[test]
fn test_commit_editing_null() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("data".to_string(), JsonNode::new(JsonValue::String("old".to_string()))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    state.clear_edit_buffer();
    for ch in "null".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_ok());

    let node = state.tree().get_node(&[0]).unwrap();
    assert!(matches!(node.value(), JsonValue::Null));
}

#[test]
fn test_commit_editing_invalid_number() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("count".to_string(), JsonNode::new(JsonValue::Number(42.0))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    state.clear_edit_buffer();
    for ch in "not-a-number".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid number"));
}

#[test]
fn test_commit_editing_invalid_boolean() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("active".to_string(), JsonNode::new(JsonValue::Boolean(true))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    state.clear_edit_buffer();
    for ch in "maybe".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be true or false"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_commit_editing`
Expected: FAIL with "method not found"

**Step 3: Implement commit_editing method**

```rust
// src/editor/state.rs (add method after cancel_editing, around line 875)

    /// Commits the edited value from the buffer to the tree.
    /// Parses the buffer according to the original node's type and updates the tree.
    /// Returns an error if the buffer content is invalid for the node's type.
    pub fn commit_editing(&mut self) -> anyhow::Result<()> {
        use crate::document::node::JsonValue;
        use anyhow::{anyhow, Context};

        let buffer_content = self.edit_buffer.as_ref()
            .ok_or_else(|| anyhow!("No active edit buffer"))?
            .clone();

        let path = self.cursor.path();
        let node = self.tree.get_node(path)
            .ok_or_else(|| anyhow!("Node not found at cursor"))?;

        // Determine the new value based on the original node's type
        let new_value = match node.value() {
            JsonValue::String(_) => JsonValue::String(buffer_content),
            JsonValue::Number(_) => {
                let num = buffer_content.parse::<f64>()
                    .context("Invalid number format")?;
                JsonValue::Number(num)
            }
            JsonValue::Boolean(_) => {
                let bool_val = match buffer_content.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(anyhow!("Boolean value must be true or false")),
                };
                JsonValue::Boolean(bool_val)
            }
            JsonValue::Null => {
                if buffer_content != "null" {
                    return Err(anyhow!("Null value must be 'null'"));
                }
                JsonValue::Null
            }
            JsonValue::Object(_) | JsonValue::Array(_) => {
                return Err(anyhow!("Cannot edit container types"));
            }
        };

        // Update the node in the tree
        let node_mut = self.tree.get_node_mut(path)
            .ok_or_else(|| anyhow!("Node not found for update"))?;
        *node_mut.value_mut() = new_value;

        // Clear edit buffer and mark dirty
        self.edit_buffer = None;
        self.mark_dirty();
        self.rebuild_tree_view();

        Ok(())
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_commit_editing`
Expected: All 6 tests PASS

**Step 5: Commit**

```bash
git add src/editor/state.rs tests/editor_tests.rs
git commit -m "feat: implement commit_editing to save buffer changes to tree

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 2: Insert Mode - Input Handling

### Task 3: Handle Character Input in Insert Mode

**Files:**
- Modify: `src/input/handler.rs`
- Modify: `src/input/keys.rs`

**Step 1: Add InsertCharacter input event**

```rust
// src/input/keys.rs (add to InputEvent enum around line 40)
pub enum InputEvent {
    Quit,
    MoveDown,
    MoveUp,
    MoveLeft,
    MoveRight,
    EnterInsertMode,
    EnterCommandMode,
    EnterSearchMode,
    ExitMode,
    Delete,
    Yank,
    Paste,
    NextSearchResult,
    InsertCharacter(char),     // Add this variant
    InsertBackspace,           // Add this variant
    InsertEnter,               // Add this variant
    Unknown,
}
```

**Step 2: Update key mapping for insert mode**

```rust
// src/input/keys.rs (update map_key_event function, around line 90)
        EditorMode::Insert => match key.code {
            KeyCode::Esc => InputEvent::ExitMode,
            KeyCode::Char(c) => InputEvent::InsertCharacter(c),
            KeyCode::Backspace => InputEvent::InsertBackspace,
            KeyCode::Enter => InputEvent::InsertEnter,
            _ => InputEvent::Unknown,
        },
```

**Step 3: Handle insert mode events in handler**

```rust
// src/input/handler.rs (update handle_event method)
// Find the section that handles Command mode (around line 98) and add similar handling for Insert mode before it:

        // Handle insert mode separately for character input
        if *state.mode() == EditorMode::Insert {
            match key.code {
                KeyCode::Char(c) => {
                    state.push_to_edit_buffer(c);
                    return Ok(false);
                }
                KeyCode::Backspace => {
                    state.pop_from_edit_buffer();
                    return Ok(false);
                }
                KeyCode::Enter => {
                    // Commit the edit
                    use crate::editor::state::MessageLevel;
                    match state.commit_editing() {
                        Ok(_) => {
                            state.set_mode(EditorMode::Normal);
                            state.set_message("Value updated".to_string(), MessageLevel::Info);
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Invalid value: {}", e),
                                MessageLevel::Error,
                            );
                        }
                    }
                    return Ok(false);
                }
                KeyCode::Esc => {
                    state.cancel_editing();
                    state.set_mode(EditorMode::Normal);
                    use crate::editor::state::MessageLevel;
                    state.set_message("Edit cancelled".to_string(), MessageLevel::Info);
                    return Ok(false);
                }
                _ => return Ok(false),
            }
        }
```

**Step 4: Update EnterInsertMode handler to start editing**

```rust
// src/input/handler.rs (find InputEvent::EnterInsertMode, around line 198)
                InputEvent::EnterInsertMode => {
                    use crate::editor::state::MessageLevel;
                    state.start_editing();
                    if state.edit_buffer().is_some() {
                        state.set_mode(EditorMode::Insert);
                        state.set_message("-- INSERT --".to_string(), MessageLevel::Info);
                    } else {
                        state.set_message("Cannot edit this node type".to_string(), MessageLevel::Error);
                    }
                }
```

**Step 5: Build to verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Test manually**

Create a test file:
```bash
echo '{"name": "Alice", "count": 42, "active": true}' > /tmp/test-edit.json
```

Run: `cargo run -- /tmp/test-edit.json`
Expected:
- Navigate to a value with j/k
- Press 'i' to enter insert mode
- Type to edit the value
- Press Enter to commit or Esc to cancel

**Step 7: Commit**

```bash
git add src/input/handler.rs src/input/keys.rs
git commit -m "feat: handle character input in insert mode

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Display Edit Buffer in UI

**Files:**
- Create: `src/ui/edit_prompt.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create edit prompt rendering module**

```rust
// src/ui/edit_prompt.rs
use ratatui::{
    layout::Rect,
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::theme::colors::ThemeColors;

/// Renders the edit prompt showing the current edit buffer content.
pub fn render_edit_prompt(
    f: &mut Frame,
    area: Rect,
    buffer: &str,
    colors: &ThemeColors,
) {
    let prompt_text = format!("Edit: {}", buffer);

    let line = Line::from(vec![
        Span::styled(
            prompt_text,
            Style::default()
                .fg(colors.foreground)
                .bg(colors.background)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "█",
            Style::default()
                .fg(colors.cursor)
                .bg(colors.background),
        ),
    ]);

    let prompt = Paragraph::new(line)
        .style(Style::default().bg(colors.background));

    f.render_widget(prompt, area);
}
```

**Step 2: Add edit_prompt to UI module**

```rust
// src/ui/mod.rs (add to module declarations at top)
pub mod layout;
pub mod tree_view;
pub mod status_line;
pub mod edit_prompt;  // Add this line
```

**Step 3: Update UI render to show edit prompt when editing**

```rust
// src/ui/mod.rs (update render method to check for edit buffer)
pub fn render<B: Backend>(
    &self,
    terminal: &mut Terminal<B>,
    state: &EditorState,
) -> Result<()> {
    use ratatui::layout::{Constraint, Direction, Layout};

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),      // Main view
                Constraint::Length(1),   // Status line
                Constraint::Length(1),   // Message area / Edit prompt
            ])
            .split(f.area());

        // Render tree view
        tree_view::render_tree_view(
            f,
            chunks[0],
            state.tree_view(),
            state.cursor(),
            &self.theme.colors,
        );

        // Render status line
        status_line::render_status_line(
            f,
            chunks[1],
            state,
            &self.theme.colors,
        );

        // Render edit prompt if in insert mode with active buffer
        if let Some(buffer) = state.edit_buffer() {
            edit_prompt::render_edit_prompt(
                f,
                chunks[2],
                buffer,
                &self.theme.colors,
            );
        } else if let Some(message) = state.message() {
            message_area::render_message_area(
                f,
                chunks[2],
                message,
                &self.theme.colors,
            );
        }
    })?;

    Ok(())
}
```

**Step 4: Build and test**

Run: `cargo build`
Expected: Compiles successfully

Run: `cargo run -- /tmp/test-edit.json`
Expected: When in insert mode, see "Edit: <buffer content>█" at bottom

**Step 5: Commit**

```bash
git add src/ui/edit_prompt.rs src/ui/mod.rs
git commit -m "feat: display edit buffer in UI during insert mode

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 3: Delete Operation

### Task 5: Implement Delete Node Functionality

**Files:**
- Modify: `src/document/tree.rs`
- Modify: `tests/document_tests.rs`

**Step 1: Write test for deleting nodes**

```rust
// tests/document_tests.rs (add)

#[test]
fn test_delete_object_property() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));

    // Delete second property (index 1)
    let result = tree.delete_node(&[1]);
    assert!(result.is_ok());

    // Verify only 2 properties remain
    match tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "a");
            assert_eq!(entries[1].0, "c");
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_delete_array_element() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(10.0)),
        JsonNode::new(JsonValue::Number(20.0)),
        JsonNode::new(JsonValue::Number(30.0)),
    ])));

    // Delete middle element (index 1)
    let result = tree.delete_node(&[1]);
    assert!(result.is_ok());

    // Verify only 2 elements remain
    match tree.root().value() {
        JsonValue::Array(elements) => {
            assert_eq!(elements.len(), 2);
            match elements[0].value() {
                JsonValue::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("Expected number"),
            }
            match elements[1].value() {
                JsonValue::Number(n) => assert_eq!(*n, 30.0),
                _ => panic!("Expected number"),
            }
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_delete_nested_node() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("user".to_string(), JsonNode::new(JsonValue::Object(vec![
            ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
            ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
        ]))),
    ])));

    // Delete nested property at path [0, 1] (user.age)
    let result = tree.delete_node(&[0, 1]);
    assert!(result.is_ok());

    // Verify only name remains
    let user_node = tree.get_node(&[0]).unwrap();
    match user_node.value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "name");
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_delete_root_fails() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));

    // Cannot delete root node (empty path)
    let result = tree.delete_node(&[]);
    assert!(result.is_err());
}

#[test]
fn test_delete_invalid_path() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
    ])));

    // Try to delete non-existent path
    let result = tree.delete_node(&[99]);
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_delete`
Expected: FAIL with "method not found"

**Step 3: Implement delete_node method**

```rust
// src/document/tree.rs (add method after get_node_mut, around line 320)

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
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_delete`
Expected: All 5 tests PASS

**Step 5: Commit**

```bash
git add src/document/tree.rs tests/document_tests.rs
git commit -m "feat: implement delete_node method for tree

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Wire Delete Operation to Input Handler

**Files:**
- Modify: `src/editor/state.rs`
- Modify: `src/input/handler.rs`
- Modify: `tests/editor_tests.rs`

**Step 1: Write test for delete operation in EditorState**

```rust
// tests/editor_tests.rs (add)

#[test]
fn test_delete_node_at_cursor() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Move to second element
    state.cursor_mut().set_path(vec![1]);

    // Delete it
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok());
    assert!(state.is_dirty());

    // Verify only 2 lines remain
    assert_eq!(state.tree_view().lines().len(), 2);

    // Cursor should stay at index 1 (now pointing to "c")
    assert_eq!(state.cursor().path(), &[1]);
}

#[test]
fn test_delete_last_node_moves_cursor() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Move to last element
    state.cursor_mut().set_path(vec![1]);

    // Delete it
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok());

    // Cursor should move to previous line [0]
    assert_eq!(state.cursor().path(), &[0]);
}

#[test]
fn test_delete_root_fails() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new(tree);

    // Cursor at root (empty path after tree_view initialization with no children)
    // Since there are no lines, cursor will be at []
    state.cursor_mut().set_path(vec![]);

    let result = state.delete_node_at_cursor();
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_delete_node_at_cursor`
Expected: FAIL with "method not found"

**Step 3: Implement delete_node_at_cursor in EditorState**

```rust
// src/editor/state.rs (add method after rebuild_tree_view, around line 470)

    /// Deletes the node at the current cursor position.
    /// Adjusts the cursor position after deletion and rebuilds the tree view.
    pub fn delete_node_at_cursor(&mut self) -> anyhow::Result<()> {
        let path = self.cursor.path().to_vec();

        // Find current line index before deletion
        let lines = self.tree_view.lines();
        let current_idx = lines.iter().position(|l| l.path == path);

        // Delete the node
        self.tree.delete_node(&path)?;
        self.mark_dirty();
        self.rebuild_tree_view();

        // Adjust cursor position
        let new_lines = self.tree_view.lines();
        if new_lines.is_empty() {
            // No lines left, cursor stays at root
            self.cursor.set_path(vec![]);
        } else if let Some(idx) = current_idx {
            // Try to keep cursor at same visual position
            let new_idx = idx.min(new_lines.len() - 1);
            self.cursor.set_path(new_lines[new_idx].path.clone());
        } else if !new_lines.is_empty() {
            // Cursor wasn't found, move to first line
            self.cursor.set_path(new_lines[0].path.clone());
        }

        Ok(())
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_delete_node_at_cursor`
Expected: All 3 tests PASS

**Step 5: Update input handler to call delete method**

```rust
// src/input/handler.rs (find InputEvent::Delete around line 247, replace placeholder)
                InputEvent::Delete => {
                    use crate::editor::state::MessageLevel;
                    match state.delete_node_at_cursor() {
                        Ok(_) => {
                            state.set_message("Node deleted".to_string(), MessageLevel::Info);
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Delete failed: {}", e),
                                MessageLevel::Error,
                            );
                        }
                    }
                }
```

**Step 6: Build and test manually**

Run: `cargo build`
Expected: Compiles successfully

Run: `cargo run -- /tmp/test-edit.json`
Expected: Navigate to a node, press 'd' to delete it

**Step 7: Commit**

```bash
git add src/editor/state.rs src/input/handler.rs tests/editor_tests.rs
git commit -m "feat: implement delete operation with cursor adjustment

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 4: Paste Operation

### Task 7: Implement Paste After Current Node

**Files:**
- Modify: `src/document/tree.rs`
- Modify: `tests/document_tests.rs`

**Step 1: Write test for inserting nodes**

```rust
// tests/document_tests.rs (add)

#[test]
fn test_insert_node_in_object() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));

    // Insert new node at index 1 (between a and c)
    let new_node = JsonNode::new(JsonValue::Number(2.0));
    let result = tree.insert_node_in_object(&[1], "b".to_string(), new_node);
    assert!(result.is_ok());

    // Verify three properties in order
    match tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].0, "a");
            assert_eq!(entries[1].0, "b");
            assert_eq!(entries[2].0, "c");
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_insert_node_in_array() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(10.0)),
        JsonNode::new(JsonValue::Number(30.0)),
    ])));

    // Insert new node at index 1 (between 10 and 30)
    let new_node = JsonNode::new(JsonValue::Number(20.0));
    let result = tree.insert_node_in_array(&[1], new_node);
    assert!(result.is_ok());

    // Verify three elements in order
    match tree.root().value() {
        JsonValue::Array(elements) => {
            assert_eq!(elements.len(), 3);
            match elements[0].value() {
                JsonValue::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("Expected number"),
            }
            match elements[1].value() {
                JsonValue::Number(n) => assert_eq!(*n, 20.0),
                _ => panic!("Expected number"),
            }
            match elements[2].value() {
                JsonValue::Number(n) => assert_eq!(*n, 30.0),
                _ => panic!("Expected number"),
            }
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_insert_node_at_end() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
    ])));

    // Insert at end (index 1, which equals length)
    let new_node = JsonNode::new(JsonValue::Number(2.0));
    let result = tree.insert_node_in_array(&[1], new_node);
    assert!(result.is_ok());

    match tree.root().value() {
        JsonValue::Array(elements) => {
            assert_eq!(elements.len(), 2);
        }
        _ => panic!("Expected array"),
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_insert_node`
Expected: FAIL with "method not found"

**Step 3: Implement insert methods**

```rust
// src/document/tree.rs (add methods after delete_node)

    /// Inserts a node into an object at the specified path and index.
    /// The path must point to the object, and index specifies where to insert.
    pub fn insert_node_in_object(
        &mut self,
        path: &[usize],
        key: String,
        node: JsonNode,
    ) -> anyhow::Result<()> {
        use anyhow::{anyhow, Context};

        // Get parent path (all but last index)
        let parent_path = if path.is_empty() {
            &[]
        } else {
            &path[..path.len() - 1]
        };
        let index = if path.is_empty() { 0 } else { path[path.len() - 1] };

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
                    return Err(anyhow!("Index {} out of bounds for object with {} entries", index, entries.len()));
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
    pub fn insert_node_in_array(
        &mut self,
        path: &[usize],
        node: JsonNode,
    ) -> anyhow::Result<()> {
        use anyhow::{anyhow, Context};

        // Get parent path (all but last index)
        let parent_path = if path.is_empty() {
            &[]
        } else {
            &path[..path.len() - 1]
        };
        let index = if path.is_empty() { 0 } else { path[path.len() - 1] };

        // Get mutable reference to parent (or root if path is empty)
        let target = if parent_path.is_empty() {
            self.root_mut()
        } else {
            self.get_node_mut(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        // Insert into array
        match target.value_mut() {
            JsonValue::Array(elements) => {
                if index > elements.len() {
                    return Err(anyhow!("Index {} out of bounds for array with {} elements", index, elements.len()));
                }
                elements.insert(index, node);
            }
            _ => {
                return Err(anyhow!("Target is not an array"));
            }
        }

        Ok(())
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_insert_node`
Expected: All 3 tests PASS

**Step 5: Commit**

```bash
git add src/document/tree.rs tests/document_tests.rs
git commit -m "feat: implement insert_node methods for objects and arrays

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: Wire Paste Operation to Input Handler

**Files:**
- Modify: `src/editor/state.rs`
- Modify: `src/input/handler.rs`
- Modify: `tests/editor_tests.rs`

**Step 1: Write test for paste operation**

```rust
// tests/editor_tests.rs (add)

#[test]
fn test_paste_node_in_object() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Set clipboard with a node
    let clipboard_node = JsonNode::new(JsonValue::String("test".to_string()));
    // We need to use yank first to populate clipboard
    // For this test, we'll directly access the internal clipboard (not ideal but necessary)
    // Actually, let's yank a node first
    state.cursor_mut().set_path(vec![0]);
    state.yank_node();

    // Now move to position [1] and paste
    state.cursor_mut().set_path(vec![1]);
    let result = state.paste_node_at_cursor();
    assert!(result.is_ok());
    assert!(state.is_dirty());

    // Should have 3 nodes now
    assert_eq!(state.tree_view().lines().len(), 3);
}

#[test]
fn test_paste_node_in_array() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(10.0)),
        JsonNode::new(JsonValue::Number(30.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Yank first element
    state.cursor_mut().set_path(vec![0]);
    state.yank_node();

    // Paste after first element
    let result = state.paste_node_at_cursor();
    assert!(result.is_ok());

    // Should have 3 elements now
    assert_eq!(state.tree_view().lines().len(), 3);
}

#[test]
fn test_paste_without_clipboard_fails() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Try to paste without yanking first
    let result = state.paste_node_at_cursor();
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_paste_node`
Expected: FAIL with "method not found"

**Step 3: Implement paste_node_at_cursor in EditorState**

```rust
// src/editor/state.rs (add method after yank_node, around line 708)

    /// Pastes the clipboard node after the current cursor position.
    /// For objects, generates a unique key name. For arrays, inserts after current index.
    pub fn paste_node_at_cursor(&mut self) -> anyhow::Result<()> {
        use anyhow::anyhow;
        use crate::document::node::JsonValue;

        let clipboard_node = self.clipboard.clone()
            .ok_or_else(|| anyhow!("Nothing to paste"))?;

        let current_path = self.cursor.path().to_vec();

        // Determine parent and insert position
        if current_path.is_empty() {
            return Err(anyhow!("Cannot paste at root level"));
        }

        let parent_path = &current_path[..current_path.len() - 1];
        let current_index = current_path[current_path.len() - 1];
        let insert_index = current_index + 1;

        // Get parent node to determine type
        let parent = if parent_path.is_empty() {
            self.tree.root()
        } else {
            self.tree.get_node(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        match parent.value() {
            JsonValue::Object(_) => {
                // Generate a unique key name
                let mut key_name = "pasted".to_string();
                let mut counter = 1;

                // Keep trying until we find a unique key
                loop {
                    let test_key = if counter == 1 {
                        key_name.clone()
                    } else {
                        format!("{}{}", key_name, counter)
                    };

                    // Check if key exists
                    let parent_ref = if parent_path.is_empty() {
                        self.tree.root()
                    } else {
                        self.tree.get_node(parent_path).unwrap()
                    };

                    let key_exists = if let JsonValue::Object(entries) = parent_ref.value() {
                        entries.iter().any(|(k, _)| k == &test_key)
                    } else {
                        false
                    };

                    if !key_exists {
                        key_name = test_key;
                        break;
                    }

                    counter += 1;
                }

                // Build the full path for insertion
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree.insert_node_in_object(&insert_path, key_name, clipboard_node)?;
            }
            JsonValue::Array(_) => {
                let mut insert_path = parent_path.to_vec();
                insert_path.push(insert_index);

                self.tree.insert_node_in_array(&insert_path, clipboard_node)?;
            }
            _ => {
                return Err(anyhow!("Parent is not a container type"));
            }
        }

        self.mark_dirty();
        self.rebuild_tree_view();

        // Move cursor to newly pasted node
        let mut new_cursor_path = parent_path.to_vec();
        new_cursor_path.push(insert_index);
        self.cursor.set_path(new_cursor_path);

        Ok(())
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_paste_node`
Expected: All 3 tests PASS

**Step 5: Update input handler to call paste method**

```rust
// src/input/handler.rs (find InputEvent::Paste around line 251, replace placeholder)
                InputEvent::Paste => {
                    use crate::editor::state::MessageLevel;
                    match state.paste_node_at_cursor() {
                        Ok(_) => {
                            state.set_message("Node pasted".to_string(), MessageLevel::Info);
                        }
                        Err(e) => {
                            state.set_message(
                                format!("Paste failed: {}", e),
                                MessageLevel::Error,
                            );
                        }
                    }
                }
```

**Step 6: Build and test manually**

Run: `cargo build`
Expected: Compiles successfully

Run: `cargo run -- /tmp/test-edit.json`
Expected:
- Navigate to a node, press 'y' to yank
- Navigate elsewhere, press 'p' to paste
- See the yanked node inserted after current position

**Step 7: Commit**

```bash
git add src/editor/state.rs src/input/handler.rs tests/editor_tests.rs
git commit -m "feat: implement paste operation with unique key generation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 5: Integration Testing & Documentation

### Task 9: End-to-End Integration Tests

**Files:**
- Create: `tests/integration_editing.rs`

**Step 1: Write integration tests**

```rust
// tests/integration_editing.rs
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::editor::mode::EditorMode;

#[test]
fn test_full_edit_workflow() {
    // Create initial document
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Start editing first field (name)
    state.cursor_mut().set_path(vec![0]);
    state.set_mode(EditorMode::Insert);
    state.start_editing();

    // Edit the value
    state.clear_edit_buffer();
    for ch in "Bob".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Commit the edit
    let result = state.commit_editing();
    assert!(result.is_ok());
    assert!(state.is_dirty());

    // Verify the change
    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "Bob"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_full_delete_workflow() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Delete middle element
    state.cursor_mut().set_path(vec![1]);
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok());

    // Verify only 2 elements remain
    assert_eq!(state.tree_view().lines().len(), 2);
}

#[test]
fn test_full_yank_paste_workflow() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Yank first element
    state.cursor_mut().set_path(vec![0]);
    assert!(state.yank_node());

    // Paste after first element
    let result = state.paste_node_at_cursor();
    assert!(result.is_ok());

    // Should have 3 elements now
    assert_eq!(state.tree_view().lines().len(), 3);
}

#[test]
fn test_edit_cancel_workflow() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ])));
    let mut state = EditorState::new(tree);

    // Start editing
    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    // Make changes
    state.clear_edit_buffer();
    for ch in "Bob".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Cancel instead of committing
    state.cancel_editing();

    // Verify no change was made
    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "Alice"),
        _ => panic!("Expected string"),
    }
    assert!(!state.is_dirty());
}
```

**Step 2: Run integration tests**

Run: `cargo test --test integration_editing`
Expected: All 4 tests PASS

**Step 3: Run all tests to verify nothing broke**

Run: `cargo test`
Expected: All tests PASS (should be 74+ tests now with new additions)

**Step 4: Commit**

```bash
git add tests/integration_editing.rs
git commit -m "test: add end-to-end integration tests for editing

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 10: Update Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update known issues section**

```markdown
// CLAUDE.md (update Known Issues section around line 49)

**Working Features:**
- ✅ JSON file loading (filesystem paths only)
- ✅ Tree view rendering with expand/collapse and auto-expansion
- ✅ Line numbers (enabled by default, toggle with `:set number`/`:set nonumber`)
- ✅ Navigation (j/k/h/l, arrow keys)
- ✅ Mode switching (i for INSERT, : for COMMAND, / for SEARCH, Esc to NORMAL)
- ✅ Status line showing current mode and filename
- ✅ Command mode with visible prompt and input buffer
- ✅ Command execution (`:w`, `:q`, `:q!`, `:wq`, `:x`)
- ✅ Save functionality (`:w` writes changes to disk atomically)
- ✅ Message area for errors, warnings, and info messages
- ✅ Help system (press `?` for scrollable help overlay)
- ✅ Search functionality (`/` to search, `n` for next result)
- ✅ Theme system (`:theme` to list, `:theme <name>` to switch)
- ✅ Settings system (`:set` to view, `:set <option>` to change)
- ✅ Config file support (`~/.config/jsonquill/config.toml`, `:set save` to persist)
- ✅ Yank operation (`y` copies to clipboard including system clipboard)
- ✅ Default dark theme (gray/black, not blue)
- ✅ Insert mode for editing values (strings, numbers, booleans, null)
- ✅ Delete operation (`d` key removes nodes from tree)
- ✅ Paste operation (`p` key inserts yanked nodes)
- ✅ All tests passing

**Known Issues / TODO:**
- ❌ **Stdin piping not supported** - `cat file.json | jsonquill` fails due to terminal I/O conflict
- ❌ **No rename operation** - Cannot rename object keys (only edit values)
- ❌ **No undo/redo** - Changes are permanent until file is saved/reloaded
```

**Step 2: Update usage section**

```markdown
// CLAUDE.md (update Editing section around line 116)

# Editing (NORMAL mode)
i           - Enter INSERT mode on current node
              (edits value for strings, numbers, booleans, null)
              (shows "Cannot edit" for objects/arrays)

# INSERT mode
<chars>     - Type to edit the value
Backspace   - Delete last character
Enter       - Commit changes and return to NORMAL mode
Esc         - Cancel editing and return to NORMAL mode

# Editing (NORMAL mode)
y           - Yank (copy) current node to clipboard
d           - Delete current node (removes from tree)
p           - Paste clipboard content after current node
              (generates unique key for objects, inserts after for arrays)
```

**Step 3: Build final binary**

Run: `cargo build --release`
Expected: Compiles successfully with no warnings

**Step 4: Test the release binary**

Create comprehensive test file:
```bash
cat > /tmp/comprehensive-test.json << 'EOF'
{
  "user": {
    "name": "Alice",
    "age": 30,
    "active": true,
    "metadata": null
  },
  "tags": ["rust", "json", "editor"],
  "count": 42
}
EOF
```

Run: `./target/release/jsonquill /tmp/comprehensive-test.json`

Test sequence:
1. Navigate with j/k to "Alice"
2. Press 'i', edit to "Bob", press Enter
3. Navigate to "age" (30)
4. Press 'i', edit to "31", press Enter
5. Navigate to "active" (true)
6. Press 'i', edit to "false", press Enter
7. Navigate to first tag "rust"
8. Press 'y' to yank
9. Navigate to last tag "editor"
10. Press 'p' to paste (should insert after)
11. Navigate to "metadata": null
12. Press 'd' to delete
13. Press ':w' to save
14. Press ':q' to quit

Verify saved file:
```bash
cat /tmp/comprehensive-test.json
```

Expected: File contains all the edits made

**Step 5: Commit documentation update**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with completed editing features

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Summary

This plan implements full editing functionality for jsonquill:

**Implemented Features:**
- ✅ Insert mode with character input handling
- ✅ Edit buffer for modifying values (string, number, boolean, null)
- ✅ Commit/cancel editing with Enter/Esc
- ✅ Delete operation with cursor adjustment
- ✅ Paste operation with unique key generation
- ✅ UI showing edit buffer during insert mode
- ✅ Proper dirty flag and tree view updates
- ✅ Comprehensive tests (unit + integration)
- ✅ Updated documentation

**Testing Coverage:**
- Unit tests for EditorState edit buffer methods
- Unit tests for commit_editing with type validation
- Unit tests for tree delete/insert operations
- Unit tests for paste with clipboard
- Integration tests for full workflows
- Manual testing with release binary

**User Workflow:**
1. Navigate to a value node
2. Press 'i' to enter insert mode
3. Edit the value (type/backspace)
4. Press Enter to commit or Esc to cancel
5. Press 'd' to delete nodes
6. Press 'y' to yank, 'p' to paste

All operations properly update the dirty flag and rebuild the tree view to keep the UI in sync.
