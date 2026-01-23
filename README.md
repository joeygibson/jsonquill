# JSON Quill

A terminal-based structural JSON editor with vim-style keybindings.

## Status

**Alpha Release** - Core functionality is implemented and usable. The editor supports:
- JSON file loading and editing
- Tree-based navigation with vim keybindings
- Add, edit, delete operations for JSON values
- Undo/redo functionality
- Clipboard operations (yank/paste)
- Customizable themes and settings
- Search functionality
- Configuration file support

See [CLAUDE.md](CLAUDE.md) for detailed feature list and developer documentation.

## Description

JSON Quill is a Rust-based terminal application for viewing and editing JSON files in a structured, tree-like format. It provides an intuitive vim-style interface for navigating and manipulating complex JSON documents directly in the terminal.

## Tech Stack

- **Rust**: Core language
- **ratatui**: Terminal UI framework
- **termion**: Terminal manipulation with /dev/tty support
- **serde_json**: JSON parsing and serialization
- **clap**: Command-line argument parsing
- **arboard**: Clipboard integration

## Installation & Usage

### Build from Source

```bash
# Clone the repository
git clone https://github.com/joeygibson/jsonquill
cd jsonquill

# Build release binary
cargo build --release

# Run with a file
./target/release/jsonquill examples/sample.json
```

### Basic Usage

```bash
# Open a JSON file
jsonquill file.json

# Create a new empty JSON file
jsonquill

# Specify theme
jsonquill --theme default-light file.json

# Pipe JSON from stdin
cat file.json | jsonquill
echo '{"name": "example", "count": 42}' | jsonquill

# Fetch and edit JSON from an API
curl https://api.example.com/data | jsonquill
curl -s https://jsonplaceholder.typicode.com/users/1 | jsonquill
```

### JSONL Support

JSON Quill supports JSONL (JSON Lines) files with the `.jsonl` or `.ndjson` extension:

```bash
# Open a JSONL file
jsonquill data.jsonl

# Each line displays as a collapsed object
# Press l or → to expand a line
# Edit fields within expanded lines normally
```

**JSONL Features:**
- Each line parsed as separate JSON object
- Lines start collapsed showing preview
- Flat display (no nesting at root level)
- Save preserves line-by-line format
- All edit operations work within lines

## Key Bindings

### Navigation

| Key | Action | Notes |
|-----|--------|-------|
| `j` / `k` | Move down / up | Supports count prefix (e.g., `3j` moves down 3 lines) |
| `↓` / `↑` | Move down / up | Arrow keys also work |
| `h` / `l` | Collapse / expand node | Toggle node expansion state |
| `←` / `→` | Collapse / expand node | Arrow keys also work |
| `gg` | Jump to top of document | |
| `G` | Jump to bottom of document | |
| `<count>g` | Jump to line number | e.g., `10g` jumps to line 10 |
| `Ctrl-d` | Page down | Scroll half page down |
| `Ctrl-u` | Page up | Scroll half page up |

### Modes

| Key | Action | Description |
|-----|--------|-------------|
| `i` | Enter INSERT mode | Edit value of current node |
| `:` | Enter COMMAND mode | Execute commands (`:w`, `:q`, etc.) |
| `/` | Enter SEARCH mode | Search in keys and values |
| `Esc` | Return to NORMAL mode | Exit INSERT, COMMAND, or SEARCH mode |

### Editing (NORMAL mode)

| Key | Action | Notes |
|-----|--------|-------|
| `a` | Add new field/element | Objects: prompts for key then value<br>Arrays: prompts for value directly |
| `dd` | Delete current node | Supports count prefix (e.g., `3dd` deletes 3 nodes) |
| `yy` | Yank (copy) current node | Supports count prefix (e.g., `2yy` copies 2 nodes)<br>Copies to system clipboard |
| `p` | Paste after cursor | Insert yanked content after current node |
| `P` | Paste before cursor | Insert yanked content before current node |
| `u` | Undo last change | |
| `Ctrl-r` | Redo last undone change | |
| `ZZ` | Save and quit | Only saves if file has been modified |

### INSERT Mode

| Key | Action |
|-----|--------|
| `<chars>` | Type to edit the value |
| `Backspace` | Delete last character |
| `←` / `→` | Move cursor left/right |
| `Home` / `End` | Move to start/end of line |
| `Ctrl-u` | Delete to start of line |
| `Delete` | Delete character under cursor |
| `Enter` | Commit changes and return to NORMAL mode |
| `Esc` | Cancel editing and return to NORMAL mode |

### Search

| Key | Action | Description |
|-----|--------|-------------|
| `/` | Start search | Enter SEARCH mode to type search query |
| `n` | Jump to next match | Find next occurrence of search term |
| `Esc` | Exit search mode | Return to NORMAL mode |

### Commands (COMMAND mode)

Type `:` to enter command mode, then:

| Command | Action | Notes |
|---------|--------|-------|
| `:w` | Save file | Write changes to disk |
| `:w <filename>` | Save as | Write to a different file |
| `:q` | Quit | Warns if there are unsaved changes |
| `:q!` | Force quit | Quit without saving changes |
| `:wq` | Save and quit | Also: `:x` or `ZZ` |
| `:undo` | Undo last change | Same as `u` in NORMAL mode |
| `:redo` | Redo last undone change | Same as `Ctrl-r` in NORMAL mode |
| `:theme` | List available themes | Shows all built-in themes |
| `:theme <name>` | Switch theme | e.g., `:theme default-light` |
| `:set` | Show current settings | Display all configuration values |
| `:set number` | Enable line numbers | Show line numbers in tree view |
| `:set nonumber` | Disable line numbers | Hide line numbers |
| `:set save` | Save settings to config | Write current settings to `~/.config/jsonquill/config.toml` |

### Other

| Key | Action | Notes |
|-----|--------|-------|
| `q` | Quit | Only works in NORMAL mode (same as `:q`) |
| `?` | Toggle help overlay | Shows all keybindings |
| `↑` / `↓` | Scroll help | When help overlay is open |
| `j` / `k` | Scroll help | When help overlay is open |

## Value Parsing

When adding or editing values, JSON Quill automatically detects the type:

- `true` / `false` → Boolean
- `null` → Null
- `42` / `3.14` / `-1.5` → Number
- Anything else → String

Examples:
- Type `hello` → Stored as string `"hello"`
- Type `42` → Stored as number `42`
- Type `true` → Stored as boolean `true`

## Configuration

JSON Quill supports a configuration file at `~/.config/jsonquill/config.toml`.

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

Use `:set save` to persist your current settings to the config file.

## Development Setup

### Prerequisites

- Rust toolchain (1.70+)
- Cargo package manager

### Building

```bash
# Clone the repository
git clone https://github.com/joeygibson/jsonquill
cd jsonquill

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run -- examples/sample.json

# Build release binary
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_add_field_to_object

# Run with output
cargo test -- --nocapture
```

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

### Third-Party Licenses

This project depends on several open source libraries, all of which are MIT-compatible. For a complete list of dependencies and their licenses, see [THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md).

All direct dependencies are either:
- MIT licensed, or
- Dual-licensed under Apache-2.0 OR MIT (used under MIT terms)

### Contributing

By contributing to this project, you agree that your contributions will be licensed under the same MIT License.
