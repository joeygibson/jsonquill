use jsonquill::document::node::JsonValue;
use jsonquill::editor::repeat::RepeatableCommand;

#[test]
fn test_repeatable_command_delete() {
    let cmd = RepeatableCommand::Delete { count: 3 };

    match cmd {
        RepeatableCommand::Delete { count } => assert_eq!(count, 3),
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn test_repeatable_command_yank() {
    let cmd = RepeatableCommand::Yank { count: 5 };

    match cmd {
        RepeatableCommand::Yank { count } => assert_eq!(count, 5),
        _ => panic!("Expected Yank command"),
    }
}

#[test]
fn test_repeatable_command_add() {
    let cmd = RepeatableCommand::Add {
        value: JsonValue::String("test".to_string()),
        key: Some("mykey".to_string()),
    };

    match cmd {
        RepeatableCommand::Add { value, key } => {
            assert_eq!(value, JsonValue::String("test".to_string()));
            assert_eq!(key, Some("mykey".to_string()));
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_repeatable_command_clone() {
    let cmd = RepeatableCommand::Delete { count: 2 };
    let cloned = cmd.clone();

    match cloned {
        RepeatableCommand::Delete { count } => assert_eq!(count, 2),
        _ => panic!("Expected Delete command"),
    }
}
