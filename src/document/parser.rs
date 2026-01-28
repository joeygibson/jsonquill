//! JSON parsing with metadata preservation.
//!
//! This module provides functionality to parse JSON strings into `JsonTree` structures
//! while preserving formatting metadata. The parser converts standard JSON into our
//! internal representation that tracks modification status and original text for
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

use super::node::{JsonNode, JsonValue, NodeMetadata};
use super::tree::JsonTree;
use anyhow::{Context, Result};
use serde_json::Value as SerdeValue;

/// Parses a JSON string into a `JsonTree`.
///
/// This function uses `serde_json` to parse the JSON string, then converts
/// the result into our internal `JsonTree` structure with metadata tracking.
/// The root node preserves the original JSON string for format-preserving edits.
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

    let root = convert_serde_value(serde_value);
    Ok(JsonTree::new(root))
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

fn convert_serde_value(value: SerdeValue) -> JsonNode {
    convert_serde_value_impl(&value)
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

        // Root node should have no text span initially (will be added by span tracker)
        assert!(tree.root().metadata.text_span.is_none());
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
}
