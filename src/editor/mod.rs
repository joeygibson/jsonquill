//! Editor state and mode management.
//!
//! This module provides the core editor functionality including modal editing
//! support, cursor management, and editor state. It follows vim-style modal
//! editing paradigms with Normal, Insert, and Command modes.
//!
//! # Modules
//!
//! - `mode`: Editor mode enumeration and transitions
//!
//! # Example
//!
//! ```
//! use jeditor::editor::mode::EditorMode;
//!
//! // Editor starts in Normal mode
//! let mode = EditorMode::default();
//! assert_eq!(mode, EditorMode::Normal);
//! ```

pub mod mode;
