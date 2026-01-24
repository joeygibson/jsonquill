# JSONL Support and Collapsed Object Preview Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add JSONL file support with jless-style collapsed object/array previews for both JSONL and regular JSON.

**Architecture:** Add `JsonValue::JsonlRoot` variant to distinguish JSONL from arrays, implement `format_collapsed_preview()` for inline previews, modify tree rendering to handle JSONL flat display, and add JSONL-specific save logic.

**Tech Stack:** Rust, ratatui, serde_json, anyhow

---

## Task 1: Add JsonValue::JsonlRoot variant

**Files:**
- Modify: `src/document/node.rs:35-48` (JsonValue enum)
- Test: Tests will be added in subsequent tasks

**Step 1: Add JsonlRoot variant to JsonValue enum**

Add new variant after `Null` in the `JsonValue` enum:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// A JSON object containing key-value pairs
    Object(Vec<(String, JsonNode)>),
    /// A JSON array containing ordered values
    Array(Vec<JsonNode>),
    /// A JSON string
    String(String),
    /// A JSON number (represented as f64)
    Number(f64),
    /// A JSON boolean
    Boolean(bool),
    /// A JSON null value
    Null,
    /// A JSONL document root containing lines (each line is a JsonNode)
    JsonlRoot(Vec<JsonNode>),
}
```

**Step 2: Update is_container() method**

Find the `is_container()` method in `src/document/node.rs` and update to include `JsonlRoot`:

```rust
pub fn is_container(&self) -> bool {
    matches!(self, JsonValue::Object(_) | JsonValue::Array(_) | JsonValue::JsonlRoot(_))
}
```

**Step 3: Update is_object() method**

Update `is_object()` to NOT match `JsonlRoot`:

```rust
pub fn is_object(&self) -> bool {
    matches!(self, JsonValue::Object(_))
}
```

**Step 4: Update is_array() method**

Update `is_array()` to NOT match `JsonlRoot`:

```rust
pub fn is_array(&self) -> bool {
    matches!(self, JsonValue::Array(_))
}
```

**Step 5: Build to check for compilation errors**

Run: `cargo build`
Expected: Multiple compilation errors in files that match on `JsonValue` (tree.rs, parser.rs, saver.rs, etc.)

**Step 6: Fix tree.rs get_node() method**

In `src/document/tree.rs`, update `get_node()` method to handle `JsonlRoot`:

```rust
pub fn get_node(&self, path: &[usize]) -> Option<&JsonNode> {
    let mut current = &self.root;

    for &index in path {
        match current.value() {
            JsonValue::Object(entries) => {
                current = &entries.get(index)?.1;
            }
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                current = elements.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}
```

**Step 7: Fix tree.rs get_node_mut() method**

Update `get_node_mut()` similarly:

```rust
pub fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut JsonNode> {
    let mut current = &mut self.root;

    for &index in path {
        match current.value_mut() {
            JsonValue::Object(entries) => {
                current = &mut entries.get_mut(index)?.1;
            }
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                current = elements.get_mut(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}
```

**Step 8: Fix tree.rs delete_node() method**

Update `delete_node()` to handle `JsonlRoot`:

```rust
// In the match statement around line 211
match parent.value_mut() {
    JsonValue::Object(entries) => {
        if index >= entries.len() {
            return Err(anyhow!(/*...*/));
        }
        entries.remove(index);
    }
    JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
        if index >= elements.len() {
            return Err(anyhow!(/*...*/));
        }
        elements.remove(index);
    }
    _ => {
        return Err(anyhow!("Parent is not a container type"));
    }
}
```

**Step 9: Fix tree.rs insert_node_in_array() method**

Update `insert_node_in_array()` to handle `JsonlRoot`:

```rust
// In the match statement around line 299
match target.value_mut() {
    JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
        if index > elements.len() {
            return Err(anyhow!(/*...*/));
        }
        elements.insert(index, node);
    }
    _ => {
        return Err(anyhow!("Target is not an array"));
    }
}
```

**Step 10: Fix parser.rs parse_value() function**

In `src/document/parser.rs`, update `parse_value()` to never create `JsonlRoot` (only created by JSONL loader):

No changes needed - `JsonlRoot` is only created explicitly in `load_jsonl_file()`.

**Step 11: Fix saver.rs node_to_serde_value() function**

In `src/document/saver.rs`, find `node_to_serde_value()` and add `JsonlRoot` case:

```rust
fn node_to_serde_value(node: &JsonNode) -> serde_json::Value {
    match node.value() {
        JsonValue::Null => serde_json::Value::Null,
        JsonValue::Boolean(b) => serde_json::Value::Bool(*b),
        JsonValue::Number(n) => {
            serde_json::Number::from_f64(*n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        JsonValue::String(s) => serde_json::Value::String(s.clone()),
        JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
            serde_json::Value::Array(
                elements.iter().map(node_to_serde_value).collect()
            )
        }
        JsonValue::Object(entries) => {
            let map = entries
                .iter()
                .map(|(k, v)| (k.clone(), node_to_serde_value(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}
```

**Step 12: Build to verify all match errors fixed**

Run: `cargo build`
Expected: SUCCESS - all non-exhaustive pattern errors resolved

**Step 13: Run tests**

Run: `cargo test`
Expected: All tests pass (no test changes needed yet)

**Step 14: Commit**

```bash
cd /Users/jgibson/Projects/jsonquill-worktrees/jsonl-support
git add src/document/node.rs src/document/tree.rs src/file/saver.rs
git commit -m "feat: add JsonValue::JsonlRoot variant for JSONL documents

Add new variant to distinguish JSONL documents from regular arrays.
Update all match statements to handle JsonlRoot alongside Array.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add JSONL file loading

**Files:**
- Modify: `src/file/loader.rs`
- Create: `tests/jsonl_tests.rs`

**Step 1: Write failing test for load_jsonl_file()**

Create `tests/jsonl_tests.rs`:

```rust
use jsonquill::document::node::JsonValue;
use jsonquill::file::loader::load_json_file;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_load_simple_jsonl_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.jsonl");

    let jsonl_content = r#"{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
{"id":3,"name":"Charlie"}"#;

    fs::write(&file_path, jsonl_content).unwrap();

    let tree = load_json_file(&file_path).unwrap();

    // Should be JsonlRoot with 3 lines
    match tree.root().value() {
        JsonValue::JsonlRoot(lines) => {
            assert_eq!(lines.len(), 3);

            // Check first line
            if let JsonValue::Object(fields) = lines[0].value() {
                assert_eq!(fields.len(), 2);
            } else {
                panic!("Expected object");
            }
        }
        _ => panic!("Expected JsonlRoot"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_load_simple_jsonl_file`
Expected: FAIL - function doesn't exist yet

**Step 3: Implement load_jsonl_file() function**

In `src/file/loader.rs`, add at the bottom before the `#[cfg(test)]` section:

```rust
/// Loads and parses a JSONL (JSON Lines) file from the filesystem.
///
/// Each line in the file must be a valid JSON value. Blank lines are skipped.
/// The result is a JsonTree with a JsonlRoot containing all lines.
pub fn load_jsonl_file<P: AsRef<Path>>(path: P) -> Result<JsonTree> {
    use crate::document::node::{JsonNode, JsonValue};

    let content = fs::read_to_string(path.as_ref())
        .context("Failed to read JSONL file")?;

    let mut lines = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue; // Skip blank lines
        }

        let value: serde_json::Value = serde_json::from_str(line)
            .with_context(|| format!("Invalid JSON on line {}", line_num + 1))?;

        let node = parse_value(&value);
        lines.push(node);
    }

    let root = JsonNode::new(JsonValue::JsonlRoot(lines));
    Ok(JsonTree::new(root))
}
```

**Step 4: Make parse_value accessible**

At the top of `src/file/loader.rs`, change the import:

```rust
use crate::document::parser::parse_json;
// Add:
use crate::document::parser::parse_value;
```

And in `src/document/parser.rs`, make `parse_value` public:

```rust
pub fn parse_value(value: &serde_json::Value) -> JsonNode {
    // existing implementation
}
```

**Step 5: Update load_json_file() to detect JSONL extension**

In `src/file/loader.rs`, update `load_json_file()`:

```rust
pub fn load_json_file<P: AsRef<Path>>(path: P) -> Result<JsonTree> {
    let path_ref = path.as_ref();

    // Check if this is a JSONL file
    if let Some(ext) = path_ref.extension() {
        if ext == "jsonl" || ext == "ndjson" {
            return load_jsonl_file(path_ref);
        }
    }

    // Regular JSON
    let content = fs::read_to_string(path_ref)
        .context("Failed to read file")?;
    parse_json(&content).context("Failed to parse JSON")
}
```

**Step 6: Run test to verify it passes**

Run: `cargo test test_load_simple_jsonl_file`
Expected: PASS

**Step 7: Write test for blank lines**

Add to `tests/jsonl_tests.rs`:

```rust
#[test]
fn test_load_jsonl_skips_blank_lines() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.jsonl");

    let jsonl_content = r#"{"id":1}

{"id":2}

{"id":3}"#;

    fs::write(&file_path, jsonl_content).unwrap();

    let tree = load_json_file(&file_path).unwrap();

    match tree.root().value() {
        JsonValue::JsonlRoot(lines) => {
            assert_eq!(lines.len(), 3); // Blank lines skipped
        }
        _ => panic!("Expected JsonlRoot"),
    }
}
```

**Step 8: Run test**

Run: `cargo test test_load_jsonl_skips_blank_lines`
Expected: PASS

**Step 9: Write test for invalid JSON line**

Add to `tests/jsonl_tests.rs`:

```rust
#[test]
fn test_load_jsonl_invalid_line() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.jsonl");

    let jsonl_content = r#"{"id":1}
{invalid json}
{"id":3}"#;

    fs::write(&file_path, jsonl_content).unwrap();

    let result = load_json_file(&file_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("line 2"));
}
```

**Step 10: Run test**

Run: `cargo test test_load_jsonl_invalid_line`
Expected: PASS

**Step 11: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 12: Commit**

```bash
git add src/file/loader.rs src/document/parser.rs tests/jsonl_tests.rs
git commit -m "feat: add JSONL file loading support

Detect .jsonl and .ndjson extensions and parse line-by-line.
Create JsonlRoot with vector of parsed lines.
Skip blank lines, report line numbers in errors.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Add collapsed preview formatting

**Files:**
- Modify: `src/ui/tree_view.rs`
- Test: Unit tests in same file

**Step 1: Write test for format_collapsed_preview() with simple object**

Add to `src/ui/tree_view.rs` in the `#[cfg(test)]` section:

```rust
#[test]
fn test_format_collapsed_preview_simple_object() {
    use crate::document::node::{JsonNode, JsonValue};

    let obj = JsonNode::new(JsonValue::Object(vec![
        ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ]));

    let preview = format_collapsed_preview(&obj, 100);
    assert_eq!(preview, "(2) {id: 1, name: \"Alice\"}");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_format_collapsed_preview_simple_object`
Expected: FAIL - function doesn't exist

**Step 3: Implement format_collapsed_preview() function**

Add before the `#[cfg(test)]` section in `src/ui/tree_view.rs`:

```rust
/// Formats a collapsed preview of a JSON node similar to jless.
///
/// Format: (N) {key1: val1, key2: val2, ...} for objects
///         (N) [elem1, elem2, ...] for arrays
///
/// Truncates at max_chars with "..." if needed.
pub fn format_collapsed_preview(node: &JsonNode, max_chars: usize) -> String {
    match node.value() {
        JsonValue::Object(fields) => format_collapsed_object(fields, max_chars),
        JsonValue::Array(elements) => format_collapsed_array(elements, max_chars),
        JsonValue::JsonlRoot(lines) => {
            // Shouldn't happen, but treat like array
            format_collapsed_array(lines, max_chars)
        }
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        JsonValue::Boolean(b) => format!("{}", b),
        JsonValue::Null => "null".to_string(),
    }
}

fn format_collapsed_object(fields: &[(String, JsonNode)], max_chars: usize) -> String {
    if fields.is_empty() {
        return "{…}".to_string();
    }

    let count = fields.len();
    let mut preview = format!("({}) {{", count);

    for (i, (key, value)) in fields.iter().enumerate() {
        if preview.len() >= max_chars {
            preview.push_str("...");
            break;
        }

        // Add key
        preview.push_str(key);
        preview.push_str(": ");

        // Add value
        let value_str = match value.value() {
            JsonValue::Object(_) => "{…}".to_string(),
            JsonValue::Array(_) | JsonValue::JsonlRoot(_) => "[…]".to_string(),
            JsonValue::String(s) => {
                let quoted = format!("\"{}\"", s);
                if preview.len() + quoted.len() > max_chars {
                    format!("\"{}...\"", &s[..s.len().min(10)])
                } else {
                    quoted
                }
            }
            JsonValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            JsonValue::Boolean(b) => format!("{}", b),
            JsonValue::Null => "null".to_string(),
        };

        preview.push_str(&value_str);

        // Add comma if not last
        if i < fields.len() - 1 {
            preview.push_str(", ");
        }
    }

    // Close brace if we have room
    if preview.len() < max_chars {
        preview.push('}');
    }

    preview
}

fn format_collapsed_array(elements: &[JsonNode], max_chars: usize) -> String {
    if elements.is_empty() {
        return "[…]".to_string();
    }

    let count = elements.len();
    let mut preview = format!("({}) [", count);

    for (i, element) in elements.iter().enumerate() {
        if preview.len() >= max_chars {
            preview.push_str("...");
            break;
        }

        let value_str = match element.value() {
            JsonValue::Object(_) => "{…}".to_string(),
            JsonValue::Array(_) | JsonValue::JsonlRoot(_) => "[…]".to_string(),
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            JsonValue::Boolean(b) => format!("{}", b),
            JsonValue::Null => "null".to_string(),
        };

        preview.push_str(&value_str);

        if i < elements.len() - 1 {
            preview.push_str(", ");
        }
    }

    if preview.len() < max_chars {
        preview.push(']');
    }

    preview
}
```

**Step 4: Run test**

Run: `cargo test test_format_collapsed_preview_simple_object`
Expected: PASS

**Step 5: Write test for nested objects**

Add test:

```rust
#[test]
fn test_format_collapsed_preview_nested_object() {
    use crate::document::node::{JsonNode, JsonValue};

    let obj = JsonNode::new(JsonValue::Object(vec![
        ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("user".to_string(), JsonNode::new(JsonValue::Object(vec![
            ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
        ]))),
    ]));

    let preview = format_collapsed_preview(&obj, 100);
    assert_eq!(preview, "(2) {id: 1, user: {…}}");
}
```

**Step 6: Run test**

Run: `cargo test test_format_collapsed_preview_nested_object`
Expected: PASS

**Step 7: Write test for array**

Add test:

```rust
#[test]
fn test_format_collapsed_preview_array() {
    use crate::document::node::{JsonNode, JsonValue};

    let arr = JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ]));

    let preview = format_collapsed_preview(&arr, 100);
    assert_eq!(preview, "(3) [1, 2, 3]");
}
```

**Step 8: Run test**

Run: `cargo test test_format_collapsed_preview_array`
Expected: PASS

**Step 9: Write test for truncation**

Add test:

```rust
#[test]
fn test_format_collapsed_preview_truncation() {
    use crate::document::node::{JsonNode, JsonValue};

    let obj = JsonNode::new(JsonValue::Object(vec![
        ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
        ("email".to_string(), JsonNode::new(JsonValue::String("alice@example.com".to_string()))),
        ("active".to_string(), JsonNode::new(JsonValue::Boolean(true))),
    ]));

    let preview = format_collapsed_preview(&obj, 40);
    assert!(preview.len() <= 43); // Allow a bit of overflow for "..."
    assert!(preview.contains("..."));
}
```

**Step 10: Run test**

Run: `cargo test test_format_collapsed_preview_truncation`
Expected: PASS

**Step 11: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 12: Commit**

```bash
git add src/ui/tree_view.rs
git commit -m "feat: add collapsed preview formatting like jless

Implement format_collapsed_preview() with format:
- Objects: (N) {key1: val1, key2: val2, ...}
- Arrays: (N) [elem1, elem2, ...]
- Nested containers shown as {…} or […]
- Truncate at max_chars with ...

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Use collapsed preview in tree view

**Files:**
- Modify: `src/ui/tree_view.rs` (TreeViewLine struct and rendering)

**Step 1: Update TreeViewLine struct**

Find the `TreeViewLine` struct around line 17 and update:

```rust
#[derive(Debug, Clone)]
pub struct TreeViewLine {
    pub path: Vec<usize>,
    pub depth: usize,
    pub key: Option<String>,
    pub value_type: ValueType,
    pub value_preview: String,
    pub expandable: bool,
    pub expanded: bool,
}
```

Note: `value_preview` is already a `String`, so no changes needed here. But we need to update how it's generated.

**Step 2: Find value preview generation logic**

Search for where `value_preview` is currently set. It should be in the tree rendering functions. Look for patterns like `"{ 3 fields }"` or `"[ 5 items ]"`.

Expected location: Around line 400-500 in rendering functions.

**Step 3: Replace old preview logic with collapsed preview**

Find the function that generates value previews (likely `render_tree_lines` or similar) and replace the old logic:

OLD:
```rust
let value_preview = match value {
    JsonValue::Object(fields) => format!("{{ {} fields }}", fields.len()),
    JsonValue::Array(elements) => format!("[ {} items ]", elements.len()),
    // ...
};
```

NEW:
```rust
let value_preview = if !is_expanded && node.value().is_container() {
    format_collapsed_preview(node, 60)
} else {
    match value {
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Number(n) => /* existing logic */,
        // ... other scalars
        _ => String::new(), // Containers handled by collapsed preview
    }
};
```

**Step 4: Build to check for errors**

Run: `cargo build`
Expected: May have some errors if we need to update more locations

**Step 5: Run tests**

Run: `cargo test`
Expected: Some tests may fail if they check exact preview format

**Step 6: Update test expectations**

Find tests like `test_value_preview` and update to expect new format:

OLD: `"{ 3 fields }"`
NEW: `"(3) {key1: val1, ...}"`

**Step 7: Run tests again**

Run: `cargo test`
Expected: All tests pass

**Step 8: Manual test with sample JSON**

Create `test.json`:
```json
{
  "user": {"name": "Alice", "age": 30},
  "items": [1, 2, 3, 4, 5]
}
```

Run: `cargo run -- test.json`

Expected: See collapsed previews like `(2) {name: "Alice", age: 30}` when collapsed

**Step 9: Commit**

```bash
git add src/ui/tree_view.rs
git commit -m "feat: use collapsed preview in tree view display

Replace \"{ N fields }\" / \"[ N items ]\" with jless-style previews.
Show inline content when collapsed instead of just counts.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Add JSONL tree view rendering

**Files:**
- Modify: `src/ui/tree_view.rs` (add render_jsonl_root function)

**Step 1: Write test for JSONL rendering**

Add to tests in `src/ui/tree_view.rs`:

```rust
#[test]
fn test_render_jsonl_root() {
    use crate::document::node::{JsonNode, JsonValue};
    use crate::document::tree::JsonTree;

    let lines = vec![
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ])),
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ])),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));
    let mut state = TreeViewState::new();
    state.rebuild(&tree);

    // Should have 2 lines, both collapsed
    assert_eq!(state.lines().len(), 2);
    assert!(state.lines()[0].value_preview.contains("id: 1"));
    assert_eq!(state.lines()[0].depth, 0);
    assert!(state.lines()[0].expandable);
    assert!(!state.lines()[0].expanded);
}
```

**Step 2: Run test to verify failure**

Run: `cargo test test_render_jsonl_root`
Expected: FAIL - rendering not implemented yet

**Step 3: Add render_jsonl_root() function**

Add to `src/ui/tree_view.rs`:

```rust
fn render_jsonl_root(
    lines: &[JsonNode],
    state: &TreeViewState,
    result: &mut Vec<TreeViewLine>,
) {
    for (idx, node) in lines.iter().enumerate() {
        let path = vec![idx];
        let is_expanded = state.is_expanded(&path);

        if is_expanded {
            // Render expanded tree for this line
            render_node(&path, node, 0, state, result);
        } else {
            // Show collapsed preview
            let preview = format_collapsed_preview(node, 60);
            result.push(TreeViewLine {
                path: path.clone(),
                depth: 0,
                key: None,
                value_type: ValueType::from_json_value(node.value()),
                value_preview: preview,
                expandable: true,
                expanded: false,
            });
        }
    }
}
```

**Step 4: Update main render function to call render_jsonl_root()**

Find the main tree rendering function (likely `rebuild()` or `render_tree()`) and add case for `JsonlRoot`:

```rust
pub fn rebuild(&mut self, tree: &JsonTree) {
    self.lines.clear();

    match tree.root().value() {
        JsonValue::JsonlRoot(lines) => {
            render_jsonl_root(lines, self, &mut self.lines);
        }
        _ => {
            // Existing rendering logic for regular JSON
            render_node(&vec![], tree.root(), 0, self, &mut self.lines);
        }
    }
}
```

**Step 5: Ensure JSONL lines start collapsed**

In the `rebuild()` function, don't call `expand_all()` for JSONL:

```rust
pub fn rebuild(&mut self, tree: &JsonTree) {
    self.lines.clear();

    match tree.root().value() {
        JsonValue::JsonlRoot(lines) => {
            // Don't expand - start collapsed
            render_jsonl_root(lines, self, &mut self.lines);
        }
        _ => {
            // Expand regular JSON by default
            self.expand_all(tree);
            render_node(&vec![], tree.root(), 0, self, &mut self.lines);
        }
    }
}
```

**Step 6: Run test**

Run: `cargo test test_render_jsonl_root`
Expected: PASS

**Step 7: Test expanding JSONL line**

Add test:

```rust
#[test]
fn test_expand_jsonl_line() {
    use crate::document::node::{JsonNode, JsonValue};
    use crate::document::tree::JsonTree;

    let lines = vec![
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
        ])),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));
    let mut state = TreeViewState::new();
    state.rebuild(&tree);

    // Initially collapsed
    assert_eq!(state.lines().len(), 1);

    // Expand first line
    state.toggle_expand(&vec![0]);
    state.rebuild(&tree);

    // Should now show 3 lines: object + 2 fields
    assert!(state.lines().len() > 1);
}
```

**Step 8: Run test**

Run: `cargo test test_expand_jsonl_line`
Expected: PASS

**Step 9: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 10: Commit**

```bash
git add src/ui/tree_view.rs
git commit -m "feat: add JSONL tree view rendering

Implement render_jsonl_root() for flat display of JSONL lines.
Lines start collapsed by default showing preview.
Can expand to see full object structure.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Add JSONL save functionality

**Files:**
- Modify: `src/file/saver.rs`
- Test: `tests/jsonl_tests.rs`

**Step 1: Write test for JSONL save**

Add to `tests/jsonl_tests.rs`:

```rust
#[test]
fn test_save_jsonl_format() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::file::saver::save_json_file;
    use jsonquill::config::Config;
    use std::fs;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("output.jsonl");

    let lines = vec![
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
        ])),
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(2.0))),
            ("name".to_string(), JsonNode::new(JsonValue::String("Bob".to_string()))),
        ])),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));
    let config = Config::default();

    save_json_file(&tree, &file_path, &config).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("\"id\":1"));
    assert!(lines[0].contains("\"name\":\"Alice\""));
    assert!(lines[1].contains("\"id\":2"));

    // No array brackets
    assert!(!content.starts_with('['));
    assert!(!content.ends_with(']'));
}
```

**Step 2: Run test to verify failure**

Run: `cargo test test_save_jsonl_format`
Expected: FAIL - function not implemented

**Step 3: Add save_jsonl() function**

In `src/file/saver.rs`, add before the `#[cfg(test)]` section:

