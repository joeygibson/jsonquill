//! Tests for the :format command behavior.

use jsonquill::config::Config;
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::parser::parse_json;
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::file::saver::save_json_file;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_format_command_json_uses_multiline() {
    // Create a compact JSON document
    let json = r#"{"name":"Alice","age":30,"city":"NYC"}"#;
    let tree = parse_json(json).unwrap();
    let mut editor = EditorState::new(tree, "default-dark".to_string());

    // Format the document
    editor.format_document().unwrap();

    // Save to verify the formatted output
    let temp_file = NamedTempFile::new().unwrap();
    let config = Config::default();
    save_json_file(temp_file.path(), editor.tree(), &config).unwrap();
    let formatted = fs::read_to_string(temp_file.path()).unwrap();

    // Should be multi-line with 2-space indent
    assert!(formatted.contains("{\n"));
    assert!(formatted.contains("  \"name\": \"Alice\""));
    assert!(formatted.contains("  \"age\": 30"));
    assert!(formatted.contains("  \"city\": \"NYC\""));
}

#[test]
fn test_format_command_jsonl_uses_compact() {
    // Create a JSONL document
    let line1 = JsonNode::new(JsonValue::Object(vec![
        ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
        (
            "active".to_string(),
            JsonNode::new(JsonValue::Boolean(true)),
        ),
    ]));

    let line2 = JsonNode::new(JsonValue::Object(vec![
        ("id".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Bob".to_string())),
        ),
        (
            "active".to_string(),
            JsonNode::new(JsonValue::Boolean(false)),
        ),
    ]));

    let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(vec![line1, line2])));
    let mut editor = EditorState::new(tree, "default-dark".to_string());

    // Format the document
    editor.format_document().unwrap();

    // Save to verify the formatted output
    let temp_file = NamedTempFile::new().unwrap();
    let config = Config::default();
    save_json_file(temp_file.path(), editor.tree(), &config).unwrap();
    let formatted = fs::read_to_string(temp_file.path()).unwrap();

    let lines: Vec<&str> = formatted.lines().collect();

    // Should be exactly 2 lines (compact format like jq -c)
    assert_eq!(
        lines.len(),
        2,
        "JSONL should have 2 compact lines, got: {}",
        formatted
    );

    // Each line should be compact (no internal newlines or indentation)
    assert_eq!(lines[0], r#"{"id":1,"name":"Alice","active":true}"#);
    assert_eq!(lines[1], r#"{"id":2,"name":"Bob","active":false}"#);

    // Should NOT contain multi-line formatting
    assert!(
        !formatted.contains("  \"id\""),
        "JSONL should not have indentation"
    );
}
