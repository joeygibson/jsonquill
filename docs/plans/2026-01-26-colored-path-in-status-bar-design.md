# Colored Path in Status Bar

## Overview

Add color highlighting to the path in the status bar to make it stand out more, using the theme's key color for semantic consistency.

## Design

### Color Choice

Use `colors.key` for the path display because:
- **Semantic match**: Paths are composed of keys and array indices
- **Theme consistency**: The key color is already optimized for visibility across all 8 themes
- **User intuition**: Users already associate this color with navigation structure from the tree view
- **Balanced visibility**: Not too loud (like cursor), not too subtle

### Current Implementation

The status line currently renders as a single `Span` with uniform styling:
```rust
let content = format!("{}{}{}", left, " ".repeat(padding), right);
let line = Line::from(Span::styled(
    content,
    Style::default()
        .fg(colors.status_line_fg)
        .bg(colors.status_line_bg),
));
```

### Proposed Implementation

Replace the single-span approach with multiple styled spans:

```rust
let line = Line::from(vec![
    // Mode
    Span::styled(mode_text, Style::default().fg(colors.status_line_fg).bg(colors.status_line_bg)),
    // Separator
    Span::styled(" | ", Style::default().fg(colors.status_line_fg).bg(colors.status_line_bg)),
    // Filename
    Span::styled(filename, Style::default().fg(colors.status_line_fg).bg(colors.status_line_bg)),
    // Path (colored with key color)
    Span::styled(path_display, Style::default().fg(colors.key).bg(colors.status_line_bg)),
    // Dirty indicator
    Span::styled(dirty_indicator, Style::default().fg(colors.status_line_fg).bg(colors.status_line_bg)),
    // Search results
    Span::styled(search_info, Style::default().fg(colors.status_line_fg).bg(colors.status_line_bg)),
    // Padding
    Span::styled(" ".repeat(padding), Style::default().bg(colors.status_line_bg)),
    // Position
    Span::styled(position, Style::default().fg(colors.status_line_fg).bg(colors.status_line_bg)),
]);
```

### Benefits

1. **Visual hierarchy**: Path stands out without being overwhelming
2. **Cross-theme compatibility**: Works across all 8 built-in themes
3. **Semantic clarity**: Color reinforces that path shows key/index navigation
4. **No new theme colors needed**: Uses existing `key` color

## Implementation Plan

1. Refactor status line rendering from single span to multi-span
2. Apply `colors.key` to path span
3. Update tests to verify multi-span structure
4. Manual verification across multiple themes

## Non-Goals

- Custom path color (uses existing key color)
- Coloring other status bar elements
- Per-segment path coloring (e.g., different colors for keys vs indices)