```rust
/// Saves a JSONL document to a file.
///
/// Each line is saved as a separate JSON object (one per line).
fn save_jsonl(tree: &JsonTree, path: &Path, config: &Config) -> Result<()> {
    use crate::document::node::JsonValue;

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

**Step 4: Update save_json_file() to detect JsonlRoot**

Find `save_json_file()` and update:

```rust
pub fn save_json_file(tree: &JsonTree, path: &Path, config: &Config) -> Result<()> {
    use crate::document::node::JsonValue;

    // Check if this is a JSONL document
    if matches!(tree.root().value(), JsonValue::JsonlRoot(_)) {
        return save_jsonl(tree, path, config);
    }

    // Regular JSON save logic
    // ... existing code
}
```

**Step 5: Run test**

Run: `cargo test test_save_jsonl_format`
Expected: PASS

**Step 6: Test round-trip (load → save → load)**

Add test:

```rust
#[test]
fn test_jsonl_roundtrip() {
    use jsonquill::file::loader::load_json_file;
    use jsonquill::file::saver::save_json_file;
    use jsonquill::config::Config;
    use jsonquill::document::node::JsonValue;
    use std::fs;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.jsonl");

    let original = r#"{"id":1,"name":"Alice","active":true}
{"id":2,"name":"Bob","active":false}
{"id":3,"name":"Charlie","active":true}"#;

    fs::write(&file_path, original).unwrap();

    // Load
    let tree = load_json_file(&file_path).unwrap();

    // Save
    let config = Config::default();
    save_json_file(&tree, &file_path, &config).unwrap();

    // Load again
    let tree2 = load_json_file(&file_path).unwrap();

    // Verify structure preserved
    match tree2.root().value() {
        JsonValue::JsonlRoot(lines) => {
            assert_eq!(lines.len(), 3);
        }
        _ => panic!("Expected JsonlRoot"),
    }
}
```

**Step 7: Run test**

Run: `cargo test test_jsonl_roundtrip`
Expected: PASS

**Step 8: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 9: Commit**

```bash
git add src/file/saver.rs tests/jsonl_tests.rs
git commit -m "feat: add JSONL save functionality

