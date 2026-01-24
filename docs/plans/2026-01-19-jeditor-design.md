# jsonquill Design Document

**Date:** 2026-01-19
**Status:** Approved

## Overview

jsonquill is a terminal-based JSON editor written in Rust, inspired by jless. It provides structural editing of JSON and JSONL files with vim-style modal keybindings, syntax highlighting, and an interactive tree-based interface.

## Core Features

- **Structural JSON Editing**: Edit JSON through direct manipulation of the structure (add/delete keys, change values, reorder arrays) rather than raw text editing
- **Modal Interface**: Vim-style modes (normal, insert, command) with familiar keybindings
- **JSONL Support**: Line-based view for JSON Lines files with per-line editing
- **Type-Aware Editing**: Different editing experiences for strings, numbers, booleans, and null values
- **Dual Search**: Text search (`/`) and structural path/key search (`:find`, `:path`)
- **Theme System**: Multiple built-in themes plus custom theme support
- **Format Preservation**: Non-destructive editing that preserves original formatting

## Architecture

### Core Components

1. **Parser Layer**
   - Uses `serde_json` for JSON parsing
   - Custom metadata layer preserves original formatting (whitespace, indentation, key ordering)
   - Enables non-destructive editing where unchanged portions retain original format on save

2. **Document Model**
   - Represents JSON as a tree structure
   - Each node knows its type (object, array, string, number, boolean, null) and metadata (formatting, position)
   - For JSONL files, maintains collection of independent JSON documents with line number associations

3. **Editor State**
   - Tracks current mode (normal, insert, command)
   - Cursor position in the tree
   - Undo/redo history (linear stack)
   - Registers for yank/paste operations
   - Dirty flag for unsaved changes

4. **UI Layer** (ratatui)
   - Tree view: navigable JSON structure with syntax highlighting
   - Status line: current mode, file info, position
   - Message area: errors, confirmations, command input
   - Uses selected theme for colorization

5. **Input Handler**
   - Modal command processor that interprets keybindings based on mode
   - Routes commands to appropriate operations (navigation, editing, search, file operations)

6. **File Manager**
   - Handles loading with size-based strategy switching
   - Saving with format preservation
   - Optional backup creation (`.jsonquill.bak`)

## UI Layout

### Screen Organization

```
┌────────────────────────────────────────┐
│  1  {                                  │
│  2    "users": [                       │
│  3      ▼ {                            │
│  4          "name": "Alice",           │
│  5          "email": "alice@example",  │
│  6        }                            │
│  7    ]                                │
│  8  }                                  │
│                                        │
│                                        │
└────────────────────────────────────────┘
 NORMAL | data.json [+] | $.users[0]
 :w to save
```

**Main Tree View:**
- Expandable/collapsible tree with indentation
- Expand/collapse indicators (`▶` or `▼`) for objects/arrays
- Key names in distinct color
- Type-specific value rendering (quoted strings, colored types)
- Optional line numbers in left margin

**Status Line:**
- Current mode (NORMAL, INSERT, COMMAND)
- Filename and modified indicator `[+]`
- Current position (JSONPath to selected node)
- File type (JSON/JSONL) and validation status

**Command/Message Area:**
- Command mode: `:` prompt with user input
- Messages: success/error feedback
- Modal dialogs: expanded error details with dismissal prompt

### Navigation (Normal Mode)

**Tree Navigation:**
- `j/k` or arrows: move down/up through visible nodes
- `h/l`: collapse/expand current node (or move to parent/first child)
- `gg/G`: jump to top/bottom
- `{/}`: jump to previous/next sibling at same level
- `Ctrl-d/u`: page down/up

**JSONL Navigation:**
- `J/K` (shift): move between JSON objects (lines)
- Within a line: standard tree navigation

## Editing Operations

### Modal System

**Normal Mode** (default):
- `i`: enter insert mode to edit current value
- `a`: add new field (object) or element (array)
- `o/O`: add new sibling after/before current node
- `dd`: delete current node
- `yy`: yank current node to unnamed register
- `p/P`: paste after/before current node
- `r`: rename current key (object properties only)
- `u`: undo last operation
- `Ctrl-r`: redo
- `:`: enter command mode
- `ZZ`: save if dirty, then quit
- `?`: show help overlay

