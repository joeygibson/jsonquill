<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/jsonquill-dark.png" />
    <source media="(prefers-color-scheme: light)" srcset="assets/jsonquill-light.png" />
    <img alt="jsonquill logo" src="assets/jsonquill-readme-light.png" width="600" />
  </picture>
</p>

# JSONQuill

A terminal-based structural JSON editor with vim-style keybindings.

## Status

[![CI](https://github.com/joeygibson/jsonquill/workflows/CI/badge.svg)](https://github.com/joeygibson/jsonquill/actions/workflows/ci.yml)
[![Release](https://github.com/joeygibson/jsonquill/workflows/Release/badge.svg)](https://github.com/joeygibson/jsonquill/actions/workflows/release.yml)

**Alpha Release** - Core functionality is implemented and usable. The editor supports:
- JSON file loading and editing (including JSONL format)
- Tree-based navigation with vim keybindings
- Advanced navigation (sibling jumping, screen positioning, count prefixes)
- Add, edit, delete operations for JSON values
- Undo/redo functionality with repeat command
- Clipboard operations (yank/paste with path copying, smart paste, named registers)
- Visual mode for bulk operations on multiple nodes
- Marks and jump list for quick navigation
- Motion-to-mark operations (yank/delete from cursor to mark)
- Text and JSONPath structural search (with smart case and key search)
- Customizable themes and settings
- Line numbers (absolute and relative)
- Configuration file support
- Mouse/trackpad scrolling support
- **Format Preservation**: Unmodified JSON nodes retain their exact original formatting (whitespace, indentation, newlines) when saved
- Gzip compression support (transparent `.json.gz` and `.jsonl.gz` handling)

See [CLAUDE.md](CLAUDE.md) for detailed feature list and developer documentation.

## Data Safety

- **JSON Validation Before Save**: Every file save re-parses the generated JSON to verify it's valid before writing to disk. If serialization produces invalid JSON (indicating a bug), the save fails with a clear error message instead of corrupting your data.
- **Atomic Writes**: Files are written to a temporary file first, then atomically renamed to the target path, ensuring your original file is never left in a partially written state.
- **Optional Backups**: Enable `create_backup: true` in your config to automatically create `.bak` files before saving (e.g., `file.json.bak`).
- **Format Preservation**: Unmodified nodes retain their exact original formatting, reducing the risk of unintended changes.

## Platform Support

- ✅ **Linux** (x86_64 glibc and musl)
- ✅ **macOS** (Intel and Apple Silicon)
- ❌ **Windows** (not currently supported)

## Description

JSONQuill is a Rust-based terminal application for viewing and editing JSON files in a structured, tree-like format. It provides an intuitive vim-style interface for navigating and manipulating complex JSON documents directly in the terminal.

## Tech Stack

- **Rust**: Core language
- **ratatui**: Terminal UI framework
- **termion**: Terminal manipulation with /dev/tty support
- **serde_json**: JSON parsing and serialization
- **clap**: Command-line argument parsing
- **arboard**: Clipboard integration

## Installation & Usage

### Homebrew (macOS)

```bash
# Add the tap
brew tap joeygibson/tools

# Install jsonquill
brew install jsonquill
```

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

# Open gzip-compressed JSON files (transparent decompression)
jsonquill data.json.gz
jsonquill logs.jsonl.gz

# Pipe compressed data (decompress first)
curl https://api.example.com/data.json.gz | gunzip | jsonquill
```

### JSONL Support

JSONQuill supports JSONL (JSON Lines) files with the `.jsonl` or `.ndjson` extension:

```bash
# Open a JSONL file
jsonquill data.jsonl

# Each line starts collapsed showing a preview
# Press l or → to expand a line and see its contents
# Edit fields within expanded lines normally
# Press h or ← to collapse a line back to preview
```

**JSONL Features:**
- Each line parsed as separate JSON object
- Lines start collapsed showing detailed preview (e.g., `(3) {id: 1, name: "Alice", ...}`)
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
| `E` | Fully expand subtree | Expands current node and all descendants |
| `C` | Fully collapse subtree | Collapses current node and all descendants |
| `H` | Move to parent | Navigate to parent node without collapsing |
| `gg` / `Home` | Jump to top of document | |
| `G` / `End` | Jump to bottom of document | |
| `<count>G` / `<count>gg` | Jump to line number | e.g., `10G` or `10gg` jumps to line 10 |
| `Ctrl-d` | Half-page down | Scroll half page down |
| `Ctrl-u` | Half-page up | Scroll half page up |
| `Ctrl-f` / `PgDn` | Full-page down | Scroll full page down |
| `Ctrl-b` / `PgUp` | Full-page up | Scroll full page up |
| `zz` | Center cursor on screen | Scroll viewport to center current line |
| `zt` | Move cursor to top of screen | Scroll viewport to place current line at top |
| `zb` | Move cursor to bottom of screen | Scroll viewport to place current line at bottom |
| `}` | Jump to next sibling | Move to the next node at the same level |
| `{` | Jump to previous sibling | Move to the previous node at the same level |
| `0` / `^` | Jump to first sibling | Move to first node at current level |
| `$` | Jump to last sibling | Move to last node at current level |
| `w` | Move to next at same/shallower depth | Skip over deep nested structures to next top-level node |
| `b` | Move to previous at same/shallower depth | Skip back over deep nested structures to previous top-level node |
| Scroll wheel / Trackpad | Scroll viewport | Scroll up/down 3 lines per tick (toggle with `:set mouse`/`:set nomouse`) |

### Modes

| Key | Action | Description |
|-----|--------|-------------|
| `:` | Enter COMMAND mode | Execute commands (`:w`, `:q`, etc.) |
| `/` / `?` | Enter SEARCH mode | Search forward/backward in keys and values |
| `Esc` | Return to NORMAL mode | Exit INSERT, COMMAND, or SEARCH mode |

### Editing (NORMAL mode)

| Key | Action | Notes |
|-----|--------|-------|
| `e` | Edit current value | Enters INSERT mode to edit the value at cursor |
| `i` | Add new scalar field/element | Objects: prompts for key then value<br>Arrays: prompts for value directly |
| `a` | Add empty array `[]` | Adds after current node |
| `o` | Add empty object `{}` | Adds after current node |
| `r` | Rename object key | Only works on object keys (not array elements) |
| `dd` | Delete current node | Supports count prefix (e.g., `3dd` deletes 3 nodes)<br>Deletes to unnamed register (syncs with system clipboard) |
| `yy` | Yank (copy) current node | Supports count prefix (e.g., `2yy` copies 2 nodes)<br>Copies to unnamed register (syncs with system clipboard) |
| `yp` | Yank path (dot notation) | Copy path like `.foo[3].bar` to clipboard |
| `yb` | Yank path (bracket notation) | Copy path like `["foo"][3]["bar"]` to clipboard |
| `yq` | Yank path (jq style) | Copy path in jq-style notation |
| `p` | Paste after cursor | Insert yanked content after current node<br>**Smart paste**: expanded containers paste inside, collapsed containers paste as sibling |
| `P` | Paste before cursor | Insert yanked content before current node |
| `u` | Undo last change | |
| `Ctrl-r` | Redo last undone change | |
| `.` | Repeat last edit | Repeats last `dd`, `yy`, `p`, or `P` operation |
| `ZZ` | Save and quit | Only saves if file has been modified |

### Visual Mode

| Key | Action | Notes |
|-----|--------|-------|
| `v` / `V` | Enter visual mode | Select multiple nodes for bulk operations |
| `j` / `k` / `h` / `l` | Expand/shrink selection | Move selection boundaries in visual mode |
| `d` | Delete selection | Remove all selected nodes |
| `y` | Yank (copy) selection | Copy all selected nodes |
| `p` / `P` | Replace selection | Replace selection with clipboard content |
| `Esc` | Exit visual mode | Return to NORMAL mode |

### Marks & Jump List

JSONQuill supports vim-style marks and jump list navigation:

| Key | Action | Notes |
|-----|--------|-------|
| `m{a-z}` | Set mark | Set a named mark at the current cursor position |
| `'{a-z}` | Jump to mark | Jump cursor to the previously set mark |
| `y'{a-z}` | Yank to mark | Yank from cursor to mark (motion-to-mark) |
| `d'{a-z}` | Delete to mark | Delete from cursor to mark (motion-to-mark) |
| `Ctrl-o` | Jump backward | Navigate backward in jump history |
| `Ctrl-i` | Jump forward | Navigate forward in jump history |

**Jump list records:** `gg`, `G`, line jumps (`<count>G`), search (`/`, `?`, `n`), and mark jumps (`'a`-`'z`).

**Examples:**
```bash
ma          # Set mark 'a' at current position
10j         # Move down 10 lines
'a          # Jump back to mark 'a'
y'a         # Yank from cursor to mark 'a'
mb          # Set mark 'b'
d'b         # Delete from cursor to mark 'b'
Ctrl-o      # Jump back in jump history
Ctrl-i      # Jump forward in jump history
```

### Named Registers

JSONQuill supports vim-style named registers for managing multiple clipboards:

| Key | Action | Notes |
|-----|--------|-------|
| `"a` - `"z` | Select register (a-z) | Use before `yy`, `dd`, `p`, or `P` to specify which register |
| `"A` - `"Z` | Select register (append mode) | Uppercase letters append to existing register content |
| `"0` | Yank register | Always contains the last yanked (copied) content |
| `"1` - `"9` | Delete history | `"1` = most recent delete, `"2` = previous, etc. |
| `""` | Unnamed register | Default register, syncs with system clipboard |

**Examples:**
```bash
"ayy        # Yank current node to register 'a'
"ap         # Paste from register 'a' after cursor
"bdd        # Delete current node to register 'b'
"b3dd       # Delete 3 nodes to register 'b'
"Ayy        # Append current node to register 'a'
"1p         # Paste from delete history (most recent delete)
yy          # Yank to unnamed register (system clipboard)
p           # Paste from unnamed register (system clipboard)
```

**Use cases:**
- **Multiple clipboards**: Store different content in registers a-z
- **Accumulate content**: Use uppercase (A-Z) to append to a register
- **Delete history**: Access previously deleted content with registers 1-9
- **System clipboard**: Unnamed register (`yy`/`dd`/`p`) syncs with system clipboard

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
| `/` | Start forward search | Enter SEARCH mode to search forward through document<br>Uses smart case (case-insensitive unless pattern has uppercase) |
| `?` | Start backward search | Enter SEARCH mode to search backward through document |
| `n` | Jump to next match | Find next occurrence in search direction<br>Shows match counter (e.g., "Match 2/5")<br>Shows "W" when wrapping around |
| `*` | Search forward for key | Search forward for current object key name |
| `#` | Search backward for key | Search backward for current object key name |
| `:find` | Enter text search mode | Same as pressing `/` |
| `Esc` | Exit search mode | Return to NORMAL mode |

### JSONPath Search (Structural Search)

JSONPath queries allow you to search by structure rather than text:

| Command | Action | Example |
|---------|--------|---------|
| `:path <query>` | JSONPath structural search | `:path $.store.book[*].author` |
| `:jp <query>` | Short alias for `:path` | `:jp $..price` |

**Supported JSONPath Syntax:**

- `$` - Root node
- `.property` - Named property
- `['property']` - Bracket notation
- `[index]` - Array index (supports negative: `[-1]` = last)
- `[*]` - All children
- `..property` - Recursive descent (find anywhere)
- `[start:end]` - Array slicing
- `['prop1','prop2']` - Multiple properties

**Examples:**

```bash
:path $.store.book[*].author    # All book authors
:jp $..price                    # All price fields anywhere
:path $.items[0:3]              # First 3 array items
:path $.user['name','email']    # Multiple properties
:jp $.data[*].id                # All IDs in data array
```

After executing a JSONPath search, use `n` to navigate through matches just like text search.

### Commands (COMMAND mode)

Type `:` to enter command mode, then:

**Tab Completion:** Press `Tab` to autocomplete theme names (`:theme <Tab>`) and setting names (`:set <Tab>`). Press `Tab` multiple times to cycle through options.

| Command | Action | Notes |
|---------|--------|-------|
| `:w` | Save file | Write changes to disk |
| `:w <filename>` | Save as | Write to a different file |
| `:q` | Quit | Warns if there are unsaved changes |
| `:q!` | Force quit | Quit without saving changes |
| `:wq` | Save and quit | Also: `:x` or `ZZ` |
| `:e <filename>` | Load a different file | Warns if there are unsaved changes |
| `:e!` | Reload current file | Discard in-memory changes and reload from disk |
| `:e! <filename>` | Force load a different file | Discard changes and load new file |
| `:undo` | Undo last change | Same as `u` in NORMAL mode |
| `:redo` | Redo last undone change | Same as `Ctrl-r` in NORMAL mode |
| `:format` | Reformat document | Apply jq-style formatting (2-space indent, multi-line) |
| `:help` | Show help overlay | Same as `F1` in NORMAL mode |
| `:theme` | List available themes | Shows all built-in themes |
| `:theme <name>` | Switch theme | e.g., `:theme default-light` |
| `:set` | Show current settings | Display all configuration values |
| `:set number` | Enable line numbers | Show line numbers in tree view |
| `:set nonumber` | Disable line numbers | Hide line numbers |
| `:set relativenumber` (or `:set rnu`) | Enable relative line numbers | Show distance from cursor line |
| `:set norelativenumber` (or `:set nornu`) | Disable relative line numbers | Show absolute line numbers |
| `:set mouse` | Enable mouse scrolling | Enable mouse/trackpad scrolling |
| `:set nomouse` | Disable mouse scrolling | Disable mouse/trackpad scrolling |
| `:set create_backup` | Enable backup file creation | Create `.bak` files before saving |
| `:set nocreate_backup` | Disable backup file creation | Don't create backup files |
| `:set save` | Save settings to config | Write current settings to `~/.config/jsonquill/config.toml` |
| `:path <query>` | JSONPath structural search | e.g., `:path $.store.book[*].author` |
| `:jp <query>` | Short alias for `:path` | e.g., `:jp $..price` |

### Other

| Key | Action | Notes |
|-----|--------|-------|
| `q` | Quit | Only works in NORMAL mode (same as `:q`) |
| `F1` | Toggle help overlay | Shows all keybindings (also `:help`) |
| `↑` / `↓` | Scroll help | When help overlay is open |
| `j` / `k` | Scroll help | When help overlay is open |
| Scroll wheel / Trackpad | Scroll help | When help overlay is open |

## Value Parsing

When adding or editing values, JSONQuill automatically detects the type:

- `true` / `false` → Boolean
- `null` → Null
- `42` / `3.14` / `-1.5` → Number
- Anything else → String

Examples:
- Type `hello` → Stored as string `"hello"`
- Type `42` → Stored as number `42`
- Type `true` → Stored as boolean `true`

## Themes

JSONQuill includes multiple built-in color themes. Use `:theme` to list all available themes, or `:theme <name>` to switch.

**Available themes:**
- `default-dark` - Dark theme optimized for low-light environments (default)
- `default-light` - Light theme for well-lit environments
- `gruvbox-dark` - Retro groove color scheme with warm, earthy tones
- `nord` - Arctic, north-bluish color palette
- `dracula` - Dark theme with vibrant purples and pinks
- `solarized-dark` - Precision color scheme for machines and people
- `monokai` - Popular color scheme inspired by Monokai Pro
- `one-dark` - The default dark theme from Atom editor

**Examples:**
```bash
# List all available themes
:theme

# Switch to Nord theme
:theme nord

# Switch to Dracula theme
:theme dracula
```

Themes can also be set in your configuration file (see Configuration section below).

## Configuration

JSONQuill supports a configuration file at `~/.config/jsonquill/config.toml`.

### Config File Format

```toml
# Theme name (default: "default-dark")
theme = "default-dark"

# Number of spaces per indentation level (default: 2)
indent_size = 2

# Display line numbers (default: true)
show_line_numbers = true

# Show relative line numbers (default: false)
relative_line_numbers = false

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

# Preserve original formatting for unmodified nodes (default: true)
preserve_formatting = true
```

### Format Preservation

JSONQuill preserves the original formatting of unmodified JSON nodes when saving files. This means:
- Nodes you don't edit keep their exact original whitespace, indentation, and newlines
- Only modified nodes are reformatted according to your indent settings
- This is particularly useful for large JSON files where you only need to edit specific values

You can disable format preservation if you want all output to be reformatted consistently:

```toml
# Preserve original formatting for unmodified nodes (default: true)
preserve_formatting = true
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

### Version Management

The `scripts/ver` script helps manage version updates and releases:

```bash
# Print current version
./scripts/ver
# Output: 0.2.1

# Update to new version
./scripts/ver 0.3.0
```

**What it does:**
1. Validates version format (must be X.Y.Z)
2. Updates `Cargo.toml` with the new version
3. Runs `cargo build` to update `Cargo.lock`
4. Commits both files: `chore: bump version to X.Y.Z`
5. Creates git tag: `vX.Y.Z`
6. Prints push command for you to run manually

**Example workflow:**
```bash
# Update version
./scripts/ver 0.3.0

# Review the changes
git log -1
git show v0.3.0

# Push to remote
git push && git push origin v0.3.0
```

### Regenerating Release Notes

The `scripts/release-notes` script generates GitHub release notes independently:

```bash
# Generate release notes (auto-detects current version from Cargo.toml)
./scripts/release-notes 0.7.0

# Generate release notes for specific version range
./scripts/release-notes 0.7.0 0.5.0

# Save to file
./scripts/release-notes 0.7.0 > release-notes-v0.7.0.md
```

This is useful when you need to regenerate release notes after editing commits or want to preview notes without running the full version bump workflow.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

### Third-Party Licenses

This project depends on several open source libraries, all of which are MIT-compatible. For a complete list of dependencies and their licenses, see [THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md).

All direct dependencies are either:
- MIT licensed, or
- Dual-licensed under Apache-2.0 OR MIT (used under MIT terms)

### Contributing

By contributing to this project, you agree that your contributions will be licensed under the same MIT License.