Implement save_jsonl() to write one JSON object per line.
Detect JsonlRoot in save_json_file() and route to JSONL saver.
Preserve line order and format.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Add preview color to theme system

**Files:**
- Modify: `src/theme/colors.rs`
- Modify: `src/theme/mod.rs`

**Step 1: Add preview_color to ThemeColors**

In `src/theme/colors.rs`, find the `ThemeColors` struct and add field:

```rust
pub struct ThemeColors {
    // ... existing fields
    pub preview: Color, // NEW: Color for collapsed previews
}
```

**Step 2: Update default_dark() theme**

In the same file, update `default_dark()`:

```rust
pub fn default_dark() -> Self {
    Self {
        // ... existing colors
        preview: Color::Cyan, // Cyan for collapsed previews
    }
}
```

**Step 3: Update default_light() theme**

Update `default_light()`:

```rust
pub fn default_light() -> Self {
    Self {
        // ... existing colors
        preview: Color::Blue, // Blue for collapsed previews in light theme
    }
}
```

**Step 4: Build to check for compilation errors**

Run: `cargo build`
Expected: May have errors where ThemeColors is constructed in tests

**Step 5: Fix test compilation errors**

Find any tests that construct `ThemeColors` and add `preview: Color::Cyan` field.

**Step 6: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 7: Commit**

