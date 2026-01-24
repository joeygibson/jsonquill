// tests/document_tests.rs
use jsonquill::document::node::{JsonNode, JsonValue};

// ============================================================================
// Basic Node Creation Tests
// ============================================================================

#[test]
fn test_create_string_node() {
    let node = JsonNode::new(JsonValue::String("hello".to_string()));
    assert!(matches!(node.value(), JsonValue::String(_)));
    if let JsonValue::String(s) = node.value() {
        assert_eq!(s, "hello");
    }
}

#[test]
fn test_create_number_node() {
    let node = JsonNode::new(JsonValue::Number(42.0));
    assert!(matches!(node.value(), JsonValue::Number(_)));
    if let JsonValue::Number(n) = node.value() {
        assert_eq!(*n, 42.0);
    }
}

#[test]
fn test_create_boolean_node() {
    let node = JsonNode::new(JsonValue::Boolean(true));
    assert!(matches!(node.value(), JsonValue::Boolean(true)));

    let node_false = JsonNode::new(JsonValue::Boolean(false));
    assert!(matches!(node_false.value(), JsonValue::Boolean(false)));
}

#[test]
fn test_create_null_node() {
    let node = JsonNode::new(JsonValue::Null);
    assert!(matches!(node.value(), JsonValue::Null));
}

// ============================================================================
// Object Node Tests
// ============================================================================

#[test]
fn test_create_object_node() {
    let object = JsonNode::new(JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("jsonquill".to_string())),
        ),
        ("version".to_string(), JsonNode::new(JsonValue::Number(1.0))),
    ]));

    assert!(matches!(object.value(), JsonValue::Object(_)));

    if let JsonValue::Object(fields) = object.value() {
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "name");
        assert_eq!(fields[1].0, "version");
    }
}

#[test]
fn test_create_empty_object() {
    let empty_object = JsonNode::new(JsonValue::Object(vec![]));

    if let JsonValue::Object(fields) = empty_object.value() {
        assert_eq!(fields.len(), 0);
    } else {
        panic!("Expected Object variant");
    }
}

#[test]
fn test_object_with_nested_values() {
    let nested = JsonNode::new(JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("test".to_string())),
        ),
        (
            "enabled".to_string(),
            JsonNode::new(JsonValue::Boolean(true)),
        ),
        ("count".to_string(), JsonNode::new(JsonValue::Number(5.0))),
        ("data".to_string(), JsonNode::new(JsonValue::Null)),
    ]));

    if let JsonValue::Object(fields) = nested.value() {
        assert_eq!(fields.len(), 4);

        // Verify each field
        assert!(matches!(fields[0].1.value(), JsonValue::String(_)));
        assert!(matches!(fields[1].1.value(), JsonValue::Boolean(true)));
        assert!(matches!(fields[2].1.value(), JsonValue::Number(_)));
        assert!(matches!(fields[3].1.value(), JsonValue::Null));
    } else {
        panic!("Expected Object variant");
    }
}

// ============================================================================
// Array Node Tests
// ============================================================================

#[test]
fn test_create_array_node() {
    let array = JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ]));

    assert!(matches!(array.value(), JsonValue::Array(_)));

    if let JsonValue::Array(items) = array.value() {
        assert_eq!(items.len(), 3);
    }
}

#[test]
fn test_create_empty_array() {
    let empty_array = JsonNode::new(JsonValue::Array(vec![]));

    if let JsonValue::Array(items) = empty_array.value() {
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected Array variant");
    }
}

#[test]
fn test_array_with_mixed_types() {
    let mixed_array = JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::String("text".to_string())),
        JsonNode::new(JsonValue::Number(42.0)),
        JsonNode::new(JsonValue::Boolean(true)),
        JsonNode::new(JsonValue::Null),
    ]));

    if let JsonValue::Array(items) = mixed_array.value() {
        assert_eq!(items.len(), 4);
        assert!(matches!(items[0].value(), JsonValue::String(_)));
        assert!(matches!(items[1].value(), JsonValue::Number(_)));
        assert!(matches!(items[2].value(), JsonValue::Boolean(true)));
        assert!(matches!(items[3].value(), JsonValue::Null));
    } else {
        panic!("Expected Array variant");
    }
}

