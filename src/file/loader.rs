//! JSON file loading functionality.
//!
//! This module provides functions to load JSON documents from files or stdin,
//! parsing them into `JsonTree` structures that can be edited by jsonquill.

use crate::document::parser::{parse_json, parse_value};
use crate::document::tree::JsonTree;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Loads and parses a JSON file from the filesystem.
///
/// This function reads a file from disk and parses its contents as JSON,
/// returning a `JsonTree` structure ready for editing.
///
/// # Arguments
///
/// * `path` - The path to the JSON file to load
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(JsonTree)` if the file was successfully loaded and parsed
/// - `Err(anyhow::Error)` if:
///   - The file could not be read (doesn't exist, permission denied, etc.)
///   - The file contents are not valid JSON
///
/// # Examples
///
/// ```no_run
/// use jsonquill::file::loader::load_json_file;
///
/// let tree = load_json_file("config.json").unwrap();
/// // tree is now ready for editing
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The file path does not exist
/// - The file cannot be read (permissions, etc.)
/// - The file contents are not valid JSON
pub fn load_json_file<P: AsRef<Path>>(path: P) -> Result<JsonTree> {
    let path_ref = path.as_ref();

    // Check if file is gzipped
    let is_gzipped = path_ref
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "gz")
        .unwrap_or(false);

    // Read content (decompress if needed)
    let content = if is_gzipped {
        read_gzipped_file(path_ref)?
    } else {
        fs::read_to_string(path_ref).context("Failed to read file")?
    };

    // Determine format from filename (before .gz)
    let is_jsonl = determine_jsonl_format(path_ref);

    // Parse accordingly
    if is_jsonl {
        parse_jsonl_content(&content)
    } else {
        parse_json(&content).context("Failed to parse JSON")
    }
}

/// Helper function to parse JSONL content (newline-delimited JSON).
///
/// Each line must be a valid JSON value. Blank lines are skipped.
pub fn parse_jsonl_content(content: &str) -> Result<JsonTree> {
    use crate::document::node::{JsonNode, JsonValue};

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

    if lines.is_empty() {
        anyhow::bail!("No valid JSON found in JSONL content");
    }

    let root = JsonNode::new(JsonValue::JsonlRoot(lines));
    Ok(JsonTree::new(root))
}

/// Loads and parses JSON from standard input.
///
/// This function reads from stdin until EOF and parses the contents as JSON,
/// returning a `JsonTree` structure ready for editing. This is useful for
/// piping JSON data into the editor.
///
/// The function automatically detects whether the input is regular JSON or
/// JSONL format (newline-delimited JSON). It tries regular JSON first, and
/// if that fails, it attempts to parse as JSONL.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(JsonTree)` if stdin was successfully read and parsed
/// - `Err(anyhow::Error)` if:
///   - Reading from stdin failed
///   - The input contents are not valid JSON or JSONL
///
/// # Examples
///
/// ```no_run
/// use jsonquill::file::loader::load_json_from_stdin;
///
/// // Usage: echo '{"key": "value"}' | cargo run -- -
/// let tree = load_json_from_stdin().unwrap();
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - Reading from stdin fails
/// - The input contents are not valid JSON or JSONL
pub fn load_json_from_stdin() -> Result<JsonTree> {
    use std::io::{self, Read};

    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .context("Failed to read from stdin")?;

    // Check for gzip magic bytes (0x1f 0x8b)
    let content = if buffer.starts_with(&[0x1f, 0x8b]) {
        decompress_gzip_bytes(&buffer)?
    } else {
        String::from_utf8(buffer).context("Invalid UTF-8 in stdin")?
    };

    // Try to parse as regular JSON first
    if let Ok(tree) = parse_json(&content) {
        return Ok(tree);
    }

    // If regular JSON parsing fails, try JSONL format
    parse_jsonl_content(&content)
        .context("Failed to parse JSON from stdin: input is neither valid JSON nor valid JSONL")
}

/// Loads and parses a JSONL (JSON Lines) file from the filesystem.
///
/// Each line in the file must be a valid JSON value. Blank lines are skipped.
/// The result is a JsonTree with a JsonlRoot containing all lines.
pub fn load_jsonl_file<P: AsRef<Path>>(path: P) -> Result<JsonTree> {
    let content = fs::read_to_string(path.as_ref()).context("Failed to read JSONL file")?;
    parse_jsonl_content(&content)
}

/// Determines if file is JSONL format based on filename.
///
/// Checks for .jsonl or .ndjson extension, handling .gz suffix correctly.
/// Examples:
/// - `data.jsonl` → true
/// - `data.jsonl.gz` → true
/// - `data.json.gz` → false
fn determine_jsonl_format<P: AsRef<Path>>(path: P) -> bool {
    let path_str = path.as_ref().to_string_lossy();

    // Remove .gz suffix if present
    let base = if let Some(stripped) = path_str.strip_suffix(".gz") {
        stripped
    } else {
        &path_str
    };

    base.ends_with(".jsonl") || base.ends_with(".ndjson")
}

/// Reads and decompresses a gzipped file.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened
/// - The file is not valid gzip format (corrupted)
/// - The decompressed content is not valid UTF-8
fn read_gzipped_file<P: AsRef<Path>>(path: P) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let file = fs::File::open(path).context("Failed to open gzipped file")?;
    let mut decoder = GzDecoder::new(file);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .context("Failed to decompress gzipped file - file may be corrupted")?;
    Ok(content)
}

