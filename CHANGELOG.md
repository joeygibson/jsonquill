# Changelog

All notable changes to jsonquill will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.14.1] - 2026-02-19

### Added

- open empty document when specified file doesn't exist
- add update-release script to push release notes to GitHub

### Other

- updated demo
## [0.14.0] - 2026-02-10

### Added

- remove item counts from collapsed previews
## [0.13.1] - 2026-02-09

### Added

- recursively expand nested containers in collapsed previews
## [0.13.0] - 2026-02-09

### Added

- expand collapsed previews to use full terminal width
- add command and search history with Up/Down arrow cycling
- add crossterm backend for Windows support

### Changed

- Documentation: update platform support to indicate experimental Windows support

### Other

- update README
- ci: add nightly build workflow for preview binaries on main
## [0.12.4] - 2026-02-07

### Fixed

- update ratatui to 0.30 to resolve lru security vulnerability
## [0.12.3] - 2026-02-06

### Fixed

- preserve search results after exiting search mode
## [0.12.2] - 2026-02-05

### Added

- add cursor navigation to command and search modes

### Fixed

- text search now finds matches inside collapsed nodes
- JSONPath queries now work on JSONL files

### Other

- changed video size
## [0.12.1] - 2026-02-03

### Fixed

- undo system now preserves expansion state and cursor position correctly

### Other

- updated video
- demo video
- new logos
- update readme
## [0.12.0] - 2026-01-31

### Added

- gzip compression support complete
- add gzip compression support to save functions
- add write_file_atomic with optional compression
- add gzip auto-detection for stdin input
- add gzip decompression support to load_json_file
- add read_gzipped_file helper function
- add determine_jsonl_format helper function

### Changed

- Documentation: add gzip compression support to documentation
- Documentation: add gzip compression support design
- Documentation: add motion-to-mark, smart paste, and visual mode to README

### Fixed

- use compact formatting for JSONL in :format command

### Other

- Testing: add gzip integration tests
- Testing: add gzip save, format switching, and backup tests
- Testing: add gzipped JSONL file loading test
- Testing: add corrupted gzip file error handling test
## [0.11.0] - 2026-01-30

### Added

