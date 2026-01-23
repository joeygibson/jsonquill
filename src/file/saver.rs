//! JSON file saving functionality.
//!
//! This module provides functions to save `JsonTree` structures to files with
//! atomic write operations and optional backup creation.

use crate::config::Config;
use crate::document::node::{JsonNode, JsonValue};
use crate::document::tree::JsonTree;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Saves a JSON tree to a file with optional backup creation.
///
/// This function serializes a `JsonTree` to JSON format and writes it to the
/// specified file path. The write operation is atomic (writes to a temp file
/// then renames) to prevent data loss on crashes. Optionally creates a backup
/// of the original file before writing.
///
/// For JSONL documents (JsonValue::JsonlRoot), saves in line-by-line format.
///
/// # Arguments
///
/// * `path` - The path where the JSON file should be saved
/// * `tree` - The JSON tree to serialize and save
/// * `config` - Configuration including indentation and backup settings
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(())` if the file was successfully saved
/// - `Err(anyhow::Error)` if:
///   - Creating a backup failed
///   - Writing the temp file failed
///   - Renaming the temp file to the target failed
///
/// # Examples
///
/// ```no_run
/// use jsonquill::file::saver::save_json_file;
/// use jsonquill::document::node::{JsonNode, JsonValue};
/// use jsonquill::document::tree::JsonTree;
/// use jsonquill::config::Config;
///
/// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
/// let config = Config::default();
/// save_json_file("output.json", &tree, &config).unwrap();
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - Backup creation fails (if requested)
/// - Writing to the temp file fails
/// - Renaming the temp file to the target fails
///
/// # Atomic Write
///
/// This function uses an atomic write strategy:
/// 1. Serializes the JSON to a temporary file
/// 2. Renames the temporary file to the target path
///
/// This ensures that the target file is never left in a partially written state.
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

    // Serialize to JSON
    let json_str = serialize_node(tree.root(), config.indent_size, 0);

    // Write to temp file first (atomic save)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, json_str).context("Failed to write temp file")?;

    // Rename temp to target (atomic operation)
    fs::rename(&temp_path, path).context("Failed to rename temp file")?;

    Ok(())
}

/// Saves a JSONL document to a file.
///
/// Each line is saved as a separate JSON object (one per line).
fn save_jsonl<P: AsRef<Path>>(path: P, tree: &JsonTree, config: &Config) -> Result<()> {
    let path = path.as_ref();

    // Create backup if requested and file exists
    if config.create_backup && path.exists() {
        let backup_path = path.with_extension("jsonquill.bak");
        fs::copy(path, backup_path).context("Failed to create backup")?;
    }

    let mut output = String::new();

    if let JsonValue::JsonlRoot(lines) = tree.root().value() {
        for node in lines {
            let json_value = node_to_serde_value(node);
            let line = serde_json::to_string(&json_value)?;
            output.push_str(&line);
            output.push('\n');
        }
    }

    // Write to temp file first (atomic save)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, output).context("Failed to write temp file")?;

    // Rename temp to target (atomic operation)
    fs::rename(&temp_path, path).context("Failed to rename temp file")?;

    Ok(())
}