// ============================================================================
// Nested Structure Tests
// ============================================================================

#[test]
fn test_objects_in_arrays() {
    let array_of_objects = JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("first".to_string())),
            ),
        ])),
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(2.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("second".to_string())),
            ),
        ])),
    ]));

    if let JsonValue::Array(items) = array_of_objects.value() {
        assert_eq!(items.len(), 2);

        // Check first object
        if let JsonValue::Object(fields) = items[0].value() {
            assert_eq!(fields.len(), 2);
        } else {
            panic!("Expected Object in array");
        }
    } else {
        panic!("Expected Array variant");
    }
}

#[test]
fn test_arrays_in_objects() {
    let object_with_arrays = JsonNode::new(JsonValue::Object(vec![
        (
            "numbers".to_string(),
            JsonNode::new(JsonValue::Array(vec![
                JsonNode::new(JsonValue::Number(1.0)),
                JsonNode::new(JsonValue::Number(2.0)),
            ])),
        ),
        (
            "strings".to_string(),
            JsonNode::new(JsonValue::Array(vec![
                JsonNode::new(JsonValue::String("a".to_string())),
                JsonNode::new(JsonValue::String("b".to_string())),
            ])),
        ),
    ]));

    if let JsonValue::Object(fields) = object_with_arrays.value() {
        assert_eq!(fields.len(), 2);

        // Check both arrays
        assert!(matches!(fields[0].1.value(), JsonValue::Array(_)));
        assert!(matches!(fields[1].1.value(), JsonValue::Array(_)));
    } else {
        panic!("Expected Object variant");
    }
}

#[test]
fn test_deeply_nested_structure() {
    let deeply_nested = JsonNode::new(JsonValue::Object(vec![(
        "level1".to_string(),
        JsonNode::new(JsonValue::Object(vec![(
            "level2".to_string(),
            JsonNode::new(JsonValue::Object(vec![(
                "level3".to_string(),
                JsonNode::new(JsonValue::String("deep".to_string())),
            )])),
        )])),
    )]));

    if let JsonValue::Object(l1) = deeply_nested.value() {
        if let JsonValue::Object(l2) = l1[0].1.value() {
            if let JsonValue::Object(l3) = l2[0].1.value() {
                if let JsonValue::String(s) = l3[0].1.value() {
                    assert_eq!(s, "deep");
                } else {
                    panic!("Expected String at level 3");
                }
            } else {
                panic!("Expected Object at level 2");
            }
        } else {
            panic!("Expected Object at level 1");
        }
    } else {
        panic!("Expected Object at root");
    }
}

// ============================================================================
// Modification Tracking Tests
// ============================================================================

#[test]
fn test_new_nodes_are_modified() {
    let node = JsonNode::new(JsonValue::String("test".to_string()));
    assert!(node.is_modified(), "New nodes should be marked as modified");
}

#[test]
fn test_value_mut_marks_as_modified() {
    let mut node = JsonNode::new(JsonValue::String("original".to_string()));
    assert!(node.is_modified(), "Should start as modified");

    // Access mutable value (even without changing it)
    let _ = node.value_mut();
    assert!(node.is_modified(), "Should remain modified after value_mut");
}

#[test]
fn test_value_mut_maintains_modified_flag() {
    let mut node = JsonNode::new(JsonValue::Number(1.0));
    assert!(node.is_modified());

    // Mutate the value
    *node.value_mut() = JsonValue::Number(2.0);
    assert!(node.is_modified(), "Should remain modified after mutation");

    // Access value_mut again
    let _ = node.value_mut();
    assert!(node.is_modified(), "Should remain modified");
}

#[test]
fn test_immutable_value_preserves_state() {
    let node = JsonNode::new(JsonValue::Boolean(true));
    assert!(node.is_modified());

    // Reading value immutably shouldn't change modification state
    let _ = node.value();
    assert!(node.is_modified());
}

// ============================================================================
// Clone Behavior Tests
// ============================================================================

#[test]
fn test_clone_preserves_value() {
    let original = JsonNode::new(JsonValue::String("clone me".to_string()));
    let cloned = original.clone();

    assert_eq!(original.value(), cloned.value());
}

