# Undo/Redo System Design

**Date:** 2026-01-21
**Status:** Approved for implementation

## Overview

Implement vim-style undo/redo with branching undo tree support for jsonquill. This allows users to undo tree modifications (delete, paste, value edits) and navigate through edit history.

## Requirements

### Commands to Implement
- `u` - Undo last change (Normal mode)
- `Ctrl-r` - Redo last undone change (Normal mode)
- `:undo` - Undo via command mode
- `:redo` - Redo via command mode

### Operations Captured
Only tree modifications are captured:
- Delete operations (`dd`)
- Paste operations (`p`, `P`)
- Value edits (INSERT mode commit)

NOT captured:
- Cursor movements (j/k/gg/G)
- Expand/collapse state (h/l)
- Search operations
- Mode changes

### History Model
**Branching undo tree** (vim-style):
- When you undo then make a new edit, the old "future" is preserved as a branch
- You can still access previously undone changes
- `Ctrl-r` follows the newest branch (highest sequence number)

## Architecture

### Data Structures

```rust
// src/editor/undo.rs

struct UndoNode {
    snapshot: EditorSnapshot,
    parent: Option<usize>,        // Index of parent node
    children: Vec<usize>,          // Indices of child nodes
    timestamp: SystemTime,         // When this state was created
    seq: u64,                      // Sequence number for ordering
}

struct UndoTree {
    nodes: Vec<UndoNode>,          // All nodes in the tree
    current: usize,                 // Index of current state
    next_seq: u64,                  // Next sequence number
    limit: usize,                   // Max nodes to keep
}

struct EditorSnapshot {
    tree: JsonTree,                 // Complete tree state
    cursor_path: Vec<usize>,        // Cursor position
}
```

### Key Decisions

1. **Full snapshots** - Store complete tree clones (not diffs) for simplicity
2. **Lower limit** - Default `undo_limit` reduced from 1000 to 50 to manage memory
3. **Sequence numbers** - Provide chronological ordering across branches
4. **Parent + children** - Each node tracks both for tree navigation

## Algorithms

### Undo (u key)
```
1. If current node has a parent:
   - Move current pointer to parent
   - Restore parent's snapshot
   - Show message: "Undo to seq N"
2. Else: show "Already at oldest change"
```

### Redo (Ctrl-r key)
```
1. If current node has children:
   - Find child with highest sequence number (newest branch)
   - Move current pointer to that child
   - Restore child's snapshot
   - Show message: "Redo to seq N"
2. Else: show "Already at newest change"
```

### Add Checkpoint (before mutations)
```
1. Create snapshot of current state
2. Create new UndoNode with current as parent
3. Add new node to tree
4. If current has existing children:
   - Keep them (branching!) but new node becomes latest
5. Move current pointer to new node
6. If tree size exceeds limit:
   - Prune oldest unreachable nodes (not on current branch)
```

## Integration

### EditorState Changes

```rust
// In src/editor/state.rs
pub struct EditorState {
    // ... existing fields ...
    undo_tree: UndoTree,
    undo_limit: usize,  // From config
}

impl EditorState {
    // Create checkpoint before mutating operations
    fn checkpoint(&mut self) {
        let snapshot = EditorSnapshot {
            tree: self.tree.clone(),
            cursor_path: self.cursor.path().to_vec(),
        };
        self.undo_tree.add_checkpoint(snapshot);
    }

    pub fn undo(&mut self) -> bool { /* ... */ }
    pub fn redo(&mut self) -> bool { /* ... */ }
}
```

### Checkpoint Locations

Call `checkpoint()` automatically before:
- `delete_node_at_cursor()`
- `paste_node_at_cursor()`
- `paste_node_before_cursor()`
- `commit_editing()` (value changes)

### Input Handling

- Normal mode `u` → `state.undo()`
- Normal mode `Ctrl-r` → `state.redo()`
- Command `:undo` → `state.undo()`
- Command `:redo` → `state.redo()`

## Implementation Tasks

1. **Create `src/editor/undo.rs`**
   - Implement `EditorSnapshot`, `UndoNode`, `UndoTree`
   - Unit tests for branching behavior

2. **Modify `src/editor/state.rs`**
   - Add `undo_tree` field and initialization
   - Add `checkpoint()` method
   - Inject `checkpoint()` calls before mutations
   - Add public `undo()` and `redo()` methods

3. **Modify `src/input/handler.rs`**
   - Handle `u` key in Normal mode
   - Handle `Ctrl-r` in Normal mode
   - Handle `:undo` and `:redo` commands

4. **Modify `src/config/mod.rs`**
   - Change default `undo_limit` from 1000 to 50

5. **Update `src/editor/mod.rs`**
   - Add `pub mod undo;`

6. **Testing**
   - Unit tests for undo tree branching
   - Integration tests: dd→undo, paste→undo→redo, edit→undo→edit

## Memory Considerations

- **Strategy:** Full snapshots with reduced limit (50 operations)
- **Typical memory:** ~50 copies of the JSON tree
- **For 100KB JSON:** ~5MB of undo history
- **Pruning:** Remove oldest nodes not on current branch when limit exceeded

## Future Extensions

Possible future enhancements (not in this implementation):
- `:undo N` / `:redo N` - multiple steps
- `g-` / `g+` - chronological navigation across branches
- `:earlier` / `:later` - time-based navigation
- `:undolist` - view the undo tree structure
- Persistent undo (save to disk)
