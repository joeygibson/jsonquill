# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ⚠️ IMPORTANT: Global Instructions

**ALWAYS read and follow the global instructions at `~/.claude/instructions.md` first.**

The global instructions include:
- **Rust-Specific Guidelines:** Standard cargo commands, pre-commit checklist (fmt/clippy/test), project structure
- **Testing Requirements:** Unit tests for new functions, >80% coverage on business logic
- **Git Workflow:** Wait for explicit commit request, update docs before committing
- **Security & Code Quality:** No secrets in commits, error handling, descriptive names

These global standards apply to ALL projects and override defaults when they conflict.

## Project Overview

**jsonquill** is a terminal-based structural JSON editor built in Rust. It provides vim-style keybindings for navigating and editing JSON documents in a tree-like structure, making it easy to work with complex JSON files directly in the terminal.

## Development Commands

See `~/.claude/instructions.md` for standard Rust/Cargo commands (build, run, test, fmt, clippy).

## Pre-Commit Checklist

**CRITICAL: ALWAYS run these checks before committing. Never commit without passing all checks.**

### 1. Code Quality (REQUIRED)
```bash
# Format code (REQUIRED - will fail CI if not run)
cargo fmt

# Verify formatting is correct
cargo fmt --check

# Run clippy with warnings as errors (REQUIRED - will fail CI)
cargo clippy -- -D warnings

# Run all tests
cargo test
```

### 2. Project-Specific Requirements
- **Update help screen** (`src/ui/help_overlay.rs`) when adding user-facing features
  - New commands, keybindings, or major features visible to users
  - Help screen should match README.md and CLAUDE.md documentation

### 3. Pre-Commit Command Summary
**Run this before EVERY commit:**
```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

If any of these fail, fix the issues before committing. Do not commit code that doesn't pass all three checks.

## Version Management

Use the `scripts/ver` script to manage version updates and create releases:

```bash
# Print current version
./scripts/ver

# Update to new version (validates format, updates files, commits, tags)
./scripts/ver 0.3.0
```

The script will:
1. Validate version format (X.Y.Z)
2. Update Cargo.toml
3. Generate release notes from git commits (conventional commit format)
4. Update CHANGELOG.md with new version entry and auto-generated notes
5. Prompt to review/edit CHANGELOG.md (optional)
6. Run cargo build to update Cargo.lock
7. Commit changes with message: `chore: bump version to X.Y.Z`
8. Create git tag: `vX.Y.Z`
9. Print push command (does not auto-push for safety)

**Release Notes Generation:**
The script automatically parses git commits since the last version using conventional commit format:
- `feat:` → Added section
- `fix:` → Fixed section
- `docs:`, `refactor:`, `perf:` → Changed section
- `chore:` commits are excluded (version bumps, formatting)
- Other commits → Other section

**Example output:**
```
Updating version: 0.2.1 → 0.3.0
Updated Cargo.toml: 0.2.1 → 0.3.0
Updated CHANGELOG.md with v0.3.0 entry
Generated release notes from git commits

Review/edit the generated CHANGELOG.md? [y/N]
Running cargo build...
Committing version bump...
Creating git tag: v0.3.0

Version updated successfully!
To push changes and tag, run:
git push && git push origin v0.3.0
```

### Regenerating Release Notes

Use the `scripts/release-notes` script to generate GitHub release notes independently from the version bump workflow:

```bash
# Generate release notes (auto-detects current version from Cargo.toml)
./scripts/release-notes 0.7.0

# Generate release notes for specific version range
./scripts/release-notes 0.7.0 0.5.0

# Save to file
./scripts/release-notes 0.7.0 > release-notes-v0.7.0.md
```

This is useful for:
- Previewing release notes before running `ver`
- Regenerating notes after editing commit messages
- Creating notes for custom version ranges

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
- ✅ Relative line numbers (vim-style, toggle with `:set relativenumber`/`:set norelativenumber`)
- ✅ Navigation (j/k/h/l, arrow keys) with count prefix support (e.g., `3j`, `5k`)
- ✅ Mode switching (i for INSERT, : for COMMAND, / for SEARCH, Esc to NORMAL)
- ✅ Status line showing current mode, filename, current path, and cursor position
  - Shows current JSON path in dot notation (e.g., `users[0].name`, `config.database.port`) highlighted in the theme's key color
  - Path displayed after filename (before dirty indicator and search results)
  - Root node shows no path
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
- ✅ Jump commands (`gg`/`Home` for top, `G`/`End` for bottom, `<count>G`/`<count>gg` for specific line)
- ✅ Page scrolling (`Ctrl-d`/`Ctrl-u` for half-page, `Ctrl-f`/`Ctrl-b`/`PgDn`/`PgUp` for full-page)
- ✅ Save and quit (`ZZ` saves if dirty then quits)
- ✅ Quit with dirty check (`q` warns if unsaved, matching `:q` behavior)
- ✅ Default dark theme (gray/black, not blue)
- ✅ Undo/redo (`u` to undo, `Ctrl-r` to redo, `:undo`, `:redo`)
- ✅ Add scalar values (`a` to add after current node)
- ✅ Add object/array containers (`o` to add object, `A` to add array)
- ✅ Rename object keys (`r` to rename key)
- ✅ JSONL (.jsonl, .ndjson) file support
- ✅ Collapsed object/array previews (jless-style)
- ✅ Mouse/trackpad scrolling (3 lines per tick, toggle with `:set mouse`/`:set nomouse`)
- ✅ Named registers (a-z) with append mode (A-Z) and history (0-9)
- ✅ Format preservation (whitespace, indentation for unmodified nodes)
- ✅ Visual mode (`v` to select multiple nodes, `d`/`y`/`p` to operate on selection)
- ✅ Marks (`m{a-z}` to set, `'{a-z}` to jump)
- ✅ Jump list (`Ctrl-o`/`Ctrl-i` to navigate through jump history)
- ✅ Repeat command (`.` to repeat last edit operation)
- ✅ All tests passing

