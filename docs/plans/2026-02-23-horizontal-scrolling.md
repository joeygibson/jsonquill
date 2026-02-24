# Horizontal Scrolling Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add vim-style horizontal scrolling (zh/zl/zH/zL/zs/ze) with auto-reset on vertical movement and auto-scroll for search results.

**Architecture:** A single `horizontal_offset: usize` field on `EditorState` applied during rendering. Line numbers are pinned; all tree content shifts left. The existing `z` pending command pattern is extended with new second-character matches (h/l/H/L/s/e). Vertical movement methods reset the offset to 0.

**Tech Stack:** Rust, ratatui (span clipping), existing pending command system

---

### Task 1: Add horizontal_offset field to EditorState

**Files:**
- Modify: `src/editor/state.rs:229-230` (add field after `scroll_offset`)
- Modify: `src/editor/state.rs:279-354` (initialize in `new()`)

**Step 1: Write the failing test**

Create `tests/horizontal_scroll_tests.rs`:

```rust
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_horizontal_offset_defaults_to_zero() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let state = EditorState::new_with_default_theme(tree);
    assert_eq!(state.horizontal_offset(), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_horizontal_offset_defaults_to_zero`
Expected: FAIL - `horizontal_offset` method doesn't exist

**Step 3: Write minimal implementation**

In `src/editor/state.rs`:

1. Add field after `scroll_offset: usize,` (line 229):
```rust
    scroll_offset: usize,
    horizontal_offset: usize,
```

2. Initialize to 0 in `new()` (in the struct init block, after `scroll_offset: 0`):
```rust
    horizontal_offset: 0,
```

3. Add getter/setter/reset methods (near `scroll_offset()` getter around line 1030):
```rust
    pub fn horizontal_offset(&self) -> usize {
        self.horizontal_offset
    }

    pub fn set_horizontal_offset(&mut self, offset: usize) {
        self.horizontal_offset = offset;
    }

    pub fn reset_horizontal_offset(&mut self) {
        self.horizontal_offset = 0;
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_horizontal_offset_defaults_to_zero`
Expected: PASS

**Step 5: Commit**

```bash
git add src/editor/state.rs tests/horizontal_scroll_tests.rs
git commit -m "feat: add horizontal_offset field to EditorState"
```

---

### Task 2: Add horizontal scroll methods to EditorState

**Files:**
- Modify: `src/editor/state.rs` (add scroll methods near screen positioning methods ~line 1229)
- Modify: `tests/horizontal_scroll_tests.rs`

**Step 1: Write the failing tests**

Append to `tests/horizontal_scroll_tests.rs`:

```rust
#[test]
fn test_scroll_right() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    assert_eq!(state.horizontal_offset(), 5);
}

#[test]
fn test_scroll_left_clamps_to_zero() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(3);
    state.scroll_left(10);
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_scroll_left() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(10);
    state.scroll_left(3);
    assert_eq!(state.horizontal_offset(), 7);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_scroll_right test_scroll_left`
Expected: FAIL - methods don't exist

**Step 3: Write minimal implementation**

Add to `src/editor/state.rs` near the other scroll methods:

```rust
    pub fn scroll_right(&mut self, count: usize) {
        self.horizontal_offset = self.horizontal_offset.saturating_add(count);
    }

    pub fn scroll_left(&mut self, count: usize) {
        self.horizontal_offset = self.horizontal_offset.saturating_sub(count);
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_scroll_right test_scroll_left`
Expected: PASS

**Step 5: Commit**

```bash
git add src/editor/state.rs tests/horizontal_scroll_tests.rs
git commit -m "feat: add scroll_left/scroll_right methods to EditorState"
```

---

### Task 3: Reset horizontal_offset on vertical movement

**Files:**
- Modify: `src/editor/state.rs` (add `self.horizontal_offset = 0;` to vertical movement methods)
- Modify: `tests/horizontal_scroll_tests.rs`

**Step 1: Write the failing test**

Append to `tests/horizontal_scroll_tests.rs`:

```rust
#[test]
fn test_horizontal_offset_resets_on_move_down() {
    let node = JsonNode::new(JsonValue::Object(vec![
        JsonNode::new_with_key("a".to_string(), JsonValue::String("hello".to_string())),
        JsonNode::new_with_key("b".to_string(), JsonValue::String("world".to_string())),
    ]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    assert_eq!(state.horizontal_offset(), 5);
    state.move_cursor_down();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_horizontal_offset_resets_on_jump_to_top() {
    let node = JsonNode::new(JsonValue::Object(vec![
        JsonNode::new_with_key("a".to_string(), JsonValue::String("hello".to_string())),
        JsonNode::new_with_key("b".to_string(), JsonValue::String("world".to_string())),
    ]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    state.jump_to_top();
    assert_eq!(state.horizontal_offset(), 0);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_horizontal_offset_resets`
Expected: FAIL - offset still 5 after movement

**Step 3: Add resets to all vertical movement methods**

In `src/editor/state.rs`, add `self.horizontal_offset = 0;` at the start of each of these methods:

- `move_cursor_down` (~line 894)
- `move_cursor_up` (~line 944)
- `jump_to_top` (~line 1070)
- `jump_to_bottom` (~line 1079)
- `jump_to_line` (~line 1090)
- `page_down` (~line 1105)
- `page_up` (~line 1137)
- `full_page_down` (~line 1167)
- `full_page_up` (~line 1199)
- `center_cursor_on_screen` (~line 1229)
- `cursor_to_top_of_screen` (~line 1257)
- `cursor_to_bottom_of_screen` (~line 1276)
- `move_to_next_sibling` (~line 1330)
- `move_to_previous_sibling` (~line 1497)
- `move_to_first_sibling` (~line 1377)
- `move_to_last_sibling` (~line 1423)
- `move_to_parent` (~line 1582)
- `move_to_next_at_same_or_shallower_depth` (~line 1543)
- `move_to_previous_at_same_or_shallower_depth` (~line 1613)
- `next_search_result` (~line 3007)

**Step 4: Run tests to verify they pass**

Run: `cargo test test_horizontal_offset_resets`
Expected: PASS

**Step 5: Commit**

```bash
git add src/editor/state.rs tests/horizontal_scroll_tests.rs
git commit -m "feat: reset horizontal_offset on vertical movement"
```

---

### Task 4: Wire up zh/zl/zH/zL keybindings in handler

**Files:**
- Modify: `src/input/handler.rs:653-675` (extend the `z` pending command match)
- Modify: `tests/horizontal_scroll_tests.rs`

**Step 1: Write the failing test**

Append to `tests/horizontal_scroll_tests.rs`:

```rust
#[test]
fn test_zh_scrolls_left() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    // Simulate zh: set pending 'z', then handle 'h'
    state.set_pending_command('z');
    // We'll test the state method directly since handler depends on terminal I/O
    state.scroll_left(1);
    state.clear_pending();
    assert_eq!(state.horizontal_offset(), 4);
}
```

**Step 2: Run test to verify it passes** (this tests the state methods, not the handler wiring)

Run: `cargo test test_zh_scrolls_left`
Expected: PASS (state methods already work)

**Step 3: Wire up handler**

In `src/input/handler.rs`, extend the `if state.pending_command() == Some('z')` match block (line 654) to add cases before the `_ =>` fallthrough:

