# Third-Party Licenses

This project uses the following open source packages. All dependencies are compatible with the MIT license.

## Direct Dependencies

### anyhow (1.0.100)
- **License:** Apache-2.0 OR MIT (used under MIT)
- **Repository:** https://github.com/dtolnay/anyhow
- **Description:** Flexible concrete Error type built on std::error::Error

### arboard (3.6.1)
- **License:** Apache-2.0 OR MIT (used under MIT)
- **Repository:** https://github.com/1Password/arboard
- **Description:** Image and text handling for the OS clipboard

### clap (4.5.54)
- **License:** Apache-2.0 OR MIT (used under MIT)
- **Repository:** https://github.com/clap-rs/clap
- **Description:** A simple to use, efficient, and full-featured Command Line Argument Parser

### crossterm (0.28.1)
- **License:** MIT
- **Repository:** https://github.com/crossterm-rs/crossterm
- **Description:** A crossplatform terminal library for manipulating terminals

### dirs (5.0.1)
- **License:** Apache-2.0 OR MIT (used under MIT)
- **Repository:** https://github.com/soc/dirs-rs
- **Description:** A tiny low-level library that provides platform-specific standard locations of directories

### ratatui (0.29.0)
- **License:** MIT
- **Repository:** https://github.com/ratatui/ratatui
- **Description:** A library that's all about cooking up terminal user interfaces

### serde (1.0.228)
- **License:** Apache-2.0 OR MIT (used under MIT)
- **Repository:** https://github.com/serde-rs/serde
- **Description:** A generic serialization/deserialization framework

### serde_json (1.0.149)
- **License:** Apache-2.0 OR MIT (used under MIT)
- **Repository:** https://github.com/serde-rs/json
- **Description:** A JSON serialization file format

### tempfile (3.24.0)
- **License:** Apache-2.0 OR MIT (used under MIT)
- **Repository:** https://github.com/Stebalien/tempfile
- **Description:** A library for managing temporary files and directories

### toml (0.8.23)
- **License:** Apache-2.0 OR MIT (used under MIT)
- **Repository:** https://github.com/toml-rs/toml
- **Description:** A native Rust encoder and decoder of TOML-formatted files and streams

## License Compatibility Summary

All dependencies are licensed under terms compatible with the MIT license:

- **MIT**: Fully compatible
- **Apache-2.0 OR MIT**: Dual-licensed; we use these dependencies under MIT terms
- **0BSD**: Zero-Clause BSD, very permissive and MIT-compatible
- **BSL-1.0**: Boost Software License 1.0, permissive and MIT-compatible
- **MPL-2.0**: Mozilla Public License 2.0, weak copyleft but compatible for linking
- **Zlib**: Zlib license, permissive and MIT-compatible

### Transitive Dependencies

This project has various transitive dependencies (dependencies of dependencies) which include additional licenses:

- **Apache-2.0**: Used by many Rust ecosystem libraries
- **0BSD OR Apache-2.0 OR MIT**: Triple-licensed dependencies (we use under MIT)
- **Apache-2.0 OR MIT OR Zlib**: Triple-licensed dependencies (we use under MIT)
- **BSL-1.0**: Used by clipboard-win, error-code
- **MPL-2.0**: Used by option-ext
- **Zlib**: Used by foldhash and various image processing libraries

All of these licenses are compatible with MIT and allow redistribution under MIT terms.

## Generating License Report

To see a complete list of all transitive dependencies and their licenses, run:

```bash
cargo install cargo-license
cargo license
```

To see only direct dependencies:

```bash
cargo license --direct-deps-only
```

## Attribution

While not legally required by the MIT license, we acknowledge and thank all the open source maintainers whose work makes this project possible.
