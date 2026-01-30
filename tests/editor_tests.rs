use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::mode::EditorMode;
use jsonquill::editor::state::EditorState;

#[test]
fn test_mode_starts_normal() {
    let mode = EditorMode::Normal;
    assert!(matches!(mode, EditorMode::Normal));
}

#[test]
fn test_visual_mode_display() {
    let mode = EditorMode::Visual;
    assert_eq!(format!("{}", mode), "VISUAL");
}

#[test]
fn test_visual_mode_equality() {
    assert_eq!(EditorMode::Visual, EditorMode::Visual);
    assert_ne!(EditorMode::Visual, EditorMode::Normal);
}

#[test]
fn test_editor_state_creation() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let state = EditorState::new_with_default_theme(tree);

    assert_eq!(state.mode(), &EditorMode::Normal);
    assert!(!state.is_dirty());
}

#[test]
fn test_editor_state_set_dirty() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.mark_dirty();
    assert!(state.is_dirty());
}

#[test]
fn test_editor_state_clear_dirty() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.mark_dirty();
    assert!(state.is_dirty());

    state.clear_dirty();
    assert!(!state.is_dirty());
}

#[test]
fn test_editor_state_mode_transitions() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);

    // Start in Normal mode
    assert_eq!(state.mode(), &EditorMode::Normal);

    // Switch to Insert mode
    state.set_mode(EditorMode::Insert);
    assert_eq!(state.mode(), &EditorMode::Insert);

    // Switch to Command mode
    state.set_mode(EditorMode::Command);
    assert_eq!(state.mode(), &EditorMode::Command);

    // Back to Normal mode
    state.set_mode(EditorMode::Normal);
    assert_eq!(state.mode(), &EditorMode::Normal);
}

#[test]
fn test_editor_state_filename() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);

    // Initially no filename
    assert_eq!(state.filename(), None);

    // Set a filename
    state.set_filename("test.json".to_string());
    assert_eq!(state.filename(), Some("test.json"));

    // Change the filename
    state.set_filename("other.json".to_string());
    assert_eq!(state.filename(), Some("other.json"));
}

#[test]
fn test_editor_state_cursor_access() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Initial cursor is at root
    assert_eq!(state.cursor().path(), &[] as &[usize]);

    // Modify cursor through mutable reference
    state.cursor_mut().push(0);
    assert_eq!(state.cursor().path(), &[0]);

    state.cursor_mut().push(1);
    assert_eq!(state.cursor().path(), &[0, 1]);

    state.cursor_mut().pop();
    assert_eq!(state.cursor().path(), &[0]);
}

#[test]
fn test_editor_state_tree_access() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::String("test".to_string())));
    let state = EditorState::new_with_default_theme(tree);

    // Access tree through immutable reference
    let tree_ref = state.tree();
    // Verify we can access the root node
    let _root = tree_ref.root();
}

#[test]
fn test_editor_state_tree_mut_access() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::String("initial".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    // Access tree through mutable reference
    let _tree_mut = state.tree_mut();
    // Can modify tree here
}

// Cursor tests
use jsonquill::editor::cursor::Cursor;

#[test]
fn test_cursor_new() {
    let cursor = Cursor::new();
    assert_eq!(cursor.path(), &[] as &[usize]);
}

#[test]
fn test_cursor_default() {
    let cursor = Cursor::default();
    assert_eq!(cursor.path(), &[] as &[usize]);
}

#[test]
fn test_cursor_push() {
    let mut cursor = Cursor::new();
    cursor.push(0);
    assert_eq!(cursor.path(), &[0]);

    cursor.push(1);
    assert_eq!(cursor.path(), &[0, 1]);

    cursor.push(2);
    assert_eq!(cursor.path(), &[0, 1, 2]);
}

#[test]
fn test_cursor_pop() {
    let mut cursor = Cursor::new();
    cursor.push(0);
    cursor.push(1);
    cursor.push(2);

    assert_eq!(cursor.pop(), Some(2));
    assert_eq!(cursor.path(), &[0, 1]);

    assert_eq!(cursor.pop(), Some(1));
    assert_eq!(cursor.path(), &[0]);

    assert_eq!(cursor.pop(), Some(0));
    assert_eq!(cursor.path(), &[] as &[usize]);

    // Pop from empty returns None
    assert_eq!(cursor.pop(), None);
    assert_eq!(cursor.path(), &[] as &[usize]);
}

