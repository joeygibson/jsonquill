# Visual Mode, Marks, Jump List, and Repeat Command Design

**Date:** 2026-01-29
**Status:** Approved
**Features:** Visual mode (`v`), Marks (`m`/`'`), Jump list (`Ctrl-o`/`Ctrl-i`), Repeat (`.`)

## Overview

This design adds four vim-style features to jsonquill that enhance navigation and editing efficiency:

1. **Visual Mode** - Select multiple nodes for bulk operations
2. **Jump List** - Navigate backward/forward through cursor position history
3. **Marks** - Bookmark positions and jump back to them
4. **Repeat Command** - Replay the last editing operation

All features integrate with existing systems (undo/redo, count prefixes, registers) and follow vim conventions.

## Architecture

### High-Level Integration

**Visual Mode (`v`)**
- New mode: `EditorMode::Visual` added to the enum
- State tracks: selection start position, current position, selected node paths
- Operations: `d`, `y`, `p`, `P` work on entire selection
- Visual indication: Selected nodes highlighted in the tree view

**Jump List (`Ctrl-o` / `Ctrl-i`)**
- New component: `JumpList` struct in `src/editor/jumplist.rs`
- Ring buffer storing cursor positions (limit: 100 jumps)
- Current position pointer for forward/backward navigation
- Integrates with existing cursor position tracking

**Marks (`m{a-z}`, `'{a-z}`)**
- New component: `MarkSet` struct in `src/editor/marks.rs` (similar to existing `RegisterSet`)
- Stores 26 marks (a-z) as cursor paths (`Vec<usize>`)
- Session-scoped (not persisted to disk)
- Input handling: two-key sequence like registers

**Repeat Command (`.`)**
- New component: `RepeatableCommand` enum in `src/editor/repeat.rs`
- State: `last_command: Option<RepeatableCommand>` in EditorState
- Recorded on: delete, yank, add, rename, change, paste
- Replayed on: `.` key press with same parameters

### System Integration

All features integrate with:
- **Undo/redo** - Operations create undo snapshots
- **Count prefixes** - `3dd`, `5v` to select 5 nodes, `3.` to repeat 3 times
- **Status line** - Show mode, feedback messages
- **Registers** - Visual mode operations work with named registers

## Detailed Design

### 1. Visual Mode

#### State Management

```rust
// In src/editor/mode.rs
pub enum EditorMode {
    Normal,
    Insert,
    Command,
    Search,
    Visual,  // NEW
}

// In src/editor/state.rs - EditorState struct
visual_anchor: Option<Vec<usize>>,  // Where selection started
visual_selection: Vec<Vec<usize>>,  // All selected node paths
```

#### Behavior Flow

1. User presses `v` in Normal mode → Enter Visual mode, set anchor to current cursor
2. User moves cursor (`j`, `k`, `h`, `l`) → Update `visual_selection` with range from anchor to cursor
3. User presses operation key (`d`, `y`, `p`, `P`) → Execute on all selected nodes, return to Normal mode
4. User presses `Esc` → Clear selection, return to Normal mode

#### Selection Calculation

- When cursor moves, recalculate selection as all visible nodes between anchor and cursor (inclusive)
- Respects tree visibility: collapsed nodes count as one item
- Works with count prefix: `5v` enters visual mode and selects 5 nodes downward
- Selection is always a contiguous range of visible nodes

#### UI Rendering

- Selected nodes: background color from theme (new theme field: `visual_selection_bg`)
- Visual mode indicator: Status line shows "VISUAL" instead of "NORMAL"
- Selection count: Status line shows "5 nodes selected"

#### Supported Operations

- `d` - Delete all selected nodes in one undo-able operation
- `y` - Yank all selected nodes to register (stored as array of nodes)
- `p` - Paste yanked content after selection end
- `P` - Paste yanked content before selection start
- All operations exit Visual mode after execution

#### Keybindings

- `v` - Enter visual mode (Normal mode only)
- `Esc` - Exit visual mode, return to Normal
- `j`, `k`, `h`, `l`, arrows - Extend/shrink selection
- `d`, `y`, `p`, `P` - Execute operation on selection

### 2. Jump List

#### Data Structure

