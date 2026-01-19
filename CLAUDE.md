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
- ✅ Tree view rendering with expand/collapse
- ✅ Basic navigation (j/k/h/l, arrow keys)
- ✅ Mode switching (i for INSERT, : for COMMAND, Esc to NORMAL)
- ✅ Status line showing current mode and filename
- ✅ All 259 tests passing

**Known Issues / TODO:**
- ❌ **Command mode has no visible prompt** - Pressing `:` switches to COMMAND mode but shows no `:` prompt at the bottom
- ❌ **Command input buffer not implemented** - Cannot type or execute commands (`:w`, `:q`, etc.)
- ❌ **Help system missing** - `?` key not mapped, no help overlay implemented
- ❌ **Insert mode not functional** - Pressing `i` switches mode but cannot edit values
- ❌ **No editing operations** - Delete, yank, paste, rename not implemented
- ❌ **Stdin piping not supported** - `cat file.json | jeditor` fails due to terminal I/O conflict
- ❌ **No save functionality** - Cannot write changes back to disk
- ❌ **Message area empty** - Third UI line reserved but not used for messages/errors

## Usage

```bash
# Open a JSON file
./target/release/jeditor foo.json

# Basic navigation
j/k         - Move down/up
h/l         - Collapse/expand node (or move left/right)
Arrow keys  - Also work for navigation

# Mode switching (partially implemented)
i           - Enter INSERT mode (but editing not implemented yet)
:           - Enter COMMAND mode (but no prompt/commands yet)
Esc         - Return to NORMAL mode
q           - Quit (only works in NORMAL mode)
```
