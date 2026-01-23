//! Integration tests for file I/O operations.

use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::file::loader::load_json_file;
use jsonquill::file::saver::save_json_file;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_load_simple_json_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"{{"name": "test"}}"#).unwrap();

    let tree = load_json_file(temp_file.path()).unwrap();

    // Verify the tree structure
    match tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "name");
            match entries[0].1.value() {
                JsonValue::String(s) => assert_eq!(s, "test"),
                _ => panic!("Expected string value"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_load_complex_json_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(
        temp_file,
        r#"{{
        "user": {{
            "name": "Alice",
            "age": 30,
            "active": true
        }},
        "items": [1, 2, 3],
        "metadata": null
    }}"#
    )
    .unwrap();

    let tree = load_json_file(temp_file.path()).unwrap();

    match tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].0, "user");
            assert_eq!(entries[1].0, "items");
            assert_eq!(entries[2].0, "metadata");

            // Check user object
            match entries[0].1.value() {
                JsonValue::Object(user_entries) => {
                    assert_eq!(user_entries.len(), 3);
                }
                _ => panic!("Expected object"),
            }

            // Check items array
            match entries[1].1.value() {
                JsonValue::Array(items) => {
                    assert_eq!(items.len(), 3);
                }
                _ => panic!("Expected array"),
            }

            // Check null
            assert!(matches!(entries[2].1.value(), JsonValue::Null));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_load_invalid_json() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"{{invalid json}}"#).unwrap();

    let result = load_json_file(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_load_nonexistent_file() {
    let result = load_json_file("/path/that/does/not/exist/file.json");
    assert!(result.is_err());
}

#[test]
fn test_save_simple_json_file() {
    let obj = vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("test".to_string())),
    )];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, 2, false).unwrap();

    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("\"name\""));
    assert!(content.contains("\"test\""));
    // Small objects with scalar values use compact formatting
    assert_eq!(content.trim(), "{\"name\": \"test\"}");
}

#[test]
fn test_save_complex_json_file() {
    let user_obj = vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
        (
            "age".to_string(),
            JsonNode::new(JsonValue::Number(30.0)),
        ),
        (
            "active".to_string(),
            JsonNode::new(JsonValue::Boolean(true)),
        ),
    ];

    let items = vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ];

    let obj = vec![
        (
            "user".to_string(),
            JsonNode::new(JsonValue::Object(user_obj)),
        ),
        ("items".to_string(), JsonNode::new(JsonValue::Array(items))),
        ("metadata".to_string(), JsonNode::new(JsonValue::Null)),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, 2, false).unwrap();

    let content = std::fs::read_to_string(temp_file.path()).unwrap();

    // Verify key elements are present
    assert!(content.contains("\"user\""));
    assert!(content.contains("\"name\""));
    assert!(content.contains("\"Alice\""));
    assert!(content.contains("\"items\""));
    assert!(content.contains("\"metadata\""));
    assert!(content.contains("null"));
}

#[test]
fn test_save_with_different_indentation() {
    // Use nested structure to ensure multi-line formatting
    let inner = vec![(
        "nested_key".to_string(),
        JsonNode::new(JsonValue::String("nested_value".to_string())),
    )];
    let obj = vec![(
        "key".to_string(),
        JsonNode::new(JsonValue::Object(inner)),
    )];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Test with 2 spaces
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, 2, false).unwrap();
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("  \"key\""));

    // Test with 4 spaces
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, 4, false).unwrap();
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("    \"key\""));
}

#[test]
fn test_save_creates_backup() {
    let obj = vec![(
        "version".to_string(),
        JsonNode::new(JsonValue::Number(1.0)),
    )];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    let temp_file = NamedTempFile::new().unwrap();

    // First save
    save_json_file(temp_file.path(), &tree, 2, false).unwrap();

    // Update tree
    let obj = vec![(
        "version".to_string(),
        JsonNode::new(JsonValue::Number(2.0)),
    )];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Second save with backup
    save_json_file(temp_file.path(), &tree, 2, true).unwrap();

    // Check backup exists
    let backup_path = temp_file.path().with_extension("jsonquill.bak");
    assert!(backup_path.exists());

    // Verify backup contains old content
    let backup_content = std::fs::read_to_string(&backup_path).unwrap();
    assert!(backup_content.contains("1"));

    // Verify main file has new content
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("2"));
}