#[test]
fn test_cursor_set_path() {
    let mut cursor = Cursor::new();
    cursor.set_path(vec![0, 1, 2]);
    assert_eq!(cursor.path(), &[0, 1, 2]);

    cursor.set_path(vec![]);
    assert_eq!(cursor.path(), &[] as &[usize]);

    cursor.set_path(vec![5]);
    assert_eq!(cursor.path(), &[5]);
}

#[test]
fn test_cursor_clone() {
    let mut cursor = Cursor::new();
    cursor.push(0);
    cursor.push(1);

    let cloned = cursor.clone();
    assert_eq!(cursor.path(), cloned.path());
    assert_eq!(cursor, cloned);
}

#[test]
fn test_cursor_equality() {
    let mut cursor1 = Cursor::new();
    let mut cursor2 = Cursor::new();

    assert_eq!(cursor1, cursor2);

    cursor1.push(0);
    assert_ne!(cursor1, cursor2);

    cursor2.push(0);
    assert_eq!(cursor1, cursor2);

    cursor1.push(1);
    cursor2.push(2);
    assert_ne!(cursor1, cursor2);
}

#[test]
fn test_cursor_debug() {
    let mut cursor = Cursor::new();
    cursor.push(0);
    cursor.push(1);

    let debug_str = format!("{:?}", cursor);
    assert!(debug_str.contains("Cursor"));
    assert!(debug_str.contains("path"));
}

#[test]
fn test_cursor_multiple_operations() {
    let mut cursor = Cursor::new();

    // Build up a path
    cursor.push(0);
    cursor.push(1);
    cursor.push(2);
    assert_eq!(cursor.path(), &[0, 1, 2]);

    // Pop one level
    cursor.pop();
    assert_eq!(cursor.path(), &[0, 1]);

    // Push a different index
    cursor.push(5);
    assert_eq!(cursor.path(), &[0, 1, 5]);

    // Replace entire path
    cursor.set_path(vec![10, 20]);
    assert_eq!(cursor.path(), &[10, 20]);

    // Clear by setting empty path
    cursor.set_path(vec![]);
    assert_eq!(cursor.path(), &[] as &[usize]);
}

#[test]
fn test_mode_display() {
    assert_eq!(format!("{}", EditorMode::Normal), "NORMAL");
    assert_eq!(format!("{}", EditorMode::Insert), "INSERT");
    assert_eq!(format!("{}", EditorMode::Command), "COMMAND");
}

#[test]
fn test_mode_default() {
    let mode = EditorMode::default();
    assert_eq!(mode, EditorMode::Normal);
}

#[test]
fn test_mode_equality() {
    let mode1 = EditorMode::Normal;
    let mode2 = EditorMode::Normal;
    let mode3 = EditorMode::Insert;

    assert_eq!(mode1, mode2);
    assert_ne!(mode1, mode3);
    assert_ne!(mode2, mode3);
}

#[test]
fn test_mode_clone() {
    let mode = EditorMode::Insert;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

#[test]
fn test_mode_copy() {
    let mode = EditorMode::Command;
    let copied = mode;
    assert_eq!(mode, copied);
    // If mode wasn't Copy, this would have moved it
    assert_eq!(mode, EditorMode::Command);
}

#[test]
fn test_mode_debug() {
    let mode = EditorMode::Normal;
    let debug_str = format!("{:?}", mode);
    assert_eq!(debug_str, "Normal");

    let mode = EditorMode::Insert;
    let debug_str = format!("{:?}", mode);
    assert_eq!(debug_str, "Insert");

    let mode = EditorMode::Command;
    let debug_str = format!("{:?}", mode);
    assert_eq!(debug_str, "Command");
}

#[test]
fn test_all_mode_variants() {
    // Ensure all variants can be constructed
    let normal = EditorMode::Normal;
    let insert = EditorMode::Insert;
    let command = EditorMode::Command;

    // Ensure they are all different
    assert_ne!(normal, insert);
    assert_ne!(normal, command);
    assert_ne!(insert, command);

    // Ensure they all display correctly
    assert_eq!(format!("{}", normal), "NORMAL");
    assert_eq!(format!("{}", insert), "INSERT");
    assert_eq!(format!("{}", command), "COMMAND");
}

#[test]
fn test_tree_view_initialized() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "test".to_string(),
        JsonNode::new(JsonValue::String("value".to_string())),
    )])));

    let state = EditorState::new_with_default_theme(tree);

    // Verify tree view is initialized
    assert_eq!(state.tree_view().lines().len(), 1);
    assert_eq!(state.tree_view().lines()[0].key, Some("test".to_string()));
}

