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
/// Creates a backup of a file by copying it with a .bak extension.
fn create_backup<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let mut backup_path = path.to_path_buf();
    let original_name = backup_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
    backup_path.set_file_name(format!("{}.bak", original_name));
    fs::copy(path, backup_path).context("Failed to create backup")?;
    Ok(())
}

pub fn save_json_file<P: AsRef<Path>>(path: P, tree: &JsonTree, config: &Config) -> Result<()> {
    let path = path.as_ref();

    // Determine if we should compress based on target filename
    let should_compress = path.to_string_lossy().ends_with(".gz");

    // Check if this is a JSONL document
    if matches!(tree.root().value(), JsonValue::JsonlRoot(_)) {
        return save_jsonl(path, tree, config, should_compress);
    }

    // Create backup if requested and file exists
    if config.create_backup && path.exists() {
        create_backup(path)?;
    }

    // Serialize with format preservation if original source is available
    let mut json_str = if let Some(original) = tree.original_source() {
        serialize_preserving_format(tree.root(), original, config, 0)
    } else {
        // No original source, use standard serialization
        serialize_node(tree.root(), config.indent_size, 0)
    };

    // Preserve trailing newline from original if present
    if let Some(original) = tree.original_source() {
        if original.ends_with('\n') && !json_str.ends_with('\n') {
            json_str.push('\n');
        }
    }

    // Validate the serialized JSON before writing to disk
    // This catches serialization bugs before they corrupt user data
    serde_json::from_str::<serde_json::Value>(&json_str)
        .context("Generated invalid JSON - this is a bug in jsonquill's serialization")?;

    // Write atomically (compressed or uncompressed)
    write_file_atomic(path, json_str.as_bytes(), should_compress)?;

    Ok(())
}

/// Writes data to a file atomically, optionally compressing with gzip.
///
/// This function writes to a temporary file first, then atomically renames
/// it to the target path. This ensures the target file is never left in a
/// partially written state.
///
/// # Arguments
///
/// * `path` - Target file path
/// * `data` - Bytes to write
/// * `compress` - Whether to gzip-compress the data before writing
///
/// # Errors
///
/// Returns an error if:
/// - Creating the temp file fails
/// - Writing or compressing fails
/// - Renaming the temp file fails
fn write_file_atomic<P: AsRef<Path>>(path: P, data: &[u8], compress: bool) -> Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let path = path.as_ref();
    let temp_path = path.with_extension("tmp");

    if compress {
        // Write compressed
        let file = fs::File::create(&temp_path).context("Failed to create temp file")?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder
            .write_all(data)
            .context("Failed to write compressed data")?;
        encoder.finish().context("Failed to finish compression")?;
    } else {
        // Write uncompressed
        fs::write(&temp_path, data).context("Failed to write temp file")?;
    }

    // Atomic rename
    fs::rename(&temp_path, path).context("Failed to rename temp file")?;

    Ok(())
}

/// Saves a JSONL document to a file.
///
/// Each line is saved as a separate JSON object (one per line).
fn save_jsonl<P: AsRef<Path>>(
    path: P,
    tree: &JsonTree,
    config: &Config,
    compress: bool,
) -> Result<()> {
    let path = path.as_ref();

    // Create backup if requested and file exists
    if config.create_backup && path.exists() {
        create_backup(path)?;
    }

    let mut output = String::new();

    if let JsonValue::JsonlRoot(lines) = tree.root().value() {
        for (i, node) in lines.iter().enumerate() {
            // JSONL requires compact single-line JSON
            let line = serialize_node_compact(node);

            // Validate each line is valid JSON
            serde_json::from_str::<serde_json::Value>(&line).with_context(|| {
                format!(
                    "Generated invalid JSON at line {} - this is a bug in jsonquill's serialization",
                    i + 1
                )
            })?;

            output.push_str(&line);
            output.push('\n');
        }
    }

    // Write atomically with optional compression
    write_file_atomic(path, output.as_bytes(), compress)?;

    Ok(())
}

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

    // If node is unmodified and has a valid text span, extract from original
    if !node.is_modified() {
        if let Some(span) = node.metadata.text_span.as_ref() {
            return original[span.start..span.end].to_string();
        }
    }

    // Node was modified or has no span - serialize fresh
    match node.value() {
        JsonValue::Object(entries) => serialize_object_preserving(entries, original, config, depth),
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
        result.push_str(&serialize_preserving_format(
            value,
            original,
            config,
            depth + 1,
        ));
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
        result.push_str(&serialize_preserving_format(
            element,
            original,
            config,
            depth + 1,
        ));
        if i < elements.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }
    result.push_str(&indent);
    result.push(']');
    result
}