#[test]
fn test_save_without_backup_no_backup_file() {
    let obj = vec![(
        "test".to_string(),
        JsonNode::new(JsonValue::Boolean(true)),
    )];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, 2, false).unwrap();

    let backup_path = temp_file.path().with_extension("jeditor.bak");
    assert!(!backup_path.exists());
}

#[test]
fn test_roundtrip_save_and_load() {
    // Create a complex tree
    let user_obj = vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Bob".to_string())),
        ),
        (
            "email".to_string(),
            JsonNode::new(JsonValue::String("bob@example.com".to_string())),
        ),
    ];

    let obj = vec![
        (
            "user".to_string(),
            JsonNode::new(JsonValue::Object(user_obj)),
        ),
        (
            "count".to_string(),
            JsonNode::new(JsonValue::Number(42.0)),
        ),
        (
            "active".to_string(),
            JsonNode::new(JsonValue::Boolean(true)),
        ),
    ];

    let original_tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Save to file
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &original_tree, 2, false).unwrap();

    // Load from file
    let loaded_tree = load_json_file(temp_file.path()).unwrap();

    // Verify structure matches
    match loaded_tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].0, "user");
            assert_eq!(entries[1].0, "count");
            assert_eq!(entries[2].0, "active");

            // Check user object
            match entries[0].1.value() {
                JsonValue::Object(user_entries) => {
                    assert_eq!(user_entries.len(), 2);
                    assert_eq!(user_entries[0].0, "name");
                    assert_eq!(user_entries[1].0, "email");

                    match user_entries[0].1.value() {
                        JsonValue::String(s) => assert_eq!(s, "Bob"),
                        _ => panic!("Expected string"),
                    }
                }
                _ => panic!("Expected object"),
            }

            // Check count
            match entries[1].1.value() {
                JsonValue::Number(n) => assert_eq!(*n, 42.0),
                _ => panic!("Expected number"),
            }

            // Check active
            match entries[2].1.value() {
                JsonValue::Boolean(b) => assert!(*b),
                _ => panic!("Expected boolean"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_save_special_characters() {
    let obj = vec![
        (
            "newline".to_string(),
            JsonNode::new(JsonValue::String("line1\nline2".to_string())),
        ),
        (
            "quote".to_string(),
            JsonNode::new(JsonValue::String("say \"hello\"".to_string())),
        ),
        (
            "backslash".to_string(),
            JsonNode::new(JsonValue::String("path\\to\\file".to_string())),
        ),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, 2, false).unwrap();

    // Load it back and verify
    let loaded_tree = load_json_file(temp_file.path()).unwrap();

    match loaded_tree.root().value() {
        JsonValue::Object(entries) => {
            match entries[0].1.value() {
                JsonValue::String(s) => assert_eq!(s, "line1\nline2"),
                _ => panic!("Expected string"),
            }
            match entries[1].1.value() {
                JsonValue::String(s) => assert_eq!(s, "say \"hello\""),
                _ => panic!("Expected string"),
            }
            match entries[2].1.value() {
                JsonValue::String(s) => assert_eq!(s, "path\\to\\file"),
                _ => panic!("Expected string"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_save_empty_containers() {
    let obj = vec![
        (
            "empty_object".to_string(),
            JsonNode::new(JsonValue::Object(vec![])),
        ),
        (
            "empty_array".to_string(),
            JsonNode::new(JsonValue::Array(vec![])),
        ),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, 2, false).unwrap();

    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("{}"));
    assert!(content.contains("[]"));
}
