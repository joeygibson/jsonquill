# Format Preservation Design

**Date**: 2026-01-27
**Status**: Approved

## Overview

This design adds whitespace and indentation preservation to jsonquill's save functionality. When users load and edit a JSON file, unmodified portions retain their exact original formatting, while modified portions use the configured formatting style. This provides clear visual feedback about changes and meets user expectations for format-preserving editors.

## Goals

- Preserve whitespace, indentation, and newlines for unmodified nodes
- Use configured formatting (indent_size) for modified/new nodes
- Maintain exact byte-for-byte output when no edits are made
- Keep implementation efficient and maintainable
- No breaking changes to existing functionality

## Non-Goals

- Comment preservation (would require different parser)
- Number format preservation (1.0 vs 1, scientific notation)
- Advanced formatting options (trailing commas, spacing rules)
- Format preservation for very large files (>100MB lazy-loading threshold)

## Design Decisions

### 1. Hybrid Formatting Approach

**Decision**: Unmodified nodes preserve exact original formatting. Modified nodes use configured formatting style.

**Rationale**:
- Provides clear visual feedback about what changed
- Predictable behavior for users
- Simpler than trying to infer and maintain formatting style for edits

**Alternatives Considered**:
- Best-effort preservation: More complex, harder to predict
- Whole-document switch: Too coarse-grained, loses useful information
- User-controlled toggle: Adds UI complexity, hybrid approach handles most cases

### 2. Text Span Storage

**Decision**: Store byte ranges pointing into the original JSON string.

**Rationale**:
- Memory efficient (two usizes per node vs entire text)
- Exact preservation (no formatting interpretation needed)
- Simple implementation (just slice original string)

**Alternatives Considered**:
- Parse formatting metadata: Complex, requires detailed whitespace analysis
- Store full text per node: Memory-intensive, lots of duplication
- Specialized parser: Large architectural change, may impact performance

### 3. Span Tracking During Parse

**Decision**: Use serde_json for parsing, track spans via secondary traversal.

**Rationale**:
- Keeps existing parser (well-tested, fast)
- Minimal changes to parsing logic
- Acceptable performance overhead

**Alternatives Considered**:
- Switch to span-aware parser (json-rust, serde-json-span): Requires parser migration
- Custom parser: Significant engineering effort, maintenance burden

### 4. Structural Change Handling

**Decision**: New nodes inherit parent's detected indentation style.

