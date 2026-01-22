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

**jeditor** is a terminal-based structural JSON editor built in Rust. It provides vim-style keybindings for navigating and editing JSON documents in a tree-like structure, making it easy to work with complex JSON files directly in the terminal.

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
- **crossterm (0.28)**: Cross-platform terminal manipulation library (backend for ratatui)
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
- **src/file/** - JSON file loading and saving (filesystem only, stdin piping not supported)
- **src/config/** - Configuration structures

## Current Status

**Working Features:**
- ✅ JSON file loading (filesystem paths only)
- ✅ Tree view rendering with expand/collapse and auto-expansion
- ✅ Array indices displayed as `[0]`, `[1]`, `[2]` when expanded
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
- ✅ Config file support (`~/.config/jeditor/config.toml`, `:set save` to persist)
- ✅ Yank operation (`yy` copies to clipboard including system clipboard)
- ✅ Delete operation (`dd` removes nodes from tree)
- ✅ Paste operation (`p` inserts yanked nodes after, `P` inserts before)
- ✅ Insert mode for editing values (strings, numbers, booleans, null)
- ✅ Viewport scrolling (automatically scrolls when navigating off-screen)
- ✅ Jump commands (`gg` for top, `G` for bottom)
- ✅ Page scrolling (`Ctrl-d` for half-page down, `Ctrl-u` for half-page up)
- ✅ Save and quit (`ZZ` saves if dirty then quits)
- ✅ Quit with dirty check (`q` warns if unsaved, matching `:q` behavior)
- ✅ Default dark theme (gray/black, not blue)
- ✅ Undo/redo (`u` to undo, `Ctrl-r` to redo, `:undo`, `:redo`)
- ✅ All tests passing

**Known Issues / TODO:**

**High Priority (Core Editing):**
- ❌ **No add operations** - `a` (add field/element), `o/O` (add sibling) not implemented
- ❌ **No rename operation** - `r` to rename object keys not implemented

**Navigation Enhancements:**
- ❌ **No sibling navigation** - `{/}` to jump to previous/next sibling not implemented
- ❌ **No previous search** - `N` for previous search match not implemented

**Advanced Features:**
- ❌ **No named registers** - `"ayy`, `"ap` for named register operations
- ❌ **No structural search** - `:find`, `:path` for JSONPath-style queries
- ❌ **Stdin piping not supported** - `cat file.json | jeditor` fails due to terminal I/O conflict
- ❌ **No JSONL support** - Line-based JSON editing not implemented
- ❌ **No format preservation** - Original formatting not preserved on save
- ❌ **No lazy loading** - Large files (≥100MB) not optimized
- ❌ **No advanced undo** - `g-`/`g+`, `:earlier`/`:later`, `:undolist` not implemented


## Usage

```bash
# Open a JSON file
./target/release/jeditor foo.json

# Navigation (NORMAL mode)
j/k         - Move down/up
h/l         - Collapse/expand node
gg          - Jump to top of document
G           - Jump to bottom of document
Ctrl-d      - Page down (half page)
Ctrl-u      - Page up (half page)
Arrow keys  - Also work for navigation

# Search
/           - Enter SEARCH mode
n           - Jump to next search result
Esc         - Exit SEARCH mode

# Modes
i           - Enter INSERT mode on current node
:           - Enter COMMAND mode
/           - Enter SEARCH mode
Esc         - Return to NORMAL mode
?           - Toggle help overlay
q           - Quit (warns if unsaved, use :q! to force)

# INSERT mode
<chars>     - Type to edit the value
Backspace   - Delete last character
Enter       - Commit changes and return to NORMAL mode
Esc         - Cancel editing and return to NORMAL mode

# Commands (in COMMAND mode)
:w          - Save file
:q          - Quit (warns if unsaved)
:q!         - Force quit without saving
:wq / :x    - Save and quit
:theme      - List available themes
:theme <name> - Switch to theme
:set          - Show current settings
:set number   - Enable line numbers
:set nonumber - Disable line numbers
:set save     - Save settings to config file
:undo         - Undo last change
:redo         - Redo last undone change

# Editing (NORMAL mode)
yy          - Yank (copy) current node to clipboard
dd          - Delete current node (removes from tree)
p           - Paste clipboard content after current node
P           - Paste clipboard content before current node
u           - Undo last change
Ctrl-r      - Redo last undone change

# Help
j/k or ↑/↓  - Scroll help when open
? or Esc    - Close help
```

## Configuration

jeditor supports a configuration file at `~/.config/jeditor/config.toml`.

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

Settings are loaded automatically when jeditor starts:
1. Default values are used as a baseline
2. Config file values override defaults (if the file exists)
3. Command-line arguments override config file values
```