```rust
// New file: src/editor/jumplist.rs

/// Manages cursor position history for Ctrl-o/Ctrl-i navigation.
pub struct JumpList {
    /// Stored cursor paths (ring buffer)
    jumps: Vec<Vec<usize>>,
    /// Current position in jump list
    current: usize,
    /// Maximum jumps to store
    max_size: usize,
}

impl JumpList {
    pub fn new(max_size: usize) -> Self;
    pub fn record_jump(&mut self, cursor_path: Vec<usize>);
    pub fn jump_backward(&mut self) -> Option<Vec<usize>>;
    pub fn jump_forward(&mut self) -> Option<Vec<usize>>;
    pub fn len(&self) -> usize;
    pub fn current_position(&self) -> usize;
}
```

```rust
// In src/editor/state.rs - EditorState struct
jumplist: JumpList,
```

#### Jump Recording Logic

Record a jump BEFORE executing these commands:
- `gg`, `G`, `<count>G` - Line jumps
- `n`, `*`, `#` - Search navigation
- `'{a-z}` - Mark jumps
- `:path` / `:jp` JSONPath results

Do NOT record:
- Regular movement: `j`, `k`, `h`, `l`, arrows
- Sibling navigation: `{`, `}`, `0`, `$`, `w`, `b`
- Page scrolling: `Ctrl-d`, `Ctrl-u`, `Ctrl-f`, `Ctrl-b`
- Parent navigation: `H`

#### Navigation Behavior

**`Ctrl-o` - Jump backward:**
1. Move `current` pointer back one position
2. Restore cursor to that position's path
3. Show status message: "Jump 3/15" (current/total)

**`Ctrl-i` - Jump forward:**
1. Move `current` pointer forward one position
2. Restore cursor to that position's path
3. Show status message: "Jump 4/15"

**Recording new jumps:**
- When at middle of jump list and new jump recorded: Truncate everything after `current`, append new jump
- This matches vim behavior: future is discarded when you jump back then navigate elsewhere

**Ring buffer behavior:**
- When limit (100) reached, remove oldest jump
- New jumps always appended at end

#### Edge Cases

- **Jumping to deleted node:** Find closest valid ancestor node, show warning
- **Empty jump list:** `Ctrl-o`/`Ctrl-i` do nothing, show message "No jump history"
- **Jump to same position as current:** Don't record duplicate consecutive jumps
- **Jump list survives undo/redo:** Cursor positions are paths, remain valid

#### UI Feedback

- Status message after jump: "Jump 3/15" (position/total)
- At oldest jump, `Ctrl-o` shows: "At oldest jump"
- At newest jump, `Ctrl-i` shows: "At newest jump"

#### Future Enhancement

- `:jumps` command to display jump list (vim-style)

### 3. Marks

#### Data Structure

```rust
// New file: src/editor/marks.rs

use std::collections::HashMap;

/// Manages local marks (a-z) for bookmarking positions.
pub struct MarkSet {
    marks: HashMap<char, Vec<usize>>,
}

impl MarkSet {
    pub fn new() -> Self;
    pub fn set_mark(&mut self, name: char, cursor_path: Vec<usize>);
    pub fn get_mark(&self, name: char) -> Option<&Vec<usize>>;
    pub fn clear(&mut self);
    pub fn list(&self) -> Vec<(char, &Vec<usize>)>;
}
```

```rust
// In src/editor/state.rs - EditorState struct
marks: MarkSet,
pending_mark_set: bool,    // Waiting for letter after 'm'
pending_mark_jump: bool,   // Waiting for letter after '
```

#### Setting Marks

**Flow:**
1. User presses `m` in Normal mode → Set `pending_mark_set = true`, show "Mark: " in status
2. User presses letter `a`-`z` → Store current cursor path in `marks[letter]`, show message "Mark 'a' set"
3. Invalid key (not a-z) or `Esc` → Cancel, clear pending state, show "Cancelled"

**Storage:**
- Marks store cursor path (`Vec<usize>`)
- 26 available marks: lowercase letters only (a-z)
- Marks are NOT updated when tree changes (may become invalid)

#### Jumping to Marks