#[test]
fn test_tree_view_mut_toggle() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "nested".to_string(),
        JsonNode::new(JsonValue::Object(vec![(
            "inner".to_string(),
            JsonNode::new(JsonValue::Number(42.0)),
        )])),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Initially expanded (auto-expansion is default)
    assert_eq!(state.tree_view().lines().len(), 2);
    assert!(state.tree_view().is_expanded(&[0]));

    // Toggle collapse
    state.tree_view_mut().toggle_expand(&[0]);
    state.rebuild_tree_view();

    // Now collapsed - should see only one line
    assert!(!state.tree_view().is_expanded(&[0]));
    assert_eq!(state.tree_view().lines().len(), 1);
}

#[test]
fn test_rebuild_tree_view() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Empty tree
    assert_eq!(state.tree_view().lines().len(), 0);

    // This is a conceptual test - in practice you'd modify the tree
    // For now, just verify rebuild_tree_view() doesn't panic
    state.rebuild_tree_view();
    assert_eq!(state.tree_view().lines().len(), 0);
}

// Navigation tests

#[test]
fn test_move_cursor_down_in_empty_tree() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Moving down in empty tree should do nothing
    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[] as &[usize]);
}

#[test]
fn test_move_cursor_up_in_empty_tree() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Moving up in empty tree should do nothing
    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[] as &[usize]);
}

#[test]
fn test_move_cursor_down_basic() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    // Create a simple flat object
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Initially at [0] (first element "a")
    assert_eq!(state.cursor().path(), &[0]);

    // Move down to [1] ("b")
    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[1]);

    // Move down to [2] ("c")
    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[2]);

    // Move down at last line - should stay at [2]
    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[2]);
}

#[test]
fn test_move_cursor_up_basic() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Start at [2]
    state.cursor_mut().set_path(vec![2]);
    assert_eq!(state.cursor().path(), &[2]);

    // Move up to [1]
    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[1]);

    // Move up to [0]
    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[0]);

    // Move up at first line - should stay at [0]
    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[0]);
}

#[test]
fn test_move_cursor_with_invalid_position() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Set cursor to invalid position
    state.cursor_mut().set_path(vec![99, 99]);

    // Move down should reset to first line
    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[0]);

    // Set cursor to invalid position again
    state.cursor_mut().set_path(vec![99]);

    // Move up should reset to first line
    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[0]);
}

#[test]
fn test_toggle_expand_at_cursor_expandable() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    // Create nested object
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "user".to_string(),
        JsonNode::new(JsonValue::Object(vec![(
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        )])),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Initially expanded (auto-expansion is default) - 2 lines visible
    assert_eq!(state.tree_view().lines().len(), 2);
    assert!(state.tree_view().is_expanded(&[0]));

    // Toggle collapse at cursor (which is at [0])
    state.toggle_expand_at_cursor();

    // Now should be collapsed - 1 line visible
    assert_eq!(state.tree_view().lines().len(), 1);
    assert!(!state.tree_view().is_expanded(&[0]));

    // Toggle again to expand
    state.toggle_expand_at_cursor();
    assert_eq!(state.tree_view().lines().len(), 2);
    assert!(state.tree_view().is_expanded(&[0]));
}

#[test]
fn test_toggle_expand_at_cursor_non_expandable() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);

    // At [0] which is a string (not expandable)
    let initial_lines = state.tree_view().lines().len();

    // Toggle expand should do nothing for non-expandable nodes
    state.toggle_expand_at_cursor();

    // Lines count should be the same
    assert_eq!(state.tree_view().lines().len(), initial_lines);
}

