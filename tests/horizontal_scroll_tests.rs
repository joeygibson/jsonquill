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

#[test]
fn test_viewport_width_defaults_to_zero() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let state = EditorState::new_with_default_theme(tree);
    assert_eq!(state.viewport_width(), 0);
}

#[test]
fn test_set_viewport_width() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_viewport_width(80);
    assert_eq!(state.viewport_width(), 80);
}

#[test]
fn test_cursor_line_display_width() {
    // Object with key "name" and value "Alice"
    let node = JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    // Move to the "name" child
    state.move_cursor_down();
    let width = state.cursor_line_display_width();
    assert!(width > 0, "cursor line display width should be positive");
}

#[test]
fn test_scroll_cursor_to_left_edge() {
    // Create a nested object so children are at depth > 0
    // Root: { "outer": { "inner": "value" } }
    // Lines:  [0] outer: {inner: "value"}  (depth 0)
    //         [0,0] inner: "value"          (depth 1)
    let inner = JsonNode::new(JsonValue::Object(vec![(
        "inner".to_string(),
        JsonNode::new(JsonValue::String("value".to_string())),
    )]));
    let node = JsonNode::new(JsonValue::Object(vec![("outer".to_string(), inner)]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    // Cursor starts at [0] (outer), move down to [0,0] (inner) at depth 1
    state.move_cursor_down();
    state.scroll_cursor_to_left_edge();
    // The horizontal offset should be the indent of the cursor line
    assert_eq!(state.horizontal_offset(), 2); // depth 1 * 2 = 2
}

#[test]
fn test_scroll_cursor_to_right_edge() {
    let node = JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_viewport_width(10);
    state.move_cursor_down();
    state.scroll_cursor_to_right_edge();
    // The offset should be set so the line ends at the right edge
    let width = state.cursor_line_display_width();
    if width > 10 {
        assert_eq!(state.horizontal_offset(), width - 10);
    } else {
        assert_eq!(state.horizontal_offset(), 0);
    }
}

#[test]
fn test_scroll_cursor_to_right_edge_narrow_content() {
    // When the content is narrower than the viewport, offset should be 0
    let node = JsonNode::new(JsonValue::Object(vec![(
        "a".to_string(),
        JsonNode::new(JsonValue::Number(1.0)),
    )]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_viewport_width(200); // very wide viewport
    state.move_cursor_down();
    state.scroll_cursor_to_right_edge();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_search_resets_horizontal_offset() {
    let node = JsonNode::new(JsonValue::Object(vec![(
        "a".to_string(),
        JsonNode::new(JsonValue::String("hello".to_string())),
    )]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(20);
    assert_eq!(state.horizontal_offset(), 20);
    // Navigating to top (simulating what happens after search) should reset
    state.jump_to_top();
    assert_eq!(state.horizontal_offset(), 0);
}
