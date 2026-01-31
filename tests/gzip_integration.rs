use jsonquill::config::Config;
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::parser::parse_json;
use jsonquill::document::tree::JsonTree;
use jsonquill::file::loader::load_json_file;
use jsonquill::file::saver::save_json_file;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a temporary file path with the given extension
fn temp_file_path(dir: &TempDir, name: &str) -> PathBuf {
    dir.path().join(name)
}

/// Helper function to compare two JSON trees for structural equality
fn trees_equal(tree1: &JsonTree, tree2: &JsonTree) -> bool {
    nodes_equal(tree1.root(), tree2.root())
}

/// Recursively compare two JSON nodes for structural equality
fn nodes_equal(node1: &JsonNode, node2: &JsonNode) -> bool {
    match (node1.value(), node2.value()) {
        (JsonValue::Object(obj1), JsonValue::Object(obj2)) => {
            if obj1.len() != obj2.len() {
                return false;
            }
            for ((k1, v1), (k2, v2)) in obj1.iter().zip(obj2.iter()) {
                if k1 != k2 || !nodes_equal(v1, v2) {
                    return false;
                }
            }
            true
        }
        (JsonValue::Array(arr1), JsonValue::Array(arr2)) => {
            if arr1.len() != arr2.len() {
                return false;
            }
            for (v1, v2) in arr1.iter().zip(arr2.iter()) {
                if !nodes_equal(v1, v2) {
                    return false;
                }
            }
            true
        }
        (JsonValue::String(s1), JsonValue::String(s2)) => s1 == s2,
        (JsonValue::Number(n1), JsonValue::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
        (JsonValue::Boolean(b1), JsonValue::Boolean(b2)) => b1 == b2,
        (JsonValue::Null, JsonValue::Null) => true,
        (JsonValue::JsonlRoot(lines1), JsonValue::JsonlRoot(lines2)) => {
            if lines1.len() != lines2.len() {
                return false;
            }
            for (v1, v2) in lines1.iter().zip(lines2.iter()) {
                if !nodes_equal(v1, v2) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

#[test]
fn test_roundtrip_compressed_json() {
    // Create a temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let json_gz_path = temp_file_path(&temp_dir, "test.json.gz");

    // Create a JSON document with various data types
    let original_json = r#"{
  "name": "Alice",
  "age": 30,
  "active": true,
  "balance": null,
  "scores": [95, 87, 92],
  "metadata": {
    "created": "2024-01-01",
    "tags": ["test", "sample"]
  }
}"#;

    // Parse the original JSON
    let original_tree = parse_json(original_json).expect("Failed to parse JSON");

    // Save as compressed file
    let config = Config::default();
    save_json_file(&json_gz_path, &original_tree, &config).expect("Failed to save compressed JSON");

    // Verify the file was created and is compressed
    assert!(json_gz_path.exists());
    let compressed_size = fs::metadata(&json_gz_path)
        .expect("Failed to get file metadata")
        .len();
    assert!(compressed_size > 0);

    // Load the compressed file
    let loaded_tree = load_json_file(&json_gz_path).expect("Failed to load compressed JSON");

    // Verify the content is identical
    assert!(
        trees_equal(&original_tree, &loaded_tree),
        "Roundtrip JSON content mismatch"
    );
}

#[test]
fn test_roundtrip_compressed_jsonl() {
    // Create a temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let jsonl_gz_path = temp_file_path(&temp_dir, "test.jsonl.gz");

    // Create JSONL tree directly
    let lines = vec![
        parse_json_value(r#"{"id": 1, "name": "Alice", "score": 95}"#),
        parse_json_value(r#"{"id": 2, "name": "Bob", "score": 87}"#),
        parse_json_value(r#"{"id": 3, "name": "Charlie", "score": 92}"#),
        parse_json_value(r#"{"id": 4, "name": "Diana", "score": 88}"#),
        parse_json_value(r#"{"id": 5, "name": "Eve", "score": 94}"#),
    ];

    let original_tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));

    // Save as compressed file
    let config = Config::default();
    save_json_file(&jsonl_gz_path, &original_tree, &config)
        .expect("Failed to save compressed JSONL");

    // Verify the file was created and is compressed
    assert!(jsonl_gz_path.exists());
    let compressed_size = fs::metadata(&jsonl_gz_path)
        .expect("Failed to get file metadata")
        .len();
    assert!(compressed_size > 0);

    // Load the compressed file
    let loaded_tree = load_json_file(&jsonl_gz_path).expect("Failed to load compressed JSONL");

    // Verify it's still JSONL format
    assert!(
        matches!(loaded_tree.root().value(), JsonValue::JsonlRoot(_)),
        "Loaded tree should be JSONL format"
    );

    // Verify the content is identical
    assert!(
        trees_equal(&original_tree, &loaded_tree),
        "Roundtrip JSONL content mismatch"
    );
}

/// Helper to parse a JSON string into a JsonNode
fn parse_json_value(json: &str) -> JsonNode {
    use jsonquill::document::parser::parse_value;
    let serde_value: serde_json::Value = serde_json::from_str(json).unwrap();
    parse_value(&serde_value)
}

#[test]
fn test_large_compressed_file() {
    // Create a temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let large_json_gz_path = temp_file_path(&temp_dir, "large.json.gz");

    // Generate a large JSON array with 100k items programmatically
    let mut items = Vec::new();
    for i in 0..100_000 {
        let obj = vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(i as f64))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String(format!("User{}", i))),
            ),
            (
                "email".to_string(),
                JsonNode::new(JsonValue::String(format!("user{}@example.com", i))),
            ),
            (
                "active".to_string(),
                JsonNode::new(JsonValue::Boolean(i % 2 == 0)),
            ),
        ];
        items.push(JsonNode::new(JsonValue::Object(obj)));
    }

    let original_tree = JsonTree::new(JsonNode::new(JsonValue::Array(items)));

    // Save as compressed file
    let config = Config::default();
    save_json_file(&large_json_gz_path, &original_tree, &config)
        .expect("Failed to save large compressed JSON");

    // Verify the file was created
    assert!(large_json_gz_path.exists());

    // Check compression ratio
    let compressed_size = fs::metadata(&large_json_gz_path)
        .expect("Failed to get file metadata")
        .len();

    // Read the uncompressed file to get its size
    let uncompressed_content = fs::read_to_string(&large_json_gz_path.with_extension(""))
        .unwrap_or_else(|_| {
            // If we can't read uncompressed, save it temporarily
            let temp_json_path = temp_file_path(&temp_dir, "large.json");
            save_json_file(&temp_json_path, &original_tree, &config).unwrap();
            fs::read_to_string(&temp_json_path).unwrap()
        });
    let uncompressed_size = uncompressed_content.len() as u64;

    println!(
        "Compression ratio: {:.2}% (uncompressed: {} bytes, compressed: {} bytes)",
        (compressed_size as f64 / uncompressed_size as f64) * 100.0,
        uncompressed_size,
        compressed_size
    );

    // Verify compression is effective (compressed size should be significantly smaller)
    // JSON with repetitive structure should compress to less than 50% of original size
    assert!(
        compressed_size < uncompressed_size / 2,
        "Compression ratio not effective: {} compressed to {} bytes",
        uncompressed_size,
        compressed_size
    );

    // Load the compressed file
    let loaded_tree =
        load_json_file(&large_json_gz_path).expect("Failed to load large compressed JSON");

    // Verify the structure
    match loaded_tree.root().value() {
        JsonValue::Array(loaded_items) => {
            assert_eq!(loaded_items.len(), 100_000, "Array should have 100k items");

            // Verify first item structure
            if let JsonValue::Object(first_obj) = &loaded_items[0].value() {
                assert_eq!(first_obj.len(), 4);
                let keys: Vec<&String> = first_obj.iter().map(|(k, _)| k).collect();
                assert!(keys.contains(&&"id".to_string()));
                assert!(keys.contains(&&"name".to_string()));
                assert!(keys.contains(&&"email".to_string()));
                assert!(keys.contains(&&"active".to_string()));
            } else {
                panic!("First item should be an object");
            }

            // Verify last item structure
            if let JsonValue::Object(last_obj) = &loaded_items[99_999].value() {
                assert_eq!(last_obj.len(), 4);
                let keys: Vec<&String> = last_obj.iter().map(|(k, _)| k).collect();
                assert!(keys.contains(&&"id".to_string()));
                assert!(keys.contains(&&"name".to_string()));
                assert!(keys.contains(&&"email".to_string()));
                assert!(keys.contains(&&"active".to_string()));
            } else {
                panic!("Last item should be an object");
            }

            // Verify structural equality with original
            assert!(
                trees_equal(&original_tree, &loaded_tree),
                "Large file roundtrip content mismatch"
            );
        }
        _ => panic!("Root should be an array"),
    }
}
