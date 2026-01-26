# Clear Search Info on Non-Search Keys Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Automatically clear search results from the status bar when the user presses any key other than `n` (next search result).

**Architecture:** Add a `clear_search_results()` method to EditorState, then call it in the input handler for all InputEvents except `NextSearchResult`. The status bar will automatically hide search info when results are empty.

**Tech Stack:** Rust

---

## Task 1: Add clear_search_results Method

**Files:**
- Modify: `src/editor/state.rs`
- Test: `tests/jsonpath_tests.rs` (existing integration test file)

**Step 1: Write failing test**

Add this test to `tests/jsonpath_tests.rs`:

```rust
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
        (
            "age".to_string(),
            JsonNode::new(JsonValue::Number(30.0)),
        ),
    ])));

    let mut state = EditorState::new(tree);
    state.rebuild_tree_view();

    // Execute a search to populate results
    state.execute_text_search("name", false);

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
```

**Step 2: Run test to verify it fails**

```bash
cargo test test_clear_search_results
```

Expected: Compilation error - `clear_search_results` method not found

**Step 3: Add clear_search_results method**

In `src/editor/state.rs`, find the search-related methods (around line 1920-2100) and add this method:

```rust
/// Clears search results but preserves search buffer and type.
/// This removes search info from the status bar while keeping
/// the search query available for potential "repeat search" features.
pub fn clear_search_results(&mut self) {
    self.search_results.clear();
    self.search_index = 0;
}
```

**Step 4: Run test to verify it passes**

```bash
cargo test test_clear_search_results
```

Expected: Test passes

**Step 5: Run all tests**

```bash
cargo test
```

Expected: All 122 tests pass (or 123 with new test)

**Step 6: Commit**

```bash
git add src/editor/state.rs tests/jsonpath_tests.rs
git commit -m "feat(editor): add clear_search_results method

- Clears search results vector and resets index
- Preserves search buffer for potential repeat search feature
- Add test to verify clearing behavior"
```

---

## Task 2: Call clear_search_results in Input Handler

**Files:**
- Modify: `src/input/handler.rs` (lines 580-700+)

**Step 1: Add clear_search_results call to all non-search events**

In `src/input/handler.rs`, in the `handle_event` method's match statement for `EditorMode::Normal`, add `state.clear_search_results();` after `state.clear_pending();` for all events EXCEPT:
- `InputEvent::NextSearchResult`
- `InputEvent::EnterSearchMode`
- `InputEvent::EnterReverseSearchMode`

The pattern should be:
```rust
InputEvent::SomeEvent => {
    state.clear_pending();
    state.clear_search_results(); // ← Add this line
    // ... rest of handler
}
```

**Events that need the call** (find these around lines 608-700+):
- `InputEvent::Help`
- `InputEvent::ExitMode`
- `InputEvent::MoveDown`
- `InputEvent::MoveUp`
- `InputEvent::MoveLeft`
- `InputEvent::MoveRight`
- `InputEvent::PageDown`
- `InputEvent::PageUp`
- `InputEvent::JumpToTop`
- `InputEvent::JumpToBottom`
- `InputEvent::EnterCommandMode`
- `InputEvent::EnterInsertMode`
- `InputEvent::Quit`
- `InputEvent::SaveAndQuit`
- `InputEvent::Delete`
- `InputEvent::Yank`
- `InputEvent::Paste`
- `InputEvent::PasteBefore`
- `InputEvent::Add`
- `InputEvent::AddArray`
- `InputEvent::AddObject`
- `InputEvent::Rename`
- `InputEvent::ExpandAll`
- `InputEvent::CollapseAll`
- `InputEvent::MoveToParent`
- `InputEvent::ScreenPosition`
- `InputEvent::NextSibling`
- `InputEvent::PreviousSibling`
- `InputEvent::FirstSibling`
- `InputEvent::LastSibling`
- `InputEvent::SearchKeyForward`
- `InputEvent::SearchKeyBackward`
- `InputEvent::NextAtSameOrShallowerDepth`
- `InputEvent::PreviousAtSameOrShallowerDepth`
- `InputEvent::Undo`
- `InputEvent::Redo`
- Any others in the match statement

