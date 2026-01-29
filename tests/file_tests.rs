//! Integration tests for file I/O operations.

use jsonquill::config::Config;
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
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

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
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
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
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

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
    let obj = vec![("key".to_string(), JsonNode::new(JsonValue::Object(inner)))];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Test with 2 spaces
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("  \"key\""));

    // Test with 4 spaces
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(
        temp_file.path(),
        &tree,
        &Config {
            indent_size: 4,
            ..Config::default()
        },
    )
    .unwrap();
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("    \"key\""));
}

#[test]
fn test_save_creates_backup() {
    let obj = vec![("version".to_string(), JsonNode::new(JsonValue::Number(1.0)))];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    let temp_file = NamedTempFile::new().unwrap();

    // First save
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Update tree
    let obj = vec![("version".to_string(), JsonNode::new(JsonValue::Number(2.0)))];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Second save with backup
    save_json_file(
        temp_file.path(),
        &tree,
        &Config {
            create_backup: true,
            ..Config::default()
        },
    )
    .unwrap();

    // Check backup exists
    let mut backup_path = temp_file.path().to_path_buf();
    let original_name = backup_path.file_name().unwrap().to_str().unwrap();
    backup_path.set_file_name(format!("{}.bak", original_name));
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
    let obj = vec![("test".to_string(), JsonNode::new(JsonValue::Boolean(true)))];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    let mut backup_path = temp_file.path().to_path_buf();
    let original_name = backup_path.file_name().unwrap().to_str().unwrap();
    backup_path.set_file_name(format!("{}.bak", original_name));
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
        ("count".to_string(), JsonNode::new(JsonValue::Number(42.0))),
        (
            "active".to_string(),
            JsonNode::new(JsonValue::Boolean(true)),
        ),
    ];

    let original_tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Save to file
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &original_tree, &Config::default()).unwrap();

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
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

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
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("{}"));
    assert!(content.contains("[]"));
}

// ============================================================================
// Document Corruption Tests
// ============================================================================
// These tests verify that save operations never corrupt JSON documents

#[test]
fn test_no_corruption_deeply_nested_structure() {
    // Create a deeply nested structure (10 levels)
    let mut current = JsonNode::new(JsonValue::String("deep value".to_string()));

    for i in (0..10).rev() {
        let obj = vec![(format!("level_{}", i), current)];
        current = JsonNode::new(JsonValue::Object(obj));
    }

    let tree = JsonTree::new(current);

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify the saved JSON is valid by parsing with serde
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Saved JSON should be valid, but got: {}",
        content
    );

    // Verify structure integrity by loading and checking
    let loaded = load_json_file(temp_file.path()).unwrap();
    let mut node = loaded.root();

    // Navigate down 10 levels
    for i in 0..10 {
        match node.value() {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, format!("level_{}", i));
                node = &entries[0].1;
            }
            _ => panic!("Expected object at level {}", i),
        }
    }

    // Verify final value
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "deep value"),
        _ => panic!("Expected string at bottom"),
    }
}