/// Converts a JsonNode to serde_json::Value for serialization.
fn node_to_serde_value(node: &JsonNode) -> serde_json::Value {
    match node.value() {
        JsonValue::Null => serde_json::Value::Null,
        JsonValue::Boolean(b) => serde_json::Value::Bool(*b),
        JsonValue::Number(n) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        JsonValue::String(s) => serde_json::Value::String(s.clone()),
        JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
            serde_json::Value::Array(elements.iter().map(node_to_serde_value).collect())
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

/// Recursively serializes a JSON node to a formatted string.
///
/// This function converts a `JsonNode` and all its children into a JSON string
/// with proper indentation and formatting. It handles all JSON value types
/// including nested objects and arrays.
///
/// For arrays and objects containing only scalar values, uses compact single-line
/// formatting if the result would be reasonably short (< 80 characters).
///
/// # Arguments
///
/// * `node` - The JSON node to serialize
/// * `indent_size` - Number of spaces per indentation level
/// * `current_depth` - Current nesting depth (used for recursion)
///
/// # Returns
///
/// A formatted JSON string representing the node
fn serialize_node(node: &JsonNode, indent_size: usize, current_depth: usize) -> String {
    let indent = " ".repeat(indent_size * current_depth);
    let next_indent = " ".repeat(indent_size * (current_depth + 1));

    match node.value() {
        JsonValue::Object(entries) => {
            if entries.is_empty() {
                return "{}".to_string();
            }

            // Try compact formatting for objects with only scalar values
            if should_use_compact_format_object(entries) {
                let compact = serialize_object_compact(entries);
                if compact.len() <= 80 {
                    return compact;
                }
            }

            // Use multi-line formatting
            let mut result = "{\n".to_string();
            for (i, (key, value)) in entries.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&format!("\"{}\": ", escape_json_string(key)));
                result.push_str(&serialize_node(value, indent_size, current_depth + 1));
                if i < entries.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent);
            result.push('}');
            result
        }
        JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
            if elements.is_empty() {
                return "[]".to_string();
            }

            // Try compact formatting for arrays with only scalar values
            if should_use_compact_format_array(elements) {
                let compact = serialize_array_compact(elements);
                if compact.len() <= 80 {
                    return compact;
                }
            }

            // Use multi-line formatting
            let mut result = "[\n".to_string();
            for (i, element) in elements.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&serialize_node(element, indent_size, current_depth + 1));
                if i < elements.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent);
            result.push(']');
            result
        }
        JsonValue::String(s) => format!("\"{}\"", escape_json_string(s)),
        JsonValue::Number(n) => {
            // Format numbers cleanly - remove unnecessary decimal points
            if n.fract() == 0.0 && n.is_finite() {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        JsonValue::Boolean(b) => b.to_string(),
        JsonValue::Null => "null".to_string(),
    }
}

/// Checks if an object should use compact (single-line) formatting.
///
/// Returns true if all values in the object are scalar (not containers).
fn should_use_compact_format_object(entries: &[(String, JsonNode)]) -> bool {
    entries.iter().all(|(_, node)| !node.value().is_container())
}

/// Checks if an array should use compact (single-line) formatting.
///
/// Returns true if all elements in the array are scalar (not containers).
fn should_use_compact_format_array(elements: &[JsonNode]) -> bool {
    elements.iter().all(|node| !node.value().is_container())
}

/// Serializes an object in compact (single-line) format.
///
/// Example: `{"a": 1, "b": "hello", "c": true}`
fn serialize_object_compact(entries: &[(String, JsonNode)]) -> String {
    let parts: Vec<String> = entries
        .iter()
        .map(|(key, value)| {
            format!(
                "\"{}\": {}",
                escape_json_string(key),
                serialize_scalar(value.value())
            )
        })
        .collect();
    format!("{{{}}}", parts.join(", "))
}

/// Serializes an array in compact (single-line) format.
///
/// Example: `[1, 2, 3, 4, 5]`
fn serialize_array_compact(elements: &[JsonNode]) -> String {
    let parts: Vec<String> = elements
        .iter()
        .map(|node| serialize_scalar(node.value()))
        .collect();
    format!("[{}]", parts.join(", "))
}

