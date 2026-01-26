# Current Path in Status Bar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Display the current JSON path (e.g., `users[0].name`) in the status bar to help users understand their location in the document.

**Architecture:** Expose the existing `compute_path_string()` method as a public API, then call it from the status line renderer to display the path between the filename and search results.

**Tech Stack:** Rust, ratatui (TUI framework)

---

## Task 1: Add Public API for Path String

**Files:**
- Modify: `src/editor/state.rs` (lines 1551-1617)
- Test: `src/editor/state.rs` (add new test)

**Step 1: Write failing test for public path API**

Add this test to the `#[cfg(test)]` section at the end of `src/editor/state.rs`:

```rust
#[test]
fn test_get_current_path_dot_notation() {
    // Create tree: {"users": [{"name": "Alice"}]}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "users".to_string(),
        JsonNode::new(JsonValue::Array(vec![JsonNode::new(JsonValue::Object(
            vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            )],
        ))])),
    )])));

    let mut state = EditorState::new(tree);

    // Navigate to root
    assert_eq!(state.get_current_path(), "");

    // Navigate to "users" key
    state.move_down();
    assert_eq!(state.get_current_path(), "users");

    // Navigate to first array element
    state.move_down();
    assert_eq!(state.get_current_path(), "users[0]");

    // Navigate to "name" key
    state.move_down();
    assert_eq!(state.get_current_path(), "users[0].name");
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test test_get_current_path_dot_notation
```

Expected output: Compilation error - method `get_current_path` not found

**Step 3: Make compute_path_string public and add get_current_path method**

In `src/editor/state.rs`, change line 1551 from:
```rust
fn compute_path_string(&self, format: &str) -> Option<String> {
```

To:
```rust
pub fn compute_path_string(&self, format: &str) -> Option<String> {
```

Then add this new method after `compute_path_string()` (around line 1617):

```rust
/// Returns the current cursor path in dot notation (e.g., "users[0].name").
/// Returns empty string for root node.
///
/// # Examples
///
/// ```
/// # use jsonquill::document::node::{JsonNode, JsonValue};
/// # use jsonquill::document::tree::JsonTree;
/// # use jsonquill::editor::state::EditorState;
/// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
///     "key".to_string(),
///     JsonNode::new(JsonValue::String("value".to_string())),
/// )])));
/// let state = EditorState::new(tree);
/// assert_eq!(state.get_current_path(), "");
/// ```
pub fn get_current_path(&self) -> String {
    if self.cursor.path().is_empty() {
        return String::new();
    }

    self.compute_path_string("dot")
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_string()
}
```

**Step 4: Run test to verify it passes**

```bash
cargo test test_get_current_path_dot_notation
```

Expected output: test passes

**Step 5: Run all tests to ensure no regressions**

```bash
cargo test
```

Expected output: All tests pass

**Step 6: Commit**

```bash
git add src/editor/state.rs
git commit -m "feat(editor): add public API for getting current path in dot notation

- Make compute_path_string() public
- Add get_current_path() that returns path without leading dot
- Add test for path generation at various navigation positions"
```

---

## Task 2: Display Path in Status Line

**Files:**
- Modify: `src/ui/status_line.rs` (lines 48-73)
- Test: `src/ui/status_line.rs` (add new test)

**Step 1: Write failing test for path display**

Add this test to the `#[cfg(test)]` section in `src/ui/status_line.rs`:

```rust
#[test]
fn test_status_line_shows_current_path() {
    let backend = TestBackend::new(100, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    // Create tree: {"users": [{"name": "Alice"}]}
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "users".to_string(),
        JsonNode::new(JsonValue::Array(vec![JsonNode::new(JsonValue::Object(
            vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            )],
        ))])),
    )])));

    let mut state = EditorState::new(tree);
    state.set_filename("test.json".to_string());

    // Navigate to users[0].name
    state.move_down(); // users key
    state.move_down(); // users[0]
    state.move_down(); // name key

    let theme = theme::get_builtin_theme("default-dark").unwrap();

    terminal
        .draw(|f| {
            let area = f.area();
            render_status_line(f, area, &state, &theme.colors);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer.content();

    let text: String = content.iter().take(100).map(|c| c.symbol()).collect();
    assert!(
        text.contains("users[0].name"),
        "Status line should show current path: {}",
        text
    );
}

#[test]
fn test_status_line_no_path_at_root() {
    let backend = TestBackend::new(80, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
        "key".to_string(),
        JsonNode::new(JsonValue::String("value".to_string())),
    )])));

    let mut state = EditorState::new(tree);
    state.set_filename("test.json".to_string());
    // Stay at root - don't navigate

    let theme = theme::get_builtin_theme("default-dark").unwrap();

    terminal
        .draw(|f| {
            let area = f.area();
            render_status_line(f, area, &state, &theme.colors);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer.content();

    let text: String = content.iter().take(80).map(|c| c.symbol()).collect();

    // Should show filename but no path (no brackets or dots after filename)
    assert!(text.contains("test.json"), "Should show filename: {}", text);

    // Check that there's no path indicators after filename
    // Format should be: "NORMAL | test.json             1/2"
    // Not: "NORMAL | test.json key             1/2"
    let parts: Vec<&str> = text.split("test.json").collect();
    if parts.len() > 1 {
        let after_filename = parts[1].trim_start();
        // Should start with spaces (padding) or position counter, not a path
        assert!(
            !after_filename.starts_with(char::is_alphabetic),
            "Should not have path at root: {}",
            text
        );
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test test_status_line_shows_current_path test_status_line_no_path_at_root
```

Expected output: Tests fail - status line doesn't contain path

**Step 3: Update status line to display path**

In `src/ui/status_line.rs`, modify the `render_status_line` function. Change lines 48-73 to:

