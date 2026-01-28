use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_named_register_yank_paste() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank to register 'a'
    state.set_pending_register('a', false);
    assert!(state.yank_nodes(1));
    state.clear_register_pending();

    // Move to second element
    state.move_cursor_down();

    // Paste from register 'a'
    state.set_pending_register('a', false);
    assert!(state.paste_nodes_at_cursor().is_ok());
}

#[test]
fn test_append_mode() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank first node to 'a'
    state.set_pending_register('a', false);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Move to second node and append to 'a'
    state.move_cursor_down();
    state.set_pending_register('a', true); // Append mode
    state.yank_nodes(1);
    state.clear_register_pending();

    // Register 'a' should have 2 nodes
    let reg_a = state.registers().get_named('a').unwrap();
    assert_eq!(reg_a.nodes.len(), 2);
}

#[test]
fn test_delete_history() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Delete three nodes
    let _ = state.delete_node_at_cursor();
    let _ = state.delete_node_at_cursor();
    let _ = state.delete_node_at_cursor();

    // Check history
    assert_eq!(state.registers().get_numbered(1).nodes.len(), 1); // Most recent
    assert_eq!(state.registers().get_numbered(2).nodes.len(), 1);
    assert_eq!(state.registers().get_numbered(3).nodes.len(), 1);
}

#[test]
fn test_yank_register_zero() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank
    state.yank_nodes(1);

    // Register "0 should have the yank
    assert_eq!(state.registers().get_numbered(0).nodes.len(), 1);

    // Delete shouldn't affect "0
    let _ = state.delete_node_at_cursor();
    assert_eq!(state.registers().get_numbered(0).nodes.len(), 1);
}

#[test]
fn test_named_register_lowercase_uppercase() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank to lowercase 'a'
    state.set_pending_register('a', false);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Move to second element
    state.move_cursor_down();

    // Append using uppercase 'A' (should map to same register)
    state.set_pending_register('A', true);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Register 'a' should have 2 nodes (lowercase and uppercase reference same register)
    let reg_a = state.registers().get_named('a').unwrap();
    assert_eq!(reg_a.nodes.len(), 2);
}

#[test]
fn test_unnamed_register_sync() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(42.0),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank without specifying a register (goes to unnamed)
    assert!(state.yank_nodes(1));

    // Unnamed register should have the yanked content
    assert_eq!(state.registers().get_unnamed().nodes.len(), 1);
}

#[test]
fn test_delete_history_overflow() {
    // Create an array with more than 9 elements to test history overflow
    let mut nodes = Vec::new();
    for i in 0..12 {
        nodes.push(JsonNode::new(JsonValue::Number(i as f64)));
    }

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(nodes)));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Delete 10 nodes (more than history can hold)
    for _ in 0..10 {
        let _ = state.delete_node_at_cursor();
    }

    // Check that newest deletes are in registers "1 through "9
    // Register "1 should have the most recent delete
    assert_eq!(state.registers().get_numbered(1).nodes.len(), 1);
    // Register "9 should have the 9th most recent delete
    assert_eq!(state.registers().get_numbered(9).nodes.len(), 1);

    // The oldest delete should have been pushed out of history
}

#[test]
fn test_multiple_named_registers() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank to register 'a'
    state.set_pending_register('a', false);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Move down and yank to register 'b'
    state.move_cursor_down();
    state.set_pending_register('b', false);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Move down and yank to register 'c'
    state.move_cursor_down();
    state.set_pending_register('c', false);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Verify all registers have their content
    assert_eq!(state.registers().get_named('a').unwrap().nodes.len(), 1);
    assert_eq!(state.registers().get_named('b').unwrap().nodes.len(), 1);
    assert_eq!(state.registers().get_named('c').unwrap().nodes.len(), 1);
}

#[test]
fn test_yank_count_to_named_register() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank 2 nodes to register 'a'
    state.set_pending_register('a', false);
    state.yank_nodes(2);
    state.clear_register_pending();

    // Register 'a' should have 2 nodes
    let reg_a = state.registers().get_named('a').unwrap();
    assert_eq!(reg_a.nodes.len(), 2);
}

#[test]
fn test_paste_from_numbered_register() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Delete first node (goes to "1)
    let _ = state.delete_node_at_cursor();

    // Delete second node (goes to "1, previous delete moves to "2)
    let _ = state.delete_node_at_cursor();

    // Cursor should still be valid on remaining element
    // Now paste from "2 (the first delete)
    state.set_pending_register('2', false);
    let result = state.paste_nodes_at_cursor();
    assert!(result.is_ok());
}

#[test]
fn test_register_isolation() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank to register 'a'
    state.set_pending_register('a', false);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Move down and yank to unnamed register
    state.move_cursor_down();
    state.yank_nodes(1);

    // Register 'a' should still have its original content
    let reg_a = state.registers().get_named('a').unwrap();
    assert_eq!(reg_a.nodes.len(), 1);

    // Unnamed register should have different content
    let unnamed = state.registers().get_unnamed();
    assert_eq!(unnamed.nodes.len(), 1);
}

#[test]
fn test_empty_register_get() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    // Getting from non-existent named register should return None
    assert!(state.registers().get_named('z').is_none());

    // Numbered registers exist but are empty
    assert!(state.registers().get_numbered(5).is_empty());
}

#[test]
fn test_append_to_nonexistent_register() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Append to non-existent register 'z' (should create it)
    state.set_pending_register('z', true);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Register 'z' should now exist with the content
    let reg_z = state.registers().get_named('z').unwrap();
    assert_eq!(reg_z.nodes.len(), 1);
}

#[test]
fn test_register_pending_state() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);

    // Initially no pending register
    assert!(state.get_pending_register().is_none());

    // Set pending register
    state.set_pending_register('a', false);
    assert!(state.get_pending_register().is_some());

    // Clear pending register
    state.clear_register_pending();
    assert!(state.get_pending_register().is_none());
}

#[test]
fn test_append_mode_state() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);

    // Initially not in append mode
    assert!(!state.get_append_mode());

    // Set append mode
    state.set_pending_register('a', true);
    assert!(state.get_append_mode());

    // Clear should reset append mode
    state.clear_register_pending();
    assert!(!state.get_append_mode());
}

#[test]
fn test_register_content_with_object_keys() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.cursor_mut().set_path(vec![0]);

    // Yank object property to register 'a'
    state.set_pending_register('a', false);
    state.yank_nodes(1);
    state.clear_register_pending();

    // Register should have both the node and the key
    let reg_a = state.registers().get_named('a').unwrap();
    assert_eq!(reg_a.nodes.len(), 1);
    assert_eq!(reg_a.keys.len(), 1);
    assert_eq!(reg_a.keys[0].as_ref().unwrap(), "name");
}