#[test]
fn test_clone_preserves_modified_flag() {
    let original = JsonNode::new(JsonValue::Number(42.0));
    let cloned = original.clone();

    assert_eq!(original.is_modified(), cloned.is_modified());
}

#[test]
fn test_clone_is_independent() {
    let original = JsonNode::new(JsonValue::String("original".to_string()));
    let mut cloned = original.clone();

    // Modify the clone
    *cloned.value_mut() = JsonValue::String("modified".to_string());

    // Original should be unchanged
    if let JsonValue::String(s) = original.value() {
        assert_eq!(s, "original");
    } else {
        panic!("Expected String variant");
    }
}

#[test]
fn test_clone_complex_structure() {
    let original = JsonNode::new(JsonValue::Object(vec![(
        "array".to_string(),
        JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
        ])),
    )]));

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ============================================================================
// Equality Tests
// ============================================================================

#[test]
fn test_equality_same_values() {
    let node1 = JsonNode::new(JsonValue::String("test".to_string()));
    let node2 = JsonNode::new(JsonValue::String("test".to_string()));

    assert_eq!(node1, node2);
}

#[test]
fn test_equality_different_values() {
    let node1 = JsonNode::new(JsonValue::String("test1".to_string()));
    let node2 = JsonNode::new(JsonValue::String("test2".to_string()));

    assert_ne!(node1, node2);
}

#[test]
fn test_equality_different_types() {
    let node1 = JsonNode::new(JsonValue::String("42".to_string()));
    let node2 = JsonNode::new(JsonValue::Number(42.0));

    assert_ne!(node1, node2);
}

#[test]
fn test_equality_complex_structures() {
    let obj1 = JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        (
            "b".to_string(),
            JsonNode::new(JsonValue::String("test".to_string())),
        ),
    ]));

    let obj2 = JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        (
            "b".to_string(),
            JsonNode::new(JsonValue::String("test".to_string())),
        ),
    ]));

    assert_eq!(obj1, obj2);
}

#[test]
fn test_equality_order_matters_in_objects() {
    let obj1 = JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    ]));

    let obj2 = JsonNode::new(JsonValue::Object(vec![
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
    ]));

    // Order matters in our Vec-based representation
    assert_ne!(obj1, obj2);
}

#[test]
fn test_equality_empty_collections() {
    let empty_obj1 = JsonNode::new(JsonValue::Object(vec![]));
    let empty_obj2 = JsonNode::new(JsonValue::Object(vec![]));
    assert_eq!(empty_obj1, empty_obj2);

    let empty_arr1 = JsonNode::new(JsonValue::Array(vec![]));
    let empty_arr2 = JsonNode::new(JsonValue::Array(vec![]));
    assert_eq!(empty_arr1, empty_arr2);

    // Empty object and empty array are different
    assert_ne!(empty_obj1, empty_arr1);
}

// ============================================================================
// Tree Structure Tests
// ============================================================================

use jsonquill::document::tree::JsonTree;

#[test]
fn test_create_empty_object_tree() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    assert!(tree.root().value().is_object());
}

#[test]
fn test_tree_get_child() {
    let obj = vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )];

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
    let path = vec![0]; // First child
    let child = tree.get_node(&path);
    assert!(child.is_some());
}

#[test]
fn test_tree_root_access() {
    let root_node = JsonNode::new(JsonValue::Number(42.0));
    let tree = JsonTree::new(root_node.clone());

    assert_eq!(tree.root().value(), root_node.value());
}

#[test]
fn test_tree_root_mut_access() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::String("original".to_string())));

    *tree.root_mut().value_mut() = JsonValue::String("modified".to_string());

    if let JsonValue::String(s) = tree.root().value() {
        assert_eq!(s, "modified");
    } else {
        panic!("Expected String value");
    }
}

#[test]
fn test_tree_get_empty_path() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Boolean(true)));
    let empty_path: Vec<usize> = vec![];

    // Empty path should return root
    let node = tree.get_node(&empty_path);
    assert!(node.is_some());
    assert!(matches!(node.unwrap().value(), JsonValue::Boolean(true)));
}

