use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_undo_after_delete() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "key1".to_string(),
            JsonNode::new(JsonValue::String("value1".to_string())),
        ),
        (
            "key2".to_string(),
            JsonNode::new(JsonValue::String("value2".to_string())),
        ),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Delete first node
    state.cursor_mut().set_path(vec![0]);
    state.delete_node_at_cursor().unwrap();

    // Tree should have only one node now
    assert!(state.tree().get_node(&[0]).is_some());
    assert!(state.tree().get_node(&[1]).is_none());

    // Undo
    assert!(state.undo());

    // Both nodes should be restored
    assert!(state.tree().get_node(&[0]).is_some());
    assert!(state.tree().get_node(&[1]).is_some());
}

#[test]
fn test_redo_after_undo() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "key".to_string(),
        JsonNode::new(JsonValue::String("value".to_string())),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Delete and undo
    state.cursor_mut().set_path(vec![0]);
    state.delete_node_at_cursor().unwrap();
    state.undo();

    // Redo
    assert!(state.redo());

    // Node should be deleted again
    assert!(state.tree().get_node(&[0]).is_none());
}

#[test]
fn test_branching_after_undo() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Delete first node
    state.cursor_mut().set_path(vec![0]);
    state.delete_node_at_cursor().unwrap();

    // Undo
    state.undo();

    // Delete second node (creates branch)
    state.cursor_mut().set_path(vec![1]);
    state.delete_node_at_cursor().unwrap();

    // Should have node [0] but not [1]
    assert!(state.tree().get_node(&[0]).is_some());
    assert!(state.tree().get_node(&[1]).is_none());

    // Undo
    state.undo();

    // Both nodes restored
    assert!(state.tree().get_node(&[0]).is_some());
    assert!(state.tree().get_node(&[1]).is_some());

    // Redo should go to newest branch (deleted [1])
    state.redo();
    assert!(state.tree().get_node(&[0]).is_some());
    assert!(state.tree().get_node(&[1]).is_none());
}

#[test]
fn test_undo_at_start_returns_false() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);

    // No changes made, cannot undo
    assert!(!state.undo());
}

#[test]
fn test_redo_at_end_returns_false() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);

    // No redo available
    assert!(!state.redo());
}

#[test]
fn test_undo_after_paste() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Yank and paste
    state.cursor_mut().set_path(vec![0]);
    state.yank_nodes(1);
    state.paste_node_at_cursor().unwrap();

    // Should have 2 elements
    assert!(state.tree().get_node(&[0]).is_some());
    assert!(state.tree().get_node(&[1]).is_some());

    // Undo paste
    state.undo();

    // Back to 1 element
    assert!(state.tree().get_node(&[0]).is_some());
    assert!(state.tree().get_node(&[1]).is_none());
}

#[test]
fn test_undo_after_edit() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::String("old".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    // Start editing
    state.cursor_mut().set_path(vec![]);
    state.set_mode(jsonquill::editor::mode::EditorMode::Insert);
    state.start_editing();

    // Clear pre-populated value and type new value
    state.clear_edit_buffer();
    for ch in "new".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Commit
    state.commit_editing().unwrap();

    // Value should be "new"
    if let JsonValue::String(s) = state.tree().root().value() {
        assert_eq!(s, "new");
    } else {
        panic!("Expected string value");
    }

    // Undo
    state.undo();

    // Value should be "old" again
    if let JsonValue::String(s) = state.tree().root().value() {
        assert_eq!(s, "old");
    } else {
        panic!("Expected string value");
    }
}
