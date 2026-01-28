# Interactive Theme Picker Design

**Date:** 2026-01-28
**Status:** Approved
**Inspired by:** [whosthere](https://github.com/ramonvermeulen/whosthere) - Ctrl-t theme picker with live preview

## Overview

Add an interactive theme picker popup that appears when the user types `:theme` (without arguments). The picker shows a list of available themes, allows navigation with arrow keys, previews themes live as you move the cursor, and applies the selected theme on Enter or cancels with Esc.

## Goals

- **Discoverability**: Make theme switching more intuitive than typing theme names
- **Live Preview**: See themes applied in real-time as you navigate
- **Non-destructive**: Easy to cancel and return to original theme
- **Consistent UX**: Match the existing help overlay pattern

## User Flow

1. User types `:theme` and presses Enter
2. Popup overlay appears centered on screen
3. Current theme is highlighted with `(current)` label
4. User presses `↑`/`↓` or `j`/`k` to navigate
5. As cursor moves, theme is applied immediately (live preview)
6. User presses `Enter` to apply and close, or `Esc` to revert and close

## Architecture

### State Management Pattern

Use a **state flag pattern** (like help overlay) instead of adding a new `EditorMode`:

```rust
pub struct EditorState {
    // Existing fields
    show_help: bool,

    // New fields for theme picker
    show_theme_picker: bool,
    theme_picker_state: Option<ThemePickerState>,
    current_theme_name: String,  // Track active theme
}
```

**Why not a new mode?**
- Help overlay doesn't use a mode, just a flag
- Simpler implementation, less coupling with mode system
- Theme picker has similar behavior (overlay, Esc to close)

### ThemePickerState Structure

```rust
pub struct ThemePickerState {
    pub themes: Vec<String>,        // All available themes
    pub selected_index: usize,      // Currently highlighted index
    pub original_theme: String,     // Theme when picker opened (for Esc)
    pub current_theme: String,      // Currently applied theme (for UI label)
}

impl ThemePickerState {
    pub fn new(current_theme: String) -> Self {
        let themes = crate::theme::list_builtin_themes();
        let selected_index = themes
            .iter()
            .position(|t| t == &current_theme)
            .unwrap_or(0);

        Self {
            themes,
            selected_index,
            original_theme: current_theme.clone(),
            current_theme,
        }
    }
}
```

## UI Design

### Visual Layout

```
┌─────────────────────────────────────┐
│          Select Theme               │
├─────────────────────────────────────┤
│                                     │
│  > default-dark          (current) │
│    default-light                    │
│    gruvbox-dark                     │
│    nord                             │
│    dracula                          │
│    solarized-dark                   │
│    monokai                          │
│    one-dark                         │
│                                     │
├─────────────────────────────────────┤
│ ↑/↓: Navigate  Enter: Apply  Esc: Cancel │
└─────────────────────────────────────┘
```

### Visual Elements

- **Cursor indicator**: `>` arrow shows selected theme
- **Current label**: `(current)` suffix on active theme
- **Highlight**: Selected line has different background color
- **Sizing**: ~40% screen width, fit content height (max 60% screen)
- **Centering**: Use `centered_rect()` helper from help overlay
- **Border color**: `colors.info` (consistent with help)

### Rendering Module

New file: `src/ui/theme_picker.rs`

```rust
pub fn render_theme_picker(
    f: &mut Frame,
    state: &ThemePickerState,
    colors: &ThemeColors
) {
    let area = centered_rect(40, 60, f.area());

    // Clear background
    f.render_widget(Clear, area);

    // Build theme list with highlighting
    let mut lines = vec![];
    for (idx, theme) in state.themes.iter().enumerate() {
        let is_selected = idx == state.selected_index;
        let is_current = theme == &state.current_theme;

        let cursor = if is_selected { "> " } else { "  " };
        let label = if is_current { " (current)" } else { "" };

        let style = if is_selected {
            Style::default().bg(colors.cursor_line_bg)
        } else {
            Style::default()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{}{}{}", cursor, theme, label), style)
        ]));
    }

    // Render with border and footer
    // ...
}
```

### Integration in UI::render()

```rust
pub fn render(&mut self, f: &mut Frame, state: &EditorState) {
    // Render main tree view
    render_tree_view(f, main_area, state, &self.theme.colors);

    // Render status line
    render_status_line(f, status_area, state, &self.theme.colors);

    // Render message area
    render_message_area(f, message_area, state, &self.theme.colors);

    // Overlays (highest priority)
    if state.show_help() {
        render_help_overlay(f, &self.theme.colors, state.help_scroll());
    }

    if state.show_theme_picker() {
        if let Some(picker_state) = state.theme_picker_state() {
            render_theme_picker(f, picker_state, &self.theme.colors);
        }
    }
}
```

## Input Handling

### Event Priority

When `show_theme_picker` is true, theme picker gets first priority:

```rust
pub fn handle_event(&mut self, event: Event, state: &mut EditorState) -> Result<bool> {
    // Theme picker has highest priority
    if state.show_theme_picker() {
        return self.handle_theme_picker_event(event, state);
    }

    // Then help overlay
    if state.show_help() {
        return self.handle_help_event(event, state);
    }

    // Then normal mode handling
    // ...
}
```

### Key Bindings

```rust
fn handle_theme_picker_event(
    &mut self,
    event: Event,
    state: &mut EditorState
) -> Result<bool> {
    match event {
        Event::Key(Key::Up | Key::Char('k')) => {
            state.theme_picker_previous();
            Ok(false)
        }
        Event::Key(Key::Down | Key::Char('j')) => {
            state.theme_picker_next();
            Ok(false)
        }
        Event::Key(Key::Char('\n')) => {
            state.theme_picker_apply();
            Ok(false)
        }
        Event::Key(Key::Esc | Key::Char('q')) => {
            state.theme_picker_cancel();
            Ok(false)
        }
        _ => Ok(false)  // Ignore other keys
    }
}
```

## EditorState Methods

### Opening the Picker

```rust
pub fn open_theme_picker(&mut self) {
    let current = self.current_theme_name.clone();
    self.theme_picker_state = Some(ThemePickerState::new(current));
    self.show_theme_picker = true;
    self.clear_message();
}
```

### Navigation with Live Preview

```rust
pub fn theme_picker_previous(&mut self) {
    if let Some(picker) = &mut self.theme_picker_state {
        if picker.selected_index > 0 {
            picker.selected_index -= 1;
            let theme = picker.themes[picker.selected_index].clone();
            self.preview_theme(&theme);
        }
    }
}

pub fn theme_picker_next(&mut self) {
    if let Some(picker) = &mut self.theme_picker_state {
        if picker.selected_index < picker.themes.len() - 1 {
            picker.selected_index += 1;
            let theme = picker.themes[picker.selected_index].clone();
            self.preview_theme(&theme);
        }
    }
}

fn preview_theme(&mut self, theme_name: &str) {
    self.request_theme_change(theme_name.to_string());
    if let Some(picker) = &mut self.theme_picker_state {
        picker.current_theme = theme_name.to_string();
    }
}
```

### Apply and Cancel

```rust
pub fn theme_picker_apply(&mut self) {
    if let Some(picker) = &self.theme_picker_state {
        // Theme already applied via preview, just update state
        self.current_theme_name = picker.current_theme.clone();
    }
    self.theme_picker_state = None;
    self.show_theme_picker = false;
}

pub fn theme_picker_cancel(&mut self) {
    if let Some(picker) = &self.theme_picker_state {
        // Revert to original theme
        self.request_theme_change(picker.original_theme.clone());
        self.current_theme_name = picker.original_theme.clone();
    }
    self.theme_picker_state = None;
    self.show_theme_picker = false;
}
```

## Theme Name Tracking

Currently, `EditorState` doesn't track the current theme name. We need to add this:

```rust
pub struct EditorState {
    // ... existing fields
    current_theme_name: String,  // NEW - track active theme
}

// Update constructor
pub fn new(tree: JsonTree, initial_theme_name: String) -> Self {
    Self {
        // ... existing initialization
        current_theme_name: initial_theme_name,
        show_theme_picker: false,
        theme_picker_state: None,
    }
}

// Update theme change to track name
pub fn request_theme_change(&mut self, theme_name: String) {
    self.pending_theme_change = Some(theme_name.clone());
    self.current_theme_name = theme_name;
}
```

### Initialization in main.rs

```rust
// Load config and theme
let config = Config::load()?;
let theme = get_builtin_theme(&config.theme)
    .unwrap_or_else(|| get_builtin_theme("default-dark").unwrap());

// Pass theme name to EditorState
let theme_name = config.theme.clone();
let mut state = EditorState::new(tree, theme_name);
let mut ui = UI::new(theme);
```

## Command Integration

Update the `:theme` command handler in `src/input/handler.rs`:

```rust
// Current behavior: list themes in message area
if command == "theme" {
    use crate::theme::list_builtin_themes;
    let themes = list_builtin_themes();
    let theme_list = themes.join(", ");
    state.set_message(
        format!("Available themes: {}", theme_list),
        MessageLevel::Info,
    );
    return Ok(false);
}

// New behavior: open theme picker
if command == "theme" {
    state.open_theme_picker();
    return Ok(false);
}
```

The existing `:theme <name>` behavior (setting theme directly) remains unchanged.

## Implementation Plan

### Phase 1: State Management
1. Add `ThemePickerState` struct to `src/editor/state.rs`
2. Add fields to `EditorState`: `show_theme_picker`, `theme_picker_state`, `current_theme_name`
3. Add methods: `open_theme_picker()`, `theme_picker_next()`, `theme_picker_previous()`, `theme_picker_apply()`, `theme_picker_cancel()`
4. Update `EditorState::new()` to accept `initial_theme_name` parameter
5. Update `request_theme_change()` to track `current_theme_name`

### Phase 2: Rendering
1. Create `src/ui/theme_picker.rs`
2. Implement `render_theme_picker()` function
3. Use `centered_rect()` pattern from help overlay
4. Style theme list with cursor, current label, highlighting
5. Add footer with key hints
6. Export module in `src/ui/mod.rs`
7. Integrate rendering in `UI::render()` after help overlay

### Phase 3: Input Handling
1. Add `handle_theme_picker_event()` to `InputHandler`
2. Update `handle_event()` to check `show_theme_picker` first
3. Handle arrow keys, j/k, Enter, Esc
4. Update `:theme` command to call `open_theme_picker()`

### Phase 4: Main Integration
1. Update `main.rs` to pass theme name to `EditorState::new()`
2. Ensure theme name comes from config or default

### Phase 5: Testing
1. Manual testing: `:theme` opens picker
2. Verify live preview works on navigation
3. Test Esc reverts to original theme
4. Test Enter applies selected theme
5. Test boundary cases (first/last theme navigation)

## Testing Strategy

**Manual Test Cases:**
1. Open picker with `:theme` - should show current theme highlighted
2. Navigate up/down - theme should change immediately
3. Press Esc - should revert to original theme and close
4. Press Enter - should keep selected theme and close
5. Navigate beyond boundaries - should stop at first/last theme
6. Test with different starting themes (from config)

**Edge Cases:**
- Empty theme list (shouldn't happen, but handle gracefully)
- Invalid current theme in config (picker should still open)
- Rapid navigation (no UI glitches)

## Future Enhancements (Out of Scope)

- Mouse support for clicking themes
- Search/filter themes by typing
- Theme preview samples (show colors without full apply)
- Custom theme support
- Keyboard shortcut (Ctrl-t) to open picker directly from Normal mode

## Files Modified

1. `src/editor/state.rs` - Add picker state and methods
2. `src/ui/mod.rs` - Export theme_picker module, integrate rendering
3. `src/ui/theme_picker.rs` - NEW - Rendering logic
4. `src/input/handler.rs` - Add event handling, update `:theme` command
5. `src/main.rs` - Pass theme name to EditorState
6. `CLAUDE.md` - Document new `:theme` behavior (if applicable)
7. `README.md` - Update commands section (if applicable)

## Success Criteria

- ✅ Typing `:theme` opens centered popup overlay
- ✅ Arrow keys and j/k navigate theme list
- ✅ Theme changes are previewed live as cursor moves
- ✅ Enter applies selected theme and closes picker
- ✅ Esc reverts to original theme and closes picker
- ✅ Current theme is indicated with `(current)` label
- ✅ Selected theme is highlighted with `>` cursor
- ✅ No mode changes required (state flag pattern works)
- ✅ Consistent with help overlay UX
