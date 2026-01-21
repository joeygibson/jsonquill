# jeditor

A terminal-based structural JSON editor with vim-style keybindings.

## Status

Under active development. The project is in its initial implementation phase and not yet ready for use.

## Description

jeditor is a Rust-based terminal application for viewing and editing JSON files in a structured, tree-like format. It aims to provide an intuitive vim-style interface for navigating complex JSON documents directly in the terminal.

## Tech Stack

- **Rust**: Core language
- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal manipulation
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

The project is not yet ready for installation or production use. Check back later for release information.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

### Third-Party Licenses

This project depends on several open source libraries, all of which are MIT-compatible. For a complete list of dependencies and their licenses, see [THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md).

All direct dependencies are either:
- MIT licensed, or
- Dual-licensed under Apache-2.0 OR MIT (used under MIT terms)

### Contributing

By contributing to this project, you agree that your contributions will be licensed under the same MIT License.
