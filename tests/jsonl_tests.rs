use jsonquill::document::node::JsonValue;
use jsonquill::file::loader::load_json_file;
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
