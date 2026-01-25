//! Error types for JSONPath parsing and evaluation.

use std::fmt;

/// Errors that can occur during JSONPath parsing or evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonPathError {
    /// Unexpected token at a specific position.
    UnexpectedToken {
        position: usize,
        found: String,
        expected: String,
    },
    /// Unexpected end of input.
    UnexpectedEnd { expected: String },
    /// Invalid syntax with description.
    InvalidSyntax { message: String },
}

impl fmt::Display for JsonPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonPathError::UnexpectedToken {
                position,
                found,
                expected,
            } => write!(
                f,
                "Unexpected token '{}' at position {}, expected {}",
                found, position, expected
            ),
            JsonPathError::UnexpectedEnd { expected } => {
                write!(f, "Unexpected end of input, expected {}", expected)
            }
            JsonPathError::InvalidSyntax { message } => {
                write!(f, "Invalid JSONPath syntax: {}", message)
            }
        }
    }
}

impl std::error::Error for JsonPathError {}
