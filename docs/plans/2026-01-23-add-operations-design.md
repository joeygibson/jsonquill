# Add Operations Design

**Date:** 2026-01-23
**Feature:** Add scalar values with `a` command
**Status:** Ready for implementation

## Overview

Add support for inserting new scalar values (strings, numbers, booleans, null) into JSON arrays and objects using the `a` key, similar to vim's append command. This addresses the "No add operations" TODO item.

## User Interaction Flow

### Adding to Arrays

1. User navigates to any element in an array (e.g., cursor on `1` in `[1, 2, 3]`)
2. User presses `a`
3. Editor creates an empty string `""` as a new array element after the current position
4. Editor enters Insert mode with the new element selected
5. User types a value (e.g., `42` or `hello` or `true`)
6. User presses Enter to commit
7. Editor parses the value: detects numbers, booleans (`true`/`false`), `null`, or treats as string
8. Cursor moves to the newly created element

### Adding to Objects

1. User navigates to any field in an object (e.g., cursor on `"age": 30`)
2. User presses `a`
3. Editor shows a prompt: `Key: ` (similar to search/command prompt)
4. User types the key name (e.g., `email`) and presses Enter
5. Editor creates `"email": ""` after the current field
6. Editor enters Insert mode with the empty string value selected
7. User types a value, presses Enter to commit
8. Value is parsed and cursor moves to the new field

### Edge Cases

- **Root scalar:** Pressing `a` when cursor is on root scalar shows error "Cannot add sibling to root node"
- **Root container:** If root is object/array, add to it as normal
- **Scalar inside container:** Add sibling after it
- **Duplicate keys:** Allowed (JSON technically permits it, though not recommended)
- **Canceling:** Esc during key prompt or insert mode cancels and doesn't create the node

### Arrays of Objects

**Phase 1 (Initial):** Pressing `a` inside an array of objects creates an empty string `""`, even if siblings are objects. Simple and consistent rule.

**Phase 2 (Future):** Detect sibling types and create matching empties (e.g., create `{}` when siblings are objects, then prompt for first key).

**Phase 3 (Implemented):** Add dedicated commands like `o` (add object), `A` (add array) for explicit type creation.

## State Management

### New Editor State Fields

Add to `EditorState`:

```rust
add_mode_stage: AddModeStage,
add_key_buffer: String,
add_insertion_point: Option<Vec<usize>>, // path where we're inserting
```

### Add Mode Stage Enum

```rust
pub enum AddModeStage {
    None,              // Not in add mode
    AwaitingKey,       // Pressed 'a' in object, waiting for key input
    AwaitingValue,     // Key entered or skipped (arrays), waiting for value
}
```

### Mode Transitions

The `a` command keeps editor in Normal mode but sets internal `add_mode_stage`:

**Array flow:**
```
Normal mode + 'a' press in array
  → add_mode_stage = AwaitingValue
  → enter Insert mode with edit buffer
```

**Object flow:**
```
Normal mode + 'a' press in object
  → add_mode_stage = AwaitingKey
  → show "Key: " prompt
  → user types, presses Enter
  → add_mode_stage = AwaitingValue
  → enter Insert mode with edit buffer
```

### Committing the Add

When user presses Enter in Insert mode while `add_mode_stage != None`:

1. Parse the edit buffer value (number/boolean/null/string)
2. Create the JsonNode
3. Call `tree.insert_node_in_array()` or `tree.insert_node_in_object()`
4. Clear `add_mode_stage` and `add_key_buffer`
5. Mark dirty and create undo checkpoint
6. Move cursor to newly created node

### Canceling

Esc during key prompt or insert mode:
- Clear `add_mode_stage`
- Clear `add_key_buffer`
- Clear `add_insertion_point`
- Return to Normal mode
- Don't create any node

## Implementation Details

### Files to Modify

**src/editor/state.rs:**
- Add fields: `add_mode_stage`, `add_key_buffer`, `add_insertion_point`
- Add `AddModeStage` enum
- Add methods:
  - `start_add_operation()` - determines if parent is object/array, sets stage
  - `add_mode_stage()` - getter for current stage
  - `add_key_buffer()`, `add_key_buffer_mut()` - access key buffer
  - `push_to_add_key_buffer(char)` - append character to key
  - `pop_from_add_key_buffer()` - remove last character from key
  - `commit_add_operation()` - creates node, inserts it, clears state
  - `cancel_add_operation()` - clears state without inserting

**src/input/handler.rs:**
- Add `InputEvent::Add` for the `a` key
- In Normal mode handler: when `Add` event, call `state.start_add_operation()`
- When in `AwaitingKey` stage: intercept character/backspace/enter events to build key buffer
- When Enter pressed in `AwaitingKey`: transition to `AwaitingValue` and enter Insert mode
- When Enter pressed in Insert mode with `AwaitingValue`: call `state.commit_add_operation()`
- When Esc pressed: call `state.cancel_add_operation()`

**src/input/keys.rs:**
- Map `Key::Char('a')` in Normal mode to `InputEvent::Add`

**src/ui/mod.rs:**
- When rendering, check if `add_mode_stage == AwaitingKey`
- Render "Key: {buffer}" prompt similar to command/search prompt

### Value Parsing Logic

Parse edit buffer when committing:

```rust
fn parse_scalar_value(input: &str) -> JsonValue {
    let trimmed = input.trim();

    // Try boolean
    if trimmed == "true" {
        return JsonValue::Boolean(true);
    }
    if trimmed == "false" {
        return JsonValue::Boolean(false);
    }

    // Try null
    if trimmed == "null" {
        return JsonValue::Null;
    }

    // Try number
    if let Ok(num) = trimmed.parse::<f64>() {
        return JsonValue::Number(num);
    }

    // Default to string (use original input, not trimmed)
    JsonValue::String(input.to_string())
}
```

