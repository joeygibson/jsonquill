# Format Preservation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add whitespace and indentation preservation so unmodified JSON nodes retain their exact original formatting while modified nodes use configured formatting.

**Architecture:** Use text spans (byte ranges) to track node positions in the original JSON string. When saving, splice together original text chunks for unmodified nodes with freshly serialized JSON for modified nodes.

**Tech Stack:** Rust, serde_json (existing parser), custom span tracking, hybrid serialization

---

## Phase 1: Core Infrastructure

### Task 1: Add TextSpan struct

**Files:**
- Modify: `src/document/node.rs:1-74`

**Step 1: Write the failing test**

Add to `src/document/node.rs` after existing tests:

```rust
#[cfg(test)]
mod text_span_tests {
    use super::*;

    #[test]
    fn test_text_span_creation() {
        let span = TextSpan { start: 10, end: 25 };
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 25);
    }

    #[test]
    fn test_text_span_equality() {
        let span1 = TextSpan { start: 5, end: 10 };
        let span2 = TextSpan { start: 5, end: 10 };
        let span3 = TextSpan { start: 5, end: 11 };

        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }

    #[test]
    fn test_text_span_clone() {
        let span1 = TextSpan { start: 0, end: 100 };
        let span2 = span1.clone();

        assert_eq!(span1, span2);
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cd ../jsonquill-worktrees/format-preservation
cargo test text_span_tests
```

Expected: FAIL with "cannot find type `TextSpan`"

**Step 3: Write minimal implementation**

Add before `JsonValue` enum (around line 29):

```rust
/// A byte range in the original JSON source.
///
/// TextSpan tracks the position of a node's text in the original JSON string,
/// enabling exact format preservation for unmodified nodes.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct TextSpan {
    /// Start byte offset in original JSON
    pub start: usize,
    /// End byte offset in original JSON (exclusive)
    pub end: usize,
}
```

**Step 4: Run test to verify it passes**

```bash
cargo test text_span_tests
```

Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/document/node.rs
git commit -m "feat: add TextSpan struct for tracking byte ranges"
```

---

### Task 2: Update NodeMetadata with text_span field

**Files:**
- Modify: `src/document/node.rs:63-74`

**Step 1: Write the failing test**

Add to the test section:

```rust
#[test]
fn test_node_metadata_with_text_span() {
    let metadata = NodeMetadata {
        text_span: Some(TextSpan { start: 0, end: 10 }),
        modified: false,
    };

    assert!(metadata.text_span.is_some());
    assert_eq!(metadata.text_span.unwrap().start, 0);
    assert_eq!(metadata.text_span.unwrap().end, 10);
    assert!(!metadata.modified);
}

#[test]
fn test_node_metadata_without_text_span() {
    let metadata = NodeMetadata {
        text_span: None,
        modified: true,
    };

    assert!(metadata.text_span.is_none());
    assert!(metadata.modified);
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test test_node_metadata
```

Expected: FAIL with "no field `text_span`"

**Step 3: Update NodeMetadata struct**

Replace the `NodeMetadata` struct (around line 68-74):

```rust
/// Metadata associated with a JSON node.
///
/// This structure tracks information about a node beyond its value, including
/// whether it has been modified since loading and its byte position in the
/// original source for format preservation.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeMetadata {
    /// Byte range in the original JSON string (for unmodified nodes)
    pub text_span: Option<TextSpan>,
    /// Whether this node has been modified
    pub modified: bool,
}
```

**Step 4: Update JsonNode::new() to use new metadata structure**

Find the `JsonNode::new()` method (around line 149-157) and update:

```rust
pub fn new(value: JsonValue) -> Self {
    Self {
        value,
        metadata: NodeMetadata {
            text_span: None,
            modified: true,
        },
    }
}
```

**Step 5: Run test to verify it passes**

```bash
cargo test test_node_metadata
```

Expected: PASS (2 tests)

**Step 6: Run all tests to ensure no regressions**

```bash
cargo test
```

Expected: All existing tests still pass

**Step 7: Commit**

```bash
git add src/document/node.rs
git commit -m "feat: add text_span field to NodeMetadata"
```

---

### Task 3: Add original_source to JsonTree

**Files:**
- Modify: `src/document/tree.rs:1-50`

**Step 1: Write the failing test**

Add to `src/document/tree.rs` test section:

```rust
#[test]
fn test_tree_with_original_source() {
    let root = JsonNode::new(JsonValue::String("test".to_string()));
    let tree = JsonTree::with_source(root.clone(), Some("\"test\"".to_string()));

    assert_eq!(tree.original_source(), Some("\"test\""));
}