#[test]
fn test_navigation_with_nested_expanded_tree() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    // Create nested structure
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "user".to_string(),
            JsonNode::new(JsonValue::Object(vec![
                (
                    "name".to_string(),
                    JsonNode::new(JsonValue::String("Alice".to_string())),
                ),
                (
                    "email".to_string(),
                    JsonNode::new(JsonValue::String("alice@example.com".to_string())),
                ),
            ])),
        ),
        ("count".to_string(), JsonNode::new(JsonValue::Number(42.0))),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Initially with auto-expansion: 4 lines visible ([0]=user, [0,0]=name, [0,1]=email, [1]=count)
    assert_eq!(state.tree_view().lines().len(), 4);
    assert_eq!(state.cursor().path(), &[0]);

    // Navigate through all lines
    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[0, 0]); // name

    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[0, 1]); // email

    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[1]); // count

    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[1]); // stay at last line

    // Navigate back up
    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[0, 1]); // email

    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[0, 0]); // name

    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[0]); // user

    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[0]); // stay at first line
}

#[test]
fn test_navigation_with_array() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
        JsonNode::new(JsonValue::Number(3.0)),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Start at [0]
    assert_eq!(state.cursor().path(), &[0]);

    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[1]);

    state.move_cursor_down();
    assert_eq!(state.cursor().path(), &[2]);

    state.move_cursor_up();
    assert_eq!(state.cursor().path(), &[1]);
}

// Edit buffer tests

#[test]
fn test_edit_buffer_starts_empty() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("test".to_string())));
    let state = EditorState::new_with_default_theme(tree);

    assert_eq!(state.edit_buffer(), None);
}

#[test]
fn test_start_editing_string_value() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Move cursor to first element
    state.cursor_mut().set_path(vec![0]);

    // Start editing
    state.start_editing();

    assert!(state.edit_buffer().is_some());
    assert_eq!(state.edit_buffer().unwrap(), "Alice");
}

#[test]
fn test_start_editing_number_value() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "count".to_string(),
        JsonNode::new(JsonValue::Number(42.0)),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    assert_eq!(state.edit_buffer().unwrap(), "42");
}

#[test]
fn test_start_editing_boolean_value() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "active".to_string(),
        JsonNode::new(JsonValue::Boolean(true)),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    assert_eq!(state.edit_buffer().unwrap(), "true");
}

#[test]
fn test_start_editing_null_value() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "data".to_string(),
        JsonNode::new(JsonValue::Null),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    assert_eq!(state.edit_buffer().unwrap(), "null");
}

#[test]
fn test_cancel_editing() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();
    assert!(state.edit_buffer().is_some());

    state.cancel_editing();
    assert_eq!(state.edit_buffer(), None);
}

// Commit editing tests

#[test]
fn test_commit_editing_string() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    // Clear pre-populated value and type new value
    state.clear_edit_buffer();
    state.push_to_edit_buffer('B');
    state.push_to_edit_buffer('o');
    state.push_to_edit_buffer('b');

    // Commit the change
    let result = state.commit_editing();
    assert!(result.is_ok());
    assert!(state.is_dirty());
    assert_eq!(state.edit_buffer(), None);

    // Verify the tree was updated
    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::String(s) => assert_eq!(s, "Bob"),
        _ => panic!("Expected string value"),
    }
}

#[test]
fn test_commit_editing_number() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "count".to_string(),
        JsonNode::new(JsonValue::Number(42.0)),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    // Clear pre-populated value and type new value
    state.clear_edit_buffer();
    for ch in "123.45".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_ok());

    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::Number(n) => assert_eq!(*n, 123.45),
        _ => panic!("Expected number value"),
    }
}

#[test]
fn test_commit_editing_boolean() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "active".to_string(),
        JsonNode::new(JsonValue::Boolean(true)),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    // Clear pre-populated value and type new value
    state.clear_edit_buffer();
    for ch in "false".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_ok());

    let node = state.tree().get_node(&[0]).unwrap();
    match node.value() {
        JsonValue::Boolean(b) => assert!(!*b),
        _ => panic!("Expected boolean value"),
    }
}