/// Decompresses gzip-encoded bytes to a UTF-8 string.
///
/// # Errors
///
/// Returns an error if:
/// - The bytes are not valid gzip format
/// - The decompressed content is not valid UTF-8
fn decompress_gzip_bytes(bytes: &[u8]) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(bytes);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .context("Failed to decompress gzipped stdin")?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::JsonValue;

    #[test]
    fn test_parse_jsonl_content_simple() {
        let content = r#"{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
{"id":3,"name":"Charlie"}"#;

        let tree = parse_jsonl_content(content).unwrap();

        match tree.root().value() {
            JsonValue::JsonlRoot(lines) => {
                assert_eq!(lines.len(), 3);

                // Check first line has correct structure
                if let JsonValue::Object(fields) = lines[0].value() {
                    assert_eq!(fields.len(), 2);
                } else {
                    panic!("Expected object on line 1");
                }
            }
            _ => panic!("Expected JsonlRoot"),
        }
    }

    #[test]
    fn test_parse_jsonl_content_skips_blank_lines() {
        let content = r#"{"id":1}

{"id":2}

{"id":3}"#;

        let tree = parse_jsonl_content(content).unwrap();

        match tree.root().value() {
            JsonValue::JsonlRoot(lines) => {
                assert_eq!(lines.len(), 3);
            }
            _ => panic!("Expected JsonlRoot"),
        }
    }

    #[test]
    fn test_parse_jsonl_content_mixed_types() {
        let content = r#"{"type":"object"}
["array","values"]
42
"string value"
true
null"#;

        let tree = parse_jsonl_content(content).unwrap();

        match tree.root().value() {
            JsonValue::JsonlRoot(lines) => {
                assert_eq!(lines.len(), 6);

                // Verify each type
                assert!(matches!(lines[0].value(), JsonValue::Object(_)));
                assert!(matches!(lines[1].value(), JsonValue::Array(_)));
                assert!(matches!(lines[2].value(), JsonValue::Number(_)));
                assert!(matches!(lines[3].value(), JsonValue::String(_)));
                assert!(matches!(lines[4].value(), JsonValue::Boolean(_)));
                assert!(matches!(lines[5].value(), JsonValue::Null));
            }
            _ => panic!("Expected JsonlRoot"),
        }
    }

    #[test]
    fn test_parse_jsonl_content_empty() {
        let content = "";
        let result = parse_jsonl_content(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No valid JSON found"));
    }

    #[test]
    fn test_parse_jsonl_content_invalid_json_line() {
        let content = r#"{"valid":true}
{invalid json}
{"valid":false}"#;

        let result = parse_jsonl_content(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid JSON on line 2"));
    }

    #[test]
    fn test_load_json_from_stdin_requires_actual_stdin() {
        // This test documents that load_json_from_stdin requires actual stdin
        // It cannot be easily tested in unit tests without mocking
        // The core JSONL parsing logic is tested via parse_jsonl_content tests
    }

    #[test]
    fn test_load_json_file_integration() {
        // Integration tests for file loading are in tests/file_tests.rs
        // This is just a placeholder to document the test structure
    }

    #[test]
    fn test_determine_jsonl_format() {
        assert!(determine_jsonl_format("data.jsonl"));
        assert!(determine_jsonl_format("data.ndjson"));
        assert!(determine_jsonl_format("path/to/data.jsonl.gz"));
        assert!(determine_jsonl_format("path/to/data.ndjson.gz"));
        assert!(!determine_jsonl_format("data.json"));
        assert!(!determine_jsonl_format("data.json.gz"));
    }

    #[test]
    fn test_read_gzipped_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with gzipped JSON
        let json_content = r#"{"test": "value"}"#;
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");

        // Write compressed content
        let file = fs::File::create(&gz_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(json_content.as_bytes()).unwrap();
        encoder.finish().unwrap();

        // Test decompression
        let decompressed = read_gzipped_file(&gz_path).unwrap();
        assert_eq!(decompressed, json_content);
    }

    #[test]
    fn test_read_gzipped_file_corrupted() {
        use tempfile::NamedTempFile;

        // Create file with .gz extension but invalid gzip data
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");
        fs::write(&gz_path, b"not gzip data").unwrap();

        // Should return error with helpful message
        let result = read_gzipped_file(&gz_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("decompress") || err_msg.contains("corrupted"));
    }

    #[test]
    fn test_load_gzipped_json_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with gzipped JSON
        let json_content = r#"{"name": "Alice", "age": 30}"#;
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");

        // Write compressed content
        let file = fs::File::create(&gz_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(json_content.as_bytes()).unwrap();
        encoder.finish().unwrap();

        // Load and verify
        let tree = load_json_file(&gz_path).unwrap();

        // Verify structure
        if let JsonValue::Object(entries) = tree.root().value() {
            assert_eq!(entries.len(), 2);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_load_gzipped_jsonl_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with gzipped JSONL
        let jsonl_content = r#"{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
{"id":3,"name":"Charlie"}"#;
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("jsonl.gz");

        // Write compressed content
        let file = fs::File::create(&gz_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(jsonl_content.as_bytes()).unwrap();
        encoder.finish().unwrap();

        // Load and verify
        let tree = load_json_file(&gz_path).unwrap();

        // Verify it's JSONL format
        if let JsonValue::JsonlRoot(lines) = tree.root().value() {
            assert_eq!(lines.len(), 3);
        } else {
            panic!("Expected JsonlRoot");
        }
    }
}
