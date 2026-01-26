# Colored Path in Status Bar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add color highlighting to the path in the status bar using the theme's key color to make it stand out.

**Architecture:** Refactor status line rendering from a single styled span to multiple styled spans, allowing the path segment to use a different color (colors.key) while other elements use the default status line foreground color.

**Tech Stack:** Rust, ratatui (TUI framework)

---

## Task 1: Refactor Status Line to Multi-Span Rendering

**Files:**
- Modify: `src/ui/status_line.rs` (lines 48-110)

**Step 1: Update status line to use multiple spans**

Currently the status line renders as a single span. We need to break it into multiple spans so we can color each part independently.

In `src/ui/status_line.rs`, replace the rendering logic (lines 62-110) with:

```rust
// Build status line components as separate variables for styling
let separator = " | ";
let search_info = if let Some((current, total)) = state.search_results_info() {
    use crate::editor::state::SearchType;
    match state.search_type() {
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
    }
} else {
    String::new()
};

// Calculate total length for padding
let left_len = mode_text.len()
    + separator.len()
    + filename.len()
    + path_display.len()
    + dirty_indicator.len()
    + search_info.len();

// Get cursor position
let row = state.cursor_position().0;
let total = state.total_lines();
let position = format!("{}/{}", row, total);
let right_len = position.len();

// Calculate padding
let total_width = area.width as usize;
let padding = if left_len + right_len + 1 < total_width {
    total_width - left_len - right_len
} else {
    1
};

// Build line from multiple styled spans
let line = Line::from(vec![
    Span::styled(
        mode_text,
        Style::default()
            .fg(colors.status_line_fg)
            .bg(colors.status_line_bg),
    ),
    Span::styled(
        separator,
        Style::default()
            .fg(colors.status_line_fg)
            .bg(colors.status_line_bg),
    ),
    Span::styled(
        filename,
        Style::default()
            .fg(colors.status_line_fg)
            .bg(colors.status_line_bg),
    ),
    Span::styled(
        path_display,
        Style::default().fg(colors.key).bg(colors.status_line_bg),
    ),
    Span::styled(
        dirty_indicator,
        Style::default()
            .fg(colors.status_line_fg)
            .bg(colors.status_line_bg),
    ),
    Span::styled(
        search_info,
        Style::default()
            .fg(colors.status_line_fg)
            .bg(colors.status_line_bg),
    ),
    Span::styled(
        " ".repeat(padding),
        Style::default().bg(colors.status_line_bg),
    ),
    Span::styled(
        position,
        Style::default()
            .fg(colors.status_line_fg)
            .bg(colors.status_line_bg),
    ),
]);

let status = Paragraph::new(line);
f.render_widget(status, area);
```

**Step 2: Add required imports**

At the top of `src/ui/status_line.rs`, ensure these imports are present:

```rust
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
```

**Step 3: Run existing tests to verify no regressions**

```bash
cargo test ui::status_line::tests
```

Expected: All 9 tests should still pass (behavior unchanged, just rendering implementation)

**Step 4: Manual verification across themes**

Create a test file:
```bash
echo '{"users": [{"name": "Alice", "age": 30}]}' > /tmp/test-colored-path.json
```

Test with default-dark theme:
```bash
cargo run -- /tmp/test-colored-path.json
```

Navigate to `users[0].name` (press j three times) and verify:
- Path `users[0].name` appears in the status bar
- Path should be in a different color (light blue in default-dark) than surrounding text

Test with light theme:
```bash
cargo run -- /tmp/test-colored-path.json
# Press :theme default-light
# Navigate to users[0].name
```

Verify path appears in magenta-ish color (different from black text).

**Step 5: Cleanup test file**

```bash
rm /tmp/test-colored-path.json
```

**Step 6: Run full test suite**

```bash
cargo test
```

Expected: All 121 tests pass

**Step 7: Run clippy**

```bash
cargo clippy -- -D warnings
```

Expected: No warnings

**Step 8: Commit**

```bash
git add src/ui/status_line.rs
git commit -m "feat(ui): add color highlighting to path in status bar

- Refactor status line from single span to multi-span rendering
- Apply theme's key color to path segment for visibility
- Other elements remain status_line_fg color
- Works across all 8 built-in themes"
```

---

## Task 2: Add Test for Multi-Span Structure

**Files:**
- Modify: `src/ui/status_line.rs` (add test in `#[cfg(test)]` section)