#[test]
fn test_commit_editing_null() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "data".to_string(),
        JsonNode::new(JsonValue::String("old".to_string())),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    // Clear pre-populated value and type new value
    state.clear_edit_buffer();
    for ch in "null".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_ok());

    let node = state.tree().get_node(&[0]).unwrap();
    assert!(matches!(node.value(), JsonValue::Null));
}

#[test]
fn test_commit_editing_invalid_number() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "count".to_string(),
        JsonNode::new(JsonValue::Number(42.0)),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    for ch in "not-a-number".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid number"));
}

#[test]
fn test_commit_editing_invalid_boolean() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "active".to_string(),
        JsonNode::new(JsonValue::Boolean(true)),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![0]);
    state.start_editing();

    for ch in "maybe".chars() {
        state.push_to_edit_buffer(ch);
    }

    let result = state.commit_editing();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be true or false"));
}

// Delete node tests

#[test]
fn test_delete_node_at_cursor() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Move to second element
    state.cursor_mut().set_path(vec![1]);

    // Delete it
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok());
    assert!(state.is_dirty());

    // Verify only 2 lines remain
    assert_eq!(state.tree_view().lines().len(), 2);

    // Cursor should stay at index 1 (now pointing to "c")
    assert_eq!(state.cursor().path(), &[1]);
}

#[test]
fn test_delete_last_node_moves_cursor() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Move to last element
    state.cursor_mut().set_path(vec![1]);

    // Delete it
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok());

    // Cursor should move to previous line [0]
    assert_eq!(state.cursor().path(), &[0]);
}

#[test]
fn test_delete_root_fails() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Cursor at root (empty path after tree_view initialization with no children)
    // Since there are no lines, cursor will be at []
    state.cursor_mut().set_path(vec![]);

    let result = state.delete_node_at_cursor();
    assert!(result.is_err());
}

// Paste node tests

#[test]
fn test_paste_node_in_object() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("c".to_string(), JsonNode::new(JsonValue::Number(3.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Yank first element
    state.cursor_mut().set_path(vec![0]);
    state.yank_nodes(1);

    // Now move to position [1] and paste
    state.cursor_mut().set_path(vec![1]);
    let result = state.paste_node_at_cursor();
    assert!(result.is_ok());
    assert!(state.is_dirty());

    // Should have 3 nodes now
    assert_eq!(state.tree_view().lines().len(), 3);
}

#[test]
fn test_paste_node_in_array() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(10.0)),
        JsonNode::new(JsonValue::Number(30.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Yank first element
    state.cursor_mut().set_path(vec![0]);
    state.yank_nodes(1);

    // Paste after first element
    let result = state.paste_node_at_cursor();
    assert!(result.is_ok());

    // Should have 3 elements now
    assert_eq!(state.tree_view().lines().len(), 3);
}

#[test]
fn test_paste_without_clipboard_fails() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "a".to_string(),
        JsonNode::new(JsonValue::Number(1.0)),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Try to paste without yanking first
    let result = state.paste_node_at_cursor();
    assert!(result.is_err());
}

// Undo/redo tests

#[test]
fn test_checkpoint_captures_state() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("a".to_string(), JsonNode::new(JsonValue::Number(1.0))),
        ("b".to_string(), JsonNode::new(JsonValue::Number(2.0))),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Delete first node
    state.cursor_mut().set_path(vec![0]);
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok());
    assert_eq!(state.tree_view().lines().len(), 1);

    // Undo should restore deleted node
    let undo_result = state.undo();
    assert!(undo_result);
    assert_eq!(state.tree_view().lines().len(), 2);

    // Redo should delete it again
    let redo_result = state.redo();
    assert!(redo_result);
    assert_eq!(state.tree_view().lines().len(), 1);
}

// Edit cursor navigation tests

#[test]
fn test_edit_cursor_starts_at_end() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("hello".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![]);
    state.start_editing();

    assert_eq!(state.edit_cursor_position(), 5); // "hello".len()
}

