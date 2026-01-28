use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::{EditorState, SearchType};

/// Helper to create a sample bookstore JSON structure
fn create_bookstore() -> JsonTree {
    JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "store".to_string(),
        JsonNode::new(JsonValue::Object(vec![
            (
                "book".to_string(),
                JsonNode::new(JsonValue::Array(vec![
                    JsonNode::new(JsonValue::Object(vec![
                        (
                            "category".to_string(),
                            JsonNode::new(JsonValue::String("reference".to_string())),
                        ),
                        (
                            "author".to_string(),
                            JsonNode::new(JsonValue::String("Nigel Rees".to_string())),
                        ),
                        (
                            "title".to_string(),
                            JsonNode::new(JsonValue::String("Sayings of the Century".to_string())),
                        ),
                        ("price".to_string(), JsonNode::new(JsonValue::Number(8.95))),
                    ])),
                    JsonNode::new(JsonValue::Object(vec![
                        (
                            "category".to_string(),
                            JsonNode::new(JsonValue::String("fiction".to_string())),
                        ),
                        (
                            "author".to_string(),
                            JsonNode::new(JsonValue::String("Herman Melville".to_string())),
                        ),
                        (
                            "title".to_string(),
                            JsonNode::new(JsonValue::String("Moby Dick".to_string())),
                        ),
                        ("price".to_string(), JsonNode::new(JsonValue::Number(8.99))),
                    ])),
                ])),
            ),
            (
                "bicycle".to_string(),
                JsonNode::new(JsonValue::Object(vec![
                    (
                        "color".to_string(),
                        JsonNode::new(JsonValue::String("red".to_string())),
                    ),
                    ("price".to_string(), JsonNode::new(JsonValue::Number(19.95))),
                ])),
            ),
        ])),
    )])))
}

#[test]
fn test_jsonpath_wildcard_search() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Search for all book authors using wildcard
    state.execute_jsonpath_search("$.store.book[*].author");

    // Should find 2 authors
    assert_eq!(state.search_results_info(), Some((1, 2)));

    // Verify search type is JSONPath
    assert!(matches!(state.search_type(), Some(SearchType::JsonPath(_))));

    // Verify cursor is at first match
    assert_eq!(state.cursor().path(), &[0, 0, 0, 1]); // First book's author
}

#[test]
fn test_jsonpath_recursive_descent() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Find all price fields anywhere in the document
    state.execute_jsonpath_search("$..price");

    // Should find 3 prices (2 books + 1 bicycle)
    assert_eq!(state.search_results_info(), Some((1, 3)));
}

#[test]
fn test_jsonpath_array_slicing() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "items".to_string(),
        JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(0.0)),
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
            JsonNode::new(JsonValue::Number(3.0)),
            JsonNode::new(JsonValue::Number(4.0)),
        ])),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Get first 3 items using slice [0:3]
    state.execute_jsonpath_search("$.items[0:3]");

    assert_eq!(state.search_results_info(), Some((1, 3)));

    // Verify cursor at first match
    assert_eq!(state.cursor().path(), &[0, 0]); // items[0]
}

#[test]
fn test_jsonpath_multiple_properties() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
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
            ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
        ])),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Select multiple properties
    state.execute_jsonpath_search("$.user['name','email']");

    assert_eq!(state.search_results_info(), Some((1, 2)));
}

#[test]
fn test_jsonpath_negative_array_index() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "items".to_string(),
        JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
            JsonNode::new(JsonValue::Number(3.0)),
        ])),
    )])));
    let mut state = EditorState::new_with_default_theme(tree);

    // Access last element with [-1]
    state.execute_jsonpath_search("$.items[-1]");

    assert_eq!(state.search_results_info(), Some((1, 1)));
    assert_eq!(state.cursor().path(), &[0, 2]); // Last item (index 2)
}

