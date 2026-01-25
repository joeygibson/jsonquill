//! JSONPath query parser and evaluator for structural JSON search.
//!
//! This module provides JSONPath query support, enabling users to search
//! JSON documents by structure rather than just text content.
//!
//! # Supported Syntax
//!
//! - `$` - Root node
//! - `.property` - Named property access
//! - `['property']` - Bracket notation
//! - `[index]` - Array index (supports negative indices)
//! - `[*]` or `.*` - All children (wildcard)
//! - `..property` or `..` - Recursive descent
//! - `[start:end]` - Array slicing
//! - `['prop1','prop2']` - Multiple properties
//!
//! # Examples
//!
//! ```
//! // $.store.book[*].author - all book authors
//! // $..price - all price fields anywhere
//! // $.items[0:3] - first 3 items
//! // $.user['name','email'] - multiple properties
//! ```

pub mod ast;
pub mod error;
pub mod evaluator;
pub mod parser;

pub use ast::{JsonPath, PathSegment};
pub use error::JsonPathError;
pub use evaluator::Evaluator;
pub use parser::Parser;
