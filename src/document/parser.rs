//! JSON parsing with metadata preservation.
//!
//! This module provides functionality to parse JSON strings into `JsonTree` structures
//! while preserving formatting metadata. The parser converts standard JSON into our
//! internal representation that tracks modification status and text spans for
//! format-preserving edits.
//!
//! # Example
//!
//! ```
//! use jsonquill::document::parser::parse_json;
//!
//! let json = r#"{"name": "Alice", "age": 30}"#;
//! let tree = parse_json(json).unwrap();
//!
//! // Navigate to the first field
//! let name_node = tree.get_node(&[0]).unwrap();
//! ```

use super::node::{JsonNode, JsonValue, NodeMetadata, TextSpan};
use super::tree::JsonTree;
use anyhow::{Context, Result};
use serde_json::Value as SerdeValue;

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
            SerdeValue::Number(_) => self.find_number_end(),
            SerdeValue::String(_) => self.find_string_end(),
            SerdeValue::Array(_) => self.find_container_end('[', ']'),
            SerdeValue::Object(_) => self.find_container_end('{', '}'),
        };

        TextSpan { start, end }
    }

    /// Find the end of a number
    fn find_number_end(&mut self) -> usize {
        while self.pos < self.source.len() {
            let ch = self.source.as_bytes()[self.pos];
            if ch.is_ascii_digit()
                || ch == b'-'
                || ch == b'+'
                || ch == b'.'
                || ch == b'e'
                || ch == b'E'
            {
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
                        self.pos += 1; // Move past backslash
                        if self.pos < self.source.len() {
                            let next_byte = self.source.as_bytes()[self.pos];
                            if next_byte == b'u' {
                                self.pos += 5; // \uXXXX is 6 bytes total, already moved past \
                            } else {
                                self.pos += 1; // Standard 2-byte escape
                            }
                        }
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
            let ch = self.source[self.pos..].chars().next().unwrap_or('\0');

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

/// Parses a JSON string into a `JsonTree`.
///
/// This function uses `serde_json` to parse the JSON string, then converts
/// the result into our internal `JsonTree` structure with metadata tracking.
/// The root node will have its text_span populated by the span tracker for format-preserving edits.
///
/// # Arguments
///
/// * `json_str` - A string slice containing valid JSON
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(JsonTree)` if parsing succeeds
/// - `Err(anyhow::Error)` if the JSON is malformed
///
/// # Note on Number Precision
///
/// JSON numbers are stored as `f64` internally. This means very large integers
/// (beyond 2^53 - 1) may lose precision during parsing. If exact integer precision
/// is required for large numbers, consider using string representations instead
///
/// # Example
///
/// ```
/// use jsonquill::document::parser::parse_json;
/// use jsonquill::document::node::JsonValue;
///
/// let json = r#"{"name": "Alice"}"#;
/// let tree = parse_json(json).unwrap();
///
/// // Root should be an object
/// assert!(tree.root().value().is_object());
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The input string is not valid JSON
/// - The JSON contains syntax errors
///
/// # Examples
///
/// Parsing a simple object:
/// ```
/// use jsonquill::document::parser::parse_json;
///
/// let json = r#"{"key": "value"}"#;
/// let tree = parse_json(json).unwrap();
/// ```
///
/// Parsing an array:
/// ```
/// use jsonquill::document::parser::parse_json;
///
/// let json = r#"[1, 2, 3]"#;
/// let tree = parse_json(json).unwrap();
/// ```
///
/// Handling errors:
/// ```
/// use jsonquill::document::parser::parse_json;
///
/// let invalid_json = r#"{"unclosed": "#;
/// assert!(parse_json(invalid_json).is_err());
/// ```
pub fn parse_json(json_str: &str) -> Result<JsonTree> {
    let serde_value: SerdeValue = serde_json::from_str(json_str).context("Failed to parse JSON")?;

    let mut tracker = SpanTracker::new(json_str);
    let root = convert_with_spans(&serde_value, &mut tracker);

    Ok(JsonTree::with_source(root, Some(json_str.to_string())))
}

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
                    if tracker.pos < tracker.source.len()
                        && tracker.source.as_bytes()[tracker.pos] == b':'
                    {
                        tracker.pos += 1;
                    }
                    tracker.skip_whitespace();

                    let node = convert_with_spans(v, tracker);

                    tracker.skip_whitespace();
                    // Skip comma if present
                    if tracker.pos < tracker.source.len()
                        && tracker.source.as_bytes()[tracker.pos] == b','
                    {
                        tracker.pos += 1;
                    }

                    (k.clone(), node)
                })
                .collect();
            // CRITICAL: Restore position to end of container after processing children
            tracker.pos = span.end;
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
                    if tracker.pos < tracker.source.len()
                        && tracker.source.as_bytes()[tracker.pos] == b','
                    {
                        tracker.pos += 1;
                    }
                    node
                })
                .collect();
            // CRITICAL: Restore position to end of container after processing children
            tracker.pos = span.end;
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

