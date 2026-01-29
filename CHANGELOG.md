# Changelog

All notable changes to jsonquill will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