```rust
pub fn render_status_line(f: &mut Frame, area: Rect, state: &EditorState, colors: &ThemeColors) {
    let mode_text = format!("{}", state.mode());
    let filename = state.filename().unwrap_or("[No Name]");
    let dirty_indicator = if state.is_dirty() { " [+]" } else { "" };

    // Get current path if not at root
    let path = state.get_current_path();
    let path_display = if path.is_empty() {
        String::new()
    } else {
        format!(" {}", path)
    };

    let mut left = format!("{} | {}{}{}", mode_text, filename, path_display, dirty_indicator);

    // Add search results info if available
    if let Some((current, total)) = state.search_results_info() {
        use crate::editor::state::SearchType;
        let search_info = match state.search_type() {
            Some(SearchType::Text) => {
                format!(
                    " [Search: \"{}\"] Match {}/{}",
                    state.search_buffer(),
                    current,
                    total
                )
            }
            Some(SearchType::JsonPath(query)) => {
                format!(" [JSONPath: {}] Match {}/{}", query, current, total)
            }
            None => format!(" Match {}/{}", current, total),
        };
        left.push_str(&search_info);
    }

    // Get cursor position
    let row = state.cursor_position().0;
    let total = state.total_lines();
    let right = format!("{}/{}", row, total);

    // Calculate padding to position right-aligned text
    let total_width = area.width as usize;
    let right_len = right.len();
    let left_len = left.len();

    // Ensure we don't overflow
    let padding = if left_len + right_len + 1 < total_width {
        total_width - left_len - right_len
    } else {
        1
    };

    let content = format!("{}{}{}", left, " ".repeat(padding), right);

    let line = Line::from(Span::styled(
        content,
        Style::default()
            .fg(colors.status_line_fg)
            .bg(colors.status_line_bg),
    ));

    let status = Paragraph::new(line);

    f.render_widget(status, area);
}
```

**Step 4: Run tests to verify they pass**

```bash
cargo test test_status_line_shows_current_path test_status_line_no_path_at_root
```

Expected output: Both tests pass

**Step 5: Run all tests to ensure no regressions**

```bash
cargo test
```

Expected output: All tests pass

**Step 6: Run clippy**

```bash
cargo clippy -- -D warnings
```

Expected output: No warnings

**Step 7: Manual test**

```bash
cargo run -- testdata/example.json
```

Manual verification:
- Navigate through the JSON tree using j/k keys
- Verify the status bar shows the current path (e.g., `users[0].name`)
- Verify root node shows no path
- Verify path appears after filename (before dirty indicator and search results)

**Step 8: Commit**

```bash
git add src/ui/status_line.rs
git commit -m "feat(ui): display current path in status bar

- Show path in dot notation (e.g., users[0].name) after filename
- Empty string at root node (no path displayed)
- Add tests for path display and root node behavior"
```

---

## Task 3: Update Documentation

**Files:**
- Modify: `CLAUDE.md` (update status line description)

**Step 1: Update CLAUDE.md to document the new feature**

In `CLAUDE.md`, find the section describing the status line (around line 58-59) and update it:

Change from:
```markdown
- ✅ Status line showing current mode, filename, and cursor position (row,col row/total)
```

To:
```markdown
- ✅ Status line showing current mode, filename, current path, and cursor position
```

Also add a description in the relevant section explaining the feature:

Find the "Working Features" section and add after the status line entry:
```markdown
  - Shows current JSON path in dot notation (e.g., `users[0].name`, `config.database.port`)
  - Path displayed after filename (before dirty indicator and search results)
  - Root node shows no path
```

**Step 2: Commit documentation**

```bash
git add CLAUDE.md
git commit -m "docs: document current path in status bar feature"
```

---

## Task 4: Final Verification

**Step 1: Run full test suite**

```bash
cargo test
```

Expected output: All tests pass

**Step 2: Run clippy**

```bash
cargo clippy -- -D warnings
```

Expected output: No warnings

**Step 3: Run fmt check**

```bash
cargo fmt -- --check
```

Expected output: No formatting issues

**Step 4: Build release binary**

```bash
cargo build --release
```

Expected output: Clean build

**Step 5: Manual integration test**

Create a test JSON file:
```bash
echo '{"store": {"books": [{"title": "1984", "author": "Orwell"}, {"title": "Brave New World", "author": "Huxley"}], "music": {"albums": [{"name": "Dark Side"}]}}}' > /tmp/test-path.json
```

Run jsonquill:
```bash
cargo run --release -- /tmp/test-path.json
```

Manual verification checklist:
- [ ] At root: status shows `NORMAL | /tmp/test-path.json` (no path)
- [ ] Navigate to "store" key: status shows `store`
- [ ] Navigate to "books": status shows `store.books`
- [ ] Navigate to first book: status shows `store.books[0]`
- [ ] Navigate to "title": status shows `store.books[0].title`
- [ ] Navigate to second book: status shows `store.books[1]`
- [ ] Navigate to music.albums[0].name: status shows `store.music.albums[0].name`
- [ ] Path doesn't break search results display (test with `/` search)
- [ ] Path doesn't break dirty indicator (test with `dd` to modify)

**Step 6: Cleanup test file**

```bash
rm /tmp/test-path.json
```

---

## Summary

This implementation adds current path display to the status bar by:

1. **Task 1**: Exposing existing path computation logic as public API (`get_current_path()`)
2. **Task 2**: Integrating path display into status line renderer
3. **Task 3**: Updating documentation
4. **Task 4**: Final verification and testing

The implementation is minimal and reuses existing code. The path appears in dot notation (e.g., `users[0].name`) between the filename and search results, helping users understand their location in the JSON document structure.