#[test]
fn test_tree_without_original_source() {
    let root = JsonNode::new(JsonValue::Null);
    let tree = JsonTree::new(root);

    assert_eq!(tree.original_source(), None);
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test test_tree_with_original_source
```

Expected: FAIL with "no method named `with_source`"

**Step 3: Update JsonTree struct**

Find the `JsonTree` struct (around line 25-28):

```rust
/// A complete JSON document tree.
///
/// `JsonTree` represents a parsed JSON document with a root node and optional
/// original source text for format preservation.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonTree {
    root: JsonNode,
    /// The original JSON string (preserved for unmodified nodes)
    original_source: Option<String>,
}
```

**Step 4: Update JsonTree::new() and add helper methods**

Replace the implementation section (around line 30-50):

```rust
impl JsonTree {
    /// Creates a new JSON tree with the given root node.
    ///
    /// The tree has no original source, so format preservation is not available.
    /// New nodes created via `JsonNode::new()` are marked as modified by default.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let root = JsonNode::new(JsonValue::Null);
    /// let tree = JsonTree::new(root);
    /// ```
    pub fn new(root: JsonNode) -> Self {
        Self {
            root,
            original_source: None,
        }
    }

    /// Creates a new JSON tree with the given root node and original source.
    ///
    /// The original source enables format preservation for unmodified nodes.
    pub fn with_source(root: JsonNode, original_source: Option<String>) -> Self {
        Self {
            root,
            original_source,
        }
    }

    /// Returns a reference to the original JSON source, if available.
    pub fn original_source(&self) -> Option<&str> {
        self.original_source.as_deref()
    }

    /// Returns a reference to the root node of the tree.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let root = JsonNode::new(JsonValue::Boolean(true));
    /// let tree = JsonTree::new(root);
    ///
    /// assert!(matches!(tree.root().value(), JsonValue::Boolean(true)));
    /// ```
    pub fn root(&self) -> &JsonNode {
        &self.root
    }

    /// Returns a mutable reference to the root node of the tree.
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    ///
    /// let root = JsonNode::new(JsonValue::Null);
    /// let mut tree = JsonTree::new(root);
    ///
    /// *tree.root_mut().value_mut() = JsonValue::Boolean(false);
    /// ```
    pub fn root_mut(&mut self) -> &mut JsonNode {
        &mut self.root
    }
```

**Step 5: Run test to verify it passes**

```bash
cargo test test_tree_with_original_source
```

Expected: PASS (2 tests)

**Step 6: Run all tests to ensure no regressions**

```bash
cargo test
```

Expected: All existing tests still pass

**Step 7: Commit**

```bash
git add src/document/tree.rs
git commit -m "feat: add original_source field to JsonTree"
```

---

## Phase 2: Span-Aware Parsing

### Task 4: Implement basic span tracker

**Files:**
- Modify: `src/document/parser.rs:20-96`

**Step 1: Write the failing test**

Add to test section in `src/document/parser.rs`:

```rust
#[test]
fn test_parse_preserves_text_spans() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let tree = parse_json(json).unwrap();

    // Root should have a span covering the entire input
    assert!(tree.root().metadata.text_span.is_some());
    let root_span = tree.root().metadata.text_span.unwrap();
    assert_eq!(root_span.start, 0);
    assert_eq!(root_span.end, json.len());
}

#[test]
fn test_parse_sets_modified_false_for_parsed_nodes() {
    let json = r#"{"key": "value"}"#;
    let tree = parse_json(json).unwrap();

    // Parsed nodes should not be marked as modified
    assert!(!tree.root().is_modified());
}