**Step 1: Write test for colored path span**

Add this test to the `#[cfg(test)]` module in `src/ui/status_line.rs`:

```rust
#[test]
fn test_status_line_path_uses_key_color() {
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

    // Navigate to users[0].name (has a path)
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

    // Find the path segment in the buffer
    let mut found_path_with_key_color = false;
    let text: String = content.iter().take(100).map(|c| c.symbol()).collect();

    // Verify the path appears in the output
    assert!(
        text.contains("users[0].name"),
        "Status line should contain path: {}",
        text
    );

    // Check that the cells containing the path have the key color
    // The path "users[0].name" should be rendered with colors.key foreground
    for (i, cell) in content.iter().enumerate().take(100) {
        let cell_text = cell.symbol();
        // Check if this cell is part of the path
        if text[..i.min(text.len())].ends_with("test.json") {
            // We're past the filename, check next segment for path
            let remaining = &text[i..];
            if remaining.starts_with(" users") || remaining.starts_with("users") {
                // This should be the path segment with key color
                if cell.fg == theme.colors.key {
                    found_path_with_key_color = true;
                    break;
                }
            }
        }
    }

    assert!(
        found_path_with_key_color,
        "Path segment should use theme's key color"
    );
}
```

**Step 2: Run the new test**

```bash
cargo test test_status_line_path_uses_key_color
```

Expected: Test passes

**Step 3: Run all status line tests**

```bash
cargo test ui::status_line::tests
```

Expected: All 10 tests pass (9 existing + 1 new)

**Step 4: Run full test suite**

```bash
cargo test
```

Expected: All 122 tests pass

**Step 5: Commit**

```bash
git add src/ui/status_line.rs
git commit -m "test(ui): add test for colored path in status bar

- Verify path segment uses theme's key color
- Check multi-span rendering structure"
```

---

## Task 3: Update Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update feature description**

In `CLAUDE.md`, find the status line bullet (around line 148-151) and update the sub-bullet about path display:

Change from:
```markdown
  - Shows current JSON path in dot notation (e.g., `users[0].name`, `config.database.port`)
```

To:
```markdown
  - Shows current JSON path in dot notation (e.g., `users[0].name`, `config.database.port`) highlighted in the theme's key color
```

**Step 2: Commit documentation**

```bash
git add CLAUDE.md
git commit -m "docs: document path color highlighting in status bar"
```

---

## Task 4: Final Verification

**Step 1: Run full test suite**

```bash
cargo test
```

Expected: All 122 tests pass

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

**Step 4: Build release binary**

```bash
cargo build --release
```

Expected: Clean build

**Step 5: Manual verification across all themes**

Create test file:
```bash
echo '{"store": {"books": [{"title": "1984", "author": "Orwell"}], "config": {"theme": "dark", "port": 8080}}}' > /tmp/test-themes.json
```

Test each theme:

**default-dark:**
```bash
cargo run --release -- /tmp/test-themes.json
# Navigate to store.books[0].title
# Verify path is light blue (different from white/gray text)
```

**default-light:**
```bash
# In the app, run :theme default-light
# Navigate to store.config.port
# Verify path is magenta (different from dark gray text)
```

**gruvbox-dark:**
```bash
# Run :theme gruvbox-dark
# Navigate to store.books[0].author
# Verify path is orange (gruvbox key color)
```

**nord:**
```bash
# Run :theme nord
# Navigate to store.config.theme
# Verify path is frost cyan (nord key color)
```

**dracula:**
```bash
# Run :theme dracula
# Navigate to store.books[0]
# Verify path is cyan (dracula key color)
```

Manual checklist:
- [ ] Path color stands out from surrounding text in all themes
- [ ] Path color matches the key color used in tree view
- [ ] Color doesn't break dirty indicator display
- [ ] Color doesn't break search results display
- [ ] Root node still shows no path

**Step 6: Cleanup test file**

```bash
rm /tmp/test-themes.json
```

---

## Summary

This implementation adds color highlighting to the path in the status bar by:

1. **Task 1**: Refactoring status line rendering from single-span to multi-span approach
2. **Task 2**: Adding test to verify path uses key color
3. **Task 3**: Updating documentation
4. **Task 4**: Comprehensive verification across all themes

The implementation uses the theme's `key` color for the path, which is semantically appropriate (paths show keys/indices) and already optimized for visibility across all 8 built-in themes. The change is purely cosmetic and doesn't affect any functionality or behavior.
