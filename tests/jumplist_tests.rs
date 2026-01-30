// tests/jumplist_tests.rs
use jsonquill::editor::jumplist::JumpList;

#[test]
fn test_jumplist_creation() {
    let jumplist = JumpList::new(100);
    assert_eq!(jumplist.len(), 0);
    assert_eq!(jumplist.current_position(), 0);
}

#[test]
fn test_record_and_backward() {
    let mut jumplist = JumpList::new(100);
    jumplist.record_jump(vec![0]);
    jumplist.record_jump(vec![1]);
    jumplist.record_jump(vec![2]);

    assert_eq!(jumplist.len(), 3);
    assert_eq!(jumplist.jump_backward(), Some(vec![1]));
    assert_eq!(jumplist.jump_backward(), Some(vec![0]));
    assert_eq!(jumplist.jump_backward(), None); // At oldest
}

#[test]
fn test_forward_navigation() {
    let mut jumplist = JumpList::new(100);
    jumplist.record_jump(vec![0]);
    jumplist.record_jump(vec![1]);
    jumplist.record_jump(vec![2]);

    jumplist.jump_backward();
    jumplist.jump_backward();

    assert_eq!(jumplist.jump_forward(), Some(vec![1]));
    assert_eq!(jumplist.jump_forward(), Some(vec![2]));
    assert_eq!(jumplist.jump_forward(), None); // At newest
}

#[test]
fn test_truncate_on_new_jump() {
    let mut jumplist = JumpList::new(100);
    jumplist.record_jump(vec![0]);
    jumplist.record_jump(vec![1]);
    jumplist.record_jump(vec![2]);

    // Jump back then record new jump
    jumplist.jump_backward();
    jumplist.record_jump(vec![3]);

    // Should truncate vec![2] and add vec![3]
    assert_eq!(jumplist.len(), 3);
    assert_eq!(jumplist.jump_backward(), Some(vec![1]));
}
