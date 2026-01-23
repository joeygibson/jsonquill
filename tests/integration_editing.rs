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

    // Clear pre-populated value and type new value
    state.clear_edit_buffer();
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

#[test]
fn test_count_accumulation() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new(tree);

    // Initially no count
    assert_eq!(state.pending_count(), None);
    assert_eq!(state.get_count(), 1);

    // Push single digit
    state.push_count_digit(3);
    assert_eq!(state.pending_count(), Some(3));
    assert_eq!(state.get_count(), 3);

    // Push more digits
    state.push_count_digit(5);
    assert_eq!(state.pending_count(), Some(35));
    assert_eq!(state.get_count(), 35);

    // Clear count
    state.clear_pending_count();
    assert_eq!(state.pending_count(), None);
    assert_eq!(state.get_count(), 1);
}

#[test]
fn test_count_with_delete() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
        JsonNode::new(JsonValue::Number(4.0)),
        JsonNode::new(JsonValue::Number(5.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Cursor starts at root, move to first element
    state.move_cursor_down();

    // Set count to 3
    state.push_count_digit(3);
    assert_eq!(state.get_count(), 3);

    // Simulate dd - first 'd' sets pending command
    state.set_pending_command('d');

    // Count should still be there
    assert_eq!(state.get_count(), 3);

    // Simulate second 'd' - this would trigger deletion
    // We'll manually do what the handler does
    let count = state.get_count();
    state.clear_pending();

    for _ in 0..count {
        state.yank_node();
        let _ = state.delete_node_at_cursor();
    }

    // Should have deleted 3 elements, leaving 2
    assert_eq!(state.tree_view().lines().len(), 2);
}

#[test]
fn test_count_with_yank() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new(tree);

    // Move to first node
    state.move_cursor_down();

    // Set count to 2
    state.push_count_digit(2);

    // Simulate yy
    state.set_pending_command('y');

    let count = state.get_count();
    state.clear_pending();

    for _ in 0..count {
        state.yank_node();
        state.move_cursor_down();
    }

    // Should have yanked (clipboard should have content)
    assert!(state.has_clipboard());
}

