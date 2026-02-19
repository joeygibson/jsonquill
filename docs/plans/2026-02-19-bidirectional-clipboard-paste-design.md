# Bidirectional Clipboard Paste

## Problem

When opening an empty/new JSON file, there's no way to paste a JSON document from the system clipboard. The `p`/`P` commands only read from internal registers, and clipboard sync is one-way (yank writes to clipboard, paste never reads from it).

## Design

### Clipboard Fallback on Paste

In `paste_nodes_at_cursor()` and `paste_nodes_before_cursor()`: when using the unnamed register (no `"a` prefix) and it's empty, read from the system clipboard:

1. `arboard::Clipboard::get_text()` to read clipboard text
2. `parse_json()` to parse as JSON
3. On success: use parsed root node as paste content
4. On failure: show error "Clipboard does not contain valid JSON"

### Root Replacement

When pasting into an empty root (object with 0 children or array with 0 elements) and the paste content is a single container node, replace the root entirely instead of inserting under a "pasted" key.

This covers both clipboard paste and internal register paste.

### Scope

- Named register paste (`"ap`) never touches clipboard
- Yank-to-clipboard unchanged
- Non-empty document paste unchanged

### Files Modified

- `src/editor/state.rs`: `paste_nodes_at_cursor()`, `paste_nodes_before_cursor()` (clipboard fallback), `paste_single_node()` (root replacement)
