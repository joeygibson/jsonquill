//! Repeatable command tracking for the '.' key.

use crate::document::node::JsonValue;

/// Represents a command that can be repeated with the '.' key.
///
/// This enum captures editing operations that modify the document
/// along with their parameters, so they can be replayed at a different
/// cursor position.
#[derive(Debug, Clone)]
pub enum RepeatableCommand {
    /// Delete with count (dd, 3dd)
    Delete { count: u32 },

    /// Yank with count (yy, 5yy)
    Yank { count: u32 },

    /// Paste after (p) or before (P)
    Paste { before: bool },

    /// Add scalar value (i)
    /// key is Some for object insertions, None for arrays
    Add {
        value: JsonValue,
        key: Option<String>,
    },

    /// Add empty array (a)
    AddArray,

    /// Add empty object (o)
    AddObject,

    /// Rename key (r)
    Rename { new_key: String },

    /// Change value in insert mode (e)
    ChangeValue { new_value: String },
}