**Flow:**
1. User presses `'` (single quote) or `` ` `` (backtick) in Normal mode → Set `pending_mark_jump = true`, show "Jump to mark: " in status
2. User presses letter `a`-`z`:
   - If mark exists: Restore cursor to mark position, record jump in jump list, show "Jumped to mark 'a'"
   - If mark doesn't exist: Show error "Mark 'a' not set"
3. Invalid key or `Esc` → Cancel, clear pending state

**Jump list integration:**
- Jumping to a mark records current position in jump list first
- This allows `Ctrl-o` to return to pre-jump position

#### Mark Lifecycle

- Marks persist during editing session
- Marks cleared when:
  - File closed / new file loaded (`:e`)
  - Explicit clear command (future: `:delmarks`)
- Marks NOT cleared by undo/redo

#### Handling Deleted Nodes

When jumping to a mark whose node was deleted:
1. Walk up the cursor path, removing segments until a valid node is found
2. Restore cursor to closest valid ancestor
3. Show warning: "Mark 'a' position no longer exists, jumped to nearest node"

#### UI Feedback

- After setting: "Mark 'a' set"
- After jumping: "Jumped to mark 'a'"
- Mark doesn't exist: "Mark 'a' not set"
- Mark position deleted: "Mark 'a' position no longer exists, jumped to nearest node"

#### Keybindings

- `m{a-z}` - Set mark
- `'{a-z}` - Jump to mark (also supports `` `{a-z} ``)

#### Future Enhancements

- `:marks` command to list all set marks
- `:delmarks {marks}` to delete specific marks
- Special marks: `` `. `` (last change), `` `" `` (last exit position)

### 4. Repeat Command

#### Data Structure

```rust
// New file: src/editor/repeat.rs

use crate::document::node::JsonValue;

/// Represents a command that can be repeated with the '.' key.
#[derive(Debug, Clone)]
pub enum RepeatableCommand {
    /// Delete with count (dd, 3dd)
    Delete { count: u32 },

    /// Yank with count (yy, 5yy)
    Yank { count: u32 },

    /// Paste after (p) or before (P)
    Paste { before: bool },

    /// Add scalar value (i)
    Add { value: JsonValue, key: Option<String> },

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

```rust
// In src/editor/state.rs - EditorState struct
last_command: Option<RepeatableCommand>,
```

#### Recording Commands

Capture these operations when successfully executed:

- `dd`, `3dd` → `RepeatableCommand::Delete { count: 3 }`
- `yy`, `5yy` → `RepeatableCommand::Yank { count: 5 }`
- `p` → `RepeatableCommand::Paste { before: false }`
- `P` → `RepeatableCommand::Paste { before: true }`
- `i` + enter value → `RepeatableCommand::Add { value, key }`
- `a` → `RepeatableCommand::AddArray`
- `o` → `RepeatableCommand::AddObject`
- `r` + new key → `RepeatableCommand::Rename { new_key }`
- `e` + change value → `RepeatableCommand::ChangeValue { new_value }`

**Recording happens AFTER successful completion**, not when initiated. This ensures we only record commands that actually modified the document.

#### Replaying with `.`

**Flow:**
1. User presses `.` in Normal mode
2. If `last_command` is None → Show message "No command to repeat"
3. Otherwise:
   - Execute the stored command with original parameters at current cursor position
   - Create undo snapshot (normal operation)
   - Update `last_command` to the replayed command (vim behavior: `.` can be repeated)
   - Show message: "Repeated: <command description>"

**Examples:**
- Last command was `dd` → `.` deletes current node
- Last command was `i` adding `"hello"` → `.` adds `"hello"` again at current position
- Last command was `r` renaming to "newKey" → `.` renames current key to "newKey"

#### Count Prefix Interaction

- `3.` → Repeat the last command 3 times
- The count applies to the repeat action, not to the stored command's count

**Examples:**
- Last command: `dd` (delete 1), press `3.` → Delete 3 nodes (deletes 1 node 3 times)
- Last command: `2dd` (delete 2), press `.` → Delete 2 nodes
- Last command: `2dd` (delete 2), press `3.` → Delete 6 nodes total (delete 2 nodes, 3 times)

#### What NOT to Repeat

These operations do NOT set `last_command`:
- Navigation movements (`j`, `k`, `h`, `l`, `gg`, `G`, etc.)
- Mode changes (`v`, `:`, `/`)
- Visual mode operations (future: may support this)
- Command mode operations (`:w`, `:format`, `:path`, etc.)
- Search (`/`, `n`, `*`, `#`)
- Undo/redo (`u`, `Ctrl-r`)

#### Error Handling

When repeating fails (e.g., trying to rename an array element):
- Show error message describing why it failed
- Don't clear `last_command` (user can retry elsewhere)
- Don't create undo snapshot

#### UI Feedback

- After successful repeat: "Repeated: delete 2 nodes"
- No command to repeat: "No command to repeat"
- Repeat failed: Show specific error (e.g., "Cannot rename array element")

