# Clear Search Info on Non-Search Keys

## Overview

Automatically clear search result information from the status bar when the user presses any key other than search navigation keys (`n` for next result).

## Problem

Currently, once you perform a search with `/` or `?`, the status bar shows search information like `[Search: "query"] Match 2/5` indefinitely. This clutters the status bar even after you've finished navigating search results and moved on to other tasks.

## Design

### User Experience

**Current behavior:**
1. User searches: `/name`
2. Status shows: `NORMAL | file.json [Search: "name"] Match 1/3`
3. User navigates with `j`, `k`, `i`, etc.
4. Status still shows: `NORMAL | file.json [Search: "name"] Match 1/3` ← cluttered

**New behavior:**
1. User searches: `/name`
2. Status shows: `NORMAL | file.json users[0].name [Search: "name"] Match 1/3`
3. User presses `n` - navigates to next match
4. Status still shows: `NORMAL | file.json users[1].name [Search: "name"] Match 2/3` ← active search
5. User presses `j` - moves down (non-search action)
6. Status shows: `NORMAL | file.json users[2]` ← search cleared

### Keys that Preserve Search Info

- `n` - Next search result (active search navigation)

### Keys that Clear Search Info

All other keys:
- Movement: `j`, `k`, `h`, `l`, arrow keys, `gg`, `G`, etc.
- Editing: `i`, `a`, `o`, `dd`, `yy`, `p`, etc.
- Mode switching: `:`, `?`, `/`, `Esc`
- Any other command

### Implementation Approach

**Add `clear_search_results()` method:**
```rust
pub fn clear_search_results(&mut self) {
    self.search_results.clear();
    self.search_index = 0;
    // Keep search_buffer and search_type for potential "repeat search" feature
}
```

**Update input handler:**
In the input handler, after processing any key command except `n`, call `state.clear_search_results()`.

**Status bar automatically updates:**
The status bar already calls `state.search_results_info()` which returns `None` when `search_results` is empty, automatically removing the search display.

## Benefits

1. **Cleaner status bar** - Shows relevant context (current path, mode) instead of stale search info
2. **Intuitive behavior** - Matches user intent: once you move away from search, you're done searching
3. **No new commands needed** - Automatic clearing, no need for `:nohl` or manual clear
4. **Preserves search buffer** - Can potentially add "repeat search" feature later

## Non-Goals

- Preserving search results for later navigation (once you clear, search is done)
- Adding explicit clear command (automatic clearing is sufficient)
- Implementing `N` for previous search (can be added separately)