/// Serializes a scalar value (not a container) to a string.
///
/// This is a simplified version of serialize_node for scalar values only.
fn serialize_scalar(value: &JsonValue) -> String {
    match value {
        JsonValue::String(s) => format!("\"{}\"", escape_json_string(s)),
        JsonValue::Number(n) => {
            if n.fract() == 0.0 && n.is_finite() {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        JsonValue::Boolean(b) => b.to_string(),
        JsonValue::Null => "null".to_string(),
        _ => panic!("serialize_scalar called on non-scalar value"),
    }
}

/// Escapes special characters in a string for JSON serialization.
///
/// This function handles all special characters that need escaping in JSON strings:
/// - Backslash (\)
/// - Double quote (")
/// - Control characters (newline, tab, carriage return, etc.)
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// A new string with all special characters properly escaped
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"),
            '\x0C' => result.push_str("\\f"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_null() {
        let node = JsonNode::new(JsonValue::Null);
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "null");
    }

    #[test]
    fn test_serialize_boolean() {
        let node = JsonNode::new(JsonValue::Boolean(true));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "true");

        let node = JsonNode::new(JsonValue::Boolean(false));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "false");
    }

    #[test]
    fn test_serialize_number() {
        let node = JsonNode::new(JsonValue::Number(42.0));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "42");

        let node = JsonNode::new(JsonValue::Number(3.14));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_serialize_string() {
        let node = JsonNode::new(JsonValue::String("hello".to_string()));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_serialize_empty_object() {
        let node = JsonNode::new(JsonValue::Object(vec![]));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_serialize_empty_array() {
        let node = JsonNode::new(JsonValue::Array(vec![]));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_serialize_simple_object() {
        let obj = vec![(
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        )];
        let node = JsonNode::new(JsonValue::Object(obj));
        let result = serialize_node(&node, 2, 0);
        // Small scalar objects use compact formatting
        assert_eq!(result, "{\"name\": \"Alice\"}");
    }

    #[test]
    fn test_serialize_simple_array() {
        let arr = vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
            JsonNode::new(JsonValue::Number(3.0)),
        ];
        let node = JsonNode::new(JsonValue::Array(arr));
        let result = serialize_node(&node, 2, 0);
        // Small scalar arrays use compact formatting
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_serialize_nested_object() {
        let inner = vec![("age".to_string(), JsonNode::new(JsonValue::Number(30.0)))];
        let outer = vec![("user".to_string(), JsonNode::new(JsonValue::Object(inner)))];
        let node = JsonNode::new(JsonValue::Object(outer));
        let result = serialize_node(&node, 2, 0);
        // Inner object with single scalar value uses compact formatting
        assert_eq!(result, "{\n  \"user\": {\"age\": 30}\n}");
    }

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_json_string("hello\\world"), "hello\\\\world");
        assert_eq!(escape_json_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_json_string("hello\tworld"), "hello\\tworld");
        assert_eq!(escape_json_string("hello\rworld"), "hello\\rworld");
    }

    #[test]
    fn test_compact_array_with_scalars() {
        let arr = vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::String("test".to_string())),
            JsonNode::new(JsonValue::Boolean(true)),
            JsonNode::new(JsonValue::Null),
        ];
        let node = JsonNode::new(JsonValue::Array(arr));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "[1, \"test\", true, null]");
    }

    #[test]
    fn test_compact_object_with_scalars() {
        let obj = vec![
            ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            (
                "b".to_string(),
                JsonNode::new(JsonValue::String("test".to_string())),
            ),
            ("c".to_string(), JsonNode::new(JsonValue::Boolean(false))),
        ];
        let node = JsonNode::new(JsonValue::Object(obj));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "{\"a\": 1, \"b\": \"test\", \"c\": false}");
    }

    #[test]
    fn test_nested_containers_use_multiline() {
        // Array containing an object should use multi-line formatting
        let inner = vec![(
            "key".to_string(),
            JsonNode::new(JsonValue::String("value".to_string())),
        )];
        let arr = vec![JsonNode::new(JsonValue::Object(inner))];
        let node = JsonNode::new(JsonValue::Array(arr));
        let result = serialize_node(&node, 2, 0);
        assert!(
            result.contains('\n'),
            "Nested containers should use multi-line formatting"
        );
    }

    #[test]
    fn test_long_compact_array_uses_multiline() {
        // Create an array that would exceed 80 characters in compact format
        let arr: Vec<JsonNode> = (0..30)
            .map(|i| JsonNode::new(JsonValue::Number(i as f64)))
            .collect();
        let node = JsonNode::new(JsonValue::Array(arr));
        let result = serialize_node(&node, 2, 0);
        // Should fall back to multi-line because compact would be > 80 chars
        assert!(
            result.contains('\n'),
            "Long arrays should use multi-line formatting"
        );
    }
}