```bash
git add src/theme/colors.rs src/theme/mod.rs
git commit -m "feat: add preview_color to theme system

Add preview field to ThemeColors for collapsed preview styling.
Default to Cyan for dark theme, Blue for light theme.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Apply preview color in tree view rendering

**Files:**
- Modify: `src/ui/tree_view.rs` (or wherever rendering happens with theme)
- Modify: `src/ui/mod.rs` (may need to pass theme to rendering)

**Step 1: Find where tree view lines are rendered with colors**

Look for the rendering code that applies theme colors to tree view output. This is likely in `src/ui/mod.rs` or a rendering module.

Expected: Function that takes `TreeViewLine` and `Theme` and produces styled output.

**Step 2: Apply preview color to collapsed previews**

In the rendering code, when displaying `value_preview` for collapsed containers:

```rust
// OLD:
let preview_style = Style::default().fg(theme.colors.value_string);

// NEW:
let preview_style = if line.expandable && !line.expanded {
    Style::default().fg(theme.colors.preview)
} else {
    // Regular value color
    Style::default().fg(/* appropriate color */)
};
```

**Step 3: Build**

Run: `cargo build`
Expected: SUCCESS

**Step 4: Manual test**

Create `test.json`:
```json
{"user": {"name": "Alice"}, "items": [1, 2, 3]}
```

Run: `cargo run -- test.json`

Expected: Collapsed previews appear in cyan color

**Step 5: Commit**

```bash
git add src/ui/mod.rs src/ui/tree_view.rs
git commit -m "feat: apply preview color to collapsed previews

