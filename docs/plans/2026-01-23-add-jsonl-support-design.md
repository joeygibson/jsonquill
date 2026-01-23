# JSONL Support and Collapsed Object Preview Design

## Overview

Add support for JSONL (JSON Lines) files and implement collapsed object/array previews similar to jless for both JSONL and regular JSON documents.

## Goals

1. **JSONL File Support**: Detect, parse, edit, and save `.jsonl` files line-by-line
2. **Collapsed Previews**: Show inline previews of collapsed objects/arrays like jless
3. **Consistent Display**: Apply collapsed preview format to both JSONL lines and regular JSON containers

## Background

**Current State:**
- JSON Quill displays collapsed containers as `{ 3 fields }` or `[ 5 items ]`
- Only supports regular JSON files (single document)
- No JSONL support

**User Research:**
- jless shows collapsed previews: `(7) {name: "TechCorp", founded: 2010, active: true, headquarters: {…}, ...}`
- Format: `(N) {key1: val1, key2: val2, nested: {…}, ...}`
- Uses cyan-ish color for preview text
- Shows nested containers as `{…}` or `[…]`

## Architecture

### 1. Collapsed Object Display

**Format Specification:**
```
Objects: (N) {key1: val1, key2: val2, nested: {…}, ...}
Arrays:  (N) [elem1, elem2, nested: […], ...]
```

**Rules:**
- Count shown as `(N)` before preview
- Keys unquoted (JSON5-style)
- String values remain quoted
- Numbers/booleans/null shown as-is
- Nested objects: `{…}` (with ellipsis)
- Nested arrays: `[…]` (with ellipsis)
- Truncate at ~60 characters, ending with `...` or `...}`/`...]`
- No closing brace/bracket unless preview fits completely

**Examples:**
```
{…}                                           // Empty or single field
(3) {id: 1, name: "Alice", active: true}      // Fits entirely
(5) {id: 1, name: "Alice", email: "al..."}    // Truncated string
(2) {user: {…}, count: 42}                    // With nested object
(3) {items: […], total: 100, page: 1}         // With nested array
(10) [1, 2, 3, 4, 5, 6, 7, 8, 9, ...]         // Truncated array
```

**Implementation Location:**
- `src/ui/tree_view.rs`: Modify `TreeViewLine::value_preview()` (around line 445)
- Add helper: `format_collapsed_preview(node: &JsonNode, max_chars: usize) -> String`

**Color/Styling:**
- Research jless source for exact ANSI color (appears cyan from screenshot)
- Add to theme system: `preview_color: Color` field
- Default to cyan/light blue for previews
- Apply via `Style::default().fg(theme.preview_color)`

**Display in Tree:**
```
∇ user: (3) {name: "Alice", email: "alice@...", age: 30}
∇ items: (10) [1, 2, 3, 4, 5, 6, 7, 8, 9, ...]
∇ config: (2) {debug: true, options: {…}}
  ▷ debug: true
  ∇ options: (5) {theme: "dark", size: 12, ...}
```

### 2. JSONL File Detection and Parsing

**File Extensions:**
- `.jsonl` (primary)
- `.ndjson` (alternative)

**Detection Logic:**
```rust
// In src/file/loader.rs
pub fn load_json_file(path: &str) -> Result<JsonTree> {
    if path.ends_with(".jsonl") || path.ends_with(".ndjson") {
        load_jsonl_file(path)
    } else {
        // Existing JSON loading
    }
}
```

**JSONL Parser:**
```rust
pub fn load_jsonl_file(path: &str) -> Result<JsonTree> {
    let contents = fs::read_to_string(path)?;
    let mut lines = Vec::new();

    for (line_num, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue; // Skip blank lines
        }

        let value: serde_json::Value = serde_json::from_str(line)
            .with_context(|| format!("Invalid JSON on line {}", line_num + 1))?;

        let node = parse_value(&value);
        lines.push(node);
    }

    // Wrap in special root variant
    let root = JsonNode::new(JsonValue::JsonlRoot(lines));
    Ok(JsonTree::new(root))
}
```

**Data Structure Changes:**
```rust
// In src/document/node.rs
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonNode>),
    Object(Vec<(String, JsonNode>)),
    JsonlRoot(Vec<JsonNode>),  // NEW: Special root for JSONL
}
```