**Insert Mode** (type-aware):
- **Strings**: Multi-line text editor, arrow keys, backspace, delete. `Esc` saves, `Ctrl-c` cancels
- **Numbers**: Numeric input only, validates as typed, blocks non-numeric characters
- **Booleans**: Toggle with space or `t`/`f` keys
- **Null**: Fixed, cannot edit (delete and add new value instead)
- **Objects/Arrays**: Prompt for key name (objects), then recursive type selection

**Command Mode:**
- `:w` - save file
- `:q` - quit (warns if unsaved)
- `:wq` or `:x` - save and quit
- `:q!` - force quit without saving
- `:set <option>=<value>` - modify settings
- `:set save` - persist current settings to config
- `:theme <name>` - switch color theme
- `:format` - reformat current file structure
- `:help` or `:h` - show help

## Search Capabilities

### Text Search (`/`)

- Searches within JSON string values and key names
- Regex support
- `n/N`: jump to next/previous match
- Case-insensitive by default (configurable with `:set ignorecase`)
- Highlights all matches in tree view
- Search wraps around at end

### Structural Search (`:find`, `:path`)

- `:find keyname` - finds all occurrences of a key
- `:find $.path.to.field` - JSONPath-style search
- `:find @.value` - searches for matching values
- Same `n/N` navigation as text search
- Highlights matching nodes

**Search History:**
- Both modes maintain separate history
- Up/down arrows at prompt navigate history
- Last search persists until new search
- Not persisted across restarts

## Copy/Paste System

### Register Operations

**Unnamed Register** (default):
- `yy`: yank current node
- `p/P`: paste after/before
- Syncs with system clipboard when available

**Named Registers** (`a-z`):
- `"ayy`: yank to register `a`
- `"ap`: paste from register `a`
- `:reg`: show all register contents

**System Clipboard:**
- `"+yy`: explicit yank to system clipboard (register `+`)
- `"+p`: paste from system clipboard
- Unnamed register auto-syncs to clipboard if available
- Graceful fallback if clipboard unavailable

**Paste Validation:**
- Ensures pasted JSON is compatible with target location
- Type checking (objects, arrays, primitives)
- Shows error dialog if paste would create invalid JSON

## Configuration

### Config File

Location: `~/.config/jsonquill/config.toml`

```toml
[general]
theme = "default-dark"
indent_size = 2  # display only
show_line_numbers = true
auto_save = false
validation_mode = "strict"  # or "lenient"
create_backup = false  # .jsonquill.bak files

[editor]
undo_limit = 1000
tab_stops = 2
wrap_lines = false

[clipboard]
sync_unnamed_register = true

[display]
show_hidden_chars = false
compact_arrays = false
max_string_preview = 100

[performance]
lazy_load_threshold = 104857600  # 100MB
```

### Runtime Configuration

- `:set <option>=<value>`: change for current session
- `:set save`: write to config.toml
- `:set <option>?`: show current value
- `:set`: show all modified settings

**Precedence:** Defaults → Config file → Runtime `:set` → CLI flags

## Theme System

### Built-in Themes

- `default-dark` - Default dark theme
- `default-light` - Light background variant
- `monokai` - Sublime Text inspired
- `solarized-dark` / `solarized-light`
- `dracula` - Dracula dark theme
- `gruvbox` - Gruvbox dark theme
- `nord` - Nord color scheme

### Color Categories

- **Syntax**: object keys, strings, numbers, booleans, null, braces/brackets
- **UI**: background, foreground, status line, selection/cursor, line numbers
- **Semantic**: errors, warnings, info, search highlights, modified indicator
- **Tree**: expand/collapse indicators, indentation guides

### Custom Themes

Location: `~/.config/jsonquill/themes/<name>.toml`

```toml
[syntax]
key = "#e06c75"
string = "#98c379"
number = "#d19a66"
boolean = "#56b6c2"
null = "#c678dd"

[ui]
background = "#282c34"
foreground = "#abb2bf"
cursor = "#528bff"
status_line_bg = "#21252b"

[semantic]
error = "#e06c75"
warning = "#e5c07b"
search_highlight = "#3e4451"
```

### Runtime Switching

- `:theme <name>`: switch immediately
- Preference saved with `:set save`
- Invalid themes fall back to `default-dark`

