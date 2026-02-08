use jsonquill::editor::mode::EditorMode;
use jsonquill::input::backend::{BackendEvent, BackendKey};
use jsonquill::input::keys::{map_key_event, InputEvent};

#[test]
fn test_visual_mode_key() {
    let event = BackendEvent::Key(BackendKey::Char('v'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::EnterVisualMode);
}

#[test]
fn test_mark_set_key() {
    let event = BackendEvent::Key(BackendKey::Char('m'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::MarkSet);
}

#[test]
fn test_mark_jump_key() {
    let event = BackendEvent::Key(BackendKey::Char('\''));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::MarkJump);
}

#[test]
fn test_jump_backward_key() {
    let event = BackendEvent::Key(BackendKey::Ctrl('o'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::JumpBackward);
}

#[test]
fn test_jump_forward_key() {
    let event = BackendEvent::Key(BackendKey::Ctrl('i'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::JumpForward);
}

#[test]
fn test_repeat_key() {
    let event = BackendEvent::Key(BackendKey::Char('.'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::Repeat);
}
