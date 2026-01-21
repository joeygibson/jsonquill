use jeditor::document::node::{JsonNode, JsonValue};
use jeditor::document::tree::JsonTree;
use jeditor::editor::state::EditorState;
use jeditor::editor::mode::EditorMode;

#[test]
fn test_full_edit_workflow() {
    // Create initial document
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Start editing first field (name)
    state.cursor_mut().set_path(vec![0]);
    state.set_mode(EditorMode::Insert);
    state.start_editing();

    // Edit the value
    for ch in "Bob".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Commit the edit
    let result = state.commit_editing();
    assert!(result.is_ok());
    assert!(state.is_dirty());

    // Verify the change
    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "Bob"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_full_delete_workflow() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Delete middle element
    state.cursor_mut().set_path(vec![1]);
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok());

    // Verify only 2 elements remain
    assert_eq!(state.tree_view().lines().len(), 2);
}

#[test]
fn test_full_yank_paste_workflow() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Yank first element
    state.cursor_mut().set_path(vec![0]);
    assert!(state.yank_node());

    // Paste after first element
    let result = state.paste_node_at_cursor();
    assert!(result.is_ok());

    // Should have 3 elements now
    assert_eq!(state.tree_view().lines().len(), 3);
}

#[test]
fn test_edit_cancel_workflow() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ])));
    let mut state = EditorState::new(tree);

    // Start editing
    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    // Make changes
    for ch in "Bob".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Cancel instead of committing
    state.cancel_editing();

    // Verify no change was made
    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "Alice"),
        _ => panic!("Expected string"),
    }
    assert!(!state.is_dirty());
}