**Known Issues / TODO:**

**Navigation Enhancements:**
- ❌ **No previous search** - `N` for previous search match not implemented (note: `/` and `?` already provide forward/backward search with `n`)

**Advanced Features:**
- ✅ **Structural search** - `:path`, `:jp` for JSONPath-style queries
- ✅ **Format preservation** - Unmodified nodes retain original formatting (whitespace, indentation)
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
E           - Fully expand current node and all descendants
C           - Fully collapse current node and all descendants
H           - Move to parent node (without collapsing)
gg / Home   - Jump to top of document
G / End     - Jump to bottom of document
<count>G or <count>gg - Jump to specific line number (e.g., `10G` or `10gg` goes to line 10)
Ctrl-d      - Half-page down
Ctrl-u      - Half-page up
Ctrl-f/PgDn - Full-page down
Ctrl-b/PgUp - Full-page up
zz          - Center cursor on screen
zt          - Move cursor to top of screen
zb          - Move cursor to bottom of screen
}           - Jump to next sibling (same parent, next index)
{           - Jump to previous sibling (same parent, previous index)
0 or ^      - Jump to first sibling (same parent, index 0)
$           - Jump to last sibling (same parent, last index)
w           - Move to next node at same or shallower depth (skip deep nesting)
b           - Move to previous node at same or shallower depth (skip deep nesting)
Arrow keys  - Also work for navigation

# Mouse
Scroll wheel/trackpad - Scroll viewport (3 lines per tick)
                      - Also scrolls help overlay when help is open
                      - Toggle with :set mouse / :set nomouse

# Search
/           - Start forward search (searches down through document)
            - Uses smart case: case-insensitive unless pattern has uppercase letters
            - Shows match counter (e.g., "Match 2/5")
            - Shows "W" prefix when wrapping around (e.g., "W Match 1/5")
?           - Start backward search (searches up through document)
n           - Jump to next match in search direction
*           - Search forward for current object key
#           - Search backward for current object key
Esc         - Exit SEARCH mode

Note: Search results info disappears from the status bar when you press any key
other than `n` (next match). This keeps the status bar clean once you're done
navigating search results.

# JSONPath Search (structural search)
:path $.store.book[*].author  - Find all book authors
:jp $..price                  - Find all price fields anywhere
:path $.items[0:3]            - First 3 items
:path $.user['name','email']  - Multiple properties
n                             - Navigate to next match

# Modes
e           - Enter INSERT mode (edit current value)
:           - Enter COMMAND mode
/ or ?      - Enter SEARCH mode (forward / backward)
Esc         - Return to NORMAL mode
F1 / :help  - Toggle help overlay
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
Tab         - Autocomplete theme names (:theme <Tab>) and settings (:set <Tab>)
            - Press Tab multiple times to cycle through completions
:w          - Save file
:w <file>   - Save to new file and update current filename
:q          - Quit (warns if unsaved)
:q!         - Force quit without saving
:wq / :x    - Save and quit
:wq <file>  - Save to new file and quit
:e <file>   - Load a different file (warns if dirty)
:e!         - Reload current file from disk, discarding changes
:e! <file>  - Load a different file, discarding changes
:theme      - List available themes
:theme <name> - Switch to theme
:set          - Show current settings
:set number   - Enable line numbers
:set nonumber - Disable line numbers
:set relativenumber (or :set rnu) - Enable relative line numbers
:set norelativenumber (or :set nornu) - Disable relative line numbers
:set mouse    - Enable mouse scrolling
:set nomouse  - Disable mouse scrolling
:set create_backup - Enable backup file creation (.bak)
:set nocreate_backup - Disable backup file creation
:set save     - Save settings to config file
:undo         - Undo last change
:redo         - Redo last undone change
:format       - Reformat entire document with jq-style indentation
:help         - Show help overlay
:path <query> - Search using JSONPath query (e.g., :path $.store.book[*].author)
:jp <query>   - Short alias for :path
:find <query> - Execute text search (e.g., :find price)
:find         - Enter text search mode (same as /)

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

Registers:
"a          - Select register 'a' for next operation (a-z)
"A          - Select register 'a' in append mode (A-Z)
"0-"9       - Select numbered register (0=last yank, 1-9=delete history)
"ayy        - Yank to register 'a'
"ap / "aP   - Paste from register 'a'
"add        - Delete to register 'a'
yy / dd     - Use unnamed register (syncs with system clipboard by default)

Yank/Delete/Paste:
yy          - Yank (copy) current node to unnamed register
yp          - Yank path in dot notation (.foo[3].bar)
yb          - Yank path in bracket notation (["foo"][3]["bar"])
yq          - Yank path in jq style
dd          - Delete current node (removes from tree)
p           - Paste register content after current node
P           - Paste register content before current node

Undo/Redo:
u           - Undo last change
Ctrl-r      - Redo last undone change

Visual Mode:
v           - Enter visual mode (select multiple nodes)
j/k/h/l     - Expand/shrink selection (in visual mode)
d           - Delete selection (in visual mode)
y           - Yank (copy) selection (in visual mode)
p/P         - Replace selection with clipboard (in visual mode)
Esc         - Exit visual mode

Marks & Jump List:
m{a-z}      - Set mark at current position
'{a-z}      - Jump to mark
y'{a-z}     - Yank from cursor to mark (motion-to-mark)
d'{a-z}     - Delete from cursor to mark (motion-to-mark)
Ctrl-o      - Jump backward in jump list
Ctrl-i      - Jump forward in jump list
            - Jump list records: gg, G, line jumps, search, marks

Repeat Command:
.           - Repeat last edit (dd, yy, p, P)

Count Prefix:
1-9         - Start accumulating a count
0-9         - Continue accumulating count (after first digit)
<count>j/k  - Move down/up <count> lines (e.g., 3j moves down 3 lines)
<count>h/l  - Collapse/expand <count> times
<count>g    - Jump to line <count> (e.g., 10g jumps to line 10)
<count>dd   - Delete <count> nodes (e.g., 3dd deletes 3 nodes)
<count>yy   - Yank <count> nodes (e.g., 5yy yanks 5 nodes)

# Help
j/k or ↑/↓ or mouse wheel - Scroll help when open
? or Esc                  - Close help
```

## Themes

jsonquill includes multiple built-in color themes:

- `default-dark` - Dark theme optimized for low-light environments (default)
- `default-light` - Light theme for well-lit environments
- `gruvbox-dark` - Retro groove color scheme with warm, earthy tones
- `nord` - Arctic, north-bluish color palette
- `dracula` - Dark theme with vibrant purples and pinks
- `solarized-dark` - Precision color scheme for machines and people
- `monokai` - Popular color scheme inspired by Monokai Pro
- `one-dark` - The default dark theme from Atom editor

**Usage:**
```bash
:theme              # List all available themes
:theme nord         # Switch to Nord theme
:theme dracula      # Switch to Dracula theme
```

Themes can also be set in the configuration file (see Configuration section below).

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

# Create backup files (e.g., file.json.bak) before saving (default: false)
create_backup = false

# Maximum number of undo operations (default: 50)
undo_limit = 50

# Sync unnamed register with system clipboard (default: true)
sync_unnamed_register = true

# Enable mouse/trackpad scrolling support (default: true)
enable_mouse = true

# File size in bytes to trigger lazy loading (default: 104857600 = 100MB)
lazy_load_threshold = 104857600
```

### Saving Settings

Use `:set save` to persist your current settings to the config file. This will save:
- Current theme
- Line number setting
- Mouse setting
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

## Gzip Compressed Files

jsonquill transparently handles gzip-compressed JSON files (`.json.gz` and `.jsonl.gz`):

**How it works:**
1. Files with `.gz` extension are automatically detected and decompressed on load
2. Editing works identically to uncompressed files
3. On save (`:w`), files are re-compressed with gzip
4. Format detection uses the full extension (e.g., `file.json.gz` → JSON, `file.jsonl.gz` → JSONL)

**Requirements:**
- File must have `.gz` extension for auto-detection
- Underlying format must be `.json` or `.jsonl` (e.g., `data.json.gz`, `logs.jsonl.gz`)
- Gzip decompression happens in-memory before JSON parsing
- Save operations preserve gzip compression

**Examples:**
```bash
# Open compressed JSON file
jsonquill data.json.gz

# Open compressed JSONL file
jsonquill logs.jsonl.gz

# Pipe compressed data from stdin
curl https://api.example.com/data.json.gz | gunzip | jsonquill

# Edit and save (automatically re-compresses)
jsonquill config.json.gz
# ... make edits ...
:w  # Saves as config.json.gz (gzip compressed)

# Save compressed file to new location
:w backup.json.gz  # Saves with gzip compression
```

**Notes:**
- Compression level is set to 6 (balance between speed and compression ratio)
- Original file permissions are preserved when saving
- Gzip format is compatible with standard tools (`gzip`, `gunzip`, `zcat`)
- Large compressed files benefit from reduced memory usage during load