```rust
// Handle screen positioning and horizontal scroll commands (z prefix)
if state.pending_command() == Some('z') {
    match c {
        'z' => {
            state.clear_pending();
            state.center_cursor_on_screen();
            return Ok(false);
        }
        't' => {
            state.clear_pending();
            state.cursor_to_top_of_screen();
            return Ok(false);
        }
        'b' => {
            state.clear_pending();
            state.cursor_to_bottom_of_screen();
            return Ok(false);
        }
        'h' => {
            let count = state.get_count() as usize;
            state.clear_pending();
            state.scroll_left(count);
            return Ok(false);
        }
        'l' => {
            let count = state.get_count() as usize;
            state.clear_pending();
            state.scroll_right(count);
            return Ok(false);
        }
        'H' => {
            let count = state.get_count() as usize;
            let half_width = state.viewport_width() / 2;
            state.clear_pending();
            state.scroll_left(half_width * count);
            return Ok(false);
        }
        'L' => {
            let count = state.get_count() as usize;
            let half_width = state.viewport_width() / 2;
            state.clear_pending();
            state.scroll_right(half_width * count);
            return Ok(false);
        }
        's' => {
            state.clear_pending();
            state.scroll_cursor_to_left_edge();
            return Ok(false);
        }
        'e' => {
            state.clear_pending();
            state.scroll_cursor_to_right_edge();
            return Ok(false);
        }
        _ => {
            // Not a screen positioning command, continue with normal processing
        }
    }
}
```

**Step 4: Add `viewport_width`, `scroll_cursor_to_left_edge`, and `scroll_cursor_to_right_edge` to EditorState**

In `src/editor/state.rs`:

1. Add `viewport_width: usize` field (next to `viewport_height`):
```rust
    viewport_height: usize,
    viewport_width: usize,
```

2. Initialize to 0 in `new()`.

3. Add getter:
```rust
    pub fn viewport_width(&self) -> usize {
        self.viewport_width
    }
```

4. Store viewport_width in `adjust_scroll_to_cursor` (or add a new method that gets called from the render path). Since `adjust_scroll_to_cursor` already receives viewport_height and stores it, add a sibling method or extend the signature. The simplest approach: add a `set_viewport_width` setter called from `render()`.

```rust
    pub fn set_viewport_width(&mut self, width: usize) {
        self.viewport_width = width;
    }
```

5. Add `cursor_line_display_width` helper that calculates how wide the current cursor line is (in characters):
```rust
    pub fn cursor_line_display_width(&self) -> usize {
        let lines = self.tree_view.lines();
        let current_path = self.cursor.path();
        if let Some(line) = lines.iter().find(|l| l.path == current_path) {
            let indent = line.depth * 2; // "  " per depth
            let indicator = 2; // "▼ " or "▶ " or "  "
            let key_len = line.key.as_ref().map(|k| k.len() + 2).unwrap_or(0); // "key: "
            let value_len = line.value_preview.len();
            indent + indicator + key_len + value_len
        } else {
            0
        }
    }
```

6. Add `zs`/`ze` methods:
```rust
    pub fn scroll_cursor_to_left_edge(&mut self) {
        // Set horizontal offset so cursor line content starts at the left edge
        // Calculate the start position of the cursor line's meaningful content
        let lines = self.tree_view.lines();
        if let Some(line) = lines.iter().find(|l| l.path == self.cursor.path()) {
            let indent = line.depth * 2;
            self.horizontal_offset = indent;
        }
    }

    pub fn scroll_cursor_to_right_edge(&mut self) {
        let width = self.cursor_line_display_width();
        if width > self.viewport_width {
            self.horizontal_offset = width - self.viewport_width;
        } else {
            self.horizontal_offset = 0;
        }
    }
```

**Step 5: Wire viewport_width from render()**

In `src/ui/mod.rs` around line 143-144, after `adjust_scroll_to_cursor`:
```rust
    let viewport_width = chunks[0].width as usize;
    state.set_viewport_width(viewport_width);
```

**Step 6: Run full tests**

Run: `cargo test`
Expected: PASS

**Step 7: Commit**

```bash
git add src/editor/state.rs src/input/handler.rs src/ui/mod.rs tests/horizontal_scroll_tests.rs
git commit -m "feat: wire up zh/zl/zH/zL/zs/ze keybindings for horizontal scroll"
```

---

### Task 5: Apply horizontal offset in render_tree_view

**Files:**
- Modify: `src/ui/tree_view.rs:533-673` (add `horizontal_offset` parameter and span clipping)
- Modify: `src/ui/mod.rs:147-157` (pass `horizontal_offset` to render call)