#[test]
fn test_edit_cursor_left_right() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("test".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![]);
    state.start_editing();
    assert_eq!(state.edit_cursor_position(), 4);

    state.edit_cursor_left();
    assert_eq!(state.edit_cursor_position(), 3);

    state.edit_cursor_left();
    state.edit_cursor_left();
    assert_eq!(state.edit_cursor_position(), 1);

    state.edit_cursor_right();
    assert_eq!(state.edit_cursor_position(), 2);

    // Can't go left past 0
    state.edit_cursor_home();
    state.edit_cursor_left();
    assert_eq!(state.edit_cursor_position(), 0);

    // Can't go right past end
    state.edit_cursor_end();
    state.edit_cursor_right();
    assert_eq!(state.edit_cursor_position(), 4);
}

#[test]
fn test_edit_cursor_home_end() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("hello world".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![]);
    state.start_editing();

    // Cursor starts at end
    assert_eq!(state.edit_cursor_position(), 11);

    // Home goes to start
    state.edit_cursor_home();
    assert_eq!(state.edit_cursor_position(), 0);

    // End goes to end
    state.edit_cursor_end();
    assert_eq!(state.edit_cursor_position(), 11);
}

#[test]
fn test_edit_insert_at_cursor() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("test".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![]);
    state.start_editing();

    // Move to middle and insert
    state.edit_cursor_home();
    state.edit_cursor_right();
    state.edit_cursor_right(); // cursor at position 2 (between 'e' and 's')

    state.push_to_edit_buffer('X');
    assert_eq!(state.edit_buffer().unwrap(), "teXst");
    assert_eq!(state.edit_cursor_position(), 3);
}

#[test]
fn test_edit_backspace_at_cursor() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("hello".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![]);
    state.start_editing();

    // Backspace from end
    state.pop_from_edit_buffer();
    assert_eq!(state.edit_buffer().unwrap(), "hell");
    assert_eq!(state.edit_cursor_position(), 4);

    // Move to middle and backspace
    state.edit_cursor_home();
    state.edit_cursor_right();
    state.edit_cursor_right(); // cursor at position 2
    state.pop_from_edit_buffer(); // delete 'e'
    assert_eq!(state.edit_buffer().unwrap(), "hll");
    assert_eq!(state.edit_cursor_position(), 1);
}

#[test]
fn test_edit_delete_at_cursor() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("hello".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![]);
    state.start_editing();

    // Delete at end does nothing
    state.edit_delete_at_cursor();
    assert_eq!(state.edit_buffer().unwrap(), "hello");
    assert_eq!(state.edit_cursor_position(), 5);

    // Move to start and delete
    state.edit_cursor_home();
    state.edit_delete_at_cursor(); // delete 'h'
    assert_eq!(state.edit_buffer().unwrap(), "ello");
    assert_eq!(state.edit_cursor_position(), 0);

    // Delete in middle
    state.edit_cursor_right();
    state.edit_delete_at_cursor(); // delete first 'l'
    assert_eq!(state.edit_buffer().unwrap(), "elo");
    assert_eq!(state.edit_cursor_position(), 1);
}

#[test]
fn test_edit_kill_to_end() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::String("hello world".to_string())));
    let mut state = EditorState::new_with_default_theme(tree);

    state.cursor_mut().set_path(vec![]);
    state.start_editing();

    // Kill from end does nothing
    state.edit_kill_to_end();
    assert_eq!(state.edit_buffer().unwrap(), "hello world");
    assert_eq!(state.edit_cursor_position(), 11);

    // Move to middle and kill to end
    state.edit_cursor_home();
    state.edit_cursor_right();
    state.edit_cursor_right();
    state.edit_cursor_right();
    state.edit_cursor_right();
    state.edit_cursor_right(); // cursor at position 5 (after "hello")
    state.edit_kill_to_end();
    assert_eq!(state.edit_buffer().unwrap(), "hello");
    assert_eq!(state.edit_cursor_position(), 5);

    // Kill from start
    state.edit_cursor_home();
    state.edit_kill_to_end();
    assert_eq!(state.edit_buffer().unwrap(), "");
    assert_eq!(state.edit_cursor_position(), 0);
}

#[test]
fn test_add_mode_stage_default() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::{AddModeStage, EditorState};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let state = EditorState::new_with_default_theme(tree);

    assert!(matches!(state.add_mode_stage(), &AddModeStage::None));
}

