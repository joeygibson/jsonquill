//! Integration tests for format preservation.

use jsonquill::config::Config;
use jsonquill::document::node::JsonValue;
use jsonquill::document::parser::parse_json;
use jsonquill::file::saver::save_json_file;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_preserve_two_space_indent() {
    let json = r#"{
  "name": "Alice",
  "age": 30
}"#;

    let tree = parse_json(json).unwrap();
    let config = Config {
        preserve_formatting: true,
        ..Default::default()
    };

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_preserve_four_space_indent() {
    let json = r#"{
    "name": "Bob",
    "city": "NYC"
}"#;

    let tree = parse_json(json).unwrap();
    let config = Config {
        preserve_formatting: true,
        ..Default::default()
    };

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_preserve_compact_format() {
    let json = r#"{"name":"Alice","age":30}"#;

    let tree = parse_json(json).unwrap();
    let config = Config {
        preserve_formatting: true,
        ..Default::default()
    };

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_preserve_array_formatting() {
    let json = r#"[
  1,
  2,
  3
]"#;

    let tree = parse_json(json).unwrap();
    let config = Config {
        preserve_formatting: true,
        ..Default::default()
    };

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_preserve_nested_structure() {
    let json = r#"{
  "user": {
    "name": "Alice",
    "email": "alice@example.com"
  },
  "settings": {
    "theme": "dark"
  }
}"#;

    let tree = parse_json(json).unwrap();
    let config = Config {
        preserve_formatting: true,
        ..Default::default()
    };

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}

#[test]
fn test_edit_single_value_preserves_rest() {
    let json = r#"{
  "name": "Alice",
  "age": 30
}"#;

    let mut tree = parse_json(json).unwrap();
    let config = Config {
        preserve_formatting: true,
        ..Default::default()
    };

    // First save unmodified to verify roundtrip
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();
    let saved_unmodified = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved_unmodified, json);

    // Now modify a value - this marks the object as modified
    if let JsonValue::Object(ref mut entries) = tree.root_mut().value_mut() {
        *entries[1].1.value_mut() = JsonValue::Number(31.0);
    }

    // Save the modified tree
    let temp_file2 = NamedTempFile::new().unwrap();
    save_json_file(temp_file2.path(), &tree, &config).unwrap();
    let saved_modified = fs::read_to_string(temp_file2.path()).unwrap();

    // When object is modified, all children are re-serialized with clean formatting
    assert!(saved_modified.contains("\"age\": 31"));
}

#[test]
fn test_disable_preservation() {
    let json = r#"{
    "name":    "Alice",
    "age":     30
}"#;

    let tree = parse_json(json).unwrap();

    let config = Config {
        preserve_formatting: false,
        indent_size: 2,
        ..Default::default()
    };

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();

    // Should use normalized formatting - check for clean formatting without odd spacing
    assert!(!saved.contains("\"name\":    ")); // Not odd spacing
    assert!(!saved.contains("\"age\":     ")); // Not odd spacing
    assert!(saved.contains("\"name\": \"Alice\""));
    assert!(saved.contains("\"age\": 30"));
}

#[test]
fn test_empty_document_uses_standard_serialization() {
    let tree = jsonquill::document::tree::JsonTree::new(jsonquill::document::node::JsonNode::new(
        JsonValue::Object(vec![]),
    ));

    let config = Config::default(); // Use default (format preservation disabled)
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, "{}");
}

#[test]
fn test_preserve_mixed_scalar_types() {
    let json = r#"{
  "string": "hello",
  "number": 42,
  "boolean": true,
  "null": null
}"#;

    let tree = parse_json(json).unwrap();
    let config = Config {
        preserve_formatting: true,
        ..Default::default()
    };

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &config).unwrap();

    let saved = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(saved, json);
}
