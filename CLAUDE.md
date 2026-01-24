# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ⚠️ IMPORTANT: Global Instructions

**ALWAYS read and follow the global instructions at `~/.claude/instructions.md` first.**

Key requirements from global instructions:
- **Testing:** Include unit tests for new functions, aim for >80% code coverage on business logic
- **Git Workflow:** NEVER offer to commit changes until user explicitly requests it (e.g., "commit", "commit this")
- **Documentation:** ALWAYS update README.md, CLAUDE.md, etc. before committing
- **Security:** Never commit secrets, API keys, or credentials
- **Code Quality:** Descriptive variable names, error handling, input validation

These global standards apply to ALL projects and override defaults when they conflict.

## Project Overview

**jsonquill** is a terminal-based structural JSON editor built in Rust. It provides vim-style keybindings for navigating and editing JSON documents in a tree-like structure, making it easy to work with complex JSON files directly in the terminal.

## Development Commands

Standard Rust/Cargo commands:

```bash
# Build the project
cargo build

# Run the binary
cargo run

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Build optimized release binary
cargo build --release
```

## Architecture

The project follows a standard Rust binary + library structure:

- **src/main.rs**: Entry point for the terminal application
- **src/lib.rs**: Library code that can be imported by other modules
- **Binary structure**: Currently minimal, will expand with TUI implementation

### Key Dependencies

- **ratatui (0.29)**: Terminal UI framework for building the interface
- **termion (4.0)**: Terminal manipulation library (backend for ratatui) with native /dev/tty support
- **serde (1.0)** + **serde_json (1.0)**: JSON serialization/deserialization
- **clap (4.5)**: Command-line argument parsing
- **toml (0.8)**: Configuration file support
- **anyhow (1.0)**: Error handling utilities
- **arboard (3.4)**: Clipboard support for copy/paste operations

### Current Module Structure