Use theme.colors.preview for collapsed object/array displays.
Distinguishes previews visually from regular values.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Update documentation

**Files:**
- Modify: `README.md`
- Modify: `CLAUDE.md`

**Step 1: Update README.md with JSONL support**

Add to README.md after the stdin piping section:

```markdown
### JSONL Support

JSON Quill supports JSONL (JSON Lines) files with the `.jsonl` or `.ndjson` extension:

```bash
# Open a JSONL file
jsonquill data.jsonl

# Each line displays as a collapsed object
# Press l or → to expand a line
# Edit fields within expanded lines normally
```

**JSONL Features:**
- Each line parsed as separate JSON object
- Lines start collapsed showing preview
- Flat display (no nesting at root level)
- Save preserves line-by-line format
- All edit operations work within lines
```

**Step 2: Update CLAUDE.md status**

In `CLAUDE.md`, update the "Working Features" section:

```markdown
- ✅ JSONL (.jsonl, .ndjson) file support
- ✅ Collapsed object/array previews (jless-style)
```

Remove from "Known Issues":

```markdown
- ❌ **No JSONL support** - Line-based JSON editing not implemented
```

**Step 3: Add to CLAUDE.md architecture section**

Add under "Key Dependencies":

```markdown
**JSONL Handling:**
- `JsonValue::JsonlRoot` variant distinguishes JSONL from regular arrays
- Flat rendering in tree view (no root container)
- Separate save logic (one JSON object per line)
- Lines stored as `Vec<JsonNode>` in JsonlRoot variant
```

