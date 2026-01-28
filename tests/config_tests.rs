use jsonquill::config::Config;

#[test]
fn test_config_defaults() {
    let config = Config::default();

    assert_eq!(config.theme, "default-dark");
    assert_eq!(config.indent_size, 2);
    assert!(!config.auto_save);
    assert_eq!(config.validation_mode, "strict");
    assert!(!config.create_backup);
}

#[test]
fn test_all_default_values() {
    let config = Config::default();

    // Display settings
    assert_eq!(config.theme, "default-dark");
    assert_eq!(config.indent_size, 2);
    assert!(config.show_line_numbers);

    // Behavior settings
    assert!(!config.auto_save);
    assert!(!config.create_backup);
    assert!(config.sync_unnamed_register);

    // Validation settings
    assert_eq!(config.validation_mode, "strict");

    // Performance settings
    assert_eq!(config.undo_limit, 50);
    assert_eq!(config.lazy_load_threshold, 104_857_600); // 100MB

    // Input settings
    assert!(config.enable_mouse);
}

#[test]
fn test_custom_config() {
    let config = Config {
        theme: "gruvbox".to_string(),
        indent_size: 4,
        show_line_numbers: false,
        auto_save: true,
        validation_mode: "permissive".to_string(),
        create_backup: true,
        undo_limit: 500,
        sync_unnamed_register: false,
        lazy_load_threshold: 52_428_800, // 50MB
        enable_mouse: false,
        relative_line_numbers: false,
        preserve_formatting: true,
    };

    assert_eq!(config.theme, "gruvbox");
    assert_eq!(config.indent_size, 4);
    assert!(!config.show_line_numbers);
    assert!(config.auto_save);
    assert_eq!(config.validation_mode, "permissive");
    assert!(config.create_backup);
    assert_eq!(config.undo_limit, 500);
    assert!(!config.sync_unnamed_register);
    assert_eq!(config.lazy_load_threshold, 52_428_800);
    assert!(!config.enable_mouse);
}

#[test]
fn test_serialize_default_config() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).expect("Failed to serialize config");

    assert!(toml_str.contains("theme = \"default-dark\""));
    assert!(toml_str.contains("indent_size = 2"));
    assert!(toml_str.contains("show_line_numbers = true"));
    assert!(toml_str.contains("auto_save = false"));
    assert!(toml_str.contains("validation_mode = \"strict\""));
    assert!(toml_str.contains("create_backup = false"));
    assert!(toml_str.contains("undo_limit = 50"));
    assert!(toml_str.contains("sync_unnamed_register = true"));
    assert!(toml_str.contains("lazy_load_threshold = 104857600"));
    assert!(toml_str.contains("enable_mouse = true"));
}

#[test]
fn test_deserialize_full_config() {
    let toml_str = r#"
        theme = "monokai"
        indent_size = 4
        show_line_numbers = false
        auto_save = true
        validation_mode = "permissive"
        create_backup = true
        undo_limit = 500
        sync_unnamed_register = false
        lazy_load_threshold = 52428800
        enable_mouse = false
    "#;

    let config: Config = toml::from_str(toml_str).expect("Failed to deserialize config");

    assert_eq!(config.theme, "monokai");
    assert_eq!(config.indent_size, 4);
    assert!(!config.show_line_numbers);
    assert!(config.auto_save);
    assert_eq!(config.validation_mode, "permissive");
    assert!(config.create_backup);
    assert_eq!(config.undo_limit, 500);
    assert!(!config.sync_unnamed_register);
    assert_eq!(config.lazy_load_threshold, 52_428_800);
    assert!(!config.enable_mouse);
}

#[test]
fn test_deserialize_partial_config() {
    // Only specify some fields; others should use defaults
    let toml_str = r#"
        theme = "solarized"
        indent_size = 4
    "#;

    let config: Config = toml::from_str(toml_str).expect("Failed to deserialize config");

    // Custom values
    assert_eq!(config.theme, "solarized");
    assert_eq!(config.indent_size, 4);

    // Default values
    assert!(config.show_line_numbers);
    assert!(!config.auto_save);
    assert_eq!(config.validation_mode, "strict");
    assert!(!config.create_backup);
    assert_eq!(config.undo_limit, 50);
    assert!(config.sync_unnamed_register);
    assert_eq!(config.lazy_load_threshold, 104_857_600);
    assert!(config.enable_mouse);
}

#[test]
fn test_deserialize_empty_config() {
    // Empty TOML should use all defaults
    let toml_str = "";

    let config: Config = toml::from_str(toml_str).expect("Failed to deserialize config");

    assert_eq!(config.theme, "default-dark");
    assert_eq!(config.indent_size, 2);
    assert!(config.show_line_numbers);
    assert!(!config.auto_save);
    assert_eq!(config.validation_mode, "strict");
    assert!(!config.create_backup);
    assert_eq!(config.undo_limit, 50);
    assert!(config.sync_unnamed_register);
    assert_eq!(config.lazy_load_threshold, 104_857_600);
    assert!(config.enable_mouse);
}

#[test]
fn test_roundtrip_serialization() {
    let original = Config {
        theme: "nord".to_string(),
        indent_size: 8,
        show_line_numbers: false,
        auto_save: true,
        validation_mode: "none".to_string(),
        create_backup: true,
        undo_limit: 2000,
        sync_unnamed_register: false,
        lazy_load_threshold: 1_048_576, // 1MB
        enable_mouse: false,
        relative_line_numbers: true,
        preserve_formatting: true,
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&original).expect("Failed to serialize");

    // Deserialize back
    let deserialized: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    // Should be identical
    assert_eq!(original.theme, deserialized.theme);
    assert_eq!(original.indent_size, deserialized.indent_size);
    assert_eq!(original.show_line_numbers, deserialized.show_line_numbers);
    assert_eq!(original.auto_save, deserialized.auto_save);
    assert_eq!(original.validation_mode, deserialized.validation_mode);
    assert_eq!(original.create_backup, deserialized.create_backup);
    assert_eq!(original.undo_limit, deserialized.undo_limit);
    assert_eq!(
        original.sync_unnamed_register,
        deserialized.sync_unnamed_register
    );
    assert_eq!(
        original.lazy_load_threshold,
        deserialized.lazy_load_threshold
    );
    assert_eq!(original.enable_mouse, deserialized.enable_mouse);
}

#[test]
fn test_config_clone() {
    let config1 = Config::default();
    let config2 = config1.clone();

    assert_eq!(config1.theme, config2.theme);
    assert_eq!(config1.indent_size, config2.indent_size);
    assert_eq!(config1.show_line_numbers, config2.show_line_numbers);
    assert_eq!(config1.auto_save, config2.auto_save);
    assert_eq!(config1.validation_mode, config2.validation_mode);
    assert_eq!(config1.create_backup, config2.create_backup);
    assert_eq!(config1.undo_limit, config2.undo_limit);
    assert_eq!(config1.sync_unnamed_register, config2.sync_unnamed_register);
    assert_eq!(config1.lazy_load_threshold, config2.lazy_load_threshold);
    assert_eq!(config1.enable_mouse, config2.enable_mouse);
}

#[test]
fn test_config_debug() {
    let config = Config::default();
    let debug_str = format!("{:?}", config);

    // Debug output should contain key field names
    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("theme"));
    assert!(debug_str.contains("indent_size"));
}

#[test]
fn test_undo_limit_default_is_50() {
    let config = Config::default();
    assert_eq!(config.undo_limit, 50);
}