Implemented modules:
- **src/document/** - JSON parsing, tree representation, and node structures
- **src/editor/** - Editor state, cursor, and mode management
- **src/input/** - Keyboard event handling and key mapping
- **src/ui/** - Terminal UI rendering (tree view, status line, layout)
- **src/theme/** - Color themes and theming system
- **src/file/** - JSON file loading and saving (filesystem and stdin piping supported)
- **src/config/** - Configuration structures

**JSONL Handling:**
- `JsonValue::JsonlRoot` variant distinguishes JSONL from regular arrays
- Flat rendering in tree view (no root container)
- Separate save logic (one JSON object per line)
- Lines stored as `Vec<JsonNode>` in JsonlRoot variant
- Collapsed previews show inline content for all containers

## Current Status

**Working Features:**
- ✅ JSON file loading (filesystem paths and stdin piping supported)
- ✅ Tree view rendering with expand/collapse and auto-expansion
- ✅ Array indices displayed as `[0]`, `[1]`, `[2]` when expanded
- ✅ Line numbers (enabled by default, toggle with `:set number`/`:set nonumber`)
- ✅ Navigation (j/k/h/l, arrow keys) with count prefix support (e.g., `3j`, `5k`)
- ✅ Mode switching (i for INSERT, : for COMMAND, / for SEARCH, Esc to NORMAL)
- ✅ Status line showing current mode, filename, and cursor position (row,col row/total)
- ✅ Command mode with visible prompt and input buffer
- ✅ Command execution (`:w`, `:q`, `:q!`, `:wq`, `:x`)
- ✅ Save functionality (`:w` writes changes to disk atomically)
- ✅ Message area for errors, warnings, and info messages
- ✅ Help system (press `?` for scrollable help overlay)
- ✅ Search functionality (`/` to search, `n` for next result)
- ✅ Theme system (`:theme` to list, `:theme <name>` to switch)
- ✅ Settings system (`:set` to view, `:set <option>` to change)
- ✅ Config file support (`~/.config/jsonquill/config.toml`, `:set save` to persist)
- ✅ Yank operation (`yy` copies to clipboard including system clipboard)
- ✅ Delete operation (`dd` removes nodes from tree)
- ✅ Paste operation (`p` inserts yanked nodes after, `P` inserts before)
- ✅ Insert mode for editing values (strings, numbers, booleans, null)
- ✅ Viewport scrolling (automatically scrolls when navigating off-screen)
- ✅ Jump commands (`gg` for top, `G` for bottom, `<count>g` for specific line)
- ✅ Page scrolling (`Ctrl-d` for half-page down, `Ctrl-u` for half-page up)
- ✅ Save and quit (`ZZ` saves if dirty then quits)
- ✅ Quit with dirty check (`q` warns if unsaved, matching `:q` behavior)
- ✅ Default dark theme (gray/black, not blue)
- ✅ Undo/redo (`u` to undo, `Ctrl-r` to redo, `:undo`, `:redo`)
- ✅ Add scalar values (`a` to add after current node)
- ✅ Add object/array containers (`o` to add object, `A` to add array)
- ✅ Rename object keys (`r` to rename key)
- ✅ JSONL (.jsonl, .ndjson) file support
- ✅ Collapsed object/array previews (jless-style)
- ✅ All tests passing

**Known Issues / TODO:**

**Navigation Enhancements:**
- ❌ **No sibling navigation** - `{/}` to jump to previous/next sibling not implemented
- ❌ **No previous search** - `N` for previous search match not implemented

**Advanced Features:**
- ❌ **No named registers** - `"ayy`, `"ap` for named register operations
- ❌ **No structural search** - `:find`, `:path` for JSONPath-style queries
- ❌ **No format preservation** - Original formatting not preserved on save
- ❌ **No lazy loading** - Large files (≥100MB) not optimized
- ❌ **No advanced undo** - `g-`/`g+`, `:earlier`/`:later`, `:undolist` not implemented


## Usage

```bash
# Open a JSON file
./target/release/jsonquill foo.json

# Pipe JSON from stdin (requires /dev/tty for keyboard input)
cat foo.json | ./target/release/jsonquill
echo '{"key": "value"}' | ./target/release/jsonquill
curl https://api.example.com/data | ./target/release/jsonquill

# Start with empty document (interactive mode)
./target/release/jsonquill

# Navigation (NORMAL mode)
Movement commands can be prefixed with a count (e.g., `3j` to move down 3 lines, `5k` to move up 5 lines).

j/k         - Move down/up
h/l         - Collapse/expand node
gg          - Jump to top of document
G           - Jump to bottom of document
<count>g    - Jump to specific line number (e.g., `10g` goes to line 10)
Ctrl-d      - Page down (half page)
Ctrl-u      - Page up (half page)
Arrow keys  - Also work for navigation

# Search
/           - Enter SEARCH mode
n           - Jump to next search result
Esc         - Exit SEARCH mode

# Modes
e           - Enter INSERT mode (edit current value)
:           - Enter COMMAND mode
/           - Enter SEARCH mode
Esc         - Return to NORMAL mode
?           - Toggle help overlay
q           - Quit (warns if unsaved, use :q! to force)

# INSERT mode
When you press `i` to edit a value, the current value is pre-populated in the edit buffer
with the cursor positioned at the end. A blinking block cursor highlights the character at
the insertion point (or shows a space if at the end of the buffer).

Editing:
<chars>     - Insert character at cursor position
Backspace   - Delete character before cursor
Ctrl-d      - Delete character at cursor
Ctrl-k      - Delete from cursor to end of buffer

Navigation:
Left/Right  - Move cursor within the edit buffer
Ctrl-a      - Jump to beginning of buffer
Ctrl-e      - Jump to end of buffer

Commit/Cancel:
Enter       - Commit changes and return to NORMAL mode
Esc         - Cancel editing and return to NORMAL mode

# Commands (in COMMAND mode)
:w          - Save file
:w <file>   - Save to new file and update current filename
:q          - Quit (warns if unsaved)
:q!         - Force quit without saving
:wq / :x    - Save and quit
:wq <file>  - Save to new file and quit
:theme      - List available themes
:theme <name> - Switch to theme
:set          - Show current settings
:set number   - Enable line numbers
:set nonumber - Disable line numbers
:set save     - Save settings to config file
:undo         - Undo last change
:redo         - Redo last undone change

# Editing (NORMAL mode)
Commands can be prefixed with a count (e.g., `3dd` to delete 3 nodes, `5yy` to yank 5 nodes).

i           - Insert/add scalar value (context-sensitive)
            - On a container (object/array): adds first child inside the container
            - On a scalar: adds sibling after it
            - Arrays: immediately enter Insert mode to type value
            - Objects: prompt for key, then enter Insert mode for value
            - Values are parsed: true/false → boolean, null → null, numbers → number, else → string
a           - Add empty array [] after current node
            - Arrays: adds directly
            - Objects: prompts for key first
o           - Add empty object {} after current node
            - Arrays: adds directly
            - Objects: prompts for key first
r           - Rename object key (only works on object keys, not array elements)
            - Pre-populates with current key name
            - Enter to commit, Esc to cancel
yy          - Yank (copy) current node to clipboard
dd          - Delete current node (removes from tree)
p           - Paste clipboard content after current node
P           - Paste clipboard content before current node
u           - Undo last change
Ctrl-r      - Redo last undone change

Count Prefix:
1-9         - Start accumulating a count
0-9         - Continue accumulating count (after first digit)
<count>j/k  - Move down/up <count> lines (e.g., 3j moves down 3 lines)
<count>h/l  - Collapse/expand <count> times
<count>g    - Jump to line <count> (e.g., 10g jumps to line 10)
<count>dd   - Delete <count> nodes (e.g., 3dd deletes 3 nodes)
<count>yy   - Yank <count> nodes (e.g., 5yy yanks 5 nodes)

# Help
j/k or ↑/↓  - Scroll help when open
? or Esc    - Close help
```

## Configuration

jsonquill supports a configuration file at `~/.config/jsonquill/config.toml`.

### Config File Format

```toml
# Theme name (default: "default-dark")
theme = "default-dark"

# Number of spaces per indentation level (default: 2)
indent_size = 2

# Display line numbers (default: true)
show_line_numbers = true

# Automatically save on changes (default: false)
auto_save = false

# JSON validation strictness: "strict", "permissive", or "none" (default: "strict")
validation_mode = "strict"

# Create .bak files before saving (default: false)
create_backup = false

# Maximum number of undo operations (default: 50)
undo_limit = 50

# Sync unnamed register with system clipboard (default: true)
sync_unnamed_register = true

# File size in bytes to trigger lazy loading (default: 104857600 = 100MB)
lazy_load_threshold = 104857600
```

### Saving Settings

Use `:set save` to persist your current settings to the config file. This will save:
- Current theme
- Line number setting
- Other default values

The config file is created automatically when you run `:set save` for the first time.

### Loading Settings

Settings are loaded automatically when jsonquill starts:
1. Default values are used as a baseline
2. Config file values override defaults (if the file exists)
3. Command-line arguments override config file values

## Stdin Piping

jsonquill supports reading JSON data from stdin while maintaining full keyboard interactivity. This is accomplished using `/dev/tty` for keyboard input:

**How it works:**
1. When stdin is piped (not a terminal), jsonquill detects this automatically
2. JSON data is read from stdin before setting up the terminal UI
3. The input handler opens `/dev/tty` for keyboard events
4. termion reads keyboard input from the controlling terminal (`/dev/tty`)
5. The TUI remains fully interactive even though stdin was consumed for data

**Requirements:**
- A controlling terminal must be available (`/dev/tty` must be accessible)
- Works in interactive terminal sessions
- Will fail gracefully in non-interactive environments (CI/CD, detached sessions)

**Examples:**
```bash
# Read JSON from curl
curl https://api.github.com/users/octocat | jsonquill

# Read from file via cat
cat config.json | jsonquill

# Read from echo
echo '{"test": [1,2,3]}' | jsonquill
```