#[test]
fn test_tree_navigate_object() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "first".to_string(),
            JsonNode::new(JsonValue::String("a".to_string())),
        ),
        (
            "second".to_string(),
            JsonNode::new(JsonValue::String("b".to_string())),
        ),
        (
            "third".to_string(),
            JsonNode::new(JsonValue::String("c".to_string())),
        ),
    ])));

    // Test accessing each field by index
    let node0 = tree.get_node(&[0]).unwrap();
    if let JsonValue::String(s) = node0.value() {
        assert_eq!(s, "a");
    } else {
        panic!("Expected String");
    }

    let node1 = tree.get_node(&[1]).unwrap();
    if let JsonValue::String(s) = node1.value() {
        assert_eq!(s, "b");
    } else {
        panic!("Expected String");
    }

    let node2 = tree.get_node(&[2]).unwrap();
    if let JsonValue::String(s) = node2.value() {
        assert_eq!(s, "c");
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_tree_navigate_array() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));

    // Test accessing each element by index
    for i in 0..3 {
        let node = tree.get_node(&[i]).unwrap();
        if let JsonValue::Number(n) = node.value() {
            assert_eq!(*n, (i + 1) as f64);
        } else {
            panic!("Expected Number");
        }
    }
}

#[test]
fn test_tree_navigate_nested() {
    // Create: {"data": {"items": [1, 2, 3]}}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "data".to_string(),
        JsonNode::new(JsonValue::Object(vec![(
            "items".to_string(),
            JsonNode::new(JsonValue::Array(vec![
                JsonNode::new(JsonValue::Number(1.0)),
                JsonNode::new(JsonValue::Number(2.0)),
                JsonNode::new(JsonValue::Number(3.0)),
            ])),
        )])),
    )])));

    // Navigate: root -> "data" (0) -> "items" (0) -> element 1 (1)
    let path = vec![0, 0, 1];
    let node = tree.get_node(&path).unwrap();

    if let JsonValue::Number(n) = node.value() {
        assert_eq!(*n, 2.0);
    } else {
        panic!("Expected Number(2.0)");
    }
}

#[test]
fn test_tree_navigate_deeply_nested() {
    // Create: {"a": {"b": {"c": {"d": "deep"}}}}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "a".to_string(),
        JsonNode::new(JsonValue::Object(vec![(
            "b".to_string(),
            JsonNode::new(JsonValue::Object(vec![(
                "c".to_string(),
                JsonNode::new(JsonValue::Object(vec![(
                    "d".to_string(),
                    JsonNode::new(JsonValue::String("deep".to_string())),
                )])),
            )])),
        )])),
    )])));

    // Navigate through all levels: each level has index 0
    let path = vec![0, 0, 0, 0];
    let node = tree.get_node(&path).unwrap();

    if let JsonValue::String(s) = node.value() {
        assert_eq!(s, "deep");
    } else {
        panic!("Expected String(\"deep\")");
    }
}

#[test]
fn test_tree_get_node_out_of_bounds() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));

    // Array has only 1 element (index 0), try to access index 1
    let node = tree.get_node(&[1]);
    assert!(node.is_none());

    // Try to access index 99
    let node = tree.get_node(&[99]);
    assert!(node.is_none());
}

#[test]
fn test_tree_get_node_invalid_path_non_container() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "value".to_string(),
        JsonNode::new(JsonValue::Number(42.0)),
    )])));

    // Try to navigate into a Number (which is not a container)
    let path = vec![0, 0]; // First field, then try to index into the number
    let node = tree.get_node(&path);
    assert!(node.is_none());
}

#[test]
fn test_tree_get_node_invalid_path_through_string() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::String("test".to_string())));

    // Try to navigate into a String
    let path = vec![0];
    let node = tree.get_node(&path);
    assert!(node.is_none());
}

#[test]
fn test_tree_get_node_mut() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )])));

    // Modify the first field
    let path = vec![0];
    if let Some(node) = tree.get_node_mut(&path) {
        *node.value_mut() = JsonValue::String("Bob".to_string());
    }

    // Verify the change
    let node = tree.get_node(&path).unwrap();
    if let JsonValue::String(s) = node.value() {
        assert_eq!(s, "Bob");
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_tree_get_node_mut_nested() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "data".to_string(),
        JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
        ])),
    )])));

    // Modify nested array element
    let path = vec![0, 1]; // "data" field, second array element
    if let Some(node) = tree.get_node_mut(&path) {
        *node.value_mut() = JsonValue::Number(99.0);
    }

    // Verify the change
    let node = tree.get_node(&path).unwrap();
    if let JsonValue::Number(n) = node.value() {
        assert_eq!(*n, 99.0);
    } else {
        panic!("Expected Number(99.0)");
    }
}