**Important**: Do NOT add it to:
- `InputEvent::NextSearchResult` (line 589)
- `InputEvent::EnterSearchMode` (line 576)
- `InputEvent::EnterReverseSearchMode` (line 583)

**Step 2: Run tests**

```bash
cargo test
```

Expected: All tests pass

**Step 3: Run clippy**

```bash
cargo clippy -- -D warnings
```

Expected: No warnings

**Step 4: Manual test**

```bash
echo '{"users": [{"name": "Alice"}, {"name": "Bob"}], "count": 2}' > /tmp/test-search-clear.json
cargo run -- /tmp/test-search-clear.json
```

Manual verification:
1. Press `/` and search for "name"
2. Status bar shows: `[Search: "name"] Match 1/2`
3. Press `n` - navigates to next match
4. Status bar still shows: `[Search: "name"] Match 2/2`
5. Press `j` - moves down
6. Status bar no longer shows search info (should just show path and position)
7. Press `k` - moves up
8. Status bar still has no search info

**Step 5: Cleanup**

```bash
rm /tmp/test-search-clear.json
```

**Step 6: Commit**

```bash
git add src/input/handler.rs
git commit -m "feat(input): clear search results on non-search keys

- Call clear_search_results() for all input events except search navigation
- Search info automatically removed from status bar after any non-search action
- Preserves search results only while actively navigating with 'n'"
```

---

## Task 3: Update Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update search functionality description**

In `CLAUDE.md`, find the Search section (around line 250-270) and update the search behavior description.

Add this after the `/`, `?`, `n` key descriptions:

```markdown
Note: Search results info disappears from the status bar when you press any key
other than `n` (next match). This keeps the status bar clean once you're done
navigating search results.
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: document search results auto-clear behavior"
```

---

## Task 4: Final Verification

**Step 1: Run full test suite**

```bash
cargo test
```

Expected: All tests pass

**Step 2: Run clippy**

```bash
cargo clippy -- -D warnings
```

Expected: No warnings

**Step 3: Run fmt check**

```bash
cargo fmt -- --check
```

Expected: No formatting issues

**Step 4: Manual integration test**

Create test file:
```bash
echo '{"store": {"books": [{"title": "1984", "author": "Orwell"}, {"title": "Brave New World", "author": "Huxley"}], "music": {"albums": [{"name": "Dark Side"}]}}}' > /tmp/test-final.json
```

Run jsonquill:
```bash
cargo run --release -- /tmp/test-final.json
```

Manual verification checklist:
- [ ] Search with `/title` - status shows `[Search: "title"] Match 1/2`
- [ ] Press `n` - status still shows `[Search: "title"] Match 2/2`
- [ ] Press `j` - search info disappears, shows normal status
- [ ] Search with `?author` (backward) - status shows search info
- [ ] Press `h` (collapse) - search info disappears
- [ ] Search with `/name` - status shows search info
- [ ] Press `i` (insert mode) - search info disappears
- [ ] Search with `*` (search current key) - status shows search info
- [ ] Press `k` - search info disappears

**Step 5: Cleanup**

```bash
rm /tmp/test-final.json
```

---

## Summary

This implementation adds automatic clearing of search results when pressing non-search keys by:

1. **Task 1**: Adding `clear_search_results()` method to EditorState with test
2. **Task 2**: Calling the method for all InputEvents except search navigation
3. **Task 3**: Updating documentation to explain the behavior
4. **Task 4**: Comprehensive verification

The implementation is minimal and leverages existing infrastructure - the status bar already checks for empty search results and hides the display accordingly. No changes to status_line.rs are needed.
