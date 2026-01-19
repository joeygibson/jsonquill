// tests/document_tests.rs
use jeditor::document::node::{JsonNode, JsonValue};

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
        ("name".to_string(), JsonNode::new(JsonValue::String("jeditor".to_string()))),
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
        ("name".to_string(), JsonNode::new(JsonValue::String("test".to_string()))),
        ("enabled".to_string(), JsonNode::new(JsonValue::Boolean(true))),
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
            ("name".to_string(), JsonNode::new(JsonValue::String("first".to_string()))),
        ])),
        JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(2.0))),
            ("name".to_string(), JsonNode::new(JsonValue::String("second".to_string()))),
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
        ("numbers".to_string(), JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
        ]))),
        ("strings".to_string(), JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::String("a".to_string())),
            JsonNode::new(JsonValue::String("b".to_string())),
        ]))),
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
    let deeply_nested = JsonNode::new(JsonValue::Object(vec![
        ("level1".to_string(), JsonNode::new(JsonValue::Object(vec![
            ("level2".to_string(), JsonNode::new(JsonValue::Object(vec![
                ("level3".to_string(), JsonNode::new(JsonValue::String("deep".to_string()))),
            ]))),
        ]))),
    ]));

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
    let original = JsonNode::new(JsonValue::Object(vec![
        ("array".to_string(), JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
        ]))),
    ]));

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
        ("b".to_string(), JsonNode::new(JsonValue::String("test".to_string()))),
    ]));

    let obj2 = JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::String("test".to_string()))),
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