/// Converts a `serde_json::Value` into a `JsonNode`.
///
/// This is a recursive function that traverses the serde_json value tree
/// and converts each value into our internal representation with metadata.
/// Text spans will be added by the span tracker in a later implementation phase.
///
/// # Arguments
///
/// * `value` - The `serde_json::Value` to convert
///
/// # Returns
///
/// Returns a `JsonNode` with:
/// - The converted value
/// - `modified: false` (since it's freshly parsed, not user-modified)
/// - `text_span: None` (will be populated by span tracker later)
pub fn parse_value(value: &SerdeValue) -> JsonNode {
    convert_serde_value_impl(value)
}

fn convert_serde_value_impl(value: &SerdeValue) -> JsonNode {
    let json_value = match value {
        SerdeValue::Object(map) => {
            let entries = map
                .iter()
                .map(|(k, v)| (k.clone(), convert_serde_value_impl(v)))
                .collect();
            JsonValue::Object(entries)
        }
        SerdeValue::Array(arr) => {
            let elements = arr.iter().map(convert_serde_value_impl).collect();
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
            text_span: None,
            modified: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        let json = r#""hello""#;
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_number() {
        let json = "42.5";
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Number(n) => assert_eq!(*n, 42.5),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_boolean() {
        let json = "true";
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Boolean(b) => assert!(*b),
            _ => panic!("Expected boolean"),
        }
    }

    #[test]
    fn test_parse_null() {
        let json = "null";
        let tree = parse_json(json).unwrap();

        assert!(matches!(tree.root().value(), JsonValue::Null));
    }

    #[test]
    fn test_parse_empty_object() {
        let json = "{}";
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Object(entries) => assert_eq!(entries.len(), 0),
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let json = "[]";
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Array(elements) => assert_eq!(elements.len(), 0),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_object_with_fields() {
        let json = r#"{"name": "Alice", "age": 30, "active": true}"#;
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 3);
                assert_eq!(entries[0].0, "name");
                assert_eq!(entries[1].0, "age");
                assert_eq!(entries[2].0, "active");

                // Check values
                match entries[0].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "Alice"),
                    _ => panic!("Expected string"),
                }

                match entries[1].1.value() {
                    JsonValue::Number(n) => assert_eq!(*n, 30.0),
                    _ => panic!("Expected number"),
                }

                match entries[2].1.value() {
                    JsonValue::Boolean(b) => assert!(*b),
                    _ => panic!("Expected boolean"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_array_with_elements() {
        let json = r#"[1, "two", true, null]"#;
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Array(elements) => {
                assert_eq!(elements.len(), 4);

                assert!(matches!(elements[0].value(), JsonValue::Number(n) if *n == 1.0));
                assert!(matches!(elements[1].value(), JsonValue::String(s) if s == "two"));
                assert!(matches!(elements[2].value(), JsonValue::Boolean(true)));
                assert!(matches!(elements[3].value(), JsonValue::Null));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_nested_objects() {
        let json = r#"{"user": {"name": "Bob", "email": "bob@example.com"}}"#;
        let tree = parse_json(json).unwrap();

        // Navigate to nested object
        let user_node = tree.get_node(&[0]).unwrap();
        match user_node.value() {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "name");
                assert_eq!(entries[1].0, "email");
            }
            _ => panic!("Expected nested object"),
        }
    }

    #[test]
    fn test_parse_nested_arrays() {
        let json = r#"[[1, 2], [3, 4], [5, 6]]"#;
        let tree = parse_json(json).unwrap();

        // Check that root is an array
        match tree.root().value() {
            JsonValue::Array(outer) => {
                assert_eq!(outer.len(), 3);

                // Check first nested array
                match outer[0].value() {
                    JsonValue::Array(inner) => {
                        assert_eq!(inner.len(), 2);
                    }
                    _ => panic!("Expected nested array"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_complex_nested_structure() {
        let json = r#"{
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ],
            "metadata": {
                "count": 2,
                "active": true
            }
        }"#;

        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "users");
                assert_eq!(entries[1].0, "metadata");

                // Check users array
                match entries[0].1.value() {
                    JsonValue::Array(users) => {
                        assert_eq!(users.len(), 2);
                    }
                    _ => panic!("Expected array"),
                }

                // Check metadata object
                match entries[1].1.value() {
                    JsonValue::Object(meta) => {
                        assert_eq!(meta.len(), 2);
                    }
                    _ => panic!("Expected object"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_invalid_json() {
        let invalid_cases = vec![
            r#"{"unclosed": "#,
            r#"{"key": }"#,
            r#"{key: "value"}"#, // Unquoted key
            r#"[1, 2,"#,
            r#"{"trailing": "comma",}"#,
        ];

        for invalid in invalid_cases {
            let result = parse_json(invalid);
            assert!(result.is_err(), "Expected error for: {}", invalid);
        }
    }

    #[test]
    fn test_parse_initializes_metadata() {
        let json = r#"{"name": "Alice"}"#;
        let tree = parse_json(json).unwrap();

        // Root node should have text span populated by span tracker
        assert!(tree.root().metadata.text_span.is_some());
        // Parsed nodes should not be marked as modified
        assert!(!tree.root().is_modified());
    }

    #[test]
    fn test_parse_nodes_not_modified() {
        let json = r#"{"name": "Alice"}"#;
        let tree = parse_json(json).unwrap();

        // Parsed nodes should not be marked as modified
        assert!(!tree.root().is_modified());
    }

    #[test]
    fn test_parse_special_characters() {
        let json = r#"{"text": "Hello\nWorld", "emoji": "😀", "quote": "Say \"hi\""}"#;
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 3);

                // Check newline
                match entries[0].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "Hello\nWorld"),
                    _ => panic!("Expected string"),
                }

                // Check emoji
                match entries[1].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "😀"),
                    _ => panic!("Expected string"),
                }

                // Check escaped quotes
                match entries[2].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "Say \"hi\""),
                    _ => panic!("Expected string"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_numbers_edge_cases() {
        let test_cases = vec![
            ("0", 0.0),
            ("-1", -1.0),
            ("3.15", 3.15),
            ("-0.5", -0.5),
            ("1e10", 1e10),
            ("1.5e-5", 1.5e-5),
        ];

        for (json, expected) in test_cases {
            let tree = parse_json(json).unwrap();
            match tree.root().value() {
                JsonValue::Number(n) => assert_eq!(*n, expected),
                _ => panic!("Expected number for: {}", json),
            }
        }
    }

    #[test]
    fn test_parse_deep_nesting() {
        let json = r#"{"a": {"b": {"c": {"d": {"e": "deep"}}}}}"#;
        let tree = parse_json(json).unwrap();

        // Navigate deep into structure
        let path = vec![0, 0, 0, 0, 0];
        let deep_node = tree.get_node(&path).unwrap();

        match deep_node.value() {
            JsonValue::String(s) => assert_eq!(s, "deep"),
            _ => panic!("Expected string at deep nesting"),
        }
    }

    #[test]
    fn test_parse_unicode_strings() {
        let json = r#"{"chinese": "你好", "arabic": "مرحبا", "russian": "привет"}"#;
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 3);

                match entries[0].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "你好"),
                    _ => panic!("Expected string"),
                }

                match entries[1].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "مرحبا"),
                    _ => panic!("Expected string"),
                }

                match entries[2].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "привет"),
                    _ => panic!("Expected string"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

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

    #[test]
    fn test_parse_unicode_escape_in_string() {
        let json = r#"{"emoji": "Hello\u0041World", "chinese": "\u4f60\u597d"}"#;
        let tree = parse_json(json).unwrap();

        match tree.root().value() {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);

                // Check Unicode escape sequence \u0041 (A)
                match entries[0].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "HelloAWorld"),
                    _ => panic!("Expected string"),
                }

                // Check Unicode escape sequences for Chinese characters
                match entries[1].1.value() {
                    JsonValue::String(s) => assert_eq!(s, "你好"),
                    _ => panic!("Expected string"),
                }
            }
            _ => panic!("Expected object"),
        }
    }
}
