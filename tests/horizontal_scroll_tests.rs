use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_horizontal_offset_defaults_to_zero() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let state = EditorState::new_with_default_theme(tree);
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_set_horizontal_offset() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_horizontal_offset(10);
    assert_eq!(state.horizontal_offset(), 10);
}

#[test]
fn test_reset_horizontal_offset() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_horizontal_offset(25);
    assert_eq!(state.horizontal_offset(), 25);
    state.reset_horizontal_offset();
    assert_eq!(state.horizontal_offset(), 0);
}