**Step 4: Commit**

```bash
git add README.md CLAUDE.md
git commit -m "docs: add JSONL support documentation

Document JSONL file support in README and CLAUDE.md.
Update feature status and architecture notes.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Integration testing and polish

**Files:**
- Create: `tests/integration_jsonl.rs`
- Create: `examples/sample.jsonl`

**Step 1: Create sample JSONL file**

Create `examples/sample.jsonl`:

```jsonl
{"id":1,"name":"Alice Smith","email":"alice@example.com","age":30,"active":true,"roles":["admin","user"]}
{"id":2,"name":"Bob Jones","email":"bob@example.com","age":25,"active":true,"roles":["user"]}
{"id":3,"name":"Charlie Brown","email":"charlie@example.com","age":35,"active":false,"roles":["user","guest"]}
```

**Step 2: Create integration test file**

Create `tests/integration_jsonl.rs`:

```rust
use jsonquill::document::node::JsonValue;
use jsonquill::document::tree::JsonTree;
use jsonquill::file::loader::load_json_file;
use jsonquill::file::saver::save_json_file;
use jsonquill::config::Config;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_full_jsonl_workflow() {
    // Load sample JSONL
    let tree = load_json_file("examples/sample.jsonl").unwrap();

    // Verify structure
    match tree.root().value() {
        JsonValue::JsonlRoot(lines) => {
            assert_eq!(lines.len(), 3);
        }
        _ => panic!("Expected JsonlRoot"),
    }

    // Save to temp file
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("output.jsonl");
    let config = Config::default();

    save_json_file(&tree, &output_path, &config).unwrap();

    // Verify format
    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content.lines().count(), 3);
    assert!(!content.contains('['));
    assert!(!content.contains("[\n"));
}