**Step 1: Write the failing test**

Add to `src/ui/tree_view.rs` tests (at the bottom of the file within the existing `#[cfg(test)]` block, or create one):

This is a rendering test, so we test the span clipping helper function directly.

Append to `tests/horizontal_scroll_tests.rs`:

```rust
#[test]
fn test_horizontal_offset_is_passed_to_render() {
    // Verify that EditorState exposes horizontal_offset for the render path
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(10);
    assert_eq!(state.horizontal_offset(), 10);
}
```

**Step 2: Implement span clipping**

Add a helper function in `src/ui/tree_view.rs` before `render_tree_view`:

```rust
/// Clips a list of spans by skipping `offset` characters from the left.
/// Preserves styling across span boundaries.
fn clip_spans_horizontal(spans: Vec<Span<'_>>, offset: usize) -> Vec<Span<'_>> {
    if offset == 0 {
        return spans;
    }
    let mut remaining = offset;
    let mut result = Vec::new();
    for span in spans {
        let len = span.content.len();
        if remaining >= len {
            remaining -= len;
            continue;
        }
        if remaining > 0 {
            // Partial clip: skip `remaining` chars from this span
            let content: String = span.content.chars().skip(remaining).collect();
            result.push(Span::styled(content, span.style));
            remaining = 0;
        } else {
            result.push(span);
        }
    }
    result
}
```

**Step 3: Modify `render_tree_view` signature and rendering loop**

Update the function signature to accept `horizontal_offset`:

```rust
pub fn render_tree_view(
    f: &mut Frame,
    area: Rect,
    tree_view: &TreeViewState,
    cursor: &Cursor,
    colors: &ThemeColors,
    show_line_numbers: bool,
    relative_line_numbers: bool,
    scroll_offset: usize,
    horizontal_offset: usize,
    visual_selection: &[Vec<usize>],
)
```

In the rendering loop, split line number spans from content spans:

Replace the section from the final line assembly (after `spans.push(Span::styled(&line.value_preview, value_style));`, around line 649) through the `lines_to_render.push(final_line);` (line 665) with:

```rust
        // Separate line number spans from content spans for horizontal scrolling
        let (line_num_spans, content_spans) = if show_line_numbers && !spans.is_empty() {
            // First span is the line number - pin it
            let line_num_span = spans.remove(0);
            (vec![line_num_span], spans)
        } else {
            (vec![], spans)
        };

        // Apply horizontal offset to content spans only
        let clipped_content = clip_spans_horizontal(content_spans, horizontal_offset);

        // Reassemble: pinned line numbers + clipped content
        let mut all_spans = line_num_spans;
        all_spans.extend(clipped_content);

        // Apply visual selection background if this line is selected
        let final_line = if is_selected {
            Line::from(
                all_spans
                    .into_iter()
                    .map(|span| {
                        Span::styled(span.content, span.style.bg(colors.visual_selection_bg))
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            Line::from(all_spans)
        };

        lines_to_render.push(final_line);
```

**Step 4: Update render call site**

In `src/ui/mod.rs` line 147-157, add `horizontal_offset`:

```rust
            tree_view::render_tree_view(
                f,
                chunks[0],
                state.tree_view(),
                state.cursor(),
                &self.theme.colors,
                state.show_line_numbers(),
                state.relative_line_numbers(),
                state.scroll_offset(),
                state.horizontal_offset(),
                state.visual_selection(),
            );
```

**Step 5: Update doctest and any other callers**

Update the doctest in `render_tree_view` (around line 529) and any test callers that call `render_tree_view` directly with the new parameter (add `0` for horizontal_offset).

Search for all call sites: `render_tree_view(` — update lines ~995, ~1046, ~1116 in tree_view.rs tests to add the `0` parameter.

**Step 6: Run all tests**

Run: `cargo fmt && cargo clippy -- -D warnings && cargo test`
Expected: PASS

**Step 7: Commit**

