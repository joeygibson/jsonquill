//! Editor state and mode management.
//!
//! This module provides the core editor functionality including modal editing
//! support, cursor management, and editor state. It follows vim-style modal
//! editing paradigms with Normal, Insert, and Command modes.
//!
//! # Modules
//!
//! - `mode`: Editor mode enumeration and transitions
//! - `cursor`: Cursor position tracking in the JSON tree
//! - `state`: Editor state management (tree, mode, cursor, etc.)
//!
//! # Example
//!
//! ```
//! use jsonquill::editor::mode::EditorMode;
//!
//! // Editor starts in Normal mode
//! let mode = EditorMode::default();
//! assert_eq!(mode, EditorMode::Normal);
//! ```

pub mod cursor;
pub mod jumplist;
pub mod marks;
pub mod mode;
pub mod registers;
pub mod repeat;
pub mod state;
pub mod undo;
