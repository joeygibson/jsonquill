//! Editor mode management for modal editing.
//!
//! This module provides the `EditorMode` enum that represents the current editing
//! mode in jsonquill. Following vim-style modal editing, the editor can be in one
//! of three modes, each with different keybindings and behaviors.
//!
//! # Modes
//!
//! - **Normal**: The default mode for navigation and commands
//! - **Insert**: Mode for modifying JSON values
//! - **Command**: Mode for executing editor commands (search, save, quit, etc.)
//!
//! # Example
//!
//! ```
//! use jsonquill::editor::mode::EditorMode;
//!
//! // Editor starts in Normal mode by default
//! let mode = EditorMode::default();
//! assert_eq!(mode, EditorMode::Normal);
//! assert_eq!(format!("{}", mode), "NORMAL");
//!
//! // Switch to Insert mode
//! let mode = EditorMode::Insert;
//! assert_eq!(format!("{}", mode), "INSERT");
//! ```

use std::fmt;

/// Represents the current editing mode of the editor.
///
/// jsonquill follows a vim-style modal editing paradigm where the behavior of
/// keystrokes depends on the current mode. The mode is typically displayed
/// in the status bar using the `Display` implementation.
///
/// # Modes
///
/// - `Normal`: Default mode for navigation and structural commands. In this mode,
///   keys like `j`/`k` move the cursor, `d` deletes nodes, etc.
/// - `Insert`: Mode for editing JSON values. Typing characters modifies the
///   currently selected value.
/// - `Command`: Mode for executing editor commands like `:w` (save), `:q` (quit),
///   or `/` (search).
///
/// # Examples
///
/// ```
/// use jsonquill::editor::mode::EditorMode;
///
/// let mode = EditorMode::Normal;
/// assert_eq!(format!("{}", mode), "NORMAL");
///
/// let mode = EditorMode::Insert;
/// assert_eq!(format!("{}", mode), "INSERT");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Normal mode for navigation and structural operations.
    Normal,
    /// Insert mode for editing JSON values.
    Insert,
    /// Command mode for executing editor commands.
    Command,
    /// Search mode for finding text in keys and values.
    Search,
    /// Visual mode for selecting multiple nodes.
    Visual,
}

impl fmt::Display for EditorMode {
    /// Formats the mode as an uppercase string suitable for display in the status bar.
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::editor::mode::EditorMode;
    ///
    /// assert_eq!(format!("{}", EditorMode::Normal), "NORMAL");
    /// assert_eq!(format!("{}", EditorMode::Insert), "INSERT");
    /// assert_eq!(format!("{}", EditorMode::Command), "COMMAND");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorMode::Normal => write!(f, "NORMAL"),
            EditorMode::Insert => write!(f, "INSERT"),
            EditorMode::Command => write!(f, "COMMAND"),
            EditorMode::Search => write!(f, "SEARCH"),
            EditorMode::Visual => write!(f, "VISUAL"),
        }
    }
}

impl Default for EditorMode {
    /// Returns `EditorMode::Normal` as the default mode.
    ///
    /// The editor always starts in Normal mode, following vim conventions.
    ///
    /// # Examples
    ///
    /// ```
    /// use jsonquill::editor::mode::EditorMode;
    ///
    /// let mode = EditorMode::default();
    /// assert_eq!(mode, EditorMode::Normal);
    /// ```
    fn default() -> Self {
        EditorMode::Normal
    }
}