#[test]
fn test_no_corruption_wide_structure() {
    // Create an object with 100 properties
    let mut obj = Vec::new();
    for i in 0..100 {
        obj.push((
            format!("key_{:03}", i),
            JsonNode::new(JsonValue::Number(i as f64)),
        ));
    }

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(parsed.is_ok(), "Saved JSON should be valid");

    // Load and verify all properties
    let loaded = load_json_file(temp_file.path()).unwrap();
    match loaded.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 100);
            for i in 0..100 {
                assert_eq!(entries[i].0, format!("key_{:03}", i));
                match entries[i].1.value() {
                    JsonValue::Number(n) => assert_eq!(*n, i as f64),
                    _ => panic!("Expected number at index {}", i),
                }
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_no_corruption_all_value_types() {
    // Create a document with all JSON value types
    let nested_obj = vec![(
        "nested".to_string(),
        JsonNode::new(JsonValue::Boolean(false)),
    )];

    let nested_arr = vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ];

    let obj = vec![
        ("null_val".to_string(), JsonNode::new(JsonValue::Null)),
        (
            "bool_true".to_string(),
            JsonNode::new(JsonValue::Boolean(true)),
        ),
        (
            "bool_false".to_string(),
            JsonNode::new(JsonValue::Boolean(false)),
        ),
        (
            "int_positive".to_string(),
            JsonNode::new(JsonValue::Number(42.0)),
        ),
        (
            "int_negative".to_string(),
            JsonNode::new(JsonValue::Number(-17.0)),
        ),
        (
            "float_val".to_string(),
            JsonNode::new(JsonValue::Number(3.14159)),
        ),
        (
            "string_empty".to_string(),
            JsonNode::new(JsonValue::String("".to_string())),
        ),
        (
            "string_simple".to_string(),
            JsonNode::new(JsonValue::String("hello".to_string())),
        ),
        (
            "array_empty".to_string(),
            JsonNode::new(JsonValue::Array(vec![])),
        ),
        (
            "array_vals".to_string(),
            JsonNode::new(JsonValue::Array(nested_arr)),
        ),
        (
            "object_empty".to_string(),
            JsonNode::new(JsonValue::Object(vec![])),
        ),
        (
            "object_nested".to_string(),
            JsonNode::new(JsonValue::Object(nested_obj)),
        ),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Saved JSON should be valid, but got: {}",
        content
    );

    // Verify roundtrip
    let loaded = load_json_file(temp_file.path()).unwrap();
    match loaded.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 12);
            assert!(matches!(entries[0].1.value(), JsonValue::Null));
            assert!(matches!(entries[1].1.value(), JsonValue::Boolean(true)));
            assert!(matches!(entries[2].1.value(), JsonValue::Boolean(false)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_no_corruption_unicode_strings() {
    // Test various unicode characters
    let obj = vec![
        (
            "emoji".to_string(),
            JsonNode::new(JsonValue::String("🎉🚀✨".to_string())),
        ),
        (
            "chinese".to_string(),
            JsonNode::new(JsonValue::String("你好世界".to_string())),
        ),
        (
            "arabic".to_string(),
            JsonNode::new(JsonValue::String("مرحبا".to_string())),
        ),
        (
            "cyrillic".to_string(),
            JsonNode::new(JsonValue::String("Привет".to_string())),
        ),
        (
            "mixed".to_string(),
            JsonNode::new(JsonValue::String("Hello 世界 🌍".to_string())),
        ),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(parsed.is_ok(), "Saved JSON with unicode should be valid");

    // Verify exact string preservation
    let loaded = load_json_file(temp_file.path()).unwrap();
    match loaded.root().value() {
        JsonValue::Object(entries) => {
            match entries[0].1.value() {
                JsonValue::String(s) => assert_eq!(s, "🎉🚀✨"),
                _ => panic!("Expected string"),
            }
            match entries[1].1.value() {
                JsonValue::String(s) => assert_eq!(s, "你好世界"),
                _ => panic!("Expected string"),
            }
            match entries[4].1.value() {
                JsonValue::String(s) => assert_eq!(s, "Hello 世界 🌍"),
                _ => panic!("Expected string"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_no_corruption_complex_numbers() {
    // Test various number formats
    let obj = vec![
        ("zero".to_string(), JsonNode::new(JsonValue::Number(0.0))),
        (
            "negative_zero".to_string(),
            JsonNode::new(JsonValue::Number(-0.0)),
        ),
        (
            "large_int".to_string(),
            JsonNode::new(JsonValue::Number(9007199254740991.0)),
        ),
        (
            "small_float".to_string(),
            JsonNode::new(JsonValue::Number(0.000001)),
        ),
        (
            "negative".to_string(),
            JsonNode::new(JsonValue::Number(-999.999)),
        ),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Saved JSON with complex numbers should be valid, but got: {}",
        content
    );

    // Load and verify numbers
    let loaded = load_json_file(temp_file.path()).unwrap();
    match loaded.root().value() {
        JsonValue::Object(entries) => {
            match entries[0].1.value() {
                JsonValue::Number(n) => assert_eq!(*n, 0.0),
                _ => panic!("Expected number"),
            }
            match entries[2].1.value() {
                JsonValue::Number(n) => assert_eq!(*n, 9007199254740991.0),
                _ => panic!("Expected number"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_no_corruption_after_single_edit() {
    // Load a document, make one edit, save, verify integrity
    let original_json = r#"{
  "config": {
    "version": "1.0",
    "features": [
      "auth",
      "logging",
      "metrics"
    ],
    "database": {
      "host": "localhost",
      "port": 5432,
      "name": "mydb"
    }
  },
  "users": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"}
  ]
}"#;

    let mut tree = jsonquill::document::parser::parse_json(original_json).unwrap();

    // Make a single edit: change version from "1.0" to "2.0"
    if let JsonValue::Object(root) = tree.root_mut().value_mut() {
        if let JsonValue::Object(config) = &mut root[0].1.value_mut() {
            if let JsonValue::String(version) = &mut config[0].1.value_mut() {
                *version = "2.0".to_string();
            }
        }
    }

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Saved JSON after edit should be valid, but got: {}",
        content
    );

    // Verify structure integrity
    let serde_parsed = parsed.unwrap();

    // Check that version changed
    assert_eq!(serde_parsed["config"]["version"], "2.0");

    // Check that other fields are intact
    assert_eq!(serde_parsed["config"]["database"]["host"], "localhost");
    assert_eq!(serde_parsed["config"]["database"]["port"], 5432);
    assert_eq!(serde_parsed["users"][0]["name"], "Alice");
    assert_eq!(serde_parsed["users"][1]["name"], "Bob");
    assert_eq!(serde_parsed["config"]["features"][0], "auth");
    assert_eq!(serde_parsed["config"]["features"][1], "logging");
    assert_eq!(serde_parsed["config"]["features"][2], "metrics");
}

#[test]
fn test_no_corruption_after_multiple_edits() {
    let original_json = r#"{
  "app": {
    "name": "MyApp",
    "version": "1.0.0",
    "config": {
      "timeout": 30,
      "retries": 3
    }
  },
  "data": [1, 2, 3, 4, 5]
}"#;

    let mut tree = jsonquill::document::parser::parse_json(original_json).unwrap();

    // Make multiple edits
    if let JsonValue::Object(root) = tree.root_mut().value_mut() {
        // Change app name
        if let JsonValue::Object(app) = &mut root[0].1.value_mut() {
            if let JsonValue::String(name) = &mut app[0].1.value_mut() {
                *name = "UpdatedApp".to_string();
            }

            // Change timeout
            if let JsonValue::Object(config) = &mut app[2].1.value_mut() {
                if let JsonValue::Number(timeout) = &mut config[0].1.value_mut() {
                    *timeout = 60.0;
                }
            }
        }

        // Modify array
        if let JsonValue::Array(data) = &mut root[1].1.value_mut() {
            if let JsonValue::Number(first) = &mut data[0].value_mut() {
                *first = 10.0;
            }
        }
    }

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Saved JSON after multiple edits should be valid, but got: {}",
        content
    );

    // Verify changes
    let serde_parsed = parsed.unwrap();
    assert_eq!(serde_parsed["app"]["name"], "UpdatedApp");
    assert_eq!(serde_parsed["app"]["config"]["timeout"], 60);
    assert_eq!(serde_parsed["data"][0], 10);

    // Verify unmodified parts
    assert_eq!(serde_parsed["app"]["version"], "1.0.0");
    assert_eq!(serde_parsed["app"]["config"]["retries"], 3);
    assert_eq!(serde_parsed["data"][1], 2);
    assert_eq!(serde_parsed["data"][4], 5);
}

#[test]
fn test_no_corruption_array_of_objects() {
    // This is a common structure that could be prone to corruption
    let original_json = r#"[
  {"id": 1, "name": "Item 1", "tags": ["a", "b"]},
  {"id": 2, "name": "Item 2", "tags": ["c", "d"]},
  {"id": 3, "name": "Item 3", "tags": ["e", "f"]}
]"#;

    let tree = jsonquill::document::parser::parse_json(original_json).unwrap();

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Saved JSON should be valid, but got: {}",
        content
    );

    // Verify structure
    let serde_parsed = parsed.unwrap();
    assert!(serde_parsed.is_array());
    assert_eq!(serde_parsed.as_array().unwrap().len(), 3);
    assert_eq!(serde_parsed[0]["id"], 1);
    assert_eq!(serde_parsed[0]["name"], "Item 1");
    assert_eq!(serde_parsed[0]["tags"][0], "a");
    assert_eq!(serde_parsed[2]["tags"][1], "f");
}

#[test]
fn test_no_corruption_mixed_nesting() {
    // Objects containing arrays containing objects containing arrays
    let original_json = r#"{
  "level1": {
    "items": [
      {
        "name": "first",
        "values": [1, 2, 3]
      },
      {
        "name": "second",
        "values": [4, 5, 6]
      }
    ],
    "metadata": {
      "count": 2,
      "tags": ["important", "reviewed"]
    }
  }
}"#;

    let tree = jsonquill::document::parser::parse_json(original_json).unwrap();

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Saved JSON with mixed nesting should be valid, but got: {}",
        content
    );

    // Verify deep structure
    let serde_parsed = parsed.unwrap();
    assert_eq!(serde_parsed["level1"]["items"][0]["values"][2], 3);
    assert_eq!(serde_parsed["level1"]["items"][1]["name"], "second");
    assert_eq!(serde_parsed["level1"]["metadata"]["tags"][1], "reviewed");
}

#[test]
fn test_no_corruption_large_document() {
    // Generate a large document (1000 objects in an array)
    let mut items = Vec::new();
    for i in 0..1000 {
        let item = vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(i as f64))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String(format!("Item {}", i))),
            ),
            (
                "active".to_string(),
                JsonNode::new(JsonValue::Boolean(i % 2 == 0)),
            ),
        ];
        items.push(JsonNode::new(JsonValue::Object(item)));
    }

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(items)));

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    save_json_file(temp_file.path(), &tree, &Config::default()).unwrap();

    // Verify valid JSON
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Large JSON document should be valid after save"
    );

    // Spot check some values
    let serde_parsed = parsed.unwrap();
    assert_eq!(serde_parsed[0]["id"], 0);
    assert_eq!(serde_parsed[0]["active"], true);
    assert_eq!(serde_parsed[500]["name"], "Item 500");
    assert_eq!(serde_parsed[999]["id"], 999);
    assert_eq!(serde_parsed[999]["active"], false);
}

