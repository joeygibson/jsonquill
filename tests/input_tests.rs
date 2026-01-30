use jsonquill::editor::mode::EditorMode;
use jsonquill::input::keys::{map_key_event, InputEvent};
use termion::event::{Event, Key};

#[test]
fn test_visual_mode_key() {
    let event = Event::Key(Key::Char('v'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::EnterVisualMode);
}

#[test]
fn test_mark_set_key() {
    let event = Event::Key(Key::Char('m'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::MarkSet);
}

#[test]
fn test_mark_jump_key() {
    let event = Event::Key(Key::Char('\''));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::MarkJump);
}

#[test]
fn test_jump_backward_key() {
    let event = Event::Key(Key::Ctrl('o'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::JumpBackward);
}

#[test]
fn test_jump_forward_key() {
    let event = Event::Key(Key::Ctrl('i'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::JumpForward);
}

#[test]
fn test_repeat_key() {
    let event = Event::Key(Key::Char('.'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::Repeat);
}