**Parsing behavior:**
- `"true"` → `Boolean(true)`
- `"false"` → `Boolean(false)`
- `"null"` → `Null`
- `"42"` → `Number(42.0)`
- `"-1.5"` → `Number(-1.5)`
- `"0"` → `Number(0.0)`
- `"hello"` → `String("hello")`
- `"123abc"` → `String("123abc")` (not a valid number)
- `""` → `String("")` (empty string)

### Error Handling

1. **Empty key in objects:**
   - If Enter pressed with empty `add_key_buffer`
   - Show error: "Key cannot be empty"
   - Stay in `AwaitingKey` stage

2. **Cannot add sibling to root scalar:**
   - If tree root is a scalar and user presses `a`
   - Show error: "Cannot add sibling to root node"
   - Don't change mode

3. **Invalid cursor position:**
   - If cursor path not found in tree
   - Show error: "Invalid cursor position"
   - Cancel operation

4. **Parent is not a container:**
   - If trying to add after a node whose parent isn't object/array
   - Show error: "Parent is not a container"
   - Cancel operation

5. **Insertion failures:**
   - If `insert_node_in_array()` or `insert_node_in_object()` returns error
   - Display error message
   - Cancel operation without modifying tree

**Success messages:**
- "Added field 'email'" (for objects, info level)
- "Added element" (for arrays, info level)

**All errors:**
- Display in message area (error level)
- Don't modify tree or enter inconsistent state

## Testing Strategy

### Unit Tests (tests/editor_tests.rs)

1. **test_start_add_in_array**
   - Cursor on array element, press `a`
   - Verify `add_mode_stage` is `AwaitingValue`
   - Verify Insert mode entered

2. **test_start_add_in_object**
   - Cursor on object field, press `a`
   - Verify `add_mode_stage` is `AwaitingKey`
   - Verify key prompt shown

3. **test_add_key_buffer_operations**
   - Test `push_to_add_key_buffer()` with multiple characters
   - Test `pop_from_add_key_buffer()`
   - Verify buffer contents correct

4. **test_cancel_add_during_key_entry**
   - Press Esc during key prompt
   - Verify `add_mode_stage` cleared to `None`
   - Verify `add_key_buffer` cleared
   - Verify no node created

5. **test_cancel_add_during_value_entry**
   - Press Esc during value insert mode
   - Verify state cleared
   - Verify no node created

### Integration Tests (tests/integration_editing.rs)

1. **test_add_string_to_array**
   - Full flow: cursor on array element, `a`, type "hello", Enter
   - Verify new element exists at correct position
   - Verify it's a String("hello")
   - Verify cursor moved to new element

2. **test_add_number_to_array**
   - Type "42", press Enter
   - Verify parsed as Number(42.0), not String("42")

3. **test_add_boolean_to_array**
   - Type "true", press Enter
   - Verify parsed as Boolean(true)

4. **test_add_null_to_array**
   - Type "null", press Enter
   - Verify parsed as Null

5. **test_add_field_to_object**
   - Full flow: cursor on object field, `a`
   - Type key "email", Enter
   - Type value "test@example.com", Enter
   - Verify new field exists with correct key and value
   - Verify cursor moved to new field

6. **test_add_with_empty_key_fails**
   - Try to add to object with empty key
   - Press Enter without typing
   - Verify error message shown
   - Verify still in `AwaitingKey` stage
   - Verify no node created

7. **test_add_to_root_scalar_fails**
   - Tree root is a scalar (number/string/etc)
   - Press `a`
   - Verify error message shown
   - Verify mode unchanged

8. **test_add_creates_undo_checkpoint**
   - Add element to array
   - Press `u` to undo
   - Verify element removed

9. **test_cursor_moves_to_new_node**
   - After adding element
   - Verify cursor path points to newly created node

### Value Parsing Tests

Separate unit test for `parse_scalar_value()`:

- `"0"` → Number(0.0)
- `"-1.5"` → Number(-1.5)
- `"123abc"` → String("123abc")
- `""` → String("")
- `"  true  "` → Boolean(true) (trimmed)
- `"  null  "` → Null (trimmed)
- `"True"` → String("True") (case sensitive)
- `"NULL"` → String("NULL") (case sensitive)

## Future Enhancements

### Phase 2: Smart Type Detection

When pressing `a` in an array, detect sibling types:
- If all siblings are objects: create `{}` and prompt for first key
- If all siblings are arrays: create `[]`
- If mixed or all scalars: create empty string (current behavior)

### Phase 3: Explicit Type Commands (Implemented)

Add dedicated commands for specific types:
- `o` - add object (creates `{}`) - **IMPLEMENTED**
- `A` - add array (creates `[]`) - **IMPLEMENTED**
- `a` - add scalar (auto-parsed: number/boolean/null/string) - **IMPLEMENTED**
- Future: could add `as` for explicit string, `an` for explicit null if needed

This gives users explicit control when needed while keeping `a` smart.

### Phase 4: Multi-field Object Creation

For objects, allow adding multiple fields in one operation:
- After first field created, show prompt "Add another field? (y/n)"
- If yes, loop back to key prompt
- If no, commit and return to Normal mode

## Open Questions

None - design validated and ready for implementation.

## Summary

This design provides a clean, vim-like interface for adding scalar values to JSON structures. It starts simple (string creation with parsing) and has a clear path to future enhancements (smart type detection, explicit type commands). The two-stage flow for objects (key then value) matches JSON semantics, while the single-stage flow for arrays keeps it fast. Error handling is comprehensive, and the testing strategy ensures correctness.
