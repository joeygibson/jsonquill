# Named Registers Design

**Date:** 2026-01-26
**Status:** Approved

## Overview

Add vim-style named registers to jsonquill, providing 26 named registers (a-z) with append support (A-Z), plus automatic yank/delete history (0-9). This matches vim's register behavior for familiar and powerful clipboard management.

## Register Types

### 1. Unnamed Register (`""`)
- Current clipboard, used by `yy` and `dd` without register prefix
- Syncs to system clipboard via arboard
- Used by `p`/`P` when no register specified
- Separate from named registers (vim-style separation)

### 2. Named Registers (a-z)
- 26 user-controlled registers
- Explicitly specified with `"a`, `"b`, etc.
- Lowercase replaces: `"ayy` replaces register 'a'
- Uppercase appends: `"Ayy` appends to register 'a' (same storage as 'a')
- Never sync to system clipboard
- Persist across yank/delete operations

### 3. Numbered Registers (0-9)
- Automatic yank/delete history
- `"0`: Always contains last yank (from any yank operation)
- `"1-"9`: Delete history
  - On `dd`: current `"1` → `"2`, `"2` → `"3`, ... `"9` is lost
  - New delete goes to `"1`
  - Yank operations don't affect `"1-"9`, only `"0`

## Data Structures

### Register Content

Each register stores multiple nodes (for count-prefixed yanks like `3yy`):

```rust
pub struct RegisterContent {
    pub nodes: Vec<JsonNode>,
    pub keys: Vec<Option<String>>, // One key per node (for object members)
}
```

### Register Set

```rust
pub struct RegisterSet {
    unnamed: RegisterContent,           // "" register
    named: HashMap<char, RegisterContent>, // a-z (lowercase keys)
    numbered: [RegisterContent; 10],    // 0-9
}

impl RegisterSet {
    pub fn new() -> Self;
    pub fn get(&self, register: char) -> Option<&RegisterContent>;
    pub fn set(&mut self, register: char, content: RegisterContent);
    pub fn append(&mut self, register: char, content: RegisterContent);
    pub fn push_delete_history(&mut self, content: RegisterContent);
    pub fn update_yank_register(&mut self, content: RegisterContent);
}
```

## Register Update Behavior

### Yank Operations (`yy`)

Updates:
1. Target register (unnamed if not specified, or `"a` if specified)
2. Register `"0` (last yank)
3. System clipboard (only if unnamed register)

Uppercase (e.g., `"Ayy`) appends to the register instead of replacing.

### Delete Operations (`dd`)

Updates:
1. Target register (unnamed if not specified, or `"a` if specified)
2. Register `"1` (most recent delete), pushes history `"1` → `"2`, etc.
3. System clipboard (only if unnamed register)

**Important:** Yank and delete have separate histories. `"0` always has your last yank even after deletes. `"1` has your most recent delete.

### Path Yank Operations (`yp`, `yb`, `yq`)

These yank path strings (not JSON nodes):
- With register prefix (`"ayp`): Store in specified register as internal string
- Without register: Copy directly to system clipboard (current behavior)
- Numbered registers (`"0-"9`) not updated by path yanks

## User Interface

### Register Specification Syntax

The `"` character enters register selection mode:
- `"a` → select register 'a'
- `"A` → select register 'a' with append mode
- `"5` → select numbered register 5
- `""` → explicitly select unnamed register (rarely needed)

### Supported Commands

All yank/paste operations support register prefixes:
- `"ayy` / `"a3yy` / `3"ayy` - Yank to register 'a'
- `"Ayy` - Append to register 'a'
- `"ap` / `"aP` - Paste from register 'a'
- `"add` / `"a3dd` - Delete to register 'a'
- `"0p` - Paste from last yank
- `"1p` - Paste from last delete
- `"5p` - Paste from delete history 5

Path yank commands:
- `"ayp` - Yank path (dot notation) to register 'a'
- `"ayb` - Yank path (bracket notation) to register 'a'
- `"ayq` - Yank path (jq style) to register 'a'

### Status Bar Indication

When a register is pending (after pressing `"`), show it in the status bar:
- `"a` - register 'a' selected
- `"A` - register 'a' selected (append mode)
- `"5` - register 5 selected

Similar to how pending count is displayed.

### Error Handling

