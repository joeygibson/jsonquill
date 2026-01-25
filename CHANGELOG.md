# Changelog

All notable changes to jsonquill will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
