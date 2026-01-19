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

### Planned Module Structure

Future modules will include:
- JSON parsing and tree representation
- TUI rendering and event handling
- Vim-style keybinding implementation
- Configuration management