#[test]
fn test_parse_stores_original_source() {
    let json = r#"[1, 2, 3]"#;
    let tree = parse_json(json).unwrap();

    assert_eq!(tree.original_source(), Some(json));
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test test_parse_preserves_text_spans
```

Expected: FAIL (text_span is None, original_source is None)

**Step 3: Add span tracking helper**

Add before the `parse_json` function (around line 91):

```rust
/// Tracks byte positions while parsing JSON.
struct SpanTracker<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> SpanTracker<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let ch = self.source.as_bytes()[self.pos];
            if ch == b' ' || ch == b'\n' || ch == b'\r' || ch == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Find the span of a value in the source
    fn find_value_span(&mut self, value: &SerdeValue) -> TextSpan {
        self.skip_whitespace();
        let start = self.pos;

        // Calculate end position based on value type
        let end = match value {
            SerdeValue::Null => {
                self.pos += 4; // "null"
                self.pos
            }
            SerdeValue::Bool(true) => {
                self.pos += 4; // "true"
                self.pos
            }
            SerdeValue::Bool(false) => {
                self.pos += 5; // "false"
                self.pos
            }
            SerdeValue::Number(_) => {
                self.find_number_end()
            }
            SerdeValue::String(_) => {
                self.find_string_end()
            }
            SerdeValue::Array(_) => {
                self.find_container_end('[', ']')
            }
            SerdeValue::Object(_) => {
                self.find_container_end('{', '}')
            }
        };

        TextSpan { start, end }
    }

    /// Find the end of a number
    fn find_number_end(&mut self) -> usize {
        while self.pos < self.source.len() {
            let ch = self.source.as_bytes()[self.pos];
            if ch.is_ascii_digit() || ch == b'-' || ch == b'+' || ch == b'.' || ch == b'e' || ch == b'E' {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.pos
    }

    /// Find the end of a string
    fn find_string_end(&mut self) -> usize {
        if self.source.as_bytes()[self.pos] == b'"' {
            self.pos += 1; // Skip opening quote

            while self.pos < self.source.len() {
                match self.source.as_bytes()[self.pos] {
                    b'\\' => {
                        self.pos += 2; // Skip escape sequence
                    }
                    b'"' => {
                        self.pos += 1; // Skip closing quote
                        break;
                    }
                    _ => {
                        self.pos += 1;
                    }
                }
            }
        }
        self.pos
    }

    /// Find the end of a container (array or object)
    fn find_container_end(&mut self, open: char, close: char) -> usize {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        while self.pos < self.source.len() {
            let ch = self.source.chars().nth(self.pos - self.source.char_indices().take(self.pos).count()).unwrap_or('\0');

            if escape_next {
                escape_next = false;
                self.pos += ch.len_utf8();
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                c if c == open && !in_string => depth += 1,
                c if c == close && !in_string => {
                    depth -= 1;
                    self.pos += ch.len_utf8();
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                _ => {}
            }

            self.pos += ch.len_utf8();
        }

        self.pos
    }
}
```

**Step 4: Update parse_json to use span tracking**

Replace the `parse_json` function:

```rust
pub fn parse_json(json_str: &str) -> Result<JsonTree> {
    let serde_value: SerdeValue = serde_json::from_str(json_str).context("Failed to parse JSON")?;

    let mut tracker = SpanTracker::new(json_str);
    let root = convert_with_spans(&serde_value, &mut tracker);

    Ok(JsonTree::with_source(root, Some(json_str.to_string())))
}
```

**Step 5: Add convert_with_spans function**

Add after `parse_json`:

```rust
/// Converts a serde_json::Value to JsonNode with span tracking.
fn convert_with_spans(value: &SerdeValue, tracker: &mut SpanTracker) -> JsonNode {
    let span = tracker.find_value_span(value);

    let json_value = match value {
        SerdeValue::Object(map) => {
            tracker.pos = span.start + 1; // Skip opening brace
            let entries = map
                .iter()
                .map(|(k, v)| {
                    tracker.skip_whitespace();
                    // Skip the key string
                    tracker.find_string_end();
                    tracker.skip_whitespace();
                    // Skip the colon
                    if tracker.pos < tracker.source.len() && tracker.source.as_bytes()[tracker.pos] == b':' {
                        tracker.pos += 1;
                    }
                    tracker.skip_whitespace();

                    let node = convert_with_spans(v, tracker);

                    tracker.skip_whitespace();
                    // Skip comma if present
                    if tracker.pos < tracker.source.len() && tracker.source.as_bytes()[tracker.pos] == b',' {
                        tracker.pos += 1;
                    }

                    (k.clone(), node)
                })
                .collect();
            JsonValue::Object(entries)
        }
        SerdeValue::Array(arr) => {
            tracker.pos = span.start + 1; // Skip opening bracket
            let elements = arr
                .iter()
                .map(|v| {
                    tracker.skip_whitespace();
                    let node = convert_with_spans(v, tracker);
                    tracker.skip_whitespace();
                    // Skip comma if present
                    if tracker.pos < tracker.source.len() && tracker.source.as_bytes()[tracker.pos] == b',' {
                        tracker.pos += 1;
                    }
                    node
                })
                .collect();
            JsonValue::Array(elements)
        }
        SerdeValue::String(s) => JsonValue::String(s.clone()),
        SerdeValue::Number(n) => JsonValue::Number(n.as_f64().unwrap_or(0.0)),
        SerdeValue::Bool(b) => JsonValue::Boolean(*b),
        SerdeValue::Null => JsonValue::Null,
    };

    JsonNode {
        value: json_value,
        metadata: NodeMetadata {
            text_span: Some(span),
            modified: false,
        },
    }
}
```

**Step 6: Add TextSpan import**

At the top of the file, update the imports (around line 20):

```rust
use super::node::{JsonNode, JsonValue, NodeMetadata, TextSpan};
```

**Step 7: Run test to verify it passes**

```bash
cargo test test_parse_preserves_text_spans
```

Expected: PASS (3 tests)

**Step 8: Run all parser tests**

```bash
cargo test document::parser
```

Expected: All parser tests pass

**Step 9: Commit**

```bash
git add src/document/parser.rs
git commit -m "feat: implement span tracking during JSON parsing"
```

---

## Phase 3: Format-Preserving Serialization

### Task 5: Add config option for format preservation

**Files:**
- Modify: `src/config/mod.rs`

**Step 1: Write the failing test**

Add to test section:

```rust
#[test]
fn test_preserve_formatting_default() {
    let config = Config::default();
    assert!(config.preserve_formatting);
}

#[test]
fn test_preserve_formatting_can_be_disabled() {
    let mut config = Config::default();
    config.preserve_formatting = false;
    assert!(!config.preserve_formatting);
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test test_preserve_formatting
```

Expected: FAIL with "no field `preserve_formatting`"

**Step 3: Add preserve_formatting field to Config**

Find the `Config` struct and add the field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... existing fields ...

    /// Preserve original formatting for unmodified nodes (default: true)
    #[serde(default = "default_preserve_formatting")]
    pub preserve_formatting: bool,
}
```

**Step 4: Add default function**

Add after the other default functions:

```rust
fn default_preserve_formatting() -> bool {
    true
}
```

**Step 5: Update Config::default() implementation**

Update the `Default` impl to include the new field:

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            indent_size: default_indent_size(),
            show_line_numbers: default_show_line_numbers(),
            auto_save: default_auto_save(),
            validation_mode: default_validation_mode(),
            create_backup: default_create_backup(),
            undo_limit: default_undo_limit(),
            sync_unnamed_register: default_sync_unnamed_register(),
            enable_mouse: default_enable_mouse(),
            lazy_load_threshold: default_lazy_load_threshold(),
            preserve_formatting: default_preserve_formatting(),
        }
    }
}
```

**Step 6: Run test to verify it passes**

```bash
cargo test test_preserve_formatting
```

Expected: PASS (2 tests)

**Step 7: Run all config tests**

```bash
cargo test config
```

Expected: All config tests pass

**Step 8: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat: add preserve_formatting config option"
```

---

### Task 6: Implement format-preserving serialization

**Files:**
- Modify: `src/file/saver.rs`

**Step 1: Write the failing test**

Add to test section in `src/file/saver.rs`:

```rust
#[test]
fn test_roundtrip_preserves_formatting() {
    use crate::document::parser::parse_json;
    use tempfile::NamedTempFile;
    use std::fs;

    let original_json = r#"{
  "name": "Alice",
  "age": 30,
  "active": true
}"#;

    // Parse
    let tree = parse_json(original_json).unwrap();
    let config = Config::default();

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    // Read back
    let saved_json = fs::read_to_string(temp_file.path()).unwrap();

    // Should be byte-for-byte identical
    assert_eq!(saved_json, original_json);
}

#[test]
fn test_modified_node_uses_config_formatting() {
    use crate::document::parser::parse_json;
    use crate::document::node::JsonValue;
    use tempfile::NamedTempFile;
    use std::fs;

    let original_json = r#"{"name":    "Alice"}"#; // Odd spacing

    // Parse
    let mut tree = parse_json(original_json).unwrap();

    // Modify a value
    if let JsonValue::Object(ref mut entries) = tree.root_mut().value_mut() {
        *entries[0].1.value_mut() = JsonValue::String("Bob".to_string());
    }

    let config = Config::default();

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    // Read back
    let saved_json = fs::read_to_string(temp_file.path()).unwrap();

    // Modified node should use clean formatting
    assert!(saved_json.contains("\"name\": \"Bob\""));
    // Should NOT preserve odd spacing
    assert!(!saved_json.contains("\"name\":    "));
}

#[test]
fn test_preserve_formatting_can_be_disabled() {
    use crate::document::parser::parse_json;
    use tempfile::NamedTempFile;
    use std::fs;

    let original_json = r#"{
    "name":    "Alice",
    "age":     30
}"#;

    // Parse
    let tree = parse_json(original_json).unwrap();

    // Disable format preservation
    let mut config = Config::default();
    config.preserve_formatting = false;

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    // Read back
    let saved_json = fs::read_to_string(temp_file.path()).unwrap();

    // Should use normalized formatting
    assert!(saved_json.contains("\"name\": \"Alice\""));
    assert!(saved_json.contains("\"age\": 30"));
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test test_roundtrip_preserves_formatting
```

Expected: FAIL (formatting not preserved)

**Step 3: Add format-preserving serialization function**

Add before the existing `serialize_node` function:

```rust
/// Serializes a node with format preservation for unmodified nodes.
///
/// If the node is unmodified and has a text span, extracts the original text.
/// Otherwise, serializes using the configured formatting.
fn serialize_preserving_format(
    node: &JsonNode,
    original: &str,
    config: &Config,
    depth: usize,
) -> String {
    // If format preservation is disabled, always use fresh serialization
    if !config.preserve_formatting {
        return serialize_node(node, config.indent_size, depth);
    }

    // If node is unmodified and has a span, use original text
    if !node.is_modified() && node.metadata.text_span.is_some() {
        let span = node.metadata.text_span.as_ref().unwrap();
        return original[span.start..span.end].to_string();
    }

    // Node was modified or has no span - serialize fresh
    match node.value() {
        JsonValue::Object(entries) => {
            serialize_object_preserving(entries, original, config, depth)
        }
        JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
            serialize_array_preserving(elements, original, config, depth)
        }
        _ => serialize_node(node, config.indent_size, depth),
    }
}

/// Serializes an object with format preservation for children.
fn serialize_object_preserving(
    entries: &[(String, JsonNode)],
    original: &str,
    config: &Config,
    depth: usize,
) -> String {
    if entries.is_empty() {
        return "{}".to_string();
    }

    let indent = " ".repeat(config.indent_size * depth);
    let next_indent = " ".repeat(config.indent_size * (depth + 1));

    let mut result = "{\n".to_string();
    for (i, (key, value)) in entries.iter().enumerate() {
        result.push_str(&next_indent);
        result.push_str(&format!("\"{}\": ", escape_json_string(key)));
        result.push_str(&serialize_preserving_format(value, original, config, depth + 1));
        if i < entries.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }
    result.push_str(&indent);
    result.push('}');
    result
}

/// Serializes an array with format preservation for children.
fn serialize_array_preserving(
    elements: &[JsonNode],
    original: &str,
    config: &Config,
    depth: usize,
) -> String {
    if elements.is_empty() {
        return "[]".to_string();
    }

    let indent = " ".repeat(config.indent_size * depth);
    let next_indent = " ".repeat(config.indent_size * (depth + 1));

    let mut result = "[\n".to_string();
    for (i, element) in elements.iter().enumerate() {
        result.push_str(&next_indent);
        result.push_str(&serialize_preserving_format(element, original, config, depth + 1));
        if i < elements.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }
    result.push_str(&indent);
    result.push(']');
    result
}
```

**Step 4: Update save_json_file to use format preservation**

Replace the serialization logic in `save_json_file` (around line 64-89):

```rust
pub fn save_json_file<P: AsRef<Path>>(path: P, tree: &JsonTree, config: &Config) -> Result<()> {
    let path = path.as_ref();

    // Check if this is a JSONL document
    if matches!(tree.root().value(), JsonValue::JsonlRoot(_)) {
        return save_jsonl(path, tree, config);
    }

    // Create backup if requested and file exists
    if config.create_backup && path.exists() {
        let backup_path = path.with_extension("jsonquill.bak");
        fs::copy(path, backup_path).context("Failed to create backup")?;
    }

    // Serialize with format preservation if original source is available
    let json_str = if let Some(original) = tree.original_source() {
        serialize_preserving_format(tree.root(), original, config, 0)
    } else {
        // No original source, use standard serialization
        serialize_node(tree.root(), config.indent_size, 0)
    };

    // Write to temp file first (atomic save)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, json_str).context("Failed to write temp file")?;

    // Rename temp to target (atomic operation)
    fs::rename(&temp_path, path).context("Failed to rename temp file")?;

    Ok(())
}
```

**Step 5: Run test to verify it passes**

```bash
cargo test test_roundtrip_preserves_formatting
```

Expected: PASS (3 tests)

**Step 6: Run all saver tests**

```bash
cargo test file::saver
```

Expected: All saver tests pass

**Step 7: Run full test suite**

```bash
cargo test
```

Expected: All tests pass

**Step 8: Commit**

```bash
git add src/file/saver.rs
git commit -m "feat: implement format-preserving serialization"
```

---

## Phase 4: Integration Testing

### Task 7: Add comprehensive integration tests

**Files:**
- Create: `tests/format_preservation_tests.rs`

**Step 1: Create test file**

```rust
//! Integration tests for format preservation.

use jsonquill::config::Config;
use jsonquill::document::node::JsonValue;
use jsonquill::document::parser::parse_json;
use jsonquill::file::saver::save_json_file;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_preserve_two_space_indent() {
    let json = r#"{
  "name": "Alice",
  "age": 30
}"#;

    let tree = parse_json(json).unwrap();
    let config = Config::default();

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_preserve_four_space_indent() {
    let json = r#"{
    "name": "Bob",
    "city": "NYC"
}"#;

    let tree = parse_json(json).unwrap();
    let config = Config::default();

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_preserve_compact_format() {
    let json = r#"{"name":"Alice","age":30}"#;

    let tree = parse_json(json).unwrap();
    let config = Config::default();

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_preserve_array_formatting() {
    let json = r#"[
  1,
  2,
  3
]"#;

    let tree = parse_json(json).unwrap();
    let config = Config::default();

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_preserve_nested_structure() {
    let json = r#"{
  "user": {
    "name": "Alice",
    "email": "alice@example.com"
  },
  "settings": {
    "theme": "dark"
  }
}"#;

    let tree = parse_json(json).unwrap();
    let config = Config::default();

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_edit_single_value_preserves_rest() {
    let json = r#"{
  "name":     "Alice",
  "age":      30,
  "active":   true
}"#;

    let mut tree = parse_json(json).unwrap();

    // Modify only the age
    if let JsonValue::Object(ref mut entries) = tree.root_mut().value_mut() {
        *entries[1].1.value_mut() = JsonValue::Number(31.0);
    }

    let config = Config::default();
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();

    // Name and active should preserve their spacing
    assert!(saved.contains("\"name\":     \"Alice\""));
    assert!(saved.contains("\"active\":   true"));

    // Age should be reformatted (modified)
    assert!(saved.contains("\"age\": 31"));
    assert!(!saved.contains("\"age\":      31"));
}

#[test]
fn test_disable_preservation() {
    let json = r#"{
    "name":    "Alice",
    "age":     30
}"#;

    let tree = parse_json(json).unwrap();

    let mut config = Config::default();
    config.preserve_formatting = false;
    config.indent_size = 2;

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();

    // Should use normalized formatting
    assert!(saved.contains("  \"name\": \"Alice\""));
    assert!(saved.contains("  \"age\": 30"));
}

#[test]
fn test_empty_document_uses_standard_serialization() {
    let tree = jsonquill::document::tree::JsonTree::new(
        jsonquill::document::node::JsonNode::new(JsonValue::Object(vec![]))
    );

    let config = Config::default();
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, "{}");
}

#[test]
fn test_preserve_mixed_scalar_types() {
    let json = r#"{
  "string": "hello",
  "number": 42,
  "boolean": true,
  "null": null
}"#;

    let tree = parse_json(json).unwrap();
    let config = Config::default();

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}
```

**Step 2: Run tests**

```bash
cargo test format_preservation_tests
```

Expected: PASS (10 tests)

**Step 3: Commit**

```bash
git add tests/format_preservation_tests.rs
git commit -m "test: add comprehensive format preservation integration tests"
```

---

### Task 8: Update documentation

**Files:**
- Modify: `README.md`
- Modify: `CLAUDE.md`

**Step 1: Update README.md**

Find the "Features" section and add:

```markdown
- **Format Preservation**: Unmodified JSON nodes retain their exact original formatting (whitespace, indentation, newlines) when saved
```

Find the configuration section and add:

```markdown
### Format Preservation

By default, jsonquill preserves the original formatting of unmodified JSON nodes. When you edit a value, that node and its parent containers are reformatted using your configured settings, while the rest of the document maintains its original style.

To disable format preservation and always use normalized formatting:

```toml
# ~/.config/jsonquill/config.toml
preserve_formatting = false
```

**Step 2: Update CLAUDE.md**

Find the "Known Issues / TODO" section and remove:

```markdown
- ❌ **No format preservation** - Original formatting not preserved on save
```

Add to the features section:

```markdown
- ✅ Format preservation (whitespace, indentation for unmodified nodes)
```

**Step 3: Commit**

```bash
git add README.md CLAUDE.md
git commit -m "docs: document format preservation feature"
```

---

### Task 9: Final verification

**Step 1: Run full test suite**

```bash
cargo test
```

Expected: All tests pass

**Step 2: Run clippy**

```bash
cargo clippy -- -D warnings
```

Expected: No warnings

**Step 3: Run fmt check**

```bash
cargo fmt -- --check
```

Expected: All files properly formatted

**Step 4: If fmt issues, fix them**

```bash
cargo fmt
git add -u
git commit -m "style: run cargo fmt"
```

**Step 5: Build release binary**

```bash
cargo build --release
```

Expected: Successful build

**Step 6: Manual test with real file**

Create a test file:

```bash
cat > /tmp/test.json << 'EOF'
{
    "name":    "Alice",
    "age":     30,
    "active":  true
}
EOF
```

Test roundtrip:

```bash
./target/release/jsonquill /tmp/test.json
# Don't make any edits, just save with :wq
diff /tmp/test.json <(cat /tmp/test.json)
```

Expected: No differences (byte-for-byte identical)

**Step 7: Tag completion**

```bash
git tag feature/format-preservation-complete
```

---

## Success Criteria

- [ ] All tests pass (142+ tests)
- [ ] No clippy warnings
- [ ] Code is properly formatted
- [ ] Roundtrip preservation works (unmodified files are byte-for-byte identical)
- [ ] Modified nodes use configured formatting
- [ ] Format preservation can be disabled via config
- [ ] Documentation updated
- [ ] No regressions in existing functionality

## Notes for Implementation

- **DRY**: Reuse existing `serialize_node` for modified nodes and fallback cases
- **YAGNI**: Don't add indent detection yet - that's a future enhancement
- **TDD**: Write failing tests first, implement minimal code to pass
- **Frequent commits**: Commit after each task completion
- **Error handling**: Existing error handling patterns are sufficient

## Known Limitations

1. **Indent detection not implemented**: New nodes always use `config.indent_size`, not the detected style from the parent
2. **Tab support**: Tabs are preserved in roundtrips but new content always uses spaces
3. **Comment preservation**: Not supported (requires different parser)
4. **Number format**: Numbers are normalized (1.0 becomes 1, scientific notation is expanded)

These limitations are documented and can be addressed in future work.