- add motion-to-mark, smart paste, and fix UTF-8/highlighting bugs
- add visual selection rendering
- implement repeat command (.)
- implement visual mode operations (d, y, p, P)
- implement visual mode entry and exit
- implement mark set and jump (m and ')
- record jumps for big navigation commands
- implement jump list navigation (Ctrl-o/Ctrl-i)
- add input events for visual mode, marks, jumplist, and repeat
- add state fields for jumplist, marks, visual mode, and repeat
- add Visual mode to EditorMode enum
- add RepeatableCommand enum for '.' key functionality
- add MarkSet data structure for bookmark management
- add JumpList data structure for Ctrl-o/Ctrl-i navigation

### Changed

- Documentation: update CLAUDE.md with new features
- Documentation: update help overlay with new features
- Documentation: add design for visual mode, marks, jump list, and repeat

### Fixed

- visual selection now excludes children of selected containers
- add must_use attributes to JumpList query methods
## [0.10.0] - 2026-01-29

### Added

- add panic hook to restore terminal on crash
- add :format command for jq-style JSON formatting
- exit command mode when backspacing past colon
- add :set create_backup command and fix backup file extension

### Changed

- Documentation: add explicit pre-commit checklist for fmt and clippy

### Fixed

- paste corruption and cursor visibility issues
## [Unreleased]

### Added

- add :set create_backup and :set nocreate_backup commands
- add tab completion for create_backup settings

### Changed

- align descriptions in help screen Commands section for better readability

### Fixed

- fix backup file extension from .jsonquill.bak to .bak (now creates file.json.bak)

## [0.9.0] - 2026-01-29

### Added

- add vim-style :e commands for file reloading
- enable JSONL row additions and improve paste operations
- show key name in edit prompt and document data safety
- validate JSON before saving to catch serialization bugs
- add 7 new themes and sort theme picker alphabetically
- add interactive theme picker with live preview
- add Ctrl-f/b, PgUp/PgDn, Home/End navigation keys
- enable format preservation by default
- implement format-preserving serialization
- add preserve_formatting config option
- implement span tracking during JSON parsing
- add original_source field to JsonTree
- add text_span field to NodeMetadata
- add TextSpan struct for tracking byte ranges

### Changed

- Documentation: reorder :e commands to show non-destructive version first
- Documentation: update help screen, README, and CLAUDE.md for :e commands
- Documentation: add interactive theme picker design
- Documentation: document format preservation feature
- Documentation: fix outdated comments after text_span migration
- Documentation: add format preservation design
- Documentation: add named registers documentation to README

### Fixed

- reset to default expansion state on :e! reload
- improve :e! to preserve cursor position and fix expansion state
- preserve tree expansion state when reloading with :e!
- JSONL integer formatting and root-level add operations
- show (current) label only on original theme in picker
- use pattern matching instead of is_some + unwrap
- preserve tree expansion state when deleting nodes
- make <count>G jump to line to match vim behavior
- correct `a` and `o` commands to add containers instead of scalars
- preserve trailing newline in saved files
- correct span tracking in parser to prevent corruption
- correctly propagate inside_modified_container flag in format preservation
- disable format preservation due to data corruption
- don't extract spans from modified containers
- correct escape handling and char indexing in SpanTracker

### Other

- edit readme
- Testing: add comprehensive document corruption tests
- style: run cargo fmt
- Testing: add comprehensive format preservation integration tests
## [0.8.0] - 2026-01-27

### Added

- add register selection UI, delete history, and tests
- update paste operations to use register system
- update yank operations to use register system
- add register fields to EditorState
- add append mode and history operations
- add RegisterSet with get/set operations
- add RegisterContent struct

### Changed

- Documentation: document named register feature
- Documentation: add detailed implementation plan for named registers
- Documentation: add named registers design
- Documentation: add Homebrew installation instructions

### Fixed

- auto-detect JSONL format when piping to stdin
- add temp_container field for add container operations

### Other

- Testing: add comprehensive tests for JSONL parsing
- Merge branch 'feature/named-registers'
## [0.7.0] - 2026-01-26

### Added

- clear search results on non-search keys
- add clear_search_results method
- add color highlighting to path in status bar
- display current path in status bar
- add public API for getting current path in dot notation
- add standalone release-notes generation script

### Changed

- Documentation: document search results auto-clear behavior
- Documentation: add design for clearing search info on non-search keys
- Documentation: document path color highlighting in status bar
- Documentation: add design for colored path in status bar
- Documentation: update plan to reflect actual implementation order
- Documentation: document current path in status bar feature
- Documentation: add design for current path in status bar
- Documentation: document release-notes script

### Fixed

- use unique delimiter to avoid parsing conflicts
- filter Co-Authored-By lines from release notes
- extract full commit messages in release notes generation

### Other

- Testing: use text search in clear_search_results test per spec
- style: apply cargo fmt formatting
- Testing: add test for colored path in status bar
- style: format multi-line format! macro in status line
## [0.6.0] - 2026-01-25

### Added

- add tab-completion for :theme and :set commands
- add six popular color themes
- add parent focus command (H)
- add depth-based navigation with w and b commands
- generate GitHub-style release notes in ver script

### Fixed

- correct depth calculation for w/b commands
## [0.5.0] - 2026-01-25

### Added

- add relative line numbers (:set relativenumber)
- add smart case search and wrapping indicator
- add key search commands (*, #)
- add sibling jumping commands (0/^, $)
- add screen positioning commands (zz, zt, zb)
- add path copying commands (yp, yb, yq)
- add automatic changelog generation to ver script

### Changed

- Documentation: update README with new navigation and search features
- Documentation: simplify CLAUDE.md by referencing global Rust guidelines

### Fixed

- fix awk error in ver script with multi-line strings
## [0.4.0] - 2026-01-25

### Added

- **JSONPath Structural Search**: Query JSON documents by structure, not just text content
  - `:path <query>` command for JSONPath queries (e.g., `:path $.store.book[*].author`)
  - `:jp <query>` short alias for `:path`
  - `:find <query>` to execute text search from command mode
  - `:find` to enter text search mode (equivalent to `/`)
- **JSONPath Syntax Support**:
  - Root selector: `$`
  - Child access: `.property` or `['property']`
  - Wildcard: `*` for all children
  - Recursive descent: `..` for all descendants
  - Array slicing: `[start:end]` for array ranges
  - Multiple properties: `['name','email']`
- **UI Enhancements**:
  - Status line now shows search type (Text/JSONPath)
  - Navigate through JSONPath results using `n` (next match)
  - Help screen updated with JSONPath documentation
- **New Module**: `src/jsonpath/` with complete JSONPath implementation
  - AST (Abstract Syntax Tree) definitions
  - Tokenizer and parser
  - Query evaluator
  - Error handling
- **Search System Improvements**:
  - `SearchType` enum to track search mode (Text vs JSONPath)
  - Extended `EditorState` with JSONPath search capabilities
  - Unified search navigation with `n` key

### Changed

- Reorganized search sections in README and help screen for better clarity
- Simplified CLAUDE.md by referencing global Rust guidelines

### Fixed

- `:find` command now properly accepts search query arguments

### Testing

- Added comprehensive JSONPath integration tests
- Full test coverage for parser, evaluator, and search integration

## [0.3.0] - (Previous release)

(Release notes to be added)

---

[0.4.0]: https://github.com/yourusername/jsonquill/compare/v0.3.0...v0.4.0
[0.5.0]: https://github.com/yourusername/jsonquill/compare/v0.4.0...v0.5.0
[0.6.0]: https://github.com/yourusername/jsonquill/compare/v0.5.0...v0.6.0
[0.7.0]: https://github.com/yourusername/jsonquill/compare/v0.6.0...v0.7.0
[0.8.0]: https://github.com/yourusername/jsonquill/compare/v0.7.0...v0.8.0
[0.9.0]: https://github.com/yourusername/jsonquill/compare/v0.8.0...v0.9.0
[0.10.0]: https://github.com/yourusername/jsonquill/compare/v0.9.0...v0.10.0
[0.11.0]: https://github.com/yourusername/jsonquill/compare/v0.10.0...v0.11.0
[0.12.0]: https://github.com/yourusername/jsonquill/compare/v0.11.0...v0.12.0
[0.12.1]: https://github.com/yourusername/jsonquill/compare/v0.12.0...v0.12.1
[0.12.2]: https://github.com/yourusername/jsonquill/compare/v0.12.1...v0.12.2
[0.12.3]: https://github.com/yourusername/jsonquill/compare/v0.12.2...v0.12.3
[0.12.4]: https://github.com/yourusername/jsonquill/compare/v0.12.3...v0.12.4
[0.13.0]: https://github.com/yourusername/jsonquill/compare/v0.12.4...v0.13.0
[0.13.1]: https://github.com/yourusername/jsonquill/compare/v0.13.0...v0.13.1
[0.14.0]: https://github.com/yourusername/jsonquill/compare/v0.13.1...v0.14.0
[0.14.1]: https://github.com/yourusername/jsonquill/compare/v0.14.0...v0.14.1
