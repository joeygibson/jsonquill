// tests/document_tests.rs
use jeditor::document::node::{JsonNode, JsonValue};

#[test]
fn test_create_string_node() {
    let node = JsonNode::new(JsonValue::String("hello".to_string()));
    assert!(matches!(node.value(), JsonValue::String(_)));
}

#[test]
fn test_create_number_node() {
    let node = JsonNode::new(JsonValue::Number(42.0));
    assert!(matches!(node.value(), JsonValue::Number(_)));
}

#[test]
fn test_create_boolean_node() {
    let node = JsonNode::new(JsonValue::Boolean(true));
    assert!(matches!(node.value(), JsonValue::Boolean(true)));
}

#[test]
fn test_create_null_node() {
    let node = JsonNode::new(JsonValue::Null);
    assert!(matches!(node.value(), JsonValue::Null));
}