- Invalid register names (e.g., `"@yy`) → show error, clear pending
- Empty register on paste → show "Nothing in register 'a'"
- Register selection cancelled (Esc) → clear pending

## Implementation Plan

### 1. New Module: src/editor/registers.rs

Create dedicated module for register management:
- `RegisterContent` struct
- `RegisterSet` struct with full API
- Unit tests for register operations

### 2. EditorState Changes (src/editor/state.rs)

Remove old fields:
```rust
clipboard: Option<JsonNode>,
clipboard_key: Option<String>,
```

Add new fields:
```rust
registers: RegisterSet,
pending_register: Option<char>,
append_mode: bool,
```

Update method signatures:
- `yank_node()` → `yank_nodes(count, register, append)`
- `paste_node_at_cursor()` → `paste_nodes_at_cursor(register)`
- `paste_node_before_cursor()` → `paste_nodes_before_cursor(register)`
- `delete_node_at_cursor()` → keep signature, but update register internally

Add new methods:
- `set_pending_register(char, bool)`
- `get_pending_register() -> Option<char>`
- `get_append_mode() -> bool`
- `clear_register_pending()`

### 3. Input Event Changes (src/input/keys.rs)

Add new event:
```rust
pub enum InputEvent {
    // Existing events...
    RegisterSelect, // Triggered by '"' key
}
```

Map `"` key to `RegisterSelect` in key parsing.

### 4. Input Handler Changes (src/input/handler.rs)

Handle `InputEvent::RegisterSelect`:
1. Set `awaiting_register` flag
2. Next keypress validates and stores register
3. Execute subsequent command with stored register

Update existing handlers:
- `InputEvent::Yank` - pass register and append mode to `yank_nodes()`
- `InputEvent::Paste` - pass register to `paste_nodes_at_cursor()`
- `InputEvent::PasteBefore` - pass register to `paste_nodes_before_cursor()`
- `InputEvent::Delete` - update to use register system

Integrate with pending command system:
- Clear all pending state (count, register, command) together
- Status bar shows all pending elements

### 5. Help Screen Update (src/ui/help_overlay.rs)

Add documentation for:
- Register selection with `"`
- Named registers (a-z)
- Append mode (A-Z)
- Numbered registers (0-9)
- Examples: `"ayy`, `"ap`, `"0p`, etc.

### 6. Documentation Updates

Update CLAUDE.md and README.md:
- Add register commands to usage section
- Document register types and behavior
- Add examples of common workflows

### 7. Testing

Add comprehensive tests:
- Register storage and retrieval
- Append mode behavior
- Numbered register history
- System clipboard sync (unnamed only)
- Count prefix with registers (`"a3yy`, `3"ayy`)
- Error cases (invalid registers, empty registers)

## Success Criteria

- [ ] All 36 registers (unnamed + 26 named + 10 numbered) work correctly
- [ ] Append mode (`"Ayy`) works as expected
- [ ] Automatic history (`"0`, `"1-"9`) updates correctly
- [ ] System clipboard only syncs with unnamed register
- [ ] Count prefixes work with registers (`"a3yy`, `3"ayy`)
- [ ] Status bar shows pending register
- [ ] All tests pass
- [ ] Help screen documents new commands
- [ ] Documentation updated

## Examples

### Basic Usage
```
# Yank current node to register 'a'
"ayy

# Yank 3 nodes to register 'b'
"b3yy
# or
3"byy

# Append to register 'a'
"Ayy

# Paste from register 'a'
"ap
```

### Delete History
```
# Delete some nodes
dd    # Goes to "1
dd    # Previous "1 → "2, new delete → "1
dd    # Previous "1 → "2, "2 → "3, new delete → "1

# Recover second-to-last delete
"2p
```

### Yank History
```
# Yank some nodes
yy    # Goes to unnamed and "0
dd    # Goes to "1 (doesn't affect "0)
yy    # Goes to unnamed and "0 (old "0 is lost)

# Paste last yank even after deletes
"0p
```

## Future Enhancements

Potential future additions (not in this design):
- Special registers like `"+` for explicit clipboard control
- Small delete register `"-` (single character/value deletes)
- Last insert register `".`
- Read-only registers like `"%` (current filename)
- Command to view register contents (`:registers` or `:reg`)