#### Keybindings

- `.` - Repeat last command (Normal mode only)

#### Future Enhancements

- Visual mode operations (e.g., repeat `vjjjd`)
- Macro recording/playback (`q`, `@`) - more powerful than simple repeat

## Implementation Plan

### Phase 1: Core Infrastructure
1. Add `EditorMode::Visual` to mode enum
2. Create `src/editor/jumplist.rs` with `JumpList` struct
3. Create `src/editor/marks.rs` with `MarkSet` struct
4. Create `src/editor/repeat.rs` with `RepeatableCommand` enum
5. Add new state fields to `EditorState`

### Phase 2: Visual Mode
1. Implement visual mode entry/exit (`v`, `Esc`)
2. Implement selection tracking and calculation
3. Add visual selection rendering in tree view
4. Implement visual operations (`d`, `y`, `p`, `P`)
5. Add status line display for visual mode
6. Add count prefix support (`5v`)

### Phase 3: Jump List
1. Implement `JumpList` data structure with ring buffer
2. Add jump recording before big navigation commands
3. Implement `Ctrl-o` / `Ctrl-i` navigation
4. Add edge case handling (deleted nodes, empty list)
5. Add status messages for jump feedback

### Phase 4: Marks
1. Implement `MarkSet` data structure
2. Add `m` key handling for setting marks
3. Add `'` key handling for jumping to marks
4. Add pending state handling in input system
5. Integrate with jump list (mark jumps record in jump list)
6. Add edge case handling (deleted nodes, invalid marks)

### Phase 5: Repeat Command
1. Implement `RepeatableCommand` enum
2. Add command recording after successful operations
3. Implement `.` key handling for replay
4. Add count prefix support (`3.`)
5. Add error handling for failed repeats

### Phase 6: Integration & Testing
1. Add keybindings to `src/input/keys.rs`
2. Update help overlay with new features
3. Update CLAUDE.md documentation
4. Write integration tests
5. Update README.md

### Phase 7: Theme Support
1. Add `visual_selection_bg` to theme struct
2. Update all built-in themes with visual selection color

## Testing Strategy

### Unit Tests

**Visual Mode:**
- Selection calculation with various anchor/cursor positions
- Selection with collapsed nodes
- Operations on multi-node selections
- Count prefix behavior

**Jump List:**
- Ring buffer behavior (wrapping, max size)
- Forward/backward navigation
- Truncation when jumping back then navigating
- Edge cases (empty, deleted nodes)

**Marks:**
- Setting and retrieving marks
- Invalid mark names
- Deleted node handling
- Mark lifecycle (persist/clear)

**Repeat Command:**
- Recording various command types
- Replay with different cursor positions
- Count prefix interaction
- Error handling

### Integration Tests

- Visual mode + registers (`"ayy` in visual mode)
- Visual mode + undo/redo
- Jump list + marks (mark jumps recorded in jump list)
- Repeat + count prefix (`3.` after `2dd`)
- All features with undo/redo

### Manual Testing

- Visual mode selection rendering across themes
- Status line messages for all features
- Keyboard interaction flows (two-key sequences)
- Edge cases with real JSON files

## Documentation Updates

### CLAUDE.md
- Add visual mode commands to usage section
- Add jump list commands (`Ctrl-o`, `Ctrl-i`)
- Add marks commands (`m{a-z}`, `'{a-z}`)
- Add repeat command (`.`)
- Update feature status checklist

### Help Overlay (`src/ui/help_overlay.rs`)
- Add "Visual Mode" section
- Add "Marks and Jumps" section
- Add repeat command to editing section

### README.md
- Add to feature list
- Update usage examples

## Future Enhancements

1. **Visual mode operations:** Format, case conversion, number increment
2. **Global marks:** `m{A-Z}` for cross-file bookmarks
3. **Special marks:** `` `. ``, `` `" ``, `` `[ ``, `` `] ``
4. **Command mode:** `:marks`, `:jumps`, `:delmarks`
5. **Macros:** `q{a-z}` to record, `@{a-z}` to replay (more powerful than `.`)
6. **Visual block mode:** `Ctrl-v` for non-contiguous selection
7. **Repeat for visual operations:** Repeat complex visual mode sequences

## Open Questions

None - design is complete and approved.

## References

- Vim visual mode: `:help visual-mode`
- Vim jump list: `:help jumplist`
- Vim marks: `:help mark-motions`
- Vim repeat: `:help .`
