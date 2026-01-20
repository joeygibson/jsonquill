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
- ✅ Yank operation (`y` copies to clipboard including system clipboard)
- ✅ Default dark theme (gray/black, not blue)
- ✅ All 74 tests passing

**Known Issues / TODO:**
- ❌ **Insert mode not functional** - Pressing `i` switches mode but cannot edit values
- ❌ **Delete/paste not implemented** - `d` and `p` keys show placeholder messages
- ❌ **Stdin piping not supported** - `cat file.json | jeditor` fails due to terminal I/O conflict
- ❌ **No rename operation** - Cannot rename object keys


## Usage

```bash
# Open a JSON file
./target/release/jeditor foo.json

# Navigation (NORMAL mode)
j/k         - Move down/up
h/l         - Collapse/expand node
Arrow keys  - Also work for navigation

# Search
/           - Enter SEARCH mode
n           - Jump to next search result
Esc         - Exit SEARCH mode

# Modes
i           - Enter INSERT mode (not yet functional)
:           - Enter COMMAND mode
Esc         - Return to NORMAL mode
?           - Toggle help overlay
q           - Quit (NORMAL mode only)

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

# Editing (NORMAL mode)
y           - Yank (copy) current node to clipboard
d           - Delete current node (not yet implemented)
p           - Paste from clipboard (not yet implemented)

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

# Maximum number of undo operations (default: 1000)
undo_limit = 1000

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
