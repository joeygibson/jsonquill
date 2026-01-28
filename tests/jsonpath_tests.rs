//! Integration tests for JSONPath search functionality.

use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::{EditorState, MessageLevel, SearchType};

/// Test that JSONPath search can find a single matching node.
#[test]
fn test_jsonpath_search_single_match() {
    // Create tree: {"users": [{"name": "Alice"}, {"name": "Bob"}]}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "users".to_string(),
        JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Bob".to_string())),
            )])),
        ])),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Execute JSONPath search for $.users[0].name
    state.execute_jsonpath_search("$.users[0].name");

    // Check search type is set to JSONPath
    assert!(matches!(state.search_type(), Some(SearchType::JsonPath(_))));

    // Check we have exactly 1 result
    assert_eq!(state.search_results_info(), Some((1, 1)));

    // Check cursor is at the correct position
    let path = state.cursor().path();
    assert_eq!(path.len(), 3); // root -> users -> [0] -> name
    assert_eq!(path[0], 0); // users is at index 0 in root object
    assert_eq!(path[1], 0); // [0] is at index 0 in users array
    assert_eq!(path[2], 0); // name is at index 0 in [0] object

    // Check message indicates success
    if let Some(msg) = state.message() {
        assert!(msg.text.contains("Found 1 matches"));
        assert_eq!(msg.level, MessageLevel::Info);
    } else {
        panic!("Expected success message after JSONPath search");
    }
}

/// Test that JSONPath search can find multiple matching nodes.
#[test]
fn test_jsonpath_search_multiple_matches() {
    // Create tree: {"items": [{"price": 10}, {"price": 20}, {"price": 30}]}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "items".to_string(),
        JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Object(vec![(
                "price".to_string(),
                JsonNode::new(JsonValue::Number(10.0)),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "price".to_string(),
                JsonNode::new(JsonValue::Number(20.0)),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "price".to_string(),
                JsonNode::new(JsonValue::Number(30.0)),
            )])),
        ])),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Execute JSONPath search for $.items[*].price (all prices)
    state.execute_jsonpath_search("$.items[*].price");

    // Check we have exactly 3 results
    assert_eq!(state.search_results_info(), Some((1, 3)));

    // Check cursor is at the first match
    let path = state.cursor().path();
    assert_eq!(path.len(), 3); // root -> items -> [0] -> price
    assert_eq!(path[0], 0); // items is at index 0 in root object
    assert_eq!(path[1], 0); // [0] is at index 0 in items array
    assert_eq!(path[2], 0); // price is at index 0 in [0] object

    // Navigate to next result
    assert!(state.next_search_result().0);
    assert_eq!(state.search_results_info(), Some((2, 3)));

    // Check cursor moved to second match
    let path = state.cursor().path();
    assert_eq!(path[1], 1); // [1] is at index 1 in items array
    assert_eq!(path[2], 0); // price is at index 0 in [1] object

    // Navigate to third result
    assert!(state.next_search_result().0);
    assert_eq!(state.search_results_info(), Some((3, 3)));

    // Check cursor moved to third match
    let path = state.cursor().path();
    assert_eq!(path[1], 2); // [2] is at index 2 in items array
    assert_eq!(path[2], 0); // price is at index 0 in [2] object
}

/// Test that JSONPath search handles no matches gracefully.
#[test]
fn test_jsonpath_search_no_matches() {
    // Create tree: {"foo": "bar"}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "foo".to_string(),
        JsonNode::new(JsonValue::String("bar".to_string())),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Execute JSONPath search for non-existent path
    state.execute_jsonpath_search("$.baz");

    // Check search type is still set to JSONPath
    assert!(matches!(state.search_type(), Some(SearchType::JsonPath(_))));

    // Check we have no results
    assert_eq!(state.search_results_info(), None);

    // Check message indicates no matches
    if let Some(msg) = state.message() {
        assert!(msg.text.contains("No matches"));
        assert_eq!(msg.level, MessageLevel::Info);
    } else {
        panic!("Expected message after JSONPath search with no results");
    }
}

/// Test that invalid JSONPath query shows error message.
#[test]
fn test_jsonpath_search_invalid_query() {
    // Create tree: {"foo": "bar"}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "foo".to_string(),
        JsonNode::new(JsonValue::String("bar".to_string())),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Execute invalid JSONPath search
    state.execute_jsonpath_search("$[invalid");

    // Check we have no results
    assert_eq!(state.search_results_info(), None);

    // Check error message
    if let Some(msg) = state.message() {
        assert!(msg.text.contains("Invalid JSONPath"));
        assert_eq!(msg.level, MessageLevel::Error);
    } else {
        panic!("Expected error message after invalid JSONPath query");
    }
}

/// Test that clear_search_buffer resets search type.
#[test]
fn test_clear_search_resets_type() {
    // Create tree: {"foo": "bar"}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "foo".to_string(),
        JsonNode::new(JsonValue::String("bar".to_string())),
    )])));

    let mut state = EditorState::new_with_default_theme(tree);

    // Execute JSONPath search
    state.execute_jsonpath_search("$.foo");

    // Verify search type is set
    assert!(state.search_type().is_some());

    // Clear search buffer
    state.clear_search_buffer();

    // Verify search type is cleared
    assert!(state.search_type().is_none());
}

/// Test that clear_search_results clears search results and resets index.
#[test]
fn test_clear_search_results() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    // Create tree: {"name": "Alice", "age": 30}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
    ])));

    let mut state = EditorState::new_with_default_theme(tree);
    state.rebuild_tree_view();

    // Execute a search to populate results
    for ch in "name".chars() {
        state.push_to_search_buffer(ch);
    }
    state.execute_search();

    // Verify search results exist
    assert!(state.search_results_info().is_some());
    let (current, total) = state.search_results_info().unwrap();
    assert_eq!(current, 1);
    assert_eq!(total, 1);

    // Clear search results
    state.clear_search_results();

    // Verify search results are cleared
    assert!(state.search_results_info().is_none());
}