**Rationale**:
- Feels natural when editing
- Maintains document consistency
- Easy to implement (detect from parent's span text)

**Alternatives Considered**:
- Preserve exact whitespace: Can look odd after multiple edits
- Smart document-wide detection: Complex, may be inconsistent
- Always use config: Creates jarring mixed styles

## Architecture

### Core Data Structures

```rust
// In src/document/node.rs
pub struct NodeMetadata {
    /// Byte range in the original JSON string (for unmodified nodes)
    pub text_span: Option<TextSpan>,
    /// Whether this node has been modified
    pub modified: bool,
}

pub struct TextSpan {
    /// Start byte offset in original JSON
    pub start: usize,
    /// End byte offset in original JSON (exclusive)
    pub end: usize,
}

// In src/document/tree.rs
pub struct JsonTree {
    root: JsonNode,
    /// The original JSON string (preserved for unmodified nodes)
    original_source: Option<String>,
}
```

**Key Points**:
- `text_span` tracks where each node lives in original file
- When `modified = false` and `text_span = Some(...)`, extract exact original text
- When `modified = true`, serialize fresh JSON using configured style
- `JsonTree` holds original source string for all nodes to reference

### Parsing with Span Tracking

```rust
pub fn parse_json(json_str: &str) -> Result<JsonTree> {
    let original_source = json_str.to_string();

    // Parse with serde_json
    let serde_value: SerdeValue = serde_json::from_str(json_str)?;

    // Track spans via secondary traversal
    let root = convert_with_spans(serde_value, json_str);

    Ok(JsonTree {
        root,
        original_source: Some(original_source),
    })
}
```

**Implementation Strategy**:
1. Parse with serde_json to get value tree
2. Walk through original string, matching structure
3. As we recognize each value (object `{`, array `[`, string, number), record span
4. Recursively assign spans to corresponding nodes

### Serialization with Format Preservation

```rust
fn serialize_preserving_format(node: &JsonNode, original: &str, config: &Config) -> String {
    if !node.is_modified() && node.metadata.text_span.is_some() {
        // Extract original text from source
        let span = node.metadata.text_span.as_ref().unwrap();
        return original[span.start..span.end].to_string();
    }

    // Node was modified - serialize fresh using configured formatting
    serialize_node_with_context(node, original, config, 0)
}
```

**Behavior**:
- Unmodified nodes: Extract exact text from original string
- Modified nodes: Serialize using configured formatting
- Container nodes: Recursively apply same logic to children

### Handling Edits

**Value Edits** (edit existing node):
1. Mark node as `modified = true`
2. Clear its `text_span = None`
3. Mark parent containers as modified

**Additions** (new nodes via `i`, `a`, `o`):
1. Node starts with `modified = true`, `text_span = None`
2. Detect indentation from parent's original span
3. Serialize using detected or configured style

**Deletions** (remove nodes via `dd`):
1. Remove node from parent's children
2. Mark parent as modified
3. Parent re-serialization handles comma/spacing

**Indent Detection**:
```rust
fn detect_indent_style(parent: &JsonNode, original: &str, config: &Config) -> usize {
    if let Some(span) = &parent.metadata.text_span {
        // Analyze original text to detect spaces per level
        detect_indent_from_text(&original[span.start..span.end])
    } else {
        // Parent was modified, use config default
        config.indent_size
    }
}
```

## Edge Cases

### Backward Compatibility

Documents without span information (`original_source = None`):
- New empty documents created in editor
- Documents piped from stdin (may add span tracking later)
- Fall back to current serialization behavior

### JSONL Files

Format preservation applies per-line:
- Each line is independently preserved/serialized
- Existing JSONL handling remains separate

### Very Large Files

For files exceeding lazy-loading threshold (100MB):
- Accept memory cost of original_source for now
- Future optimization: memory-mapped files or skip preservation

### Clipboard Operations

Yanked and pasted nodes:
- Lose their text spans (marked as modified)
- Use configured formatting when pasted
- Correct behavior: pasted content is new content

### Undo/Redo

Undo operations restore both value and text_span:
- Undo system already stores full node snapshots
- Works automatically with new metadata

## Configuration

Add optional setting:

```toml
# ~/.config/jsonquill/config.toml
preserve_formatting = true  # default: true
```

Allows users to opt for normalized output if desired.

## Implementation Plan

### Phase 1: Core Infrastructure
- Add `TextSpan` struct to `src/document/node.rs`
- Update `NodeMetadata` with `text_span` field
- Remove old `original_text: Option<String>` field
- Add `original_source` to `JsonTree` in `src/document/tree.rs`

### Phase 2: Span-Aware Parsing
- Implement span tracking in `src/document/parser.rs`
- Add helper for walking and matching JSON structure
- Update `parse_json()` to compute and assign spans
- Update `convert_serde_value_impl()` to accept spans

### Phase 3: Format-Preserving Serialization
- Add `serialize_preserving_format()` in `src/file/saver.rs`
- Update `save_json_file()` to use hybrid serialization
- Implement indent detection from original text
- Keep existing `serialize_node()` for fallback

### Phase 4: Testing and Polish
- Test: parse → save without edits → byte-for-byte identical
- Test: parse → edit single value → only that value reformatted
- Test: parse → add/delete nodes → parent reformatted
- Test: various indent styles (2 space, 4 space, tabs)
- Test: compact vs expanded format preservation
- Add config option and documentation

## Files to Modify

**Core Changes**:
- `src/document/node.rs` - Add TextSpan, update NodeMetadata
- `src/document/tree.rs` - Add original_source field
- `src/document/parser.rs` - Implement span tracking
- `src/file/saver.rs` - Add format-preserving serialization
- `src/config/mod.rs` - Add preserve_formatting config option

**No Changes Needed**:
- Editor, UI, input handling (unchanged)
- Undo/redo system (works automatically)
- JSONL handling (already separate)

## Testing Strategy

**Unit Tests**:
- Span tracking correctness (all node types)
- Format preservation (unmodified nodes)
- Fresh serialization (modified nodes)
- Indent detection accuracy

**Integration Tests**:
- Round-trip preservation (load → save → compare)
- Edit scenarios (add, delete, modify values)
- Mixed formatted documents
- Edge cases (empty files, single values, deep nesting)

**Manual Testing**:
- Various real-world JSON files
- Different formatting conventions
- Large files (performance)

## Success Criteria

1. **Exact preservation**: Files saved without edits are byte-for-byte identical
2. **Partial preservation**: Edits affect only modified nodes and their containers
3. **Natural formatting**: New content blends with existing style
4. **No regressions**: All existing tests pass
5. **Performance**: Parsing overhead < 10% for typical files

## Future Enhancements

Potential future work (not in this design):
- Comment preservation (requires JSON5/JSONC parser)
- Number format preservation (track original representation)
- Configurable formatting rules (trailing commas, spacing)
- Memory-mapped files for very large documents
- Format detection and auto-configuration