#[test]
fn test_tree_get_node_mut_out_of_bounds() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));

    // Try to mutate out of bounds
    let path = vec![5];
    let node = tree.get_node_mut(&path);
    assert!(node.is_none());
}

#[test]
fn test_tree_get_node_mut_invalid_path() {
    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "value".to_string(),
        JsonNode::new(JsonValue::Boolean(true)),
    )])));

    // Try to navigate through a non-container
    let path = vec![0, 0];
    let node = tree.get_node_mut(&path);
    assert!(node.is_none());
}

#[test]
fn test_tree_clone() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("original".to_string())),
    )])));

    let mut cloned = tree.clone();

    // Modify the clone
    if let Some(node) = cloned.get_node_mut(&[0]) {
        *node.value_mut() = JsonValue::String("modified".to_string());
    }

    // Original should be unchanged
    let original_node = tree.get_node(&[0]).unwrap();
    if let JsonValue::String(s) = original_node.value() {
        assert_eq!(s, "original");
    } else {
        panic!("Expected String");
    }

    // Clone should be modified
    let cloned_node = cloned.get_node(&[0]).unwrap();
    if let JsonValue::String(s) = cloned_node.value() {
        assert_eq!(s, "modified");
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_tree_with_mixed_array_in_object() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "values".to_string(),
        JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::String("text".to_string())),
            JsonNode::new(JsonValue::Number(123.0)),
            JsonNode::new(JsonValue::Boolean(false)),
            JsonNode::new(JsonValue::Null),
        ])),
    )])));

    // Access each type in the array
    assert!(matches!(
        tree.get_node(&[0, 0]).unwrap().value(),
        JsonValue::String(_)
    ));
    assert!(matches!(
        tree.get_node(&[0, 1]).unwrap().value(),
        JsonValue::Number(_)
    ));
    assert!(matches!(
        tree.get_node(&[0, 2]).unwrap().value(),
        JsonValue::Boolean(false)
    ));
    assert!(matches!(
        tree.get_node(&[0, 3]).unwrap().value(),
        JsonValue::Null
    ));
}

#[test]
fn test_tree_with_array_of_objects() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Object(vec![(
            "id".to_string(),
            JsonNode::new(JsonValue::Number(1.0)),
        )])),
        JsonNode::new(JsonValue::Object(vec![(
            "id".to_string(),
            JsonNode::new(JsonValue::Number(2.0)),
        )])),
    ])));

    // Navigate to second object's id field
    let path = vec![1, 0]; // Second array element, first object field
    let node = tree.get_node(&path).unwrap();

    if let JsonValue::Number(n) = node.value() {
        assert_eq!(*n, 2.0);
    } else {
        panic!("Expected Number(2.0)");
    }
}

// ============================================================================
// JsonValue Helper Method Tests
// ============================================================================

#[test]
fn test_is_object() {
    let obj = JsonValue::Object(vec![]);
    assert!(obj.is_object());

    let arr = JsonValue::Array(vec![]);
    assert!(!arr.is_object());

    let str_val = JsonValue::String("test".to_string());
    assert!(!str_val.is_object());
}

#[test]
fn test_is_array() {
    let arr = JsonValue::Array(vec![]);
    assert!(arr.is_array());

    let obj = JsonValue::Object(vec![]);
    assert!(!obj.is_array());

    let num = JsonValue::Number(42.0);
    assert!(!num.is_array());
}

#[test]
fn test_is_container() {
    let obj = JsonValue::Object(vec![]);
    assert!(obj.is_container());

    let arr = JsonValue::Array(vec![]);
    assert!(arr.is_container());

    let str_val = JsonValue::String("test".to_string());
    assert!(!str_val.is_container());

    let num = JsonValue::Number(42.0);
    assert!(!num.is_container());

    let bool_val = JsonValue::Boolean(true);
    assert!(!bool_val.is_container());

    let null_val = JsonValue::Null;
    assert!(!null_val.is_container());
}

