# Named Registers Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement vim-style named registers (a-z), append mode (A-Z), and automatic yank/delete history (0-9) for clipboard management.

**Architecture:** New `registers` module with `RegisterSet` managing 36 registers (1 unnamed + 26 named + 10 numbered). Replace single clipboard field with register system. Update input handling to support `"` register selection prefix.

**Tech Stack:** Rust, HashMap for named registers, array for numbered registers, arboard for system clipboard

---

## Task 1: Create Register Module with Basic Structure

**Files:**
- Create: `src/editor/registers.rs`
- Modify: `src/editor/mod.rs` (add module declaration)

**Step 1: Write failing test for RegisterContent creation**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::{JsonNode, JsonValue};

    #[test]
    fn test_register_content_new() {
        let node = JsonNode::new(JsonValue::String("test".to_string()));
        let content = RegisterContent::new(vec![node.clone()], vec![None]);

        assert_eq!(content.nodes.len(), 1);
        assert_eq!(content.keys.len(), 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_register_content_new`
Expected: FAIL with "RegisterContent not found"

**Step 3: Write minimal RegisterContent implementation**

```rust
use crate::document::node::JsonNode;

/// Content stored in a register (nodes + optional keys for object members)
#[derive(Debug, Clone)]
pub struct RegisterContent {
    pub nodes: Vec<JsonNode>,
    pub keys: Vec<Option<String>>,
}

impl RegisterContent {
    pub fn new(nodes: Vec<JsonNode>, keys: Vec<Option<String>>) -> Self {
        Self { nodes, keys }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
```

**Step 4: Add module declaration**

In `src/editor/mod.rs`, add:
```rust
pub mod registers;
```

**Step 5: Run test to verify it passes**

Run: `cargo test test_register_content_new`
Expected: PASS

**Step 6: Commit**

```bash
git add src/editor/registers.rs src/editor/mod.rs
git commit -m "feat(registers): add RegisterContent struct"
```

---

## Task 2: Implement RegisterSet with Get/Set Operations

**Files:**
- Modify: `src/editor/registers.rs`

**Step 1: Write failing tests for RegisterSet operations**

```rust
#[test]
fn test_register_set_new() {
    let regs = RegisterSet::new();
    assert!(regs.get_unnamed().is_empty());
    assert_eq!(regs.get_named('a'), None);
    assert!(regs.get_numbered(0).is_empty());
}

#[test]
fn test_register_set_named() {
    let mut regs = RegisterSet::new();
    let node = JsonNode::new(JsonValue::Number(42.0));
    let content = RegisterContent::new(vec![node.clone()], vec![None]);

    regs.set_named('a', content.clone());

    let retrieved = regs.get_named('a').unwrap();
    assert_eq!(retrieved.nodes.len(), 1);
}

#[test]
fn test_register_set_unnamed() {
    let mut regs = RegisterSet::new();
    let node = JsonNode::new(JsonValue::Boolean(true));
    let content = RegisterContent::new(vec![node.clone()], vec![None]);

    regs.set_unnamed(content.clone());

    let retrieved = regs.get_unnamed();
    assert_eq!(retrieved.nodes.len(), 1);
}

#[test]
fn test_register_set_numbered() {
    let mut regs = RegisterSet::new();
    let node = JsonNode::new(JsonValue::Null);
    let content = RegisterContent::new(vec![node.clone()], vec![None]);

    regs.set_numbered(5, content.clone());

    let retrieved = regs.get_numbered(5);
    assert_eq!(retrieved.nodes.len(), 1);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_register_set`
Expected: FAIL with "RegisterSet not found"

**Step 3: Write RegisterSet implementation**

```rust
use std::collections::HashMap;

/// Manages all registers (unnamed, named a-z, numbered 0-9)
#[derive(Debug, Clone)]
pub struct RegisterSet {
    unnamed: RegisterContent,
    named: HashMap<char, RegisterContent>,
    numbered: [RegisterContent; 10],
}

impl RegisterSet {
    pub fn new() -> Self {
        Self {
            unnamed: RegisterContent::new(vec![], vec![]),
            named: HashMap::new(),
            numbered: [
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
            ],
        }
    }

    pub fn get_unnamed(&self) -> &RegisterContent {
        &self.unnamed
    }

    pub fn set_unnamed(&mut self, content: RegisterContent) {
        self.unnamed = content;
    }

    pub fn get_named(&self, register: char) -> Option<&RegisterContent> {
        self.named.get(&register.to_ascii_lowercase())
    }

    pub fn set_named(&mut self, register: char, content: RegisterContent) {
        self.named.insert(register.to_ascii_lowercase(), content);
    }

    pub fn get_numbered(&self, index: usize) -> &RegisterContent {
        &self.numbered[index]
    }

    pub fn set_numbered(&mut self, index: usize, content: RegisterContent) {
        self.numbered[index] = content;
    }
}

impl Default for RegisterSet {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_register_set`
Expected: All PASS

**Step 5: Commit**

```bash
git add src/editor/registers.rs
git commit -m "feat(registers): add RegisterSet with get/set operations"
```

---

## Task 3: Add Append Mode and History Operations

**Files:**
- Modify: `src/editor/registers.rs`

**Step 1: Write failing tests for append and history**

```rust
#[test]
fn test_register_append_named() {
    let mut regs = RegisterSet::new();
    let node1 = JsonNode::new(JsonValue::Number(1.0));
    let node2 = JsonNode::new(JsonValue::Number(2.0));

    regs.set_named('a', RegisterContent::new(vec![node1.clone()], vec![None]));
    regs.append_named('a', RegisterContent::new(vec![node2.clone()], vec![None]));

    let retrieved = regs.get_named('a').unwrap();
    assert_eq!(retrieved.nodes.len(), 2);
}

#[test]
fn test_register_push_delete_history() {
    let mut regs = RegisterSet::new();
    let node1 = JsonNode::new(JsonValue::Number(1.0));
    let node2 = JsonNode::new(JsonValue::Number(2.0));
    let node3 = JsonNode::new(JsonValue::Number(3.0));

    regs.push_delete_history(RegisterContent::new(vec![node1.clone()], vec![None]));
    regs.push_delete_history(RegisterContent::new(vec![node2.clone()], vec![None]));
    regs.push_delete_history(RegisterContent::new(vec![node3.clone()], vec![None]));

    // "1 should have most recent (node3)
    assert_eq!(regs.get_numbered(1).nodes.len(), 1);
    // "2 should have node2
    assert_eq!(regs.get_numbered(2).nodes.len(), 1);
}

#[test]
fn test_register_update_yank_register() {
    let mut regs = RegisterSet::new();
    let node = JsonNode::new(JsonValue::Boolean(true));
    let content = RegisterContent::new(vec![node.clone()], vec![None]);

    regs.update_yank_register(content.clone());

    // "0 should have the yanked content
    assert_eq!(regs.get_numbered(0).nodes.len(), 1);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_register_append test_register_push test_register_update`
Expected: FAIL with method not found

**Step 3: Implement append and history methods**

```rust
impl RegisterSet {
    // ... existing methods ...

    pub fn append_named(&mut self, register: char, content: RegisterContent) {
        let key = register.to_ascii_lowercase();
        if let Some(existing) = self.named.get_mut(&key) {
            existing.nodes.extend(content.nodes);
            existing.keys.extend(content.keys);
        } else {
            self.named.insert(key, content);
        }
    }

    pub fn push_delete_history(&mut self, content: RegisterContent) {
        // Shift history: "9 lost, "8→"9, ..., "2→"3, "1→"2
        for i in (1..9).rev() {
            self.numbered[i + 1] = self.numbered[i].clone();
        }
        // New delete goes to "1
        self.numbered[1] = content;
    }

    pub fn update_yank_register(&mut self, content: RegisterContent) {
        // Update "0 with latest yank
        self.numbered[0] = content;
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_register_append test_register_push test_register_update`
Expected: All PASS

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/editor/registers.rs
git commit -m "feat(registers): add append mode and history operations"
```

---

## Task 4: Add Register Fields to EditorState

**Files:**
- Modify: `src/editor/state.rs`

**Step 1: Write failing test for register state**

Add to existing tests in `src/editor/state.rs`:

```rust
#[test]
fn test_editor_state_has_registers() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new(tree);

    // Should start with empty registers
    assert!(state.get_unnamed_register().is_empty());
    assert_eq!(state.get_pending_register(), None);
    assert_eq!(state.get_append_mode(), false);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_editor_state_has_registers`
Expected: FAIL with method not found

**Step 3: Add register fields to EditorState**

In `src/editor/state.rs`, modify the struct:

```rust
use super::registers::RegisterSet;

pub struct EditorState {
    // ... existing fields ...

    // Remove these old fields:
    // clipboard: Option<JsonNode>,
    // clipboard_key: Option<String>,

    // Add new register fields:
    registers: RegisterSet,
    pending_register: Option<char>,
    append_mode: bool,

    // ... rest of fields ...
}
```

**Step 4: Update EditorState::new()**

```rust
impl EditorState {
    pub fn new(tree: JsonTree) -> Self {
        // ... existing initialization ...
        Self {
            // ... existing fields ...

            // Remove old clipboard fields
            registers: RegisterSet::new(),
            pending_register: None,
            append_mode: false,

            // ... rest of initialization ...
        }
    }
}
```

**Step 5: Add register accessor methods**

```rust
impl EditorState {
    // ... existing methods ...

    pub fn get_unnamed_register(&self) -> &crate::editor::registers::RegisterContent {
        self.registers.get_unnamed()
    }

    pub fn get_pending_register(&self) -> Option<char> {
        self.pending_register
    }

    pub fn set_pending_register(&mut self, register: char, append: bool) {
        self.pending_register = Some(register);
        self.append_mode = append;
    }

    pub fn get_append_mode(&self) -> bool {
        self.append_mode
    }

    pub fn clear_register_pending(&mut self) {
        self.pending_register = None;
        self.append_mode = false;
    }
}
```

**Step 6: Update clear_pending() to include register**

Find the `clear_pending()` method and update:

```rust
pub fn clear_pending(&mut self) {
    self.pending_command = None;
    self.pending_count = None;
    self.pending_register = None;
    self.append_mode = false;
}
```

**Step 7: Run test to verify it passes**

Run: `cargo test test_editor_state_has_registers`
Expected: PASS

**Step 8: Fix compilation errors**

Run: `cargo build`
Expected: Errors about `clipboard` field not found

Find all references to `self.clipboard` and `self.clipboard_key` and comment them out temporarily (we'll fix them properly in next tasks).

**Step 9: Run all tests**

Run: `cargo test`
Expected: Some tests may fail (we'll fix in next tasks)

**Step 10: Commit**

```bash
git add src/editor/state.rs
git commit -m "feat(registers): add register fields to EditorState"
```

---

## Task 5: Update Yank Operations to Use Registers

**Files:**
- Modify: `src/editor/state.rs`

**Step 1: Write failing test for register-based yank**

```rust
#[test]
fn test_yank_to_named_register() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("key".to_string(), JsonNode::new(JsonValue::String("value".to_string()))),
    ])));
    tree.expand(&[]);

    let mut state = EditorState::new(tree);
    state.cursor_mut().push(0);

    // Yank to register 'a'
    state.set_pending_register('a', false);
    let result = state.yank_nodes(1);

    assert!(result);
    // Should be in register 'a'
    let reg_a = state.registers.get_named('a').unwrap();
    assert_eq!(reg_a.nodes.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_yank_to_named_register`
Expected: FAIL with method signature mismatch

**Step 3: Update yank_node() to yank_nodes()**

Find `pub fn yank_node(&mut self)` and replace with:

```rust
/// Yank nodes starting at cursor for count iterations.
/// Updates target register (unnamed if not specified), register "0, and system clipboard (unnamed only).
pub fn yank_nodes(&mut self, count: u32) -> bool {
    use crate::editor::registers::RegisterContent;

    let mut nodes = Vec::new();
    let mut keys = Vec::new();

    // Collect nodes
    for _ in 0..count {
        let path = self.cursor.path();
        if let Some(node) = self.tree.get_node(path) {
            nodes.push(node.clone());

            // Store key if yanking from object
            let key = if !path.is_empty() {
                let parent_path = &path[..path.len() - 1];
                let index = path[path.len() - 1];
                if let Some(parent) = self.tree.get_node(parent_path) {
                    if let JsonValue::Object(fields) = parent.value() {
                        Some(fields[index].0.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            keys.push(key);

            // Move down for next iteration
            if !self.move_cursor_down() {
                break;
            }
        } else {
            break;
        }
    }

    if nodes.is_empty() {
        return false;
    }

    let content = RegisterContent::new(nodes, keys);

    // Determine target register
    let target_register = self.pending_register;

    // Update target register
    if let Some(reg) = target_register {
        if self.append_mode {
            self.registers.append_named(reg, content.clone());
        } else {
            self.registers.set_named(reg, content.clone());
        }
    } else {
        // Unnamed register
        self.registers.set_unnamed(content.clone());

        // Sync to system clipboard
        if let Some(node) = content.nodes.first() {
            if let Ok(json_str) = crate::file::saver::serialize_node(node, 2) {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(json_str);
                }
            }
        }
    }

    // Update "0 (last yank)
    self.registers.update_yank_register(content);

    true
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_yank_to_named_register`
Expected: PASS

**Step 5: Update all callers of yank_node()**

Search for `yank_node()` calls and update to `yank_nodes(1)`:
- In delete operations: `state.yank_nodes(1);`
- In tests: update accordingly

**Step 6: Run full test suite**

Run: `cargo test`
Expected: Most tests pass, some may need updates

**Step 7: Commit**

```bash
git add src/editor/state.rs
git commit -m "feat(registers): update yank operations to use register system"
```

---

## Task 6: Update Paste Operations to Use Registers

**Files:**
- Modify: `src/editor/state.rs`

**Step 1: Write failing test for register-based paste**

```rust
#[test]
fn test_paste_from_named_register() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![])));
    tree.expand(&[]);

    let mut state = EditorState::new(tree);

    // Manually populate register 'a'
    let node = JsonNode::new(JsonValue::String("test".to_string()));
    let content = crate::editor::registers::RegisterContent::new(vec![node], vec![None]);
    state.registers.set_named('a', content);

    // Paste from register 'a'
    state.set_pending_register('a', false);
    let result = state.paste_nodes_at_cursor();

    assert!(result.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_paste_from_named_register`
Expected: FAIL with method not found

**Step 3: Update paste_node_at_cursor() to paste_nodes_at_cursor()**

Find and replace:

```rust
pub fn paste_nodes_at_cursor(&mut self) -> anyhow::Result<()> {
    use anyhow::anyhow;

    // Get content from appropriate register
    let content = if let Some(reg) = self.pending_register {
        self.registers.get_named(reg)
            .ok_or_else(|| anyhow!("Nothing in register '{}'", reg))?
            .clone()
    } else {
        self.registers.get_unnamed().clone()
    };

    if content.is_empty() {
        return Err(anyhow!("Nothing to paste"));
    }

    // Create undo checkpoint
    self.create_undo_checkpoint();

    // Paste each node
    for (node, key) in content.nodes.iter().zip(content.keys.iter()) {
        self.paste_single_node(node.clone(), key.clone(), true)?;
    }

    self.mark_dirty();
    Ok(())
}

fn paste_single_node(&mut self, node: JsonNode, key: Option<String>, after: bool) -> anyhow::Result<()> {
    use crate::document::node::JsonValue;
    use anyhow::anyhow;

    let current_path = self.cursor.path().to_vec();

    // ... (rest of paste logic from old paste_node_at_cursor, handling after=true)
}
```

**Step 4: Similarly update paste_node_before_cursor()**

```rust
pub fn paste_nodes_before_cursor(&mut self) -> anyhow::Result<()> {
    use anyhow::anyhow;

    let content = if let Some(reg) = self.pending_register {
        self.registers.get_named(reg)
            .ok_or_else(|| anyhow!("Nothing in register '{}'", reg))?
            .clone()
    } else {
        self.registers.get_unnamed().clone()
    };

    if content.is_empty() {
        return Err(anyhow!("Nothing to paste"));
    }

    self.create_undo_checkpoint();

    for (node, key) in content.nodes.iter().zip(content.keys.iter()) {
        self.paste_single_node(node.clone(), key.clone(), false)?;
    }

    self.mark_dirty();
    Ok(())
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test test_paste_from_named_register`
Expected: PASS

**Step 6: Run full test suite**

Run: `cargo test`
Expected: Tests pass

**Step 7: Commit**

```bash
git add src/editor/state.rs
git commit -m "feat(registers): update paste operations to use register system"
```

---

## Task 7: Update Delete Operations to Use Register History

**Files:**
- Modify: `src/editor/state.rs`

**Step 1: Write failing test for delete history**

```rust
#[test]
fn test_delete_pushes_to_history() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    tree.expand(&[]);

    let mut state = EditorState::new(tree);
    state.cursor_mut().push(0);

    // Delete should push to "1
    let _ = state.delete_node_at_cursor();

    let reg_1 = state.registers.get_numbered(1);
    assert_eq!(reg_1.nodes.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_delete_pushes_to_history`
Expected: FAIL

**Step 3: Update delete_node_at_cursor()**

Find the method and update the yank portion:

```rust
pub fn delete_node_at_cursor(&mut self) -> anyhow::Result<()> {
    use anyhow::anyhow;
    use crate::editor::registers::RegisterContent;

    let path = self.cursor.path().to_vec();

    if path.is_empty() {
        return Err(anyhow!("Cannot delete root node"));
    }

    // Yank before deleting
    if let Some(node) = self.tree.get_node(&path) {
        let key = if path.len() >= 2 {
            let parent_path = &path[..path.len() - 1];
            let index = path[path.len() - 1];
            if let Some(parent) = self.tree.get_node(parent_path) {
                if let JsonValue::Object(fields) = parent.value() {
                    Some(fields[index].0.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let content = RegisterContent::new(vec![node.clone()], vec![key]);

        // Update target register
        if let Some(reg) = self.pending_register {
            if self.append_mode {
                self.registers.append_named(reg, content.clone());
            } else {
                self.registers.set_named(reg, content.clone());
            }
        } else {
            // Unnamed register
            self.registers.set_unnamed(content.clone());

            // Sync to system clipboard
            if let Ok(json_str) = crate::file::saver::serialize_node(&node, 2) {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(json_str);
                }
            }
        }

        // Push to delete history ("1)
        self.registers.push_delete_history(content);
    }

    // ... rest of delete logic ...
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_delete_pushes_to_history`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/editor/state.rs
git commit -m "feat(registers): update delete to push to history register"
```

---

## Task 8: Add RegisterSelect Input Event

**Files:**
- Modify: `src/input/keys.rs`

**Step 1: Add RegisterSelect variant to InputEvent**

```rust
pub enum InputEvent {
    // ... existing variants ...
    RegisterSelect,
}
```

**Step 2: Map `"` key to RegisterSelect**

In the `map_key_event` function, add in Normal mode:

```rust
Key::Char('"') => Some(InputEvent::RegisterSelect),
```

**Step 3: Run tests**

Run: `cargo test`
Expected: All pass

**Step 4: Commit**

```bash
git add src/input/keys.rs
git commit -m "feat(registers): add RegisterSelect input event"
```

---

## Task 9: Implement Register Selection in Input Handler

**Files:**
- Modify: `src/input/handler.rs`

**Step 1: Write test for register selection flow**

```rust
#[test]
fn test_register_selection() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::String("test".to_string())));
    let mut state = EditorState::new(tree);
    let mut handler = InputHandler::new();

    // Press " to enter register selection
    handler.handle_event(&mut state, InputEvent::RegisterSelect, &Config::default());

    // State should be awaiting register (checked via internal state)
    // This is tested indirectly through yank operation
}
```

**Step 2: Add awaiting_register field to InputHandler**

```rust
pub struct InputHandler {
    // ... existing fields ...
    awaiting_register: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            // ... existing fields ...
            awaiting_register: false,
        }
    }
}
```

**Step 3: Handle RegisterSelect event**

```rust
pub fn handle_event(
    &mut self,
    state: &mut EditorState,
    event: InputEvent,
    config: &Config,
) -> bool {
    match event {
        // ... existing handlers ...

        InputEvent::RegisterSelect => {
            self.awaiting_register = true;
            state.clear_message();
            false
        }

        // ... rest of handlers ...
    }
}
```

**Step 4: Handle character input when awaiting register**

At the start of `handle_event`, add:

```rust
// Handle register selection
if self.awaiting_register {
    self.awaiting_register = false;

    if let InputEvent::Character(c) = event {
        // Validate register name
        if c.is_ascii_lowercase() {
            state.set_pending_register(c, false);
            return false;
        } else if c.is_ascii_uppercase() {
            state.set_pending_register(c.to_ascii_lowercase(), true);
            return false;
        } else if c.is_ascii_digit() {
            state.set_pending_register(c, false);
            return false;
        } else if c == '"' {
            // Explicit unnamed register
            return false;
        } else {
            use crate::editor::state::MessageLevel;
            state.set_message(
                format!("Invalid register name: '{}'", c),
                MessageLevel::Error,
            );
            state.clear_pending();
            return false;
        }
    } else {
        // Cancel register selection
        return false;
    }
}
```

**Step 5: Update Yank handler to use pending register**

Find the Yank handler and update:

```rust
InputEvent::Yank => {
    use crate::editor::state::MessageLevel;
    if state.pending_command() == Some('y') {
        let count = state.get_count();
        state.clear_pending();
        state.clear_search_results();

        if state.yank_nodes(count) {
            let msg = if count > 1 {
                format!("{} nodes yanked", count)
            } else {
                "Node yanked".to_string()
            };
            state.set_message(msg, MessageLevel::Info);
        } else {
            state.set_message("Nothing to yank".to_string(), MessageLevel::Error);
        }
    } else {
        state.clear_message();
        state.set_pending_command('y');
    }
}
```

**Step 6: Update Paste handlers similarly**

**Step 7: Run tests**

Run: `cargo test`
Expected: All pass

**Step 8: Commit**

```bash
git add src/input/handler.rs
git commit -m "feat(registers): implement register selection in input handler"
```

---

## Task 10: Update Status Bar to Show Pending Register

**Files:**
- Modify: `src/ui/status_line.rs`

**Step 1: Update render_status_line to show register**

Find where pending count is displayed and add register display:

```rust
// Show pending register
if let Some(reg) = state.get_pending_register() {
    let register_str = if state.get_append_mode() {
        format!("\"{}",  reg.to_ascii_uppercase())
    } else {
        format!("\"{}",  reg)
    };
    status_parts.push(Span::styled(
        register_str,
        Style::default().fg(theme.colors.warning_fg),
    ));
}
```

**Step 2: Test visually**

Run: `cargo run`
Press `"a` and verify it shows in status bar

**Step 3: Commit**

```bash
git add src/ui/status_line.rs
git commit -m "feat(registers): show pending register in status bar"
```

---

## Task 11: Update Help Screen Documentation

**Files:**
- Modify: `src/ui/help_overlay.rs`

**Step 1: Add register section to help**

Add after the Yank/Paste section:

```rust
"",
"REGISTERS",
"\"a        - Select register 'a' for next yank/paste/delete",
"\"A        - Select register 'a' (append mode)",
"\"5        - Select numbered register 5",
"\"ayy      - Yank to register 'a'",
"\"ap       - Paste from register 'a'",
"\"0p       - Paste from last yank",
"\"1p       - Paste from last delete",
"yy / dd    - Yank/delete to unnamed register (syncs clipboard)",
```

**Step 2: Test help screen**

Run: `cargo run`, press `?`, verify register section appears

**Step 3: Commit**

```bash
git add src/ui/help_overlay.rs
git commit -m "docs(help): add register commands to help screen"
```

---

## Task 12: Update Path Yank Operations for Registers

**Files:**
- Modify: `src/editor/state.rs`

**Step 1: Update yank_path_dot() for registers**

```rust
pub fn yank_path_dot(&mut self) -> bool {
    if let Some(path_str) = self.compute_path_string("dot") {
        // With register: store internally, no clipboard
        if let Some(reg) = self.pending_register {
            // Store path string as a special marker node
            // For now, skip this feature (path yanks to registers is nice-to-have)
            return false;
        }

        // Without register: copy to system clipboard (existing behavior)
        use arboard::Clipboard;
        if let Ok(mut clipboard) = Clipboard::new() {
            if clipboard.set_text(path_str.clone()).is_ok() {
                return true;
            }
        }
    }
    false
}
```

**Step 2: Similar updates for yank_path_bracket() and yank_path_jq()**

**Step 3: Run tests**

Run: `cargo test`
Expected: All pass

**Step 4: Commit**

```bash
git add src/editor/state.rs
git commit -m "feat(registers): update path yank for register compatibility"
```

---

## Task 13: Add Integration Tests for Register System

**Files:**
- Create: `tests/register_tests.rs`

**Step 1: Write comprehensive register tests**

```rust
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_named_register_yank_paste() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    tree.expand(&[]);

    let mut state = EditorState::new(tree);
    state.cursor_mut().push(0);

    // Yank to register 'a'
    state.set_pending_register('a', false);
    assert!(state.yank_nodes(1));
    state.clear_register_pending();

    // Move to second element
    state.move_cursor_down();

    // Paste from register 'a'
    state.set_pending_register('a', false);
    assert!(state.paste_nodes_at_cursor().is_ok());
}

#[test]
fn test_append_mode() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    tree.expand(&[]);

    let mut state = EditorState::new(tree);
    state.cursor_mut().push(0);

    // Yank first node to 'a'
    state.set_pending_register('a', false);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Move to second node and append to 'a'
    state.move_cursor_down();
    state.set_pending_register('a', true);  // Append mode
    state.yank_nodes(1);
    state.clear_register_pending();

    // Register 'a' should have 2 nodes
    let reg_a = state.registers.get_named('a').unwrap();
    assert_eq!(reg_a.nodes.len(), 2);
}

#[test]
fn test_delete_history() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    tree.expand(&[]);

    let mut state = EditorState::new(tree);
    state.cursor_mut().push(0);

    // Delete three nodes
    let _ = state.delete_node_at_cursor();
    let _ = state.delete_node_at_cursor();
    let _ = state.delete_node_at_cursor();

    // Check history
    assert_eq!(state.registers.get_numbered(1).nodes.len(), 1); // Most recent
    assert_eq!(state.registers.get_numbered(2).nodes.len(), 1);
    assert_eq!(state.registers.get_numbered(3).nodes.len(), 1);
}

#[test]
fn test_yank_register_zero() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
    ])));
    tree.expand(&[]);

    let mut state = EditorState::new(tree);
    state.cursor_mut().push(0);

    // Yank
    state.yank_nodes(1);

    // Register "0 should have the yank
    assert_eq!(state.registers.get_numbered(0).nodes.len(), 1);

    // Delete shouldn't affect "0
    let _ = state.delete_node_at_cursor();
    assert_eq!(state.registers.get_numbered(0).nodes.len(), 1);
}
```

**Step 2: Run tests**

Run: `cargo test`
Expected: All pass

**Step 3: Commit**

```bash
git add tests/register_tests.rs
git commit -m "test(registers): add comprehensive integration tests"
```

---

## Task 14: Run Pre-Commit Checks

**Files:**
- All modified files

**Step 1: Run cargo fmt**

Run: `cargo fmt`

**Step 2: Run cargo clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

Fix any warnings that appear.

**Step 3: Run full test suite**

Run: `cargo test`
Expected: All tests pass (122+ tests)

**Step 4: Test build**

Run: `cargo build --release`
Expected: Success

**Step 5: Manual testing**

Run: `./target/release/jsonquill test.json`

Test:
- `"ayy` - yank to register a
- `"ap` - paste from register a
- `"Ayy` - append to register a
- `dd` then `"1p` - delete and paste from history
- Status bar shows `"a` when pending

**Step 6: Commit if fixes needed**

```bash
git add -A
git commit -m "chore: fix clippy warnings and formatting"
```

---

## Task 15: Update Documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `README.md` (if exists)

**Step 1: Update CLAUDE.md usage section**

Add register documentation to the "Editing (NORMAL mode)" section:

```markdown
# Registers
"a          - Select register 'a' for next operation
"A          - Select register 'a' (append mode)
"0-"9       - Select numbered registers (0=last yank, 1-9=delete history)
"ayy        - Yank to register 'a'
"ap / "aP   - Paste from register 'a'
"add        - Delete to register 'a'
yy / dd     - Use unnamed register (syncs system clipboard)
```

**Step 2: Update "Known Issues / TODO" section**

Remove the line:
```markdown
- ❌ **No named registers** - `"ayy`, `"ap` for named register operations
```

Add to "Working Features":
```markdown
- ✅ Named registers (a-z) with append mode (A-Z) and history (0-9)
```

**Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: document named register feature"
```

---

## Task 16: Final Verification and Summary

**Step 1: Run complete test suite**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 3: Test in real usage**

Create test JSON file and test all register operations:
- Named registers (a-z)
- Append mode (A-Z)
- Numbered registers (0-9)
- System clipboard sync (unnamed only)
- Count prefix with registers
- Status bar display

**Step 4: Create summary**

List all commits made:
```bash
git log --oneline feature/named-registers ^main
```

**Step 5: Push branch (optional)**

```bash
git push -u origin feature/named-registers
```

---

## Success Criteria Checklist

- [ ] All 36 registers (unnamed + 26 named + 10 numbered) work correctly
- [ ] Append mode (`"Ayy`) works as expected
- [ ] Automatic history (`"0`, `"1-"9`) updates correctly
- [ ] System clipboard only syncs with unnamed register
- [ ] Count prefixes work with registers (`"a3yy`, `3"ayy`)
- [ ] Status bar shows pending register
- [ ] All tests pass (cargo test)
- [ ] No clippy warnings (cargo clippy)
- [ ] Help screen documents new commands
- [ ] Documentation updated (CLAUDE.md)
- [ ] Manual testing confirms all features work

---

## Notes

- TDD approach: Write test first, then implementation
- Frequent commits: After each task completion
- DRY: Reuse RegisterContent for all register types
- YAGNI: Skip path-to-register feature (nice-to-have, not in design)
- Use @superpowers:verification-before-completion before final commit