## File Handling

### Loading Strategy

**Small Files (< 100MB):**
- Load entire file into memory
- Parse complete JSON structure
- Build full document tree
- Fast navigation and searching

**Large Files (≥ 100MB):**
- Show notification: "Large file detected, using lazy loading"
- Parse top-level structure immediately
- Defer parsing nested objects/arrays until expanded
- Cache parsed nodes
- Some operations slower (global search, undo across unparsed regions)

**JSONL-Specific:**
- Each line = independent JSON document
- Line-by-line parsing (never loads all lines simultaneously)
- Index maintains line numbers and file offsets
- Load/parse only visible lines when navigating
- Edited lines cached until save

### Save Operations

- `:w`: writes to original file
- Optional `.jsonquill.bak` backup (default: off, configurable)
- Format preservation: unchanged nodes retain original whitespace
- Changed nodes formatted with configured indent size
- JSONL: only modified lines reformatted
- Atomic save: write to temp file, then rename
- Clears dirty flag after save

### stdin/stdout Support

- `cat file.json | jsonquill`: opens stdin as temporary buffer
- `:w <filename>` required to save (no original file)
- `jsonquill < input.json > output.json` not supported (needs terminal control)

## Error Handling

### Validation Modes

**Strict Mode** (default):
- All operations maintain valid JSON
- Prevents: duplicate keys, incompatible types, invalid structural changes
- Blocks operation immediately with modal error dialog
- Examples: "Cannot add duplicate key 'name'", "Cannot paste array into string value"

**Lenient Mode** (`:set validation_mode=lenient`):
- Allows temporary invalid states during edits
- Shows warnings in status line
- Blocks saving until JSON is valid
- Shows validation errors with `:validate` or on save attempt
- Useful for complex refactoring

### Error Display

**Modal Dialogs** (operation-blocking errors):
- Center screen overlay with border
- Error title and detailed message
- Suggested action if applicable
- `[Press any key to dismiss]`

**Status Line Messages** (warnings/info):
- Brief message, auto-clears after timeout
- Color-coded by level (red=error, yellow=warning, blue=info)

### Validation on Load

- Syntax errors on open: show location and message
- Enter read-only mode or offer to fix/abort
- JSONL: skip invalid lines with warning, mark in line list

## Command-Line Interface

### Usage

```bash
# Open existing file
jsonquill data.json

# Open JSONL file
jsonquill logs.jsonl
jsonquill --mode jsonl data.txt

# Create new file (starts with empty object {})
jsonquill newfile.json

# Read from stdin
cat data.json | jsonquill

# Start with empty document
jsonquill
```

### Options

```
jsonquill [OPTIONS] [FILE]

OPTIONS:
  -m, --mode <MODE>          Force file type: json or jsonl
  -t, --theme <THEME>        Start with specified theme
  -r, --readonly             Open in read-only mode
  -s, --strict               Force strict validation mode
  -l, --lenient              Force lenient validation mode
  -c, --config <PATH>        Use alternate config file
      --no-config            Ignore config file, use defaults
  -h, --help                 Show help message
  -v, --version              Show version information
```

### Exit Codes

- `0`: Success
- `1`: Error (file not found, parse error, etc.)
- `2`: Invalid command-line arguments
- `130`: Interrupted (Ctrl-C)

### File Type Detection

- `.json` → JSON mode
- `.jsonl`, `.ndjson`, `.json-lines` → JSONL mode
- No extension or stdin → attempt JSON parse, fall back to JSONL on failure
- `--mode` flag overrides auto-detection

## Technology Stack

- **Language**: Rust (stable)
- **TUI Framework**: ratatui
- **JSON Parsing**: serde_json with custom metadata layer
- **Terminal**: crossterm (via ratatui)
- **CLI Parsing**: clap (recommended)
- **Config**: toml (for config file parsing)

## Undo/Redo

- Simple linear history with `u` and `Ctrl-r`
- Each edit operation is separate undo step
- Configurable undo limit (default: 1000 operations)
- History cleared on file close

## Help System

- `?` or `:help`: show full-screen help overlay
- Help organized by category (navigation, editing, commands, search)
- Shows current keybindings
- `Esc` or `q` to close help