#[test]
fn test_count_with_movement_down() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
        JsonNode::new(JsonValue::Number(4.0)),
        JsonNode::new(JsonValue::Number(5.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Cursor starts at first element [0]
    assert_eq!(state.cursor().path(), &[0]);

    // Move down 3 times with count
    state.push_count_digit(3);
    let count = state.get_count();
    state.clear_pending();

    for _ in 0..count {
        state.move_cursor_down();
    }

    // Should be at element 3 (0-indexed)
    assert_eq!(state.cursor().path(), &[3]);
}

#[test]
fn test_count_with_movement_up() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
        JsonNode::new(JsonValue::Number(4.0)),
        JsonNode::new(JsonValue::Number(5.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Move to last element
    state.jump_to_bottom();
    assert_eq!(state.cursor().path(), &[4]);

    // Move up 2 times with count
    state.push_count_digit(2);
    let count = state.get_count();
    state.clear_pending();

    for _ in 0..count {
        state.move_cursor_up();
    }

    // Should be at element 2 (0-indexed)
    assert_eq!(state.cursor().path(), &[2]);
}

#[test]
fn test_jump_to_line() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
        JsonNode::new(JsonValue::Number(4.0)),
        JsonNode::new(JsonValue::Number(5.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Jump to line 3 (1-based, so element at index 2)
    state.jump_to_line(3);
    assert_eq!(state.cursor().path(), &[2]);

    // Jump to line 1 (first element)
    state.jump_to_line(1);
    assert_eq!(state.cursor().path(), &[0]);

    // Jump to line 5 (last element)
    state.jump_to_line(5);
    assert_eq!(state.cursor().path(), &[4]);

    // Jump to invalid line (0) should do nothing
    state.jump_to_line(0);
    assert_eq!(state.cursor().path(), &[4]); // Still at line 5

    // Jump to invalid line (beyond end) should do nothing
    state.jump_to_line(100);
    assert_eq!(state.cursor().path(), &[4]); // Still at line 5
}

#[test]
fn test_cursor_position() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Cursor starts at first line (0-indexed element 0 = 1-indexed line 1)
    let (row, col) = state.cursor_position();
    assert_eq!(row, 1);
    assert_eq!(col, 1);

    // Move to second line
    state.move_cursor_down();
    let (row, col) = state.cursor_position();
    assert_eq!(row, 2);
    assert_eq!(col, 1);

    // Move to third line
    state.move_cursor_down();
    let (row, col) = state.cursor_position();
    assert_eq!(row, 3);
    assert_eq!(col, 1);

    // Total lines
    assert_eq!(state.total_lines(), 3);
}

#[test]
fn test_add_string_to_array() {
    use jeditor::editor::state::EditorState;
    use jeditor::document::node::{JsonNode, JsonValue};
    use jeditor::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Move cursor to first element
    state.cursor_mut().set_path(vec![0]);

    // Start add operation
    state.start_add_operation();

    // Type "hello" in edit buffer
    state.clear_edit_buffer();
    for ch in "hello".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Commit the add
    let result = state.commit_add_operation();
    assert!(result.is_ok());

    // Verify new element exists at position 1
    let node = state.tree().get_node(&[1]).unwrap();
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected string"),
    }

    // Verify cursor moved to new element
    assert_eq!(state.cursor().path(), &[1]);

    // Verify tree is dirty
    assert!(state.is_dirty());
}

#[test]
fn test_add_number_to_array() {
    use jeditor::editor::state::EditorState;
    use jeditor::document::node::{JsonNode, JsonValue};
    use jeditor::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_add_operation();

    state.clear_edit_buffer();
    for ch in "42".chars() {
        state.push_to_edit_buffer(ch);
    }

    state.commit_add_operation().unwrap();

    // Verify it's a number, not a string
    let node = state.tree().get_node(&[1]).unwrap();
    match node.value() {
        JsonValue::Number(n) => assert_eq!(*n, 42.0),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_add_field_to_object() {
    use jeditor::editor::state::EditorState;
    use jeditor::document::node::{JsonNode, JsonValue};
    use jeditor::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_add_operation();

    // Type key "email"
    for ch in "email".chars() {
        state.push_to_add_key_buffer(ch);
    }

    // Transition to value stage (simulating Enter key - will be handled by input handler)
    state.transition_add_to_value();

    // Type value "test@example.com"
    state.clear_edit_buffer();
    for ch in "test@example.com".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Commit
    state.commit_add_operation().unwrap();

    // Verify new field exists
    let node = state.tree().get_node(&[1]).unwrap();
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "test@example.com"),
        _ => panic!("Expected string"),
    }

    // Verify cursor moved
    assert_eq!(state.cursor().path(), &[1]);
}

#[test]
fn test_add_with_empty_key_fails() {
    use jeditor::editor::state::{EditorState, AddModeStage, MessageLevel};
    use jeditor::document::node::{JsonNode, JsonValue};
    use jeditor::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_add_operation();

    // Verify in AwaitingKey stage
    assert!(matches!(state.add_mode_stage(), &AddModeStage::AwaitingKey));

    // Try to transition without entering a key
    state.transition_add_to_value();

    // Should still be in AwaitingKey
    assert!(matches!(state.add_mode_stage(), &AddModeStage::AwaitingKey));

    // Should have error message
    if let Some(msg) = state.message() {
        assert_eq!(msg.level, MessageLevel::Error);
        assert!(msg.text.contains("Key cannot be empty"));
    } else {
        panic!("Expected error message");
    }
}

#[test]
fn test_cancel_add_during_value_entry() {
    use jeditor::editor::state::{EditorState, AddModeStage};
    use jeditor::editor::mode::EditorMode;
    use jeditor::document::node::{JsonNode, JsonValue};
    use jeditor::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
    ])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_add_operation();

    // Type some value
    for ch in "hello".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Cancel
    state.cancel_editing();
    state.cancel_add_operation();
    state.set_mode(EditorMode::Normal);

    // Verify state cleared
    assert!(matches!(state.add_mode_stage(), &AddModeStage::None));
    assert_eq!(state.mode(), &EditorMode::Normal);

    // Verify no new element was created (still just 1 element)
    assert_eq!(state.tree_view().lines().len(), 1);
}
