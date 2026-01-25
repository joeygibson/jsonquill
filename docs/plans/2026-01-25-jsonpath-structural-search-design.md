# JSONPath Structural Search Design

**Date:** 2026-01-25
**Status:** Approved
**Author:** Claude (with user collaboration)

## Overview

Add JSONPath query support to jsonquill, enabling users to search JSON documents by structure rather than just text content. The feature integrates with existing search infrastructure, using the same `n` navigation pattern for consistency with vim-style workflows.

## Goals

- Support core JSONPath syntax for structural navigation
- Integrate seamlessly with existing text search (`/`, `?`)
- Maintain vim-style consistency (searches replace each other, `n` navigates)
- Provide clear feedback on search results and errors

## Non-Goals

- Filter expressions (`?(@.price < 10)`) - deferred to future enhancement
- Script expressions or functions - not needed for terminal editor use case
- Full JSONPath spec compliance - focus on practical navigation features

## Architecture

### New Module: `src/jsonpath/`

**ast.rs** - Abstract syntax tree for JSONPath expressions
```rust
pub enum PathSegment {
    Root,                              // $
    Current,                           // @
    Child(String),                     // .property or ['property']
    Index(isize),                      // [0] or [-1]
    Wildcard,                          // * or [*]
    RecursiveDescent(Option<String>),  // .. or ..property
    Slice(Option<isize>, Option<isize>), // [start:end]
    MultiProperty(Vec<String>),        // ['prop1','prop2']
}

pub struct JsonPath {
    pub segments: Vec<PathSegment>,
}
```

**parser.rs** - Converts query strings to AST
```rust
pub struct Parser {
    input: String,
    position: usize,
}

impl Parser {
    pub fn parse(query: &str) -> Result<JsonPath, JsonPathError>
    // Tokenize and build AST using recursive descent
}
```

**evaluator.rs** - Executes queries against JSON tree
```rust
pub struct Evaluator<'a> {
    tree: &'a JsonTree,
}

impl<'a> Evaluator<'a> {
    pub fn evaluate(&self, path: &JsonPath) -> Vec<Vec<usize>>
    // Walk tree, collect matching node paths
}
```

**error.rs** - Error types
```rust
pub enum JsonPathError {
    UnexpectedToken { position: usize, found: char, expected: String },
    UnexpectedEnd { expected: String },
    InvalidSyntax { message: String },
}
```

### Modified Components

**src/editor/state.rs**
- Add `SearchType` enum: `Text | JsonPath`
- Track active search type
- Modify `execute_search()` to handle text search
- Add `execute_jsonpath_search()` for JSONPath queries
- Unify navigation in existing `next_search_result()`

**src/input/handler.rs**
- Add `:path <query>` command
- Add `:jp <query>` as alias
- Add `:find` as alias for entering `/` search mode

**src/ui/status_line.rs**
- Show search type in status: `[JSONPath: $.store.book[*]] Match 2/5`

## JSONPath Syntax Support

### Core Features (MVP)

| Syntax | Description | Example |
|--------|-------------|---------|
| `$` | Root node | `$` |
| `.property` | Child property | `$.store.name` |
| `['property']` | Bracket notation | `$['store']['name']` |
| `[index]` | Array index | `$.items[0]` |
| `[*]` or `.*` | All children | `$.items[*]` |
| `..property` | Recursive descent | `$..price` |
| `[start:end]` | Array slice | `$.items[0:3]` |
| `['p1','p2']` | Multiple properties | `$.user['name','email']` |

### Examples

```jsonpath
# Get all book authors
$.store.book[*].author

# Get all price fields anywhere in document
$..price

# Get first 3 items
$.items[0:3]

# Get user's name and email
$.user['name','email']

# Get last item in array (negative index)
$.items[-1]
```

## User Interface

### Commands

| Command | Description |
|---------|-------------|
| `:path <query>` | Execute JSONPath search |
| `:jp <query>` | Short alias for `:path` |
| `:find` | Enter text search mode (same as `/`) |

### Navigation

- `n` - Next match (works for both text and JSONPath)
- Search results persist until new search executed
- New search replaces previous search (text or JSONPath)

### Status Line

- JSONPath active: `[JSONPath: $.store.book[*]] Match 2/5`
- Text search active: `[Search: "price"] Match 1/3`

### Messages

- Success: `Found 5 matches for $.store.book[*].author`
- No matches: `No matches for $.foo.bar`
- Parse error: `Invalid JSONPath syntax: Expected ']' after array index`

## Data Flow

1. User enters `:path $.store.book[*].author`
2. Command handler extracts query string
3. Parser validates and converts to AST
4. Evaluator walks JSON tree, collecting matching paths
5. Paths stored in `EditorState.search_results`
6. Cursor jumps to first match
7. Status line shows search type and match count
8. User presses `n` to cycle through matches

## Implementation Phases

### Phase 1: Core Parser
- Implement tokenizer for JSONPath syntax
- Build recursive descent parser
- Create AST types
- Unit tests for all syntax forms

### Phase 2: Evaluator
- Implement tree walker matching AST against JSON
- Handle all path segment types
- Collect matching paths
- Unit tests with sample JSON documents

### Phase 3: Integration
- Add `SearchType` to `EditorState`
- Implement `execute_jsonpath_search()`
- Wire up `:path` and `:jp` commands
- Add `:find` alias
- Update status line rendering

### Phase 4: Testing & Documentation
- Integration tests for end-to-end search
- Edge cases: empty results, malformed queries, JSONL
- Update CLAUDE.md with new commands
- Update README.md with JSONPath examples

## Error Handling

- Invalid syntax → Parse error in message area with position
- No matches → Info message, clear previous search
- Empty query → Ignore, show usage hint
- Malformed brackets/quotes → Detailed syntax error

## Edge Cases

- JSONL files: Search within each line, path includes line index
- Empty arrays/objects: Wildcard returns no results
- Negative indices: `-1` is last element, `-2` is second-to-last
- Slice bounds: Out of range indices clamp to array bounds
- Recursive descent on primitives: No matches (can't descend into scalars)

## Future Enhancements (Not in MVP)

- Filter expressions: `$.store.book[?(@.price < 10)]`
- Boolean logic: `[?(@.price < 10 && @.category == 'fiction')]`
- Functions: `$.store.book[*].length()`
- Named search history: Save/recall common queries
- Search highlighting: Visual indicator of matched nodes in tree view

## Testing Strategy

### Unit Tests
- Parser: All syntax forms, error cases
- Evaluator: Each segment type, combinations, edge cases

### Integration Tests
- End-to-end: Query → results → navigation
- Text vs JSONPath search switching
- JSONL support
- Error messages displayed correctly

### Manual Testing
- Complex real-world JSON documents
- Performance with large files
- Error message clarity

## Success Criteria

- Parse all core JSONPath syntax forms
- Navigate results with `n` like text search
- Clear error messages for invalid queries
- All tests passing
- Documentation updated
- No regressions in existing search functionality