**Why `JsonlRoot` variant:**
- Distinguishes JSONL from regular arrays
- Enables different rendering (flat vs nested)
- Signals different save format
- Preserves line boundaries during edits

### 3. JSONL Display

**Tree View Rendering:**
```
(5) {id: 1, name: "Alice", email: "alice@example.com", age: 30, ...}
(4) {id: 2, name: "Bob", age: 30, active: true}
(3) {id: 3, name: "Charlie", ...}
```

**Key Characteristics:**
- Flat list (no nesting indicators)
- No array brackets `[` `]` around entire document
- No line numbers/indexes shown (e.g., NOT `[0]`, `[1]`, `[2]`)
- Each line starts collapsed by default
- Shows collapsed object preview for each line
- User can expand lines to see full structure

**Expansion State:**
- JSONL: All lines collapsed by default
- Regular JSON: `expand_all()` by default (current behavior)
- User can toggle with `h`/`l` or `Space`

**Implementation:**
```rust
// In src/ui/tree_view.rs
fn render_jsonl_root(lines: &[JsonNode], state: &TreeViewState) -> Vec<TreeViewLine> {
    let mut result = Vec::new();

    for (idx, node) in lines.iter().enumerate() {
        let path = vec![idx];
        let is_expanded = state.is_expanded(&path);

        if is_expanded {
            // Show expanded tree for this line
            result.extend(render_node_tree(node, &path, state, 0));
        } else {
            // Show collapsed preview
            let preview = format_collapsed_preview(node, 60);
            result.push(TreeViewLine {
                path: path.clone(),
                depth: 0,
                key: None,
                value_preview: Some(preview),
                is_expandable: true,
                is_expanded: false,
            });
        }
    }

    result
}
```

### 4. JSONL Save Format

**Save Strategy:**
```rust
// In src/file/saver.rs
fn save_jsonl(tree: &JsonTree, path: &Path, config: &Config) -> Result<()> {
    let mut output = String::new();

    if let JsonValue::JsonlRoot(lines) = tree.root().value() {
        for node in lines {
            let json_value = node_to_serde_value(node);
            let line = serde_json::to_string(&json_value)?;
            output.push_str(&line);
            output.push('\n');
        }
    }

    atomic_write(path, output.as_bytes(), config)
}
```

**Format Requirements:**
- One JSON object per line
- No array brackets around document
- No commas between lines
- Preserve exact line order
- Each line is complete, valid JSON
- Trailing newline after last line

**Example Output:**
```jsonl
{"id":1,"name":"Alice","email":"alice@example.com","age":30,"active":true}
{"id":2,"name":"Bob","age":30,"active":true}
{"id":3,"name":"Charlie","email":"charlie@example.com"}
```

**Save Detection:**
```rust
// In src/file/saver.rs - modify save_json_file()
pub fn save_json_file(tree: &JsonTree, path: &Path, config: &Config) -> Result<()> {
    if matches!(tree.root().value(), JsonValue::JsonlRoot(_)) {
        save_jsonl(tree, path, config)
    } else {
        // Existing JSON save logic
    }
}
```

### 5. JSONL Edit Operations

**Add Operation (`a`):**
- **Cursor on collapsed JSONL line**: Expands line, then prompts for key → value (adds field to object)
- **Cursor on expanded line's object**: Standard object add (key → value prompt)
- **Cursor on expanded line's array**: Standard array add (value prompt)
- **Cannot add new lines at root**: Would need separate command (`:append` or `o` - not in scope)

**Delete Operation (`dd`):**
- **Cursor on collapsed JSONL line**: Deletes entire line/object
- **Cursor inside expanded line**: Deletes that field/subtree (standard behavior)
- **Deleting all lines**: Allowed (creates empty JSONL file)
- **Cannot delete root**: JSONL root itself is undeletable

**Paste Operation (`p`/`P`):**
- **Cursor on collapsed JSONL line**: Inserts yanked content as new line after (`p`) or before (`P`)
- **Cursor inside expanded line**: Standard paste within object/array
- **Pasting object/array**: Creates new JSONL line
- **Pasting scalar at root**: Not allowed (must paste inside a line)

