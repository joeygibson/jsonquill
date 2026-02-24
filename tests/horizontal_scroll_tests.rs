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

#[test]
fn test_scroll_right() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    assert_eq!(state.horizontal_offset(), 5);
}

#[test]
fn test_scroll_left_clamps_to_zero() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(3);
    state.scroll_left(10);
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_scroll_left() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(10);
    state.scroll_left(3);
    assert_eq!(state.horizontal_offset(), 7);
}

#[test]
fn test_horizontal_offset_resets_on_move_down() {
    let node = JsonNode::new(JsonValue::Object(vec![
        (
            "a".to_string(),
            JsonNode::new(JsonValue::String("hello".to_string())),
        ),
        (
            "b".to_string(),
            JsonNode::new(JsonValue::String("world".to_string())),
        ),
    ]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    assert_eq!(state.horizontal_offset(), 5);
    state.move_cursor_down();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_horizontal_offset_resets_on_move_up() {
    let node = JsonNode::new(JsonValue::Object(vec![
        (
            "a".to_string(),
            JsonNode::new(JsonValue::String("hello".to_string())),
        ),
        (
            "b".to_string(),
            JsonNode::new(JsonValue::String("world".to_string())),
        ),
    ]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    // Move down first so we can move up
    state.move_cursor_down();
    state.scroll_right(5);
    assert_eq!(state.horizontal_offset(), 5);
    state.move_cursor_up();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_horizontal_offset_resets_on_jump_to_top() {
    let node = JsonNode::new(JsonValue::Object(vec![(
        "a".to_string(),
        JsonNode::new(JsonValue::String("hello".to_string())),
    )]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    state.jump_to_top();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_horizontal_offset_resets_on_jump_to_bottom() {
    let node = JsonNode::new(JsonValue::Object(vec![(
        "a".to_string(),
        JsonNode::new(JsonValue::String("hello".to_string())),
    )]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    state.jump_to_bottom();
    assert_eq!(state.horizontal_offset(), 0);
}
