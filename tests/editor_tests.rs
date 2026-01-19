use jeditor::editor::mode::EditorMode;
use jeditor::editor::state::EditorState;
use jeditor::document::node::{JsonNode, JsonValue};
use jeditor::document::tree::JsonTree;

#[test]
fn test_mode_starts_normal() {
    let mode = EditorMode::Normal;
    assert!(matches!(mode, EditorMode::Normal));
}

#[test]
fn test_editor_state_creation() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let state = EditorState::new(tree);

    assert_eq!(state.mode(), &EditorMode::Normal);
    assert!(!state.is_dirty());
}

#[test]
fn test_editor_state_set_dirty() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new(tree);

    state.mark_dirty();
    assert!(state.is_dirty());
}

#[test]
fn test_editor_state_clear_dirty() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new(tree);

    state.mark_dirty();
    assert!(state.is_dirty());

    state.clear_dirty();
    assert!(!state.is_dirty());
}

#[test]
fn test_editor_state_mode_transitions() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    let mut state = EditorState::new(tree);

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
    let mut state = EditorState::new(tree);

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
    let mut state = EditorState::new(tree);

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
    let state = EditorState::new(tree);

    // Access tree through immutable reference
    let tree_ref = state.tree();
    // Verify we can access the root node
    let _root = tree_ref.root();
}

#[test]
fn test_editor_state_tree_mut_access() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::String("initial".to_string())));
    let mut state = EditorState::new(tree);

    // Access tree through mutable reference
    let _tree_mut = state.tree_mut();
    // Can modify tree here
}

// Cursor tests
use jeditor::editor::cursor::Cursor;

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
    let cloned = mode.clone();
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
    use jeditor::document::node::{JsonNode, JsonValue};
    use jeditor::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("test".to_string(), JsonNode::new(JsonValue::String("value".to_string()))),
    ])));

    let state = EditorState::new(tree);

    // Verify tree view is initialized
    assert_eq!(state.tree_view().lines().len(), 1);
    assert_eq!(state.tree_view().lines()[0].key, Some("test".to_string()));
}

#[test]
fn test_tree_view_mut_toggle() {
    use jeditor::document::node::{JsonNode, JsonValue};
    use jeditor::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        ("nested".to_string(), JsonNode::new(JsonValue::Object(vec![
            ("inner".to_string(), JsonNode::new(JsonValue::Number(42.0))),
        ]))),
    ])));

    let mut state = EditorState::new(tree);

    // Initially collapsed
    assert_eq!(state.tree_view().lines().len(), 1);
    assert!(!state.tree_view().is_expanded(&[0]));

    // Toggle expand
    state.tree_view_mut().toggle_expand(&[0]);
    state.rebuild_tree_view();

    // Now expanded - should see both lines
    assert!(state.tree_view().is_expanded(&[0]));
    assert_eq!(state.tree_view().lines().len(), 2);
}

#[test]
fn test_rebuild_tree_view() {
    use jeditor::document::node::{JsonNode, JsonValue};
    use jeditor::document::tree::JsonTree;

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new(tree);

    // Empty tree
    assert_eq!(state.tree_view().lines().len(), 0);

    // This is a conceptual test - in practice you'd modify the tree
    // For now, just verify rebuild_tree_view() doesn't panic
    state.rebuild_tree_view();
    assert_eq!(state.tree_view().lines().len(), 0);
}