#[test]
fn test_no_corruption_jsonl_format() {
    // Test JSONL (JSON Lines) format
    let original_jsonl = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}"#;

    // JSONL files need special handling - create a temp file with .jsonl extension
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("test.jsonl");
    std::fs::write(&input_path, original_jsonl).unwrap();

    let tree = load_json_file(&input_path).unwrap();

    // Save to a new file with .jsonl extension
    let output_path = temp_dir.path().join("output.jsonl");
    save_json_file(&output_path, &tree, &Config::default()).unwrap();

    // Read back and verify each line is valid JSON
    let content = std::fs::read_to_string(&output_path).unwrap();
    let lines: Vec<&str> = content.trim().split('\n').collect();

    assert_eq!(lines.len(), 3, "Should have 3 JSON lines");

    for (i, line) in lines.iter().enumerate() {
        let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "Line {} should be valid JSON: {}", i, line);
    }

    // Verify content
    let line1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    let line2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    let line3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

    assert_eq!(line1["id"].as_f64().unwrap(), 1.0);
    assert_eq!(line1["name"], "Alice");
    assert_eq!(line2["id"].as_f64().unwrap(), 2.0);
    assert_eq!(line2["name"], "Bob");
    assert_eq!(line3["id"].as_f64().unwrap(), 3.0);
    assert_eq!(line3["name"], "Charlie");
}