// ============================================================================
// JSON Parser Tests
// ============================================================================

use jsonquill::document::parser::parse_json;

#[test]
fn test_parse_simple_object() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let tree = parse_json(json).unwrap();

    match tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "name");
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_parse_nested_structure() {
    let json = r#"{"user": {"name": "Bob"}}"#;
    let tree = parse_json(json).unwrap();

    let user_node = tree.get_node(&[0]);
    assert!(user_node.is_some());
}

// ============================================================================
// Delete Node Tests
// ============================================================================

#[test]
fn test_delete_object_property() {
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));

    // Delete second property (index 1)
    let result = tree.delete_node(&[1]);
    assert!(result.is_ok());

    // Verify only 2 properties remain
    match tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "a");
            assert_eq!(entries[1].0, "c");
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_delete_array_element() {
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(10.0)),
        JsonNode::new(JsonValue::Number(20.0)),
        JsonNode::new(JsonValue::Number(30.0)),
    ])));

    // Delete middle element (index 1)
    let result = tree.delete_node(&[1]);
    assert!(result.is_ok());

    // Verify only 2 elements remain
    match tree.root().value() {
        JsonValue::Array(elements) => {
            assert_eq!(elements.len(), 2);
            match elements[0].value() {
                JsonValue::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("Expected number"),
            }
            match elements[1].value() {
                JsonValue::Number(n) => assert_eq!(*n, 30.0),
                _ => panic!("Expected number"),
            }
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_delete_nested_node() {
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "user".to_string(),
        JsonNode::new(JsonValue::Object(vec![
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
            ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
        ])),
    )])));

    // Delete nested property at path [0, 1] (user.age)
    let result = tree.delete_node(&[0, 1]);
    assert!(result.is_ok());

    // Verify only name remains
    let user_node = tree.get_node(&[0]).unwrap();
    match user_node.value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "name");
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_delete_root_fails() {
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));

    // Cannot delete root node (empty path)
    let result = tree.delete_node(&[]);
    assert!(result.is_err());
}

#[test]
fn test_delete_invalid_path() {
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "a".to_string(),
        JsonNode::new(JsonValue::Number(1.0)),
    )])));

    // Try to delete non-existent path
    let result = tree.delete_node(&[99]);
    assert!(result.is_err());
}

// ============================================================================
// Insert Node Tests
// ============================================================================

#[test]
fn test_insert_node_in_object() {
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));

    // Insert new node at index 1 (between a and c)
    let new_node = JsonNode::new(JsonValue::Number(2.0));
    let result = tree.insert_node_in_object(&[1], "b".to_string(), new_node);
    assert!(result.is_ok());

    // Verify three properties in order
    match tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].0, "a");
            assert_eq!(entries[1].0, "b");
            assert_eq!(entries[2].0, "c");
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_insert_node_in_array() {
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(10.0)),
        JsonNode::new(JsonValue::Number(30.0)),
    ])));

    // Insert new node at index 1 (between 10 and 30)
    let new_node = JsonNode::new(JsonValue::Number(20.0));
    let result = tree.insert_node_in_array(&[1], new_node);
    assert!(result.is_ok());

    // Verify three elements in order
    match tree.root().value() {
        JsonValue::Array(elements) => {
            assert_eq!(elements.len(), 3);
            match elements[0].value() {
                JsonValue::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("Expected number"),
            }
            match elements[1].value() {
                JsonValue::Number(n) => assert_eq!(*n, 20.0),
                _ => panic!("Expected number"),
            }
            match elements[2].value() {
                JsonValue::Number(n) => assert_eq!(*n, 30.0),
                _ => panic!("Expected number"),
            }
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_insert_node_at_end() {
    use jsonquill::document::tree::JsonTree;

    let mut tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));

    // Insert at end (index 1, which equals length)
    let new_node = JsonNode::new(JsonValue::Number(2.0));
    let result = tree.insert_node_in_array(&[1], new_node);
    assert!(result.is_ok());

    match tree.root().value() {
        JsonValue::Array(elements) => {
            assert_eq!(elements.len(), 2);
        }
        _ => panic!("Expected array"),
    }
}
