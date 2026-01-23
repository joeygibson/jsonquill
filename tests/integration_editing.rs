use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::mode::EditorMode;
use jsonquill::editor::state::EditorState;

#[test]
fn test_full_edit_workflow() {
    // Create initial document
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
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
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )])));
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
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

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
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));
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
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )])));
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
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::{AddModeStage, EditorState, MessageLevel};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )])));
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
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::mode::EditorMode;
    use jsonquill::editor::state::{AddModeStage, EditorState};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));
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

#[test]
fn test_add_boolean_to_array() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_add_operation();

    state.clear_edit_buffer();
    for ch in "true".chars() {
        state.push_to_edit_buffer(ch);
    }

    state.commit_add_operation().unwrap();

    let node = state.tree().get_node(&[1]).unwrap();
    match node.value() {
        JsonValue::Boolean(b) => assert_eq!(*b, true),
        _ => panic!("Expected boolean"),
    }
}

#[test]
fn test_add_null_to_array() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_add_operation();

    state.clear_edit_buffer();
    for ch in "null".chars() {
        state.push_to_edit_buffer(ch);
    }

    state.commit_add_operation().unwrap();

    let node = state.tree().get_node(&[1]).unwrap();
    assert!(matches!(node.value(), JsonValue::Null));
}

#[test]
fn test_add_creates_undo_checkpoint() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![JsonNode::new(
        JsonValue::Number(1.0),
    )])));
    let mut state = EditorState::new(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_add_operation();

    state.clear_edit_buffer();
    for ch in "42".chars() {
        state.push_to_edit_buffer(ch);
    }

    state.commit_add_operation().unwrap();

    // Verify element was added
    assert!(state.tree().get_node(&[1]).is_some());

    // Undo
    state.undo();

    // Verify element was removed
    assert!(state.tree().get_node(&[1]).is_none());
}

#[test]
fn test_cursor_moves_to_new_node() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    let mut state = EditorState::new(tree);

    // Start at first element
    state.cursor_mut().set_path(vec![0]);

    state.start_add_operation();
    state.clear_edit_buffer();
    for ch in "99".chars() {
        state.push_to_edit_buffer(ch);
    }
    state.commit_add_operation().unwrap();

    // Cursor should have moved to newly created element at position 1
    assert_eq!(state.cursor().path(), &[1]);
}

#[test]
fn test_add_to_root_scalar_fails() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::{EditorState, MessageLevel};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Number(42.0)));
    let mut state = EditorState::new(tree);

    // Cursor is at root
    state.start_add_operation();

    // Should have error message
    if let Some(msg) = state.message() {
        assert_eq!(msg.level, MessageLevel::Error);
        assert!(msg.text.contains("Cannot add sibling to root"));
    } else {
        panic!("Expected error message");
    }
}

#[test]
fn test_add_field_preserves_sibling_expansion_state() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    // Create a document with nested objects
    let inner1 = vec![
        ("x".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("y".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    ];
    let inner2 = vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(3.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(4.0))),
    ];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "first".to_string(),
            JsonNode::new(JsonValue::Object(inner1)),
        ),
        (
            "second".to_string(),
            JsonNode::new(JsonValue::Object(inner2)),
        ),
    ])));
    let mut state = EditorState::new(tree);

    // Expand both nested objects
    if !state.tree_view().is_expanded(&[0]) {
        state.tree_view_mut().toggle_expand(&[0]);
    }
    if !state.tree_view().is_expanded(&[1]) {
        state.tree_view_mut().toggle_expand(&[1]);
    }
    state.rebuild_tree_view();

    // Verify both are expanded
    assert!(state.tree_view().is_expanded(&[0]));
    assert!(state.tree_view().is_expanded(&[1]));

    // Add a new field after "first" (between index 0 and 1)
    state.cursor_mut().set_path(vec![0]); // Position on "first"
    state.start_add_operation();

    // Enter key "middle"
    for ch in "middle".chars() {
        state.push_to_add_key_buffer(ch);
    }
    state.transition_add_to_value();

    // Enter value "test"
    for ch in "test".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Commit the add
    let result = state.commit_add_operation();
    assert!(result.is_ok());

    // CRITICAL: Both sibling objects should still be expanded
    // "first" stays at [0] - should still be expanded
    assert!(
        state.tree_view().is_expanded(&[0]),
        "First object at [0] should still be expanded"
    );

    // "second" moved from [1] to [2] - should still be expanded
    assert!(
        state.tree_view().is_expanded(&[2]),
        "Second object (now at [2]) should still be expanded after insertion"
    );
}

