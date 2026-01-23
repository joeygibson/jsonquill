use jsonquill::config::Config;
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::file::loader::load_json_file;
use jsonquill::file::saver::save_json_file;
use std::fs;
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

#[test]
fn test_save_jsonl_format() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("output.jsonl");

    let lines = vec![
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
        ])),
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(2.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Bob".to_string())),
            ),
        ])),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));
    let config = Config::default();

    save_json_file(&file_path, &tree, &config).unwrap();

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

#[test]
fn test_jsonl_roundtrip() {
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
    save_json_file(&file_path, &tree, &config).unwrap();

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
