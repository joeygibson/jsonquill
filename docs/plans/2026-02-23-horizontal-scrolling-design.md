# Horizontal Scrolling Design

## Problem

Long values (deep nesting, large strings, collapsed previews) overflow the terminal edge with no way to see the clipped content. Search results in clipped areas are invisible.

## Solution

A global `horizontal_offset: usize` on `EditorState`, applied uniformly to all rendered lines during `render_tree_view()`. Line numbers are pinned; everything else (indentation, indicators, keys, values) shifts left.

Global offset (not per-line) matches vim behavior, keeps state simple, and preserves vertical tree alignment.

## Keybindings

| Key | Action |
|-----|--------|
| `zh` | Scroll left 1 column |
| `zl` | Scroll right 1 column |
| `zH` | Scroll left half-screen |
| `zL` | Scroll right half-screen |
| `zs` | Scroll so cursor line's content starts at left edge |
| `ze` | Scroll so cursor line's content ends at right edge |

Count prefixes supported (e.g., `10zl` scrolls right 10 columns).

## Auto-Reset

Horizontal offset resets to 0 on any vertical cursor movement (j/k/gg/G/search/marks/jumps/etc).

## Search Auto-Scroll

When navigating to a search result (`n`, `/` confirm), calculate the display width of the matched line. If the match text falls beyond `horizontal_offset + viewport_width`, auto-scroll horizontally to reveal it. This happens after vertical scroll adjustment.

## Rendering

In `render_tree_view()`:
1. Build the full line spans as today (line number separate)
2. Calculate total character width of content spans (excluding line number)
3. Skip `horizontal_offset` columns from the content spans
4. Render only what fits in the remaining viewport width

Span splitting: iterate through spans, consuming characters until `horizontal_offset` is reached, then render the remainder. This preserves styling across the split boundary.

## State Changes

- `EditorState`: add `horizontal_offset: usize` field
- Reset `horizontal_offset = 0` in vertical movement methods
- `cursor_position()`: return actual column for `zs`/`ze` calculation

## Edge Cases

- `horizontal_offset` clamped: can't scroll past the longest visible line
- Empty/short lines with large offset: render as blank (consistent with vim)
- Insert mode: auto-scroll to keep edit cursor visible