#[test]
fn test_add_field_preserves_child_expansion_state() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    // Simulate the exact scenario: company with headquarters that has nested structure
    let headquarters = vec![
        (
            "address".to_string(),
            JsonNode::new(JsonValue::String("123 Main St".to_string())),
        ),
        (
            "city".to_string(),
            JsonNode::new(JsonValue::String("NYC".to_string())),
        ),
    ];
    let company = vec![(
        "headquarters".to_string(),
        JsonNode::new(JsonValue::Object(headquarters)),
    )];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "company".to_string(),
        JsonNode::new(JsonValue::Object(company)),
    )])));
    let mut state = EditorState::new(tree);

    // Expand the nested structure:
    // [0] = company object
    // [0, 0] = headquarters object (child of company)
    if !state.tree_view().is_expanded(&[0]) {
        state.tree_view_mut().toggle_expand(&[0]);
    }
    if !state.tree_view().is_expanded(&[0, 0]) {
        state.tree_view_mut().toggle_expand(&[0, 0]);
    }
    state.rebuild_tree_view();

    // Verify headquarters is expanded before adding
    assert!(
        state.tree_view().is_expanded(&[0, 0]),
        "headquarters should be expanded initially"
    );

    // Position cursor on headquarters (the first field of company)
    state.cursor_mut().set_path(vec![0, 0]);

    // Add "employees: 23" after headquarters
    state.start_add_operation();

    // Enter key "employees"
    for ch in "employees".chars() {
        state.push_to_add_key_buffer(ch);
    }
    state.transition_add_to_value();

    // Enter value "23"
    for ch in "23".chars() {
        state.push_to_edit_buffer(ch);
    }

    // Commit the add
    let result = state.commit_add_operation();
    assert!(result.is_ok());

    // Debug: print expanded paths before and after
    println!("Expanded paths after commit:");
    for path in &[vec![0], vec![0, 0], vec![0, 0, 0], vec![0, 1]] {
        println!("  {:?}: {}", path, state.tree_view().is_expanded(path));
    }

    // CRITICAL: headquarters should STILL be expanded after adding employees
    assert!(
        state.tree_view().is_expanded(&[0, 0]),
        "headquarters at [0, 0] should still be expanded after adding employees at [0, 1]"
    );
}

#[test]
fn test_add_field_with_detailed_expansion_tracking() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    // Create a more complex structure to test
    let address_obj = vec![
        (
            "street".to_string(),
            JsonNode::new(JsonValue::String("123 Main".to_string())),
        ),
        (
            "city".to_string(),
            JsonNode::new(JsonValue::String("NYC".to_string())),
        ),
        (
            "zip".to_string(),
            JsonNode::new(JsonValue::String("10001".to_string())),
        ),
    ];
    let headquarters = vec![
        (
            "address".to_string(),
            JsonNode::new(JsonValue::Object(address_obj)),
        ),
        (
            "phone".to_string(),
            JsonNode::new(JsonValue::String("555-1234".to_string())),
        ),
    ];
    let company = vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Acme Corp".to_string())),
        ),
        (
            "headquarters".to_string(),
            JsonNode::new(JsonValue::Object(headquarters)),
        ),
    ];
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "company".to_string(),
        JsonNode::new(JsonValue::Object(company)),
    )])));
    let tree_copy = tree.clone();
    let mut state = EditorState::new(tree);

    // Manually expand all nodes for this test
    state.tree_view_mut().expand_all(&tree_copy);
    state.rebuild_tree_view();

    // All nodes should be expanded
    println!("Initial expansion state (should all be true):");
    println!("  [0] (company): {}", state.tree_view().is_expanded(&[0]));
    println!(
        "  [0, 1] (headquarters): {}",
        state.tree_view().is_expanded(&[0, 1])
    );
    println!(
        "  [0, 1, 0] (address): {}",
        state.tree_view().is_expanded(&[0, 1, 0])
    );

    println!("\n=== BEFORE adding employees ===");
    println!("Tree structure:");
    println!(
        "[0] = company (expanded: {})",
        state.tree_view().is_expanded(&[0])
    );
    println!("[0, 0] = name");
    println!(
        "[0, 1] = headquarters (expanded: {})",
        state.tree_view().is_expanded(&[0, 1])
    );
    println!(
        "[0, 1, 0] = address (expanded: {})",
        state.tree_view().is_expanded(&[0, 1, 0])
    );
    println!("[0, 1, 1] = phone");

    // Position on "headquarters" to add after it
    state.cursor_mut().set_path(vec![0, 1]);

    // Add "employees: 23"
    state.start_add_operation();
    for ch in "employees".chars() {
        state.push_to_add_key_buffer(ch);
    }
    state.transition_add_to_value();
    for ch in "23".chars() {
        state.push_to_edit_buffer(ch);
    }
    state.commit_add_operation().unwrap();

    println!("\n=== AFTER adding employees ===");
    println!("Tree structure:");
    println!(
        "[0] = company (expanded: {})",
        state.tree_view().is_expanded(&[0])
    );
    println!("[0, 0] = name");
    println!(
        "[0, 1] = headquarters (expanded: {})",
        state.tree_view().is_expanded(&[0, 1])
    );
    println!(
        "[0, 1, 0] = address (expanded: {})",
        state.tree_view().is_expanded(&[0, 1, 0])
    );
    println!("[0, 1, 1] = phone");
    println!("[0, 2] = employees (NEW)");

    // Verify everything is still expanded
    assert!(
        state.tree_view().is_expanded(&[0]),
        "company should be expanded"
    );
    assert!(
        state.tree_view().is_expanded(&[0, 1]),
        "headquarters should be expanded"
    );
    assert!(
        state.tree_view().is_expanded(&[0, 1, 0]),
        "address should be expanded"
    );
}