**Yank Operation (`yy`):**
- **Cursor on collapsed JSONL line**: Yanks entire object
- **Cursor inside expanded line**: Yanks that subtree (standard behavior)

**Edit Operation (`i`):**
- **Cursor on collapsed JSONL line**: Not allowed (must expand first to edit)
- **Cursor inside expanded line**: Standard value editing

**Undo/Redo:**
- Works normally - checkpoints capture entire JSONL state
- Line additions/deletions are undoable
- Edits within lines are undoable

### 6. Integration Points

**Files to Modify:**

1. **`src/document/node.rs`**:
   - Add `JsonValue::JsonlRoot(Vec<JsonNode>)` variant
   - Update all `match` statements handling `JsonValue`

2. **`src/file/loader.rs`**:
   - Add `load_jsonl_file()` function
   - Update `load_json_file()` to detect `.jsonl`/`.ndjson` extensions

3. **`src/file/saver.rs`**:
   - Add `save_jsonl()` function
   - Update `save_json_file()` to detect `JsonValue::JsonlRoot`

4. **`src/ui/tree_view.rs`**:
   - Add `format_collapsed_preview()` helper
   - Update `TreeViewLine::value_preview()` to use collapsed preview
   - Add `render_jsonl_root()` for JSONL display
   - Update `render_tree()` to handle `JsonValue::JsonlRoot`

5. **`src/theme/mod.rs`**:
   - Add `preview_color: Color` field to `Theme`
   - Set default to cyan/light blue
   - Update all theme definitions

6. **`src/editor/state.rs`**:
   - Update add/delete/paste operations to handle `JsonValue::JsonlRoot`
   - Ensure path calculations work for JSONL flat structure

7. **`src/document/tree.rs`**:
   - Update `insert_node_in_object()` to handle JSONL root
   - Update `delete_node()` to handle JSONL root
   - May need `insert_jsonl_line()` and `delete_jsonl_line()` helpers

**Error Handling:**
- Invalid JSON on line N: Show line number in error message
- Empty JSONL file: Allowed (root with empty lines vec)
- Mixed content: Not allowed (must be all objects, or detect arrays/scalars?)

**Edge Cases:**
- JSONL with blank lines: Skip during parse
- JSONL with comments: Not supported (not valid JSON)
- JSONL with arrays/scalars: Allow? Or require objects only?
- Very long collapsed preview: Truncate at 60 chars with `...`
- Deeply nested collapsed object: Show as `{…}` after first level

## Testing Strategy

**Unit Tests:**
- `format_collapsed_preview()` with various node types
- `load_jsonl_file()` with valid/invalid JSONL
- `save_jsonl()` preserves line order
- Collapsed preview truncation logic

**Integration Tests:**
- Load JSONL → edit → save → verify format
- Add field to JSONL line
- Delete JSONL line
- Yank/paste JSONL lines
- Undo/redo JSONL operations
- Regular JSON with collapsed preview display

**Manual Testing:**
- Create sample `.jsonl` file
- Verify collapsed display matches jless style
- Test all edit operations
- Verify save format
- Test with large JSONL files (100+ lines)

## Implementation Notes

**YAGNI Reminders:**
- No need to add new JSONL lines yet (`:append`, `o` command)
- No need to detect line number during search
- No need to optimize for huge JSONL files (lazy loading) in first version
- No need to preserve original formatting (compact JSON output is fine)

**Future Enhancements (Not in Scope):**
- `:append` command to add new JSONL lines at root
- `o`/`O` commands for adding sibling JSONL lines
- JSONL-specific search (jump to line N)
- Mixed JSONL (arrays/objects/scalars on different lines)
- Streaming JSONL for huge files

## Success Criteria

1. ✅ `.jsonl` files load correctly (line-by-line parsing)
2. ✅ JSONL displays as flat list with collapsed previews
3. ✅ Collapsed preview format matches jless style
4. ✅ Can expand/collapse JSONL lines
5. ✅ Can edit fields within JSONL lines
6. ✅ Can delete JSONL lines
7. ✅ Can yank/paste JSONL lines
8. ✅ JSONL saves back in correct format (line-by-line)
9. ✅ Regular JSON shows collapsed previews too
10. ✅ All tests passing
