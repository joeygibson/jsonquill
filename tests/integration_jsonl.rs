use jsonquill::config::Config;
use jsonquill::document::node::JsonValue;
use jsonquill::file::loader::load_json_file;
use jsonquill::file::saver::save_json_file;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_full_jsonl_workflow() {
    // Load sample JSONL
    let tree = load_json_file("examples/sample.jsonl").unwrap();

    // Verify structure
    match tree.root().value() {
        JsonValue::JsonlRoot(lines) => {
            assert_eq!(lines.len(), 3);
        }
        _ => panic!("Expected JsonlRoot"),
    }

    // Save to temp file
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("output.jsonl");
    let config = Config::default();

    save_json_file(&output_path, &tree, &config).unwrap();

    // Verify format
    let content = fs::read_to_string(&output_path).unwrap();
    println!("Saved content:\n{}", content);
    assert_eq!(content.lines().count(), 3);
    // Check that there are no array brackets wrapping the whole file
    assert!(!content.starts_with('['));
    assert!(!content.ends_with(']'));
}

#[test]
fn test_edit_jsonl_line() {
    // Load sample JSONL
    let mut tree = load_json_file("examples/sample.jsonl").unwrap();

    // Get first line, second field (name)
    if let JsonValue::JsonlRoot(lines) = tree.root_mut().value_mut() {
        if let JsonValue::Object(fields) = lines[0].value_mut() {
            // Change name field (assuming it's the second field)
            if let JsonValue::String(ref mut name) = fields[1].1.value_mut() {
                *name = "Alice Johnson".to_string();
            }
        }
    }

    // Save
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("edited.jsonl");
    let config = Config::default();

    save_json_file(&output_path, &tree, &config).unwrap();

    // Reload and verify
    let tree2 = load_json_file(&output_path).unwrap();

    if let JsonValue::JsonlRoot(lines) = tree2.root().value() {
        if let JsonValue::Object(fields) = lines[0].value() {
            if let JsonValue::String(name) = fields[1].1.value() {
                assert_eq!(name, "Alice Johnson");
            }
        }
    }
}

#[test]
fn test_delete_jsonl_line() {
    // Load sample JSONL
    let mut tree = load_json_file("examples/sample.jsonl").unwrap();

    // Delete second line
    tree.delete_node(&[1]).unwrap();

    // Should have 2 lines now (was 3, deleted 1)
    match tree.root().value() {
        JsonValue::JsonlRoot(lines) => {
            assert_eq!(lines.len(), 2);
        }
        _ => panic!("Expected JsonlRoot"),
    }

    // Save and verify
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("deleted.jsonl");
    let config = Config::default();

    save_json_file(&output_path, &tree, &config).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content.lines().count(), 2);
}
