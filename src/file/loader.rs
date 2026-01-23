//! JSON file loading functionality.
//!
//! This module provides functions to load JSON documents from files or stdin,
//! parsing them into `JsonTree` structures that can be edited by jeditor.

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

    // Check if this is a JSONL file
    if let Some(ext) = path_ref.extension() {
        if ext == "jsonl" || ext == "ndjson" {
            return load_jsonl_file(path_ref);
        }
    }

    // Regular JSON
    let content = fs::read_to_string(path_ref).context("Failed to read file")?;

    parse_json(&content).context("Failed to parse JSON")
}

/// Loads and parses JSON from standard input.
///
/// This function reads from stdin until EOF and parses the contents as JSON,
/// returning a `JsonTree` structure ready for editing. This is useful for
/// piping JSON data into the editor.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(JsonTree)` if stdin was successfully read and parsed
/// - `Err(anyhow::Error)` if:
///   - Reading from stdin failed
///   - The input contents are not valid JSON
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
/// - The input contents are not valid JSON
pub fn load_json_from_stdin() -> Result<JsonTree> {
    use std::io::{self, Read};

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    parse_json(&buffer).context("Failed to parse JSON from stdin")
}

/// Loads and parses a JSONL (JSON Lines) file from the filesystem.
///
/// Each line in the file must be a valid JSON value. Blank lines are skipped.
/// The result is a JsonTree with a JsonlRoot containing all lines.
pub fn load_jsonl_file<P: AsRef<Path>>(path: P) -> Result<JsonTree> {
    use crate::document::node::{JsonNode, JsonValue};

    let content = fs::read_to_string(path.as_ref()).context("Failed to read JSONL file")?;

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

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_json_from_stdin_requires_actual_stdin() {
        // This test documents that load_json_from_stdin requires actual stdin
        // It cannot be easily tested in unit tests without mocking
    }

    #[test]
    fn test_load_json_file_integration() {
        // Integration tests for file loading are in tests/file_tests.rs
        // This is just a placeholder to document the test structure
    }
}