/// Serializes a JSON node to a compact single-line string.
///
/// This is used for JSONL format where each line must be a single-line JSON object.
/// Numbers are formatted as integers when they have no fractional part.
pub fn serialize_node_compact(node: &JsonNode) -> String {
    match node.value() {
        JsonValue::Object(entries) => {
            if entries.is_empty() {
                return "{}".to_string();
            }
            let parts: Vec<String> = entries
                .iter()
                .map(|(key, value)| {
                    format!(
                        "\"{}\":{}",
                        escape_json_string(key),
                        serialize_node_compact(value)
                    )
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
            if elements.is_empty() {
                return "[]".to_string();
            }
            let parts: Vec<String> = elements.iter().map(serialize_node_compact).collect();
            format!("[{}]", parts.join(","))
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

/// Serializes a JSON node in jq style (strict multi-line formatting).
///
/// This function matches jq's formatting behavior: all objects and arrays
/// are formatted with multi-line indentation, even if they're small.
/// No compact single-line formatting is used.
///
/// # Arguments
///
/// * `node` - The JSON node to serialize
/// * `indent_size` - Number of spaces per indentation level
/// * `current_depth` - Current nesting depth (used for recursion)
///
/// # Returns
///
/// A jq-style formatted JSON string
pub fn serialize_node_jq_style(
    node: &JsonNode,
    indent_size: usize,
    current_depth: usize,
) -> String {
    let indent = " ".repeat(indent_size * current_depth);
    let next_indent = " ".repeat(indent_size * (current_depth + 1));

    match node.value() {
        JsonValue::Object(entries) => {
            if entries.is_empty() {
                return "{}".to_string();
            }

            // jq always uses multi-line formatting for objects
            let mut result = "{\n".to_string();
            for (i, (key, value)) in entries.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&format!("\"{}\": ", escape_json_string(key)));
                result.push_str(&serialize_node_jq_style(
                    value,
                    indent_size,
                    current_depth + 1,
                ));
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

            // jq always uses multi-line formatting for arrays
            let mut result = "[\n".to_string();
            for (i, element) in elements.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&serialize_node_jq_style(
                    element,
                    indent_size,
                    current_depth + 1,
                ));
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
pub fn serialize_node(node: &JsonNode, indent_size: usize, current_depth: usize) -> String {
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

        let node = JsonNode::new(JsonValue::Number(2.5));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "2.5");
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

    #[test]
    fn test_roundtrip_preserves_formatting() {
        use crate::document::parser::parse_json;
        use std::fs;
        use tempfile::NamedTempFile;

        let original_json = r#"{
  "name": "Alice",
  "age": 30,
  "active": true
}"#;

        // Parse
        let tree = parse_json(original_json).unwrap();
        let config = Config {
            preserve_formatting: true,
            ..Default::default()
        };

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
        use crate::document::node::JsonValue;
        use crate::document::parser::parse_json;
        use std::fs;
        use tempfile::NamedTempFile;

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
        use std::fs;
        use tempfile::NamedTempFile;

        let original_json = r#"{
    "name":    "Alice",
    "age":     30
}"#;

        // Parse
        let tree = parse_json(original_json).unwrap();

        // Disable format preservation
        let config = Config {
            preserve_formatting: false,
            ..Default::default()
        };

        // Save
        let temp_file = NamedTempFile::new().unwrap();
        save_json_file(temp_file.path(), &tree, &config).unwrap();

        // Read back
        let saved_json = fs::read_to_string(temp_file.path()).unwrap();

        // Should use normalized formatting
        assert!(saved_json.contains("\"name\": \"Alice\""));
        assert!(saved_json.contains("\"age\": 30"));
    }

    #[test]
    fn test_edit_parent_invalidates_child_spans() {
        use crate::document::node::JsonValue;
        use crate::document::parser::parse_json;
        use std::fs;
        use tempfile::NamedTempFile;

        // Reproduce the exact scenario: company object with products array
        // When we rename a key in company and add a field, the products array
        // byte positions shift but the array itself isn't marked modified
        let original_json = r#"{
  "company": {
    "name": "TechCorp",
    "products": [
      {
        "id": "prod-1",
        "title": "Product A"
      }
    ]
  }
}"#;

        let mut tree = parse_json(original_json).unwrap();

        // Navigate to company object and modify it
        if let JsonValue::Object(ref mut root_entries) = tree.root_mut().value_mut() {
            if let JsonValue::Object(ref mut company_entries) = root_entries[0].1.value_mut() {
                // Rename "name" to "companyName" by modifying the first entry
                company_entries[0].0 = "companyName".to_string();

                // Add a new field "employees": 23
                company_entries.insert(
                    1,
                    (
                        "employees".to_string(),
                        crate::document::node::JsonNode::new(JsonValue::Number(23.0)),
                    ),
                );
            }
        }

        let config = crate::config::Config::default();
        let temp_file = NamedTempFile::new().unwrap();
        crate::file::saver::save_json_file(temp_file.path(), &tree, &config).unwrap();

        let saved_json = fs::read_to_string(temp_file.path()).unwrap();

        // The bug: products array gets corrupted because its text_span points to
        // old byte positions, but the parent modification shifted everything

        // Verify the saved JSON is valid
        let reparsed = serde_json::from_str::<serde_json::Value>(&saved_json);
        assert!(
            reparsed.is_ok(),
            "Saved JSON should be valid, but got: {}",
            saved_json
        );

        // Verify products array is intact
        assert!(saved_json.contains("\"products\":"));
        assert!(saved_json.contains("\"prod-1\""));
        assert!(saved_json.contains("\"title\": \"Product A\""));

        // Verify no garbage text extraction
        assert!(!saved_json.contains("]: n"));
        assert!(!saved_json.contains("n\","));
        assert!(!saved_json.contains("name\","));
    }

    #[test]
    fn test_write_file_atomic_uncompressed() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let target_path = temp_file.path();
        let data = b"test content";

        write_file_atomic(target_path, data, false).unwrap();

        let written = fs::read_to_string(target_path).unwrap();
        assert_eq!(written, "test content");
    }

    #[test]
    fn test_write_file_atomic_compressed() {
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let target_path = temp_file.path().with_extension("json.gz");
        let data = b"test content";

        write_file_atomic(&target_path, data, true).unwrap();

        // Decompress and verify
        let file = fs::File::open(&target_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert_eq!(decompressed, "test content");
    }

    // Task 10: Saver gzip tests

    #[test]
    fn test_save_json_as_gzipped() {
        use crate::document::parser::parse_json;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create JSON tree
        let json = r#"{"name": "Alice", "age": 30}"#;
        let tree = parse_json(json).unwrap();
        let config = Config::default();

        // Save as .json.gz
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");
        save_json_file(&gz_path, &tree, &config).unwrap();

        // Decompress and verify
        let file = fs::File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&decompressed).unwrap();
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 30);
    }

    #[test]
    fn test_save_jsonl_as_gzipped() {
        use crate::document::node::JsonValue;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create JSONL tree manually
        let lines = vec![
            JsonNode::new(JsonValue::Object(vec![(
                "id".to_string(),
                JsonNode::new(JsonValue::Number(1.0)),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "id".to_string(),
                JsonNode::new(JsonValue::Number(2.0)),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "id".to_string(),
                JsonNode::new(JsonValue::Number(3.0)),
            )])),
        ];
        let root = JsonNode::new(JsonValue::JsonlRoot(lines));
        let tree = JsonTree::new(root);
        let config = Config::default();

        // Save as .jsonl.gz
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("jsonl.gz");
        save_json_file(&gz_path, &tree, &config).unwrap();

        // Decompress and verify
        let file = fs::File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        // Verify JSONL format (one JSON per line)
        let lines: Vec<&str> = decompressed.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(lines[0])
                .unwrap()
                .get("id")
                .unwrap(),
            1
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(lines[1])
                .unwrap()
                .get("id")
                .unwrap(),
            2
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(lines[2])
                .unwrap()
                .get("id")
                .unwrap(),
            3
        );
    }

    // Task 11: Format switching tests

    #[test]
    fn test_format_switching_json_to_gz() {
        use crate::document::parser::parse_json;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create and save as .json
        let json = r#"{"test": "value"}"#;
        let tree = parse_json(json).unwrap();
        let config = Config::default();

        let temp_file = NamedTempFile::new().unwrap();
        let json_path = temp_file.path().with_extension("json");
        save_json_file(&json_path, &tree, &config).unwrap();

        // Verify uncompressed
        let content = fs::read_to_string(&json_path).unwrap();
        assert!(content.contains("test"));

        // Save same tree as .json.gz
        let gz_path = temp_file.path().with_extension("json.gz");
        save_json_file(&gz_path, &tree, &config).unwrap();

        // Verify compressed
        let file = fs::File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert!(decompressed.contains("test"));
    }

    #[test]
    fn test_format_switching_gz_to_json() {
        use crate::document::parser::parse_json;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create and save as .json.gz
        let json = r#"{"test": "value"}"#;
        let tree = parse_json(json).unwrap();
        let config = Config::default();

        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");
        save_json_file(&gz_path, &tree, &config).unwrap();

        // Verify compressed
        let file = fs::File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert!(decompressed.contains("test"));

        // Save same tree as .json (uncompressed)
        let json_path = temp_file.path().with_extension("json");
        save_json_file(&json_path, &tree, &config).unwrap();

        // Verify uncompressed
        let content = fs::read_to_string(&json_path).unwrap();
        assert!(content.contains("test"));

        // Verify it's NOT gzip (won't start with gzip magic bytes)
        let raw_bytes = fs::read(&json_path).unwrap();
        assert_ne!(&raw_bytes[0..2], &[0x1f, 0x8b]); // gzip magic bytes
    }

    // Task 12: Backup preservation test

    #[test]
    fn test_backup_preserves_compression() {
        use crate::document::parser::parse_json;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create initial .json.gz file
        let json = r#"{"version": 1}"#;
        let tree = parse_json(json).unwrap();
        let config = Config::default();

        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");
        save_json_file(&gz_path, &tree, &config).unwrap();

        // Modify and save with backup enabled
        let json2 = r#"{"version": 2}"#;
        let tree2 = parse_json(json2).unwrap();
        let config_with_backup = Config {
            create_backup: true,
            ..Default::default()
        };
        save_json_file(&gz_path, &tree2, &config_with_backup).unwrap();

        // Verify backup was created
        let backup_path = gz_path.with_file_name(format!(
            "{}.bak",
            gz_path.file_name().unwrap().to_str().unwrap()
        ));
        assert!(backup_path.exists());

        // Verify backup is compressed (can decompress)
        let file = fs::File::open(&backup_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        // Verify backup contains original version
        let parsed: serde_json::Value = serde_json::from_str(&decompressed).unwrap();
        assert_eq!(parsed["version"], 1);

        // Verify new file contains updated version
        let file2 = fs::File::open(&gz_path).unwrap();
        let mut decoder2 = GzDecoder::new(file2);
        let mut decompressed2 = String::new();
        decoder2.read_to_string(&mut decompressed2).unwrap();
        let parsed2: serde_json::Value = serde_json::from_str(&decompressed2).unwrap();
        assert_eq!(parsed2["version"], 2);
    }
}