#[test]
fn test_add_key_buffer_operations() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new_with_default_theme(tree);

    // Initially empty
    assert_eq!(state.add_key_buffer(), "");

    // Push characters
    state.push_to_add_key_buffer('e');
    state.push_to_add_key_buffer('m');
    state.push_to_add_key_buffer('a');
    state.push_to_add_key_buffer('i');
    state.push_to_add_key_buffer('l');
    assert_eq!(state.add_key_buffer(), "email");

    // Pop character
    state.pop_from_add_key_buffer();
    assert_eq!(state.add_key_buffer(), "emai");

    // Clear buffer
    state.clear_add_key_buffer();
    assert_eq!(state.add_key_buffer(), "");
}

#[test]
fn test_parse_scalar_value() {
    use jsonquill::document::node::JsonValue;
    use jsonquill::editor::state::parse_scalar_value_for_test;

    // Booleans
    assert!(matches!(
        parse_scalar_value_for_test("true"),
        JsonValue::Boolean(true)
    ));
    assert!(matches!(
        parse_scalar_value_for_test("false"),
        JsonValue::Boolean(false)
    ));
    assert!(matches!(
        parse_scalar_value_for_test("  true  "),
        JsonValue::Boolean(true)
    ));

    // Null
    assert!(matches!(
        parse_scalar_value_for_test("null"),
        JsonValue::Null
    ));
    assert!(matches!(
        parse_scalar_value_for_test("  null  "),
        JsonValue::Null
    ));

    // Numbers
    match parse_scalar_value_for_test("42") {
        JsonValue::Number(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number"),
    }
    match parse_scalar_value_for_test("-1.5") {
        JsonValue::Number(n) => assert_eq!(n, -1.5),
        _ => panic!("Expected number"),
    }
    match parse_scalar_value_for_test("0") {
        JsonValue::Number(n) => assert_eq!(n, 0.0),
        _ => panic!("Expected number"),
    }

    // Strings (fallback)
    match parse_scalar_value_for_test("hello") {
        JsonValue::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected string"),
    }
    match parse_scalar_value_for_test("123abc") {
        JsonValue::String(s) => assert_eq!(s, "123abc"),
        _ => panic!("Expected string"),
    }
    match parse_scalar_value_for_test("") {
        JsonValue::String(s) => assert_eq!(s, ""),
        _ => panic!("Expected string"),
    }
    match parse_scalar_value_for_test("True") {
        JsonValue::String(s) => assert_eq!(s, "True"),
        _ => panic!("Expected string (case sensitive)"),
    }
    match parse_scalar_value_for_test("NULL") {
        JsonValue::String(s) => assert_eq!(s, "NULL"),
        _ => panic!("Expected string (case sensitive)"),
    }
}

#[test]
fn test_start_add_in_array() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::mode::EditorMode;
    use jsonquill::editor::state::{AddModeStage, EditorState};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
        JsonNode::new(JsonValue::Number(1.0)),
        JsonNode::new(JsonValue::Number(2.0)),
    ])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Move cursor to first element
    state.cursor_mut().set_path(vec![0]);

    // Start add operation
    state.start_add_operation();

    // Should be in AwaitingValue stage (arrays skip key prompt)
    assert!(matches!(
        state.add_mode_stage(),
        &AddModeStage::AwaitingValue
    ));
    // Should have entered Insert mode
    assert_eq!(state.mode(), &EditorMode::Insert);
    // Edit buffer should be empty
    assert_eq!(state.edit_buffer(), Some(""));
}

#[test]
fn test_start_add_in_object() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::mode::EditorMode;
    use jsonquill::editor::state::{AddModeStage, EditorState};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Move cursor to first field
    state.cursor_mut().set_path(vec![0]);

    // Start add operation
    state.start_add_operation();

    // Should be in AwaitingKey stage (objects need key first)
    assert!(matches!(state.add_mode_stage(), &AddModeStage::AwaitingKey));
    // Should still be in Normal mode (key prompt, not Insert)
    assert_eq!(state.mode(), &EditorMode::Normal);
}

