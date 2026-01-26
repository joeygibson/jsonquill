# Current Path in Status Bar

## Overview

Add the current node's path to the status bar to help users understand their location in the JSON document structure.

## Design

### Display Format

The path appears between the filename/dirty indicator and search results (if any):

```
NORMAL | data.json address.people[0].firstName [+]     5/20
```

### Path Format

- **Object keys**: Dot notation (e.g., `users.profile.name`)
- **Array indices**: Bracket notation (e.g., `users[0].addresses[2]`)
- **Combined**: Mixed notation (e.g., `store.books[0].author`)
- **Root node**: Empty string (path not shown)

### Example Status Lines

```
NORMAL | data.json users[0].name [+]                15/50
NORMAL | config.json database.port                   3/8
NORMAL | api.json                                    1/1    (root node)
NORMAL | data.json users[2] [Search: "admin"] Match 1/3  20/50
```

## Implementation Plan

### 1. Add Public Path Method to EditorState

The codebase already has path generation logic for yank operations. We need a method that:
- Returns the path to the current cursor position
- Uses dot notation format
- Returns empty string for root node

### 2. Update Status Line Rendering

In `src/ui/status_line.rs`:
- Call the path method after getting filename and dirty indicator
- Insert path into status line string between dirty indicator and search results
- Add appropriate spacing

### 3. Testing

Add tests to verify:
- Object key paths display correctly
- Array index paths display correctly
- Mixed paths display correctly
- Root node shows no path
- Path doesn't break existing status line elements

## Non-Goals

- Path truncation for narrow terminals (may add later if needed)
- Alternative path formats (JSONPath, jq, etc.)
- Clickable/interactive path elements