```bash
git add src/ui/tree_view.rs src/ui/mod.rs tests/horizontal_scroll_tests.rs
git commit -m "feat: apply horizontal_offset in render_tree_view with span clipping"
```

---

### Task 6: Auto-scroll horizontally for search results

**Files:**
- Modify: `src/editor/state.rs` (in `next_search_result` and search confirmation paths)
- Modify: `tests/horizontal_scroll_tests.rs`

**Step 1: Write the failing test**

Append to `tests/horizontal_scroll_tests.rs`:

```rust
#[test]
fn test_search_resets_horizontal_offset() {
    let node = JsonNode::new(JsonValue::Object(vec![
        JsonNode::new_with_key("a".to_string(), JsonValue::String("hello".to_string())),
    ]));
    let tree = JsonTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(20);
    assert_eq!(state.horizontal_offset(), 20);
    // Navigating to a search result should reset horizontal offset
    // (vertical movement reset handles this)
    state.jump_to_top();
    assert_eq!(state.horizontal_offset(), 0);
}
```

**Step 2: Verify test passes** (this should already pass from Task 3's resets)

Run: `cargo test test_search_resets_horizontal_offset`
Expected: PASS — `next_search_result` calls `cursor.set_path()` which we already reset in Task 3.

Note: The `next_search_result` method already resets horizontal_offset because we added the reset there in Task 3. The search auto-scroll (scrolling *to* the match position) is handled by the vertical `adjust_scroll_to_cursor`. If the match line is longer than the viewport, the user can use `zs`/`ze`/`zl` to scroll to it — this matches vim behavior where search jumps to the line but horizontal scroll adjusts based on `sidescroll` settings.

**Step 3: Commit**

```bash
git add tests/horizontal_scroll_tests.rs
git commit -m "test: verify search navigation resets horizontal offset"
```

---

### Task 7: Update help overlay and documentation

**Files:**
- Modify: `src/ui/help_overlay.rs` (add zh/zl/zH/zL/zs/ze entries after zb entry at line 97)

**Step 1: Add help entries**

In `src/ui/help_overlay.rs`, after the `zb` entry (line 95-97), add:

```rust
        Line::from(vec![
            Span::styled("  zh            ", Style::default().fg(colors.number)),
            Span::raw("Scroll left 1 column"),
        ]),
        Line::from(vec![
            Span::styled("  zl            ", Style::default().fg(colors.number)),
            Span::raw("Scroll right 1 column"),
        ]),
        Line::from(vec![
            Span::styled("  zH            ", Style::default().fg(colors.number)),
            Span::raw("Scroll left half-screen"),
        ]),
        Line::from(vec![
            Span::styled("  zL            ", Style::default().fg(colors.number)),
            Span::raw("Scroll right half-screen"),
        ]),
        Line::from(vec![
            Span::styled("  zs            ", Style::default().fg(colors.number)),
            Span::raw("Scroll cursor to left edge"),
        ]),
        Line::from(vec![
            Span::styled("  ze            ", Style::default().fg(colors.number)),
            Span::raw("Scroll cursor to right edge"),
        ]),
```

**Step 2: Run tests**

Run: `cargo fmt && cargo clippy -- -D warnings && cargo test`
Expected: PASS

**Step 3: Commit**

```bash
git add src/ui/help_overlay.rs
git commit -m "docs: add horizontal scroll keybindings to help overlay"
```

---

### Task 8: Final verification

**Step 1: Run full pre-commit checks**

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

Also test crossterm backend:
```bash
cargo clippy --no-default-features --features backend-crossterm -- -D warnings
```

**Step 2: Manual smoke test**

```bash
cargo run -- test_data/large.json  # or any JSON file with long values
```

Test:
- `zh` / `zl` scroll left/right by 1
- `10zl` scroll right by 10
- `zH` / `zL` scroll left/right by half screen
- `zs` / `ze` scroll to edges
- `j` / `k` resets horizontal scroll
- `/` search, `n` navigation resets horizontal scroll
- Line numbers stay pinned during horizontal scroll
