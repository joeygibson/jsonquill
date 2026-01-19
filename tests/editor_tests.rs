use jeditor::editor::mode::EditorMode;

#[test]
fn test_mode_starts_normal() {
    let mode = EditorMode::Normal;
    assert!(matches!(mode, EditorMode::Normal));
}

#[test]
fn test_mode_display() {
    assert_eq!(format!("{}", EditorMode::Normal), "NORMAL");
    assert_eq!(format!("{}", EditorMode::Insert), "INSERT");
    assert_eq!(format!("{}", EditorMode::Command), "COMMAND");
}

#[test]
fn test_mode_default() {
    let mode = EditorMode::default();
    assert_eq!(mode, EditorMode::Normal);
}

#[test]
fn test_mode_equality() {
    let mode1 = EditorMode::Normal;
    let mode2 = EditorMode::Normal;
    let mode3 = EditorMode::Insert;

    assert_eq!(mode1, mode2);
    assert_ne!(mode1, mode3);
    assert_ne!(mode2, mode3);
}

#[test]
fn test_mode_clone() {
    let mode = EditorMode::Insert;
    let cloned = mode.clone();
    assert_eq!(mode, cloned);
}

#[test]
fn test_mode_copy() {
    let mode = EditorMode::Command;
    let copied = mode;
    assert_eq!(mode, copied);
    // If mode wasn't Copy, this would have moved it
    assert_eq!(mode, EditorMode::Command);
}

#[test]
fn test_mode_debug() {
    let mode = EditorMode::Normal;
    let debug_str = format!("{:?}", mode);
    assert_eq!(debug_str, "Normal");

    let mode = EditorMode::Insert;
    let debug_str = format!("{:?}", mode);
    assert_eq!(debug_str, "Insert");

    let mode = EditorMode::Command;
    let debug_str = format!("{:?}", mode);
    assert_eq!(debug_str, "Command");
}

#[test]
fn test_all_mode_variants() {
    // Ensure all variants can be constructed
    let normal = EditorMode::Normal;
    let insert = EditorMode::Insert;
    let command = EditorMode::Command;

    // Ensure they are all different
    assert_ne!(normal, insert);
    assert_ne!(normal, command);
    assert_ne!(insert, command);

    // Ensure they all display correctly
    assert_eq!(format!("{}", normal), "NORMAL");
    assert_eq!(format!("{}", insert), "INSERT");
    assert_eq!(format!("{}", command), "COMMAND");
}