#[test]
fn test_jsonpath_navigation_through_results() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Search for all prices
    state.execute_jsonpath_search("$..price");
    assert_eq!(state.search_results_info(), Some((1, 3)));

    // Cursor should be at first match
    assert_eq!(state.cursor().path(), &[0, 0, 0, 3]); // First book price

    // Navigate to second match
    assert!(state.next_search_result().0);
    assert_eq!(state.search_results_info(), Some((2, 3)));
    assert_eq!(state.cursor().path(), &[0, 0, 1, 3]); // Second book price

    // Navigate to third match
    assert!(state.next_search_result().0);
    assert_eq!(state.search_results_info(), Some((3, 3)));
    assert_eq!(state.cursor().path(), &[0, 1, 1]); // Bicycle price

    // Wrap around to first match
    assert!(state.next_search_result().0);
    assert_eq!(state.search_results_info(), Some((1, 3)));
    assert_eq!(state.cursor().path(), &[0, 0, 0, 3]);
}

#[test]
fn test_jsonpath_switch_to_text_search() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Start with JSONPath search
    state.execute_jsonpath_search("$..author");
    assert!(matches!(state.search_type(), Some(SearchType::JsonPath(_))));
    assert_eq!(state.search_results_info(), Some((1, 2)));

    // Switch to text search by setting search buffer and executing
    state.clear_search_buffer();
    for ch in "Melville".chars() {
        state.push_to_search_buffer(ch);
    }
    state.execute_search();

    assert!(matches!(state.search_type(), Some(SearchType::Text)));
    assert_eq!(state.search_results_info(), Some((1, 1)));
}

#[test]
fn test_jsonpath_switch_from_text_search() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Start with text search
    state.clear_search_buffer();
    for ch in "price".chars() {
        state.push_to_search_buffer(ch);
    }
    state.execute_search();
    assert!(matches!(state.search_type(), Some(SearchType::Text)));
    let text_results = state.search_results_info();
    assert!(text_results.is_some());
    assert!(text_results.unwrap().1 >= 3); // At least 3 "price" matches

    // Switch to JSONPath search
    state.execute_jsonpath_search("$..price");
    assert!(matches!(state.search_type(), Some(SearchType::JsonPath(_))));
    assert_eq!(state.search_results_info(), Some((1, 3))); // Exactly 3 price values
}

#[test]
fn test_jsonpath_invalid_query_error() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Invalid query - missing $
    state.execute_jsonpath_search("store.book");
    // Should have no results and error message
    assert_eq!(state.search_results_info(), None);

    // Invalid query - malformed bracket
    state.execute_jsonpath_search("$.store.book[");
    assert_eq!(state.search_results_info(), None);

    // Invalid query - invalid slice (this might parse as no-op, just checking it doesn't crash)
    state.execute_jsonpath_search("$.items[a:b]");
    assert_eq!(state.search_results_info(), None);
}

#[test]
fn test_jsonpath_no_matches() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Search for non-existent field
    state.execute_jsonpath_search("$.store.magazine");

    // Should have zero matches
    assert_eq!(state.search_results_info(), None);

    // Trying to navigate should return false
    assert!(!state.next_search_result().0);
}

#[test]
fn test_jsonpath_bracket_notation() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Use bracket notation for property access
    state.execute_jsonpath_search("$['store']['book'][0]['author']");

    assert_eq!(state.search_results_info(), Some((1, 1)));
    assert_eq!(state.cursor().path(), &[0, 0, 0, 1]); // First book's author
}

#[test]
fn test_jsonpath_preserves_cursor_on_error() {
    let tree = create_bookstore();
    let mut state = EditorState::new_with_default_theme(tree);

    // Set cursor to a specific position
    state.cursor_mut().set_path(vec![0, 0, 1]);
    let original_path = state.cursor().path().to_vec();

    // Execute invalid query
    state.execute_jsonpath_search("invalid query");

    // Cursor should be unchanged (error doesn't move cursor)
    assert_eq!(state.cursor().path(), &original_path);
    assert_eq!(state.search_results_info(), None);
}
