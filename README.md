# jeditor

A terminal-based structural JSON editor with vim-style keybindings.

## Status

**Alpha Release** - Core functionality is implemented and usable. The editor supports:
- JSON file loading and editing
- Tree-based navigation with vim keybindings
- Stdin piping support (e.g., `cat file.json | jeditor`)
- Undo/redo functionality
- Clipboard operations (yank/paste)
- Customizable themes and settings

See [CLAUDE.md](CLAUDE.md) for detailed feature list and usage instructions.

## Description

jeditor is a Rust-based terminal application for viewing and editing JSON files in a structured, tree-like format. It aims to provide an intuitive vim-style interface for navigating complex JSON documents directly in the terminal.

## Tech Stack

- **Rust**: Core language
- **ratatui**: Terminal UI framework
- **termion**: Terminal manipulation with /dev/tty support for stdin piping
- **serde_json**: JSON parsing and serialization
- **clap**: Command-line argument parsing
- **arboard**: Clipboard integration

## Development Setup

### Prerequisites

- Rust toolchain (1.70+)
- Cargo package manager

### Building

```bash
# Clone the repository
git clone <repository-url>
cd jeditor

# Build the project
cargo build

# Run the application
cargo run

# Run tests
cargo test
```

## Installation & Usage

Build from source:

```bash
# Clone and build
git clone <repository-url>
cd jeditor
cargo build --release

# Run with a file
./target/release/jeditor examples/sample.json

# Or pipe JSON from stdin
cat file.json | ./target/release/jeditor
curl https://api.example.com/data | ./target/release/jeditor
```

Basic keybindings:
- `j/k` or arrow keys: Navigate (supports count prefix: `3j` moves down 3 lines)
- `h/l`: Collapse/expand nodes
- `gg/G`: Jump to top/bottom
- `<count>g`: Jump to line number (e.g., `10g` goes to line 10)
- `a`: Add scalar value (arrays: direct insert, objects: prompt for key then value)
- `i`: Edit value
- `yy`: Copy node (supports count: `3yy` copies 3 nodes)
- `dd`: Delete node (supports count: `3dd` deletes 3 nodes)
- `p`: Paste
- `u`: Undo
- `Ctrl-r`: Redo
- `:w`: Save (`:w filename` to save as new file)
- `:q`: Quit
- `?`: Help

For complete documentation, see [CLAUDE.md](CLAUDE.md).

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

### Third-Party Licenses

This project depends on several open source libraries, all of which are MIT-compatible. For a complete list of dependencies and their licenses, see [THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md).

All direct dependencies are either:
- MIT licensed, or
- Dual-licensed under Apache-2.0 OR MIT (used under MIT terms)

### Contributing

By contributing to this project, you agree that your contributions will be licensed under the same MIT License.