#[test]
fn test_start_add_in_root_array() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::mode::EditorMode;
    use jsonquill::editor::state::{AddModeStage, EditorState};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Cursor is at root (empty path because array is empty)
    state.cursor_mut().set_path(vec![]);

    // Start add operation
    state.start_add_operation();

    // Should be in AwaitingValue stage (arrays skip key prompt)
    assert!(matches!(
        state.add_mode_stage(),
        &AddModeStage::AwaitingValue
    ));
    // Should have entered Insert mode
    assert_eq!(state.mode(), &EditorMode::Insert);
    // Edit buffer should be empty
    assert_eq!(state.edit_buffer(), Some(""));
}

#[test]
fn test_start_add_in_root_object() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::mode::EditorMode;
    use jsonquill::editor::state::{AddModeStage, EditorState};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Cursor is at root (empty path because object is empty)
    state.cursor_mut().set_path(vec![]);

    // Start add operation
    state.start_add_operation();

    // Should be in AwaitingKey stage (objects need key first)
    assert!(matches!(state.add_mode_stage(), &AddModeStage::AwaitingKey));
    // Should still be in Normal mode (key prompt, not Insert)
    assert_eq!(state.mode(), &EditorMode::Normal);
}

#[test]
fn test_add_array_to_empty_object_should_add_inside() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::{AddModeStage, EditorState};

    // BUG: When cursor is on an empty object, pressing 'a' (AddArray)
    // should add INSIDE the object, not as a sibling.
    // Start with {"president": {}}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "president".to_string(),
        JsonNode::new(JsonValue::Object(vec![])),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Cursor at "president" object (path [0])
    state.cursor_mut().set_path(vec![0]);

    // Press 'a' to add an array inside the empty "president" object
    state.start_add_container_operation(false); // false = array

    // Should be in AwaitingKey stage (for the key name)
    assert!(matches!(state.add_mode_stage(), &AddModeStage::AwaitingKey));

    // Type "name" for the key
    for ch in "name".chars() {
        state.push_to_add_key_buffer(ch);
    }

    // Press Enter to commit
    let result = state.commit_container_add();
    assert!(result.is_ok());

    // Check the result - the array should be INSIDE "president", not a sibling
    let root = state.tree().root();
    if let JsonValue::Object(entries) = root.value() {
        println!("Root object has {} entries", entries.len());
        for (i, (key, value_node)) in entries.iter().enumerate() {
            println!(
                "  Entry {}: key='{}', value={:?}",
                i,
                key,
                value_node.value()
            );
        }
        // BUG: Currently this fails because the array is added as a sibling at root level
        // Expected: {"president": {"name": []}}
        // Actual: {"president": {}, "name": []}
        assert_eq!(
            entries.len(),
            1,
            "BUG: Array was added as sibling to 'president' instead of inside it"
        );

        let (key, value_node) = &entries[0];
        assert_eq!(key, "president");
        if let JsonValue::Object(president_entries) = value_node.value() {
            println!("President object has {} entries", president_entries.len());
            for (i, (key, value_node)) in president_entries.iter().enumerate() {
                println!(
                    "  Entry {}: key='{}', value={:?}",
                    i,
                    key,
                    value_node.value()
                );
            }
            assert_eq!(president_entries.len(), 1);
            let (field_key, field_value_node) = &president_entries[0];
            assert_eq!(field_key, "name");
            assert!(matches!(field_value_node.value(), JsonValue::Array(_)));
        } else {
            panic!("Expected president to be an object");
        }
    } else {
        panic!("Expected root to be an object");
    }
}

#[test]
fn test_start_add_at_root_scalar_fails() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::mode::EditorMode;
    use jsonquill::editor::state::{AddModeStage, EditorState, MessageLevel};

    let tree = JsonTree::new(JsonNode::new(JsonValue::Number(42.0)));
    let mut state = EditorState::new_with_default_theme(tree);

    // Cursor is at root (empty path)

    // Try to start add operation
    state.start_add_operation();

    // Should still be in None stage
    assert!(matches!(state.add_mode_stage(), &AddModeStage::None));
    // Should still be in Normal mode
    assert_eq!(state.mode(), &EditorMode::Normal);
    // Should have error message
    if let Some(msg) = state.message() {
        assert_eq!(msg.level, MessageLevel::Error);
        assert!(msg.text.contains("Cannot add sibling to root"));
    } else {
        panic!("Expected error message");
    }
}
