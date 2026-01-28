//! Configuration system for jsonquill.
//!
//! This module provides the configuration structure for jsonquill with sensible defaults
//! and support for serialization/deserialization via serde. Configuration can be loaded
//! from TOML files and merged with command-line arguments.
//!
//! # Example
//!
//! ```
//! use jsonquill::config::Config;
//!
//! // Use default configuration
//! let config = Config::default();
//! assert_eq!(config.theme, "default-dark");
//! assert_eq!(config.indent_size, 2);
//!
//! // Create custom configuration
//! let custom = Config {
//!     theme: "gruvbox".to_string(),
//!     indent_size: 4,
//!     ..Config::default()
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Configuration for the jsonquill application.
///
/// This structure contains all configurable settings for jsonquill, including
/// display preferences, editing behavior, performance tuning, and feature flags.
/// All fields have sensible defaults via `Config::default()`.
///
/// # Fields
///
/// * `theme` - Color scheme name (default: "default-dark")
/// * `indent_size` - Number of spaces per indentation level (default: 2)
/// * `show_line_numbers` - Display line numbers in the editor (default: true)
/// * `auto_save` - Automatically save on changes (default: false)
/// * `validation_mode` - JSON validation strictness: "strict", "permissive", or "none" (default: "strict")
/// * `create_backup` - Create .bak files before saving (default: false)
/// * `undo_limit` - Maximum number of undo operations to keep (default: 50)
/// * `sync_unnamed_register` - Sync unnamed register with system clipboard (default: true)
/// * `lazy_load_threshold` - File size in bytes to trigger lazy loading (default: 100MB)
/// * `enable_mouse` - Enable mouse/trackpad scrolling support (default: true)
/// * `preserve_formatting` - Preserve original formatting for unmodified nodes (default: true)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Color scheme name
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Number of spaces per indentation level
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,

    /// Display line numbers in the editor
    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,

    /// Automatically save on changes
    #[serde(default)]
    pub auto_save: bool,

    /// JSON validation strictness: "strict", "permissive", or "none"
    #[serde(default = "default_validation_mode")]
    pub validation_mode: String,

    /// Create .bak files before saving
    #[serde(default)]
    pub create_backup: bool,

    /// Maximum number of undo operations to keep
    #[serde(default = "default_undo_limit")]
    pub undo_limit: usize,

    /// Sync unnamed register with system clipboard
    #[serde(default = "default_sync_unnamed_register")]
    pub sync_unnamed_register: bool,

    /// File size in bytes to trigger lazy loading
    #[serde(default = "default_lazy_load_threshold")]
    pub lazy_load_threshold: usize,

    /// Enable mouse/trackpad scrolling
    #[serde(default = "default_enable_mouse")]
    pub enable_mouse: bool,

    /// Show relative line numbers (like vim's relativenumber)
    #[serde(default)]
    pub relative_line_numbers: bool,

    /// Preserve original formatting for unmodified nodes (default: true)
    /// When enabled, unmodified portions of JSON retain exact original formatting
    #[serde(default = "default_preserve_formatting")]
    pub preserve_formatting: bool,
}

/// Returns the default theme name.
fn default_theme() -> String {
    "default-dark".to_string()
}

/// Returns the default indentation size.
fn default_indent_size() -> usize {
    2
}

/// Returns the default for showing line numbers.
fn default_show_line_numbers() -> bool {
    true
}

/// Returns the default validation mode.
fn default_validation_mode() -> String {
    "strict".to_string()
}

/// Returns the default undo limit.
fn default_undo_limit() -> usize {
    50 // Changed from 1000
}

/// Returns the default for syncing unnamed register.
fn default_sync_unnamed_register() -> bool {
    true
}

/// Returns the default lazy load threshold (100MB).
fn default_lazy_load_threshold() -> usize {
    104_857_600 // 100MB
}

/// Returns the default for enabling mouse support.
fn default_enable_mouse() -> bool {
    true
}

/// Returns the default for preserving formatting.
fn default_preserve_formatting() -> bool {
    true // Enabled by default - preserves original formatting for unmodified nodes
}

impl Default for Config {
    /// Creates a new configuration with default values.
    ///
    /// # Default Values
    ///
    /// * `theme`: "default-dark"
    /// * `indent_size`: 2
    /// * `show_line_numbers`: true
    /// * `auto_save`: false
    /// * `validation_mode`: "strict"
    /// * `create_backup`: false
    /// * `undo_limit`: 50
    /// * `sync_unnamed_register`: true
    /// * `lazy_load_threshold`: 104,857,600 (100MB)
    /// * `enable_mouse`: true
    /// * `preserve_formatting`: true
    ///
    /// # Example
    ///
    /// ```
    /// use jsonquill::config::Config;
    ///
    /// let config = Config::default();
    /// assert_eq!(config.theme, "default-dark");
    /// assert_eq!(config.indent_size, 2);
    /// assert!(config.show_line_numbers);
    /// ```
    fn default() -> Self {
        Self {
            theme: default_theme(),
            indent_size: default_indent_size(),
            show_line_numbers: true,
            auto_save: false,
            validation_mode: default_validation_mode(),
            create_backup: false,
            undo_limit: default_undo_limit(),
            sync_unnamed_register: true,
            lazy_load_threshold: default_lazy_load_threshold(),
            enable_mouse: default_enable_mouse(),
            relative_line_numbers: false,
            preserve_formatting: default_preserve_formatting(),
        }
    }
}

impl Config {
    /// Returns the path to the config file.
    ///
    /// Uses `~/.config/jsonquill/config.toml` on all platforms.
    pub fn config_path() -> Option<std::path::PathBuf> {
        dirs::home_dir().map(|mut path| {
            path.push(".config");
            path.push("jsonquill");
            path.push("config.toml");
            path
        })
    }

    /// Loads configuration from the default config file.
    ///
    /// Returns the default configuration if the file doesn't exist or can't be read.
    pub fn load() -> Self {
        let config_path = match Self::config_path() {
            Some(path) => path,
            None => return Self::default(),
        };

        if !config_path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_else(|_| Self::default()),
            Err(_) => Self::default(),
        }
    }

    /// Saves configuration to the default config file.
    ///
    /// Creates the config directory if it doesn't exist.
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, toml_string)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preserve_formatting_default() {
        let config = Config::default();
        assert!(config.preserve_formatting); // Format preservation is enabled by default
    }

    #[test]
    fn test_preserve_formatting_can_be_disabled() {
        let config = Config {
            preserve_formatting: false,
            ..Default::default()
        };
        assert!(!config.preserve_formatting);
    }
}
