// Tests for new state fields in EditorState
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_jumplist_initialized() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert_eq!(state.jumplist().len(), 0);
}

#[test]
fn test_marks_initialized() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert_eq!(state.marks().get_mark('a'), None);
}

#[test]
fn test_visual_state_initialized() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert_eq!(state.visual_anchor(), None);
    assert_eq!(state.visual_selection().len(), 0);
}

#[test]
fn test_last_command_initialized() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert!(state.last_command().is_none());
}
