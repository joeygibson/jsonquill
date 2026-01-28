use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_next_sibling_in_object() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Initially at first sibling [0]
    assert_eq!(state.cursor().path(), &[0]);

    // Move to next sibling [1]
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[1]);

    // Move to next sibling [2]
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[2]);

    // At last sibling, stays at [2]
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[2]);
}

#[test]
fn test_previous_sibling_in_object() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Move to last sibling first
    state.cursor_mut().set_path(vec![2]);

    // Move to previous sibling [1]
    state.move_to_previous_sibling();
    assert_eq!(state.cursor().path(), &[1]);

    // Move to previous sibling [0]
    state.move_to_previous_sibling();
    assert_eq!(state.cursor().path(), &[0]);

    // At first sibling, stays at [0]
    state.move_to_previous_sibling();
    assert_eq!(state.cursor().path(), &[0]);
}

#[test]
fn test_next_sibling_in_array() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(10.0)),
        JsonNode::new(JsonValue::Number(20.0)),
        JsonNode::new(JsonValue::Number(30.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Initially at first element [0]
    assert_eq!(state.cursor().path(), &[0]);

    // Move to next element [1]
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[1]);

    // Move to next element [2]
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[2]);

    // At last element, stays at [2]
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[2]);
}

#[test]
fn test_previous_sibling_in_array() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(10.0)),
        JsonNode::new(JsonValue::Number(20.0)),
        JsonNode::new(JsonValue::Number(30.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Move to last element first
    state.cursor_mut().set_path(vec![2]);

    // Move to previous element [1]
    state.move_to_previous_sibling();
    assert_eq!(state.cursor().path(), &[1]);

    // Move to previous element [0]
    state.move_to_previous_sibling();
    assert_eq!(state.cursor().path(), &[0]);

    // At first element, stays at [0]
    state.move_to_previous_sibling();
    assert_eq!(state.cursor().path(), &[0]);
}

#[test]
fn test_sibling_navigation_at_root() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "key".to_string(),
        JsonNode::new(JsonValue::String("value".to_string())),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Move to root
    state.cursor_mut().set_path(vec![]);

    // Next sibling does nothing at root
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[] as &[usize]);

    // Previous sibling does nothing at root
    state.move_to_previous_sibling();
    assert_eq!(state.cursor().path(), &[] as &[usize]);
}

#[test]
fn test_sibling_navigation_in_nested_structure() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "users".to_string(),
            JsonNode::new(JsonValue::Array(vec![
                JsonNode::new(JsonValue::String("Alice".to_string())),
                JsonNode::new(JsonValue::String("Bob".to_string())),
                JsonNode::new(JsonValue::String("Charlie".to_string())),
            ])),
        ),
        ("count".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Navigate to first array element [0, 0] (Alice)
    state.cursor_mut().set_path(vec![0, 0]);

    // Move to next sibling [0, 1] (Bob)
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[0, 1]);

    // Move to next sibling [0, 2] (Charlie)
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[0, 2]);

    // At last sibling in array, stays at [0, 2]
    state.move_to_next_sibling();
    assert_eq!(state.cursor().path(), &[0, 2]);

    // Move back to Bob
    state.move_to_previous_sibling();
    assert_eq!(state.cursor().path(), &[0, 1]);
}