#[test]
fn test_edit_jsonl_line() {
    use jsonquill::document::node::JsonNode;

    // Load sample JSONL
    let mut tree = load_json_file("examples/sample.jsonl").unwrap();

    // Get first line, first field
    if let JsonValue::JsonlRoot(lines) = tree.root_mut().value_mut() {
        if let JsonValue::Object(fields) = lines[0].value_mut() {
            // Change name
            if let JsonValue::String(ref mut name) = fields[1].1.value_mut() {
                *name = "Alice Johnson".to_string();
            }
        }
    }

    // Save
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("edited.jsonl");
    let config = Config::default();

    save_json_file(&tree, &output_path, &config).unwrap();

    // Reload and verify
    let tree2 = load_json_file(&output_path).unwrap();

    if let JsonValue::JsonlRoot(lines) = tree2.root().value() {
        if let JsonValue::Object(fields) = lines[0].value() {
            if let JsonValue::String(name) = fields[1].1.value() {
                assert_eq!(name, "Alice Johnson");
            }
        }
    }
}

#[test]
fn test_delete_jsonl_line() {
    use jsonquill::document::node::JsonNode;

    // Load sample JSONL
    let mut tree = load_json_file("examples/sample.jsonl").unwrap();

    // Delete second line
    tree.delete_node(&[1]).unwrap();

    // Should have 2 lines now
    match tree.root().value() {
        JsonValue::JsonlRoot(lines) => {
            assert_eq!(lines.len(), 2);
        }
        _ => panic!("Expected JsonlRoot"),
    }

    // Save and verify
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("deleted.jsonl");
    let config = Config::default();

    save_json_file(&tree, &output_path, &config).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content.lines().count(), 2);
}
```

**Step 3: Run integration tests**

Run: `cargo test --test integration_jsonl`
Expected: All tests pass

**Step 4: Manual testing checklist**

Test manually:

1. `cargo run -- examples/sample.jsonl`
   - Verify lines show collapsed previews
   - Verify no line numbers shown
   - Verify cyan color (if terminal supports)

2. Press `l` on first line
   - Verify expands to show object fields
   - Verify indentation correct

3. Press `h` on expanded line
   - Verify collapses back to preview

4. Navigate to a field, press `i`, edit value, press Enter
   - Verify edit works

5. Press `:w` to save
   - Verify file saved
   - Check file contents: `cat examples/sample.jsonl`
   - Verify JSONL format preserved

6. Test with regular JSON file: `cargo run -- examples/sample.json`
   - Verify collapsed previews work for regular JSON too
   - Verify regular JSON still expands by default

**Step 5: Fix any issues found in manual testing**

(Address any bugs discovered)

**Step 6: Commit**

```bash
git add tests/integration_jsonl.rs examples/sample.jsonl
git commit -m "test: add JSONL integration tests and sample file

Add comprehensive integration tests for JSONL workflow.
Include sample JSONL file for manual testing.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Final cleanup and verification

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass (300+ tests)

**Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 3: Format code**

Run: `cargo fmt`

**Step 4: Build release binary**

Run: `cargo build --release`
Expected: SUCCESS

**Step 5: Test release binary with JSONL**

Run: `./target/release/jsonquill examples/sample.jsonl`

Verify:
- Loads correctly
- Collapsed previews shown
- Can expand/collapse
- Can edit
- Can save
- Format preserved

**Step 6: Test release binary with regular JSON**

Run: `./target/release/jsonquill examples/sample.json`

Verify:
- Loads correctly
- Collapsed previews shown
- Regular JSON behavior preserved

**Step 7: Final commit**

```bash
git add -A
git commit -m "chore: final cleanup and formatting

Run cargo fmt and clippy fixes.
All tests passing.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

**Step 8: Merge to main**

Switch to main branch and merge:

```bash
git checkout main
git merge jsonl-support
```

Or use @superpowers:finishing-a-development-branch skill.

---

## Success Criteria Checklist

Before considering this complete, verify:

- [ ] `.jsonl` files load correctly (line-by-line parsing)
- [ ] JSONL displays as flat list with collapsed previews
- [ ] Collapsed preview format matches jless style `(N) {key: val, ...}`
- [ ] Can expand/collapse JSONL lines
- [ ] Can edit fields within JSONL lines
- [ ] Can delete JSONL lines with `dd`
- [ ] Can yank/paste JSONL lines
- [ ] JSONL saves back in correct format (line-by-line, no array brackets)
- [ ] Regular JSON shows collapsed previews too
- [ ] All tests passing (300+ tests)
- [ ] No clippy warnings
- [ ] Documentation updated (README, CLAUDE.md)
- [ ] Sample JSONL file included

## Notes

- Use @superpowers:test-driven-development for each task
- Use @superpowers:verification-before-completion before final merge
- Use @superpowers:finishing-a-development-branch after all tasks complete
