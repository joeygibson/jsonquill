# JSONPath Structural Search Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add JSONPath query support for structural navigation of JSON documents using paths like `$.store.book[*].author` and `$..price`.

**Architecture:** Create new `jsonpath` module with parser (tokenizer + recursive descent) and evaluator (tree walker). Integrate with existing search infrastructure in `EditorState`, reusing `search_results` field. Add commands `:path`, `:jp`, and `:find` alias.

**Tech Stack:** Rust, existing jsonquill architecture (JsonTree, EditorState), no new dependencies.

---

## Task 1: Create JSONPath Module Structure

**Files:**
- Create: `src/jsonpath/mod.rs`
- Create: `src/jsonpath/ast.rs`
- Create: `src/jsonpath/error.rs`
- Modify: `src/lib.rs:1-7`

**Step 1: Create JSONPath error types**

Create `src/jsonpath/error.rs`:

```rust
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
```

**Step 2: Create JSONPath AST types**

Create `src/jsonpath/ast.rs`:

```rust
//! Abstract syntax tree types for JSONPath expressions.

/// A segment in a JSONPath expression.
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// Root node ($)
    Root,
    /// Current node (@) - reserved for future filter support
    Current,
    /// Named child (.property or ['property'])
    Child(String),
    /// Array index ([0], [-1])
    Index(isize),
    /// Wildcard (* or [*]) - all children
    Wildcard,
    /// Recursive descent (.. or ..property)
    RecursiveDescent(Option<String>),
    /// Array slice ([start:end])
    Slice(Option<isize>, Option<isize>),
    /// Multiple properties (['prop1','prop2'])
    MultiProperty(Vec<String>),
}

/// A complete JSONPath expression.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPath {
    /// Segments that make up the path.
    pub segments: Vec<PathSegment>,
}

impl JsonPath {
    /// Creates a new JSONPath with the given segments.
    pub fn new(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }
}
```

**Step 3: Create JSONPath module file**

Create `src/jsonpath/mod.rs`:

```rust
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
```

**Step 4: Add jsonpath module to lib.rs**

Modify `src/lib.rs`:

```rust
pub mod config;
pub mod document;
pub mod editor;
pub mod file;
pub mod input;
pub mod jsonpath;
pub mod theme;
pub mod ui;
```

**Step 5: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully (no implementations yet, just structure)

**Step 6: Commit module structure**

```bash
git add src/jsonpath/ src/lib.rs
git commit -m "feat(jsonpath): add module structure with AST and error types

- Create jsonpath module with ast, error, parser, evaluator submodules
- Define PathSegment enum for JSONPath syntax elements
- Define JsonPath struct for complete expressions
- Add JsonPathError for parsing/evaluation errors

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Implement JSONPath Parser (Part 1: Tokenizer)

**Files:**
- Create: `src/jsonpath/parser.rs`

**Step 1: Write failing parser tests**

Create `src/jsonpath/parser.rs`:

```rust
//! JSONPath query string parser.

use super::ast::{JsonPath, PathSegment};
use super::error::JsonPathError;

/// Parser for JSONPath query strings.
pub struct Parser {
    input: String,
    position: usize,
}

impl Parser {
    /// Creates a new parser for the given query string.
    pub fn new(query: &str) -> Self {
        Self {
            input: query.to_string(),
            position: 0,
        }
    }

    /// Parses the query string into a JsonPath.
    pub fn parse(query: &str) -> Result<JsonPath, JsonPathError> {
        let mut parser = Parser::new(query);
        parser.parse_path()
    }

    fn parse_path(&mut self) -> Result<JsonPath, JsonPathError> {
        // TODO: implement
        Ok(JsonPath::new(vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_root() {
        let result = Parser::parse("$");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0], PathSegment::Root);
    }

    #[test]
    fn test_parse_child() {
        let result = Parser::parse("$.store");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0], PathSegment::Root);
        assert_eq!(path.segments[1], PathSegment::Child("store".to_string()));
    }

    #[test]
    fn test_parse_nested_child() {
        let result = Parser::parse("$.store.book");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[2], PathSegment::Child("book".to_string()));
    }

    #[test]
    fn test_parse_array_index() {
        let result = Parser::parse("$.items[0]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[1], PathSegment::Child("items".to_string()));
        assert_eq!(path.segments[2], PathSegment::Index(0));
    }

    #[test]
    fn test_parse_wildcard() {
        let result = Parser::parse("$.items[*]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Wildcard);
    }

    #[test]
    fn test_parse_wildcard_dot() {
        let result = Parser::parse("$.items.*");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Wildcard);
    }

    #[test]
    fn test_parse_recursive_descent() {
        let result = Parser::parse("$..price");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(
            path.segments[1],
            PathSegment::RecursiveDescent(Some("price".to_string()))
        );
    }

    #[test]
    fn test_parse_empty_fails() {
        let result = Parser::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_root_fails() {
        let result = Parser::parse("store.book");
        assert!(result.is_err());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --package jsonquill --lib jsonpath::parser::tests`
Expected: Tests fail with assertion errors (parse returns empty segments)

**Step 3: Implement tokenizer helpers**

Add to `src/jsonpath/parser.rs` after the `Parser` struct:

```rust
impl Parser {
    // ... existing methods ...

    /// Returns the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /// Returns the next character and advances position.
    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    /// Checks if we've reached the end of input.
    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Expects a specific character and advances, or returns an error.
    fn expect(&mut self, expected: char) -> Result<(), JsonPathError> {
        self.skip_whitespace();
        match self.next() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(JsonPathError::UnexpectedToken {
                position: self.position - 1,
                found: ch.to_string(),
                expected: format!("'{}'", expected),
            }),
            None => Err(JsonPathError::UnexpectedEnd {
                expected: format!("'{}'", expected),
            }),
        }
    }

    /// Parses an identifier (property name).
    fn parse_identifier(&mut self) -> Result<String, JsonPathError> {
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                name.push(ch);
                self.next();
            } else {
                break;
            }
        }
        if name.is_empty() {
            Err(JsonPathError::InvalidSyntax {
                message: "Expected identifier".to_string(),
            })
        } else {
            Ok(name)
        }
    }
}
```

**Step 4: Verify helper methods compile**

Run: `cargo build`
Expected: Compiles successfully

**Step 5: Commit tokenizer helpers**

```bash
git add src/jsonpath/parser.rs
git commit -m "feat(jsonpath): add parser tokenizer helpers

- Add peek/next/skip_whitespace for character navigation
- Add expect() for required character matching
- Add parse_identifier() for property names
- Add failing tests for parser functionality

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Implement JSONPath Parser (Part 2: Core Parsing)

**Files:**
- Modify: `src/jsonpath/parser.rs`

**Step 1: Implement root and child parsing**

Replace `parse_path()` in `src/jsonpath/parser.rs`:

```rust
    fn parse_path(&mut self) -> Result<JsonPath, JsonPathError> {
        let mut segments = Vec::new();

        // Must start with $ (root)
        self.skip_whitespace();
        if self.peek() != Some('$') {
            return Err(JsonPathError::InvalidSyntax {
                message: "JSONPath must start with '$'".to_string(),
            });
        }
        self.next(); // consume $
        segments.push(PathSegment::Root);

        // Parse remaining segments
        while !self.is_eof() {
            self.skip_whitespace();
            match self.peek() {
                Some('.') => {
                    self.next(); // consume .
                    if self.peek() == Some('.') {
                        // Recursive descent
                        self.next(); // consume second .
                        segments.push(self.parse_recursive_descent()?);
                    } else if self.peek() == Some('*') {
                        // Wildcard
                        self.next(); // consume *
                        segments.push(PathSegment::Wildcard);
                    } else {
                        // Named child
                        let name = self.parse_identifier()?;
                        segments.push(PathSegment::Child(name));
                    }
                }
                Some('[') => {
                    self.next(); // consume [
                    segments.push(self.parse_bracket_expression()?);
                }
                Some(ch) => {
                    return Err(JsonPathError::UnexpectedToken {
                        position: self.position,
                        found: ch.to_string(),
                        expected: "'.' or '['".to_string(),
                    });
                }
                None => break,
            }
        }

        Ok(JsonPath::new(segments))
    }

    fn parse_recursive_descent(&mut self) -> Result<PathSegment, JsonPathError> {
        self.skip_whitespace();
        if self.peek() == Some('[') || self.is_eof() {
            // Just .. without property name
            Ok(PathSegment::RecursiveDescent(None))
        } else {
            // ..property
            let name = self.parse_identifier()?;
            Ok(PathSegment::RecursiveDescent(Some(name)))
        }
    }

    fn parse_bracket_expression(&mut self) -> Result<PathSegment, JsonPathError> {
        self.skip_whitespace();

        match self.peek() {
            Some('*') => {
                self.next(); // consume *
                self.expect(']')?;
                Ok(PathSegment::Wildcard)
            }
            Some('\'') | Some('"') => {
                // String property or multiple properties
                self.parse_bracket_string()
            }
            Some('-') | Some('0'..='9') => {
                // Number: could be index or slice
                self.parse_bracket_number()
            }
            Some(':') => {
                // Slice starting from beginning [:end]
                self.parse_slice(None)
            }
            Some(ch) => Err(JsonPathError::UnexpectedToken {
                position: self.position,
                found: ch.to_string(),
                expected: "'*', quote, or number".to_string(),
            }),
            None => Err(JsonPathError::UnexpectedEnd {
                expected: "bracket expression".to_string(),
            }),
        }
    }

    fn parse_bracket_string(&mut self) -> Result<PathSegment, JsonPathError> {
        let mut properties = Vec::new();

        loop {
            self.skip_whitespace();
            let quote = self.peek().ok_or_else(|| JsonPathError::UnexpectedEnd {
                expected: "quote".to_string(),
            })?;

            if quote != '\'' && quote != '"' {
                break;
            }

            self.next(); // consume opening quote

            let mut prop = String::new();
            loop {
                match self.next() {
                    Some(ch) if ch == quote => break,
                    Some(ch) => prop.push(ch),
                    None => {
                        return Err(JsonPathError::UnexpectedEnd {
                            expected: format!("closing {}", quote),
                        })
                    }
                }
            }

            properties.push(prop);

            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.next(); // consume comma
            } else {
                break;
            }
        }

        self.expect(']')?;

        if properties.len() == 1 {
            Ok(PathSegment::Child(properties.into_iter().next().unwrap()))
        } else {
            Ok(PathSegment::MultiProperty(properties))
        }
    }

    fn parse_bracket_number(&mut self) -> Result<PathSegment, JsonPathError> {
        let num_str = self.parse_number_string()?;
        let num: isize = num_str.parse().map_err(|_| JsonPathError::InvalidSyntax {
            message: format!("Invalid number: {}", num_str),
        })?;

        self.skip_whitespace();

        if self.peek() == Some(':') {
            // It's a slice [start:end]
            self.parse_slice(Some(num))
        } else {
            // It's just an index
            self.expect(']')?;
            Ok(PathSegment::Index(num))
        }
    }

    fn parse_number_string(&mut self) -> Result<String, JsonPathError> {
        let mut num = String::new();

        if self.peek() == Some('-') {
            num.push('-');
            self.next();
        }

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num.push(ch);
                self.next();
            } else {
                break;
            }
        }

        if num.is_empty() || num == "-" {
            Err(JsonPathError::InvalidSyntax {
                message: "Expected number".to_string(),
            })
        } else {
            Ok(num)
        }
    }

    fn parse_slice(&mut self, start: Option<isize>) -> Result<PathSegment, JsonPathError> {
        self.expect(':')?;
        self.skip_whitespace();

        let end = if self.peek() == Some(']') {
            None
        } else {
            let num_str = self.parse_number_string()?;
            Some(num_str.parse().map_err(|_| JsonPathError::InvalidSyntax {
                message: format!("Invalid number in slice: {}", num_str),
            })?)
        };

        self.expect(']')?;
        Ok(PathSegment::Slice(start, end))
    }
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --package jsonquill --lib jsonpath::parser::tests`
Expected: All parser tests pass

**Step 3: Add tests for slicing and multi-property**

Add to the test module in `src/jsonpath/parser.rs`:

```rust
    #[test]
    fn test_parse_slice() {
        let result = Parser::parse("$.items[0:3]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Slice(Some(0), Some(3)));
    }

    #[test]
    fn test_parse_slice_open_end() {
        let result = Parser::parse("$.items[2:]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Slice(Some(2), None));
    }

    #[test]
    fn test_parse_slice_open_start() {
        let result = Parser::parse("$.items[:5]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Slice(None, Some(5)));
    }

    #[test]
    fn test_parse_negative_index() {
        let result = Parser::parse("$.items[-1]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Index(-1));
    }

    #[test]
    fn test_parse_multi_property() {
        let result = Parser::parse("$.user['name','email']");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(
            path.segments[2],
            PathSegment::MultiProperty(vec!["name".to_string(), "email".to_string()])
        );
    }

    #[test]
    fn test_parse_bracket_notation() {
        let result = Parser::parse("$['store']['book']");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[1], PathSegment::Child("store".to_string()));
        assert_eq!(path.segments[2], PathSegment::Child("book".to_string()));
    }

    #[test]
    fn test_parse_complex_path() {
        let result = Parser::parse("$.store.book[*].author");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 5);
        assert_eq!(path.segments[0], PathSegment::Root);
        assert_eq!(path.segments[1], PathSegment::Child("store".to_string()));
        assert_eq!(path.segments[2], PathSegment::Child("book".to_string()));
        assert_eq!(path.segments[3], PathSegment::Wildcard);
        assert_eq!(path.segments[4], PathSegment::Child("author".to_string()));
    }
```

**Step 4: Run all parser tests**

Run: `cargo test --package jsonquill --lib jsonpath::parser::tests`
Expected: All tests pass (18 tests)

**Step 5: Commit parser implementation**

```bash
git add src/jsonpath/parser.rs
git commit -m "feat(jsonpath): implement parser for core JSONPath syntax

- Parse root ($), child (.property, ['property'])
- Parse array index ([0], [-1]) and wildcard ([*], .*)
- Parse recursive descent (.., ..property)
- Parse array slicing ([start:end], [start:], [:end])
- Parse multiple properties (['prop1','prop2'])
- Add comprehensive test coverage (18 tests)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Implement JSONPath Evaluator

**Files:**
- Create: `src/jsonpath/evaluator.rs`

**Step 1: Write failing evaluator tests**

Create `src/jsonpath/evaluator.rs`:

```rust
//! JSONPath expression evaluator.

use super::ast::{JsonPath, PathSegment};
use crate::document::node::{JsonNode, JsonValue};
use crate::document::tree::JsonTree;

/// Evaluates JSONPath expressions against a JSON tree.
pub struct Evaluator<'a> {
    tree: &'a JsonTree,
}

impl<'a> Evaluator<'a> {
    /// Creates a new evaluator for the given tree.
    pub fn new(tree: &'a JsonTree) -> Self {
        Self { tree }
    }

    /// Evaluates a JSONPath expression and returns matching node paths.
    pub fn evaluate(&self, path: &JsonPath) -> Vec<Vec<usize>> {
        // TODO: implement
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonpath::Parser;

    fn make_test_tree() -> JsonTree {
        // {"store": {"book": [{"title": "Book1", "price": 10}, {"title": "Book2", "price": 20}]}}
        let book1 = JsonNode::new(JsonValue::Object(vec![
            (
                "title".to_string(),
                JsonNode::new(JsonValue::String("Book1".to_string())),
            ),
            ("price".to_string(), JsonNode::new(JsonValue::Number(10.0))),
        ]));

        let book2 = JsonNode::new(JsonValue::Object(vec![
            (
                "title".to_string(),
                JsonNode::new(JsonValue::String("Book2".to_string())),
            ),
            ("price".to_string(), JsonNode::new(JsonValue::Number(20.0))),
        ]));

        let books = JsonNode::new(JsonValue::Array(vec![book1, book2]));

        let store = JsonNode::new(JsonValue::Object(vec![("book".to_string(), books)]));

        let root = JsonNode::new(JsonValue::Object(vec![("store".to_string(), store)]));

        JsonTree::new(root)
    }

    #[test]
    fn test_evaluate_root() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vec![]);
    }

    #[test]
    fn test_evaluate_child() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$.store").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vec![0]);
    }

    #[test]
    fn test_evaluate_nested_child() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$.store.book").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vec![0, 0]);
    }

    #[test]
    fn test_evaluate_array_index() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$.store.book[0]").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vec![0, 0, 0]);
    }

    #[test]
    fn test_evaluate_wildcard() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$.store.book[*]").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], vec![0, 0, 0]);
        assert_eq!(results[1], vec![0, 0, 1]);
    }

    #[test]
    fn test_evaluate_wildcard_child() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$.store.book[*].title").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], vec![0, 0, 0, 0]);
        assert_eq!(results[1], vec![0, 0, 1, 0]);
    }

    #[test]
    fn test_evaluate_recursive_descent() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$..price").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&vec![0, 0, 0, 1]));
        assert!(results.contains(&vec![0, 0, 1, 1]));
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --package jsonquill --lib jsonpath::evaluator::tests`
Expected: All tests fail (evaluate returns empty vec)

**Step 3: Implement core evaluator logic**

Add to `src/jsonpath/evaluator.rs`:

```rust
impl<'a> Evaluator<'a> {
    // ... existing new() method ...

    /// Evaluates a JSONPath expression and returns matching node paths.
    pub fn evaluate(&self, path: &JsonPath) -> Vec<Vec<usize>> {
        if path.segments.is_empty() {
            return vec![];
        }

        // Start from root
        let mut current_paths = vec![vec![]];

        for segment in &path.segments {
            current_paths = self.evaluate_segment(segment, current_paths);
        }

        current_paths
    }

    fn evaluate_segment(
        &self,
        segment: &PathSegment,
        current_paths: Vec<Vec<usize>>,
    ) -> Vec<Vec<usize>> {
        let mut results = Vec::new();

        for path in current_paths {
            match segment {
                PathSegment::Root => {
                    // Root just returns the current path (empty path = root)
                    results.push(path);
                }
                PathSegment::Current => {
                    // Current node (for future filter support)
                    results.push(path);
                }
                PathSegment::Child(name) => {
                    if let Some(child_path) = self.find_child(&path, name) {
                        results.push(child_path);
                    }
                }
                PathSegment::Index(idx) => {
                    if let Some(child_path) = self.get_array_element(&path, *idx) {
                        results.push(child_path);
                    }
                }
                PathSegment::Wildcard => {
                    results.extend(self.get_all_children(&path));
                }
                PathSegment::RecursiveDescent(name) => {
                    results.extend(self.recursive_descent(&path, name.as_deref()));
                }
                PathSegment::Slice(start, end) => {
                    results.extend(self.get_slice(&path, *start, *end));
                }
                PathSegment::MultiProperty(props) => {
                    for prop in props {
                        if let Some(child_path) = self.find_child(&path, prop) {
                            results.push(child_path);
                        }
                    }
                }
            }
        }

        results
    }

    fn find_child(&self, path: &[usize], name: &str) -> Option<Vec<usize>> {
        let node = self.tree.get_node(path)?;

        if let JsonValue::Object(fields) = node.value() {
            for (idx, (key, _)) in fields.iter().enumerate() {
                if key == name {
                    let mut child_path = path.to_vec();
                    child_path.push(idx);
                    return Some(child_path);
                }
            }
        }

        None
    }

    fn get_array_element(&self, path: &[usize], index: isize) -> Option<Vec<usize>> {
        let node = self.tree.get_node(path)?;

        if let JsonValue::Array(elements) = node.value() {
            let len = elements.len() as isize;
            let actual_index = if index < 0 {
                (len + index) as usize
            } else {
                index as usize
            };

            if actual_index < elements.len() {
                let mut child_path = path.to_vec();
                child_path.push(actual_index);
                return Some(child_path);
            }
        }

        None
    }

    fn get_all_children(&self, path: &[usize]) -> Vec<Vec<usize>> {
        let Some(node) = self.tree.get_node(path) else {
            return vec![];
        };

        let mut results = Vec::new();

        match node.value() {
            JsonValue::Object(fields) => {
                for idx in 0..fields.len() {
                    let mut child_path = path.to_vec();
                    child_path.push(idx);
                    results.push(child_path);
                }
            }
            JsonValue::Array(elements) => {
                for idx in 0..elements.len() {
                    let mut child_path = path.to_vec();
                    child_path.push(idx);
                    results.push(child_path);
                }
            }
            _ => {}
        }

        results
    }

    fn get_slice(
        &self,
        path: &[usize],
        start: Option<isize>,
        end: Option<isize>,
    ) -> Vec<Vec<usize>> {
        let Some(node) = self.tree.get_node(path) else {
            return vec![];
        };

        if let JsonValue::Array(elements) = node.value() {
            let len = elements.len() as isize;

            let start_idx = match start {
                Some(s) if s < 0 => ((len + s).max(0)) as usize,
                Some(s) => (s.min(len)) as usize,
                None => 0,
            };

            let end_idx = match end {
                Some(e) if e < 0 => ((len + e).max(0)) as usize,
                Some(e) => (e.min(len)) as usize,
                None => len as usize,
            };

            let mut results = Vec::new();
            for idx in start_idx..end_idx {
                let mut child_path = path.to_vec();
                child_path.push(idx);
                results.push(child_path);
            }
            return results;
        }

        vec![]
    }

    fn recursive_descent(&self, path: &[usize], name: Option<&str>) -> Vec<Vec<usize>> {
        let mut results = Vec::new();

        // Helper function to recursively collect paths
        fn collect_recursive(
            evaluator: &Evaluator,
            current_path: &[usize],
            name: Option<&str>,
            results: &mut Vec<Vec<usize>>,
        ) {
            let Some(node) = evaluator.tree.get_node(current_path) else {
                return;
            };

            // If we're looking for a specific property name
            if let Some(target_name) = name {
                if let JsonValue::Object(fields) = node.value() {
                    for (idx, (key, _)) in fields.iter().enumerate() {
                        if key == target_name {
                            let mut match_path = current_path.to_vec();
                            match_path.push(idx);
                            results.push(match_path.clone());
                        }
                        // Recurse into this child
                        let mut child_path = current_path.to_vec();
                        child_path.push(idx);
                        collect_recursive(evaluator, &child_path, name, results);
                    }
                } else if let JsonValue::Array(elements) = node.value() {
                    for idx in 0..elements.len() {
                        let mut child_path = current_path.to_vec();
                        child_path.push(idx);
                        collect_recursive(evaluator, &child_path, name, results);
                    }
                }
            } else {
                // No specific name, return all descendants
                match node.value() {
                    JsonValue::Object(fields) => {
                        for idx in 0..fields.len() {
                            let mut child_path = current_path.to_vec();
                            child_path.push(idx);
                            results.push(child_path.clone());
                            collect_recursive(evaluator, &child_path, name, results);
                        }
                    }
                    JsonValue::Array(elements) => {
                        for idx in 0..elements.len() {
                            let mut child_path = current_path.to_vec();
                            child_path.push(idx);
                            results.push(child_path.clone());
                            collect_recursive(evaluator, &child_path, name, results);
                        }
                    }
                    _ => {}
                }
            }
        }

        collect_recursive(self, path, name, &mut results);
        results
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --package jsonquill --lib jsonpath::evaluator::tests`
Expected: All 7 evaluator tests pass

**Step 5: Add more edge case tests**

Add to test module in `src/jsonpath/evaluator.rs`:

```rust
    #[test]
    fn test_evaluate_negative_index() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$.store.book[-1]").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vec![0, 0, 1]); // Last element
    }

    #[test]
    fn test_evaluate_slice() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$.store.book[0:1]").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vec![0, 0, 0]);
    }

    #[test]
    fn test_evaluate_no_match() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let path = Parser::parse("$.nonexistent").unwrap();
        let results = evaluator.evaluate(&path);
        assert_eq!(results.len(), 0);
    }
```

**Step 6: Run all evaluator tests**

Run: `cargo test --package jsonquill --lib jsonpath::evaluator::tests`
Expected: All 10 tests pass

**Step 7: Commit evaluator implementation**

```bash
git add src/jsonpath/evaluator.rs
git commit -m "feat(jsonpath): implement evaluator for JSONPath queries

- Evaluate all core path segments against JSON tree
- Support child access, array indexing, wildcards
- Support recursive descent with optional property name
- Support array slicing with negative indices
- Add comprehensive test coverage (10 tests)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Add SearchType to EditorState

**Files:**
- Modify: `src/editor/state.rs`

**Step 1: Add SearchType enum**

Add to `src/editor/state.rs` after imports:

```rust
/// Type of active search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchType {
    /// Text-based search (/ or ?)
    Text,
    /// JSONPath structural search (:path or :jp)
    JsonPath(String), // Store the query string for display
}
```

**Step 2: Add search_type field to EditorState**

In `src/editor/state.rs`, modify the `EditorState` struct (around line 143):

```rust
pub struct EditorState {
    tree: JsonTree,
    mode: EditorMode,
    cursor: Cursor,
    dirty: bool,
    filename: Option<String>,
    tree_view: TreeViewState,
    message: Option<Message>,
    command_buffer: String,
    show_help: bool,
    help_scroll: usize,
    pending_theme: Option<String>,
    current_theme: String,
    clipboard: Option<JsonNode>,
    clipboard_key: Option<String>,
    search_buffer: String,
    search_results: Vec<Vec<usize>>,
    search_index: usize,
    search_forward: bool,
    search_type: Option<SearchType>,  // Add this field
    show_line_numbers: bool,
    enable_mouse: bool,
    edit_buffer: Option<String>,
    edit_cursor: usize,
    cursor_visible: bool,
    cursor_blink_ticks: u8,
    pending_command: Option<char>,
    pending_count: Option<u32>,
    scroll_offset: usize,
    viewport_height: usize,
    undo_tree: super::undo::UndoTree,
    add_mode_stage: AddModeStage,
}
```

**Step 3: Initialize search_type in new()**

Find the `EditorState::new()` method and add initialization:

```rust
    pub fn new(tree: JsonTree) -> Self {
        let mut tree_view = TreeViewState::default();
        tree_view.set_tree(&tree);

        Self {
            tree,
            mode: EditorMode::default(),
            cursor: Cursor::default(),
            dirty: false,
            filename: None,
            tree_view,
            message: None,
            command_buffer: String::new(),
            show_help: false,
            help_scroll: 0,
            pending_theme: None,
            current_theme: "default-dark".to_string(),
            clipboard: None,
            clipboard_key: None,
            search_buffer: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            search_forward: true,
            search_type: None,  // Add this
            show_line_numbers: true,
            enable_mouse: true,
            edit_buffer: None,
            edit_cursor: 0,
            cursor_visible: true,
            cursor_blink_ticks: 0,
            pending_command: None,
            pending_count: None,
            scroll_offset: 0,
            viewport_height: 0,
            undo_tree: super::undo::UndoTree::new(50),
            add_mode_stage: AddModeStage::None,
        }
    }
```

**Step 4: Update execute_search to set search_type**

Find the `execute_search()` method and modify it:

```rust
    pub fn execute_search(&mut self) {
        if self.search_buffer.is_empty() {
            return;
        }

        let query = self.search_buffer.to_lowercase();
        self.search_results.clear();
        self.search_index = 0;
        self.search_type = Some(SearchType::Text);  // Add this line

        // ... rest of the method unchanged ...
    }
```

**Step 5: Add getter for search_type**

Add method to `EditorState`:

```rust
    /// Returns the current search type, if any.
    pub fn search_type(&self) -> Option<&SearchType> {
        self.search_type.as_ref()
    }
```

**Step 6: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 7: Commit SearchType addition**

```bash
git add src/editor/state.rs
git commit -m "feat(search): add SearchType enum to track search mode

- Add SearchType enum with Text and JsonPath variants
- Add search_type field to EditorState
- Update execute_search to set SearchType::Text
- Add search_type() getter method

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Implement JSONPath Search in EditorState

**Files:**
- Modify: `src/editor/state.rs`

**Step 1: Add execute_jsonpath_search method**

Add to `src/editor/state.rs`:

```rust
    /// Executes a JSONPath query and populates search results.
    pub fn execute_jsonpath_search(&mut self, query: &str) {
        use crate::jsonpath::{Evaluator, Parser};

        self.search_results.clear();
        self.search_index = 0;

        // Parse the JSONPath query
        let path = match Parser::parse(query) {
            Ok(p) => p,
            Err(e) => {
                self.set_message(format!("Invalid JSONPath: {}", e), MessageLevel::Error);
                return;
            }
        };

        // Evaluate against the tree
        let evaluator = Evaluator::new(&self.tree);
        self.search_results = evaluator.evaluate(&path);

        // Set search type
        self.search_type = Some(SearchType::JsonPath(query.to_string()));

        // Jump to first result or show message
        if !self.search_results.is_empty() {
            self.cursor.set_path(self.search_results[0].clone());
            self.set_message(
                format!("Found {} matches for {}", self.search_results.len(), query),
                MessageLevel::Info,
            );
        } else {
            self.set_message(format!("No matches for {}", query), MessageLevel::Info);
        }
    }
```

**Step 2: Update clear_search_buffer to clear search_type**

Find the `clear_search_buffer()` method and modify:

```rust
    pub fn clear_search_buffer(&mut self) {
        self.search_buffer.clear();
        self.search_results.clear();
        self.search_type = None;  // Add this line
    }
```

**Step 3: Write integration test**

Create `tests/jsonpath_tests.rs`:

```rust
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;

#[test]
fn test_jsonpath_search_basic() {
    let root = JsonNode::new(JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
        ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
    ]));

    let tree = JsonTree::new(root);
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.name");

    assert_eq!(state.search_results_info(), Some((1, 1)));
    assert_eq!(state.cursor().path(), &[0]);
}

#[test]
fn test_jsonpath_search_wildcard() {
    let root = JsonNode::new(JsonValue::Object(vec![
        (
            "users".to_string(),
            JsonNode::new(JsonValue::Array(vec![
                JsonNode::new(JsonValue::String("Alice".to_string())),
                JsonNode::new(JsonValue::String("Bob".to_string())),
            ])),
        ),
    ]));

    let tree = JsonTree::new(root);
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.users[*]");

    assert_eq!(state.search_results_info(), Some((1, 2)));
}

#[test]
fn test_jsonpath_search_recursive_descent() {
    let root = JsonNode::new(JsonValue::Object(vec![(
        "store".to_string(),
        JsonNode::new(JsonValue::Object(vec![(
            "book".to_string(),
            JsonNode::new(JsonValue::Array(vec![
                JsonNode::new(JsonValue::Object(vec![(
                    "price".to_string(),
                    JsonNode::new(JsonValue::Number(10.0)),
                )])),
                JsonNode::new(JsonValue::Object(vec![(
                    "price".to_string(),
                    JsonNode::new(JsonValue::Number(20.0)),
                )])),
            ])),
        )])),
    )]));

    let tree = JsonTree::new(root);
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$..price");

    assert_eq!(state.search_results_info(), Some((1, 2)));
}

#[test]
fn test_jsonpath_search_no_match() {
    let root = JsonNode::new(JsonValue::Object(vec![(
        "name".to_string(),
        JsonNode::new(JsonValue::String("Alice".to_string())),
    )]));

    let tree = JsonTree::new(root);
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.nonexistent");

    assert_eq!(state.search_results_info(), None);
}

#[test]
fn test_jsonpath_search_invalid_syntax() {
    let root = JsonNode::new(JsonValue::Object(vec![]));
    let tree = JsonTree::new(root);
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("invalid");

    // Should set error message, no results
    assert_eq!(state.search_results_info(), None);
}
```

**Step 4: Run integration tests**

Run: `cargo test --test jsonpath_tests`
Expected: All 5 tests pass

**Step 5: Commit JSONPath search integration**

```bash
git add src/editor/state.rs tests/jsonpath_tests.rs
git commit -m "feat(search): add JSONPath search to EditorState

- Add execute_jsonpath_search() method
- Parse and evaluate JSONPath queries
- Populate search_results with matching paths
- Show success/error messages
- Add integration tests for JSONPath search

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Add Command Handlers

**Files:**
- Modify: `src/input/handler.rs`

**Step 1: Add :path and :jp command handlers**

In `src/input/handler.rs`, find the `execute_command()` method and add before the final `match command` statement (around line 930):

```rust
        // Handle :path and :jp commands
        if let Some(query) = command.strip_prefix("path ") {
            let query = query.trim();
            if query.is_empty() {
                state.set_message("Usage: :path <jsonpath>".to_string(), MessageLevel::Error);
            } else {
                state.execute_jsonpath_search(query);
            }
            return Ok(false);
        }

        if let Some(query) = command.strip_prefix("jp ") {
            let query = query.trim();
            if query.is_empty() {
                state.set_message("Usage: :jp <jsonpath>".to_string(), MessageLevel::Error);
            } else {
                state.execute_jsonpath_search(query);
            }
            return Ok(false);
        }

        // Handle :find command (alias for /)
        if command == "find" {
            state.set_mode(EditorMode::Search);
            state.set_search_forward(true);
            state.clear_search_buffer();
            return Ok(false);
        }
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 3: Write manual test script**

Create `test_jsonpath_commands.sh`:

```bash
#!/bin/bash
# Manual test script for JSONPath commands

set -e

echo "Testing JSONPath commands..."

# Create test JSON file
cat > /tmp/test_jsonpath.json <<EOF
{
  "store": {
    "book": [
      {"title": "Book1", "price": 10},
      {"title": "Book2", "price": 20}
    ]
  }
}
EOF

echo "Created test file: /tmp/test_jsonpath.json"
echo ""
echo "Manual tests to run in jsonquill:"
echo "1. Open: cargo run /tmp/test_jsonpath.json"
echo "2. Test :path \$.store.book[*].title"
echo "3. Test :jp \$..price"
echo "4. Test :find (should enter search mode)"
echo "5. Press 'n' to cycle through results"
echo ""
echo "Expected: Each command shows matches and allows navigation with 'n'"
```

**Step 4: Make test script executable**

Run: `chmod +x test_jsonpath_commands.sh`

**Step 5: Commit command handlers**

```bash
git add src/input/handler.rs test_jsonpath_commands.sh
git commit -m "feat(commands): add :path, :jp, and :find commands

- Add :path <query> command for JSONPath search
- Add :jp <query> as short alias
- Add :find command as alias for entering search mode (/)
- Add manual test script for verification

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Update Status Line for Search Type

**Files:**
- Modify: `src/ui/status_line.rs`

**Step 1: Read current status line implementation**

Read `src/ui/status_line.rs` to understand current rendering.

**Step 2: Update status line to show search type**

Find the part that renders search info in `src/ui/status_line.rs` and modify to show search type:

```rust
    // Add search results info if available
    if let Some((current, total)) = state.search_results_info() {
        let search_info = match state.search_type() {
            Some(crate::editor::state::SearchType::Text) => {
                format!(" [Search: \"{}\"] Match {}/{}", state.search_buffer(), current, total)
            }
            Some(crate::editor::state::SearchType::JsonPath(query)) => {
                format!(" [JSONPath: {}] Match {}/{}", query, current, total)
            }
            None => format!(" Match {}/{}", current, total),
        };
        status_parts.push(search_info);
    }
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Add test for status line rendering**

Add to `tests/editor_tests.rs` (or create if doesn't exist):

```rust
#[test]
fn test_status_line_shows_jsonpath_search() {
    use jsonquill::document::node::{JsonNode, JsonValue};
    use jsonquill::document::tree::JsonTree;
    use jsonquill::editor::state::EditorState;

    let root = JsonNode::new(JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
    ]));

    let tree = JsonTree::new(root);
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.name");

    // Verify search type is set
    assert!(matches!(
        state.search_type(),
        Some(jsonquill::editor::state::SearchType::JsonPath(_))
    ));
}
```

**Step 5: Run test**

Run: `cargo test test_status_line_shows_jsonpath_search`
Expected: Test passes

**Step 6: Commit status line update**

```bash
git add src/ui/status_line.rs tests/editor_tests.rs
git commit -m "feat(ui): show search type in status line

- Display [Search: \"query\"] for text search
- Display [JSONPath: query] for JSONPath search
- Maintain match counter display
- Add test for JSONPath status display

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Update Documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `README.md`

**Step 1: Update CLAUDE.md with JSONPath info**

In `CLAUDE.md`, update the TODO section to mark structural search as complete:

```markdown
**Advanced Features:**
- ✅ **Structural search** - `:path`, `:jp` for JSONPath-style queries
- ❌ **No named registers** - `"ayy`, `"ap` for named register operations
- ❌ **No format preservation** - Original formatting not preserved on save
- ❌ **No lazy loading** - Large files (≥100MB) not optimized
- ❌ **No advanced undo** - `g-`/`g+`, `:earlier`/`:later`, `:undolist` not implemented
```

Add JSONPath commands to the Commands section:

```markdown
:path <query> - Search using JSONPath query (e.g., :path $.store.book[*].author)
:jp <query>   - Short alias for :path
:find         - Enter text search mode (same as /)
```

Add JSONPath examples to Usage section:

```markdown
# JSONPath Search (structural search)
:path $.store.book[*].author  - Find all book authors
:jp $..price                  - Find all price fields anywhere
:path $.items[0:3]            - First 3 items
:path $.user['name','email']  - Multiple properties
n                             - Navigate to next match
```

**Step 2: Update README.md**

Add JSONPath section to README.md after the Search section:

```markdown
### JSONPath Search (Structural Search)

JSONPath queries allow you to search by structure rather than text:

| Command | Action | Example |
|---------|--------|---------|
| `:path <query>` | JSONPath structural search | `:path $.store.book[*].author` |
| `:jp <query>` | Short alias for `:path` | `:jp $..price` |
| `:find` | Enter text search mode | Same as pressing `/` |

**Supported JSONPath Syntax:**

- `$` - Root node
- `.property` - Named property
- `['property']` - Bracket notation
- `[index]` - Array index (supports negative: `[-1]` = last)
- `[*]` - All children
- `..property` - Recursive descent (find anywhere)
- `[start:end]` - Array slicing
- `['prop1','prop2']` - Multiple properties

**Examples:**

```bash
:path $.store.book[*].author    # All book authors
:jp $..price                    # All price fields anywhere
:path $.items[0:3]              # First 3 array items
:path $.user['name','email']    # Multiple properties
:jp $.data[*].id                # All IDs in data array
```

After executing a JSONPath search, use `n` to navigate through matches just like text search.
```

**Step 3: Verify markdown formatting**

Run: `cargo build` (ensures no syntax errors in doc comments)
Expected: Builds successfully

**Step 4: Commit documentation updates**

```bash
git add CLAUDE.md README.md
git commit -m "docs: add JSONPath structural search documentation

- Mark structural search as complete in CLAUDE.md
- Add JSONPath commands to command reference
- Add JSONPath syntax and examples to README.md
- Document supported operators and usage patterns

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Integration Testing and Polish

**Files:**
- Create: `tests/integration_jsonpath.rs`

**Step 1: Write comprehensive integration tests**

Create `tests/integration_jsonpath.rs`:

```rust
//! Integration tests for JSONPath search functionality.

use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::{EditorState, SearchType};

fn make_bookstore() -> JsonTree {
    let book1 = JsonNode::new(JsonValue::Object(vec![
        (
            "title".to_string(),
            JsonNode::new(JsonValue::String("Rust Programming".to_string())),
        ),
        ("price".to_string(), JsonNode::new(JsonValue::Number(39.99))),
        (
            "author".to_string(),
            JsonNode::new(JsonValue::String("Steve Klabnik".to_string())),
        ),
    ]));

    let book2 = JsonNode::new(JsonValue::Object(vec![
        (
            "title".to_string(),
            JsonNode::new(JsonValue::String("JavaScript Guide".to_string())),
        ),
        ("price".to_string(), JsonNode::new(JsonValue::Number(29.99))),
        (
            "author".to_string(),
            JsonNode::new(JsonValue::String("Douglas Crockford".to_string())),
        ),
    ]));

    let books = JsonNode::new(JsonValue::Array(vec![book1, book2]));

    let store = JsonNode::new(JsonValue::Object(vec![("book".to_string(), books)]));

    let root = JsonNode::new(JsonValue::Object(vec![
        ("store".to_string(), store),
        (
            "owner".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        ),
    ]));

    JsonTree::new(root)
}

#[test]
fn test_jsonpath_all_authors() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.store.book[*].author");

    assert_eq!(state.search_results_info(), Some((1, 2)));
    assert!(matches!(
        state.search_type(),
        Some(SearchType::JsonPath(_))
    ));

    // Navigate through results
    let first_match = state.cursor().path().to_vec();
    state.next_search_result();
    let second_match = state.cursor().path().to_vec();

    assert_ne!(first_match, second_match);
}

#[test]
fn test_jsonpath_all_prices() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$..price");

    // Should find both price fields
    assert_eq!(state.search_results_info(), Some((1, 2)));
}

#[test]
fn test_jsonpath_first_book() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.store.book[0]");

    assert_eq!(state.search_results_info(), Some((1, 1)));
}

#[test]
fn test_jsonpath_last_book() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.store.book[-1]");

    assert_eq!(state.search_results_info(), Some((1, 1)));
}

#[test]
fn test_jsonpath_slice() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.store.book[0:1]");

    assert_eq!(state.search_results_info(), Some((1, 1)));
}

#[test]
fn test_jsonpath_wildcard() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.store.*");

    // Should find the book array
    assert_eq!(state.search_results_info(), Some((1, 1)));
}

#[test]
fn test_jsonpath_multi_property() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.store.book[0]['title','author']");

    // Should find both title and author of first book
    assert_eq!(state.search_results_info(), Some((1, 2)));
}

#[test]
fn test_jsonpath_no_results() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.nonexistent");

    assert_eq!(state.search_results_info(), None);
}

#[test]
fn test_jsonpath_invalid_query() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("invalid query");

    // Should have no results due to parse error
    assert_eq!(state.search_results_info(), None);
}

#[test]
fn test_text_search_replaces_jsonpath() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    // First do JSONPath search
    state.execute_jsonpath_search("$.store.book[*]");
    assert!(matches!(
        state.search_type(),
        Some(SearchType::JsonPath(_))
    ));

    // Then do text search - should replace
    state.push_to_search_buffer('R');
    state.push_to_search_buffer('u');
    state.push_to_search_buffer('s');
    state.push_to_search_buffer('t');
    state.execute_search();

    assert!(matches!(state.search_type(), Some(SearchType::Text)));
}

#[test]
fn test_jsonpath_navigation_wraps() {
    let tree = make_bookstore();
    let mut state = EditorState::new(tree);

    state.execute_jsonpath_search("$.store.book[*]");

    assert_eq!(state.search_results_info(), Some((1, 2)));

    // Navigate to second
    state.next_search_result();
    assert_eq!(state.search_results_info(), Some((2, 2)));

    // Navigate wraps to first
    state.next_search_result();
    assert_eq!(state.search_results_info(), Some((1, 2)));
}
```

**Step 2: Run integration tests**

Run: `cargo test --test integration_jsonpath`
Expected: All 12 tests pass

**Step 3: Run all tests**

Run: `cargo test`
Expected: All tests pass (including new JSONPath tests)

**Step 4: Run clippy**

Run: `cargo clippy`
Expected: No warnings or errors

**Step 5: Run formatter**

Run: `cargo fmt`
Expected: Code formatted successfully

**Step 6: Commit integration tests and final polish**

```bash
git add tests/integration_jsonpath.rs
git commit -m "test: add comprehensive JSONPath integration tests

- Test all JSONPath operators (wildcards, recursive, slicing)
- Test navigation through results
- Test search type switching (JSONPath ↔ text)
- Test error handling for invalid queries
- 12 integration tests covering end-to-end functionality

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Final Verification

**Files:**
- All project files

**Step 1: Run pre-commit checklist**

Run:
```bash
cargo fmt
cargo clippy
cargo test
```

Expected: All pass with no warnings

**Step 2: Build release binary**

Run: `cargo build --release`
Expected: Builds successfully

**Step 3: Manual smoke test**

Create test file:
```bash
cat > /tmp/test_jsonpath_final.json <<EOF
{
  "users": [
    {"name": "Alice", "age": 30, "email": "alice@example.com"},
    {"name": "Bob", "age": 25, "email": "bob@example.com"}
  ],
  "config": {
    "theme": "dark",
    "settings": {
      "notifications": true,
      "theme": "light"
    }
  }
}
EOF
```

Test commands:
```bash
./target/release/jsonquill /tmp/test_jsonpath_final.json

# In jsonquill, test:
# :path $.users[*].name
# :jp $..theme
# :find (should enter search mode)
# n (navigate through results)
```

**Step 4: Verify all features work**

Checklist:
- [ ] `:path $.users[*].name` finds both names
- [ ] `:jp $..theme` finds both theme fields
- [ ] `:find` enters search mode
- [ ] `n` navigates through results
- [ ] Status line shows `[JSONPath: query]` format
- [ ] Error messages for invalid queries
- [ ] Text search still works (`/`, `?`)

**Step 5: Update CLAUDE.md with final status**

Verify CLAUDE.md shows:
```markdown
- ✅ **Structural search** - `:path`, `:jp` for JSONPath-style queries
```

**Step 6: Final commit**

```bash
git add -A
git commit -m "feat: complete JSONPath structural search implementation

Adds full JSONPath query support for structural navigation:

Features:
- Core JSONPath syntax (child, index, wildcard, recursive, slice)
- Commands: :path <query>, :jp <query>, :find
- Integration with existing search (n for next match)
- Status line shows search type and query
- Error messages for invalid syntax

Implementation:
- New jsonpath module (parser, evaluator, AST, errors)
- SearchType enum tracks Text vs JSONPath search
- Reuses existing search_results infrastructure
- 30+ tests covering parser, evaluator, integration

Documentation:
- Updated CLAUDE.md and README.md
- Examples and syntax reference
- Manual test scripts

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Plan Complete

**Plan saved to:** `docs/plans/2026-01-25-jsonpath-structural-search.md`

**Summary:**
- 11 tasks covering module setup, parser, evaluator, integration, testing, docs
- Each task broken into 4-7 steps (2-5 minutes each)
- TDD approach: write tests, implement, verify
- Frequent commits after each task
- Comprehensive test coverage (parser, evaluator, integration)
- Full documentation updates

**Estimated effort:** 2-3 hours for experienced Rust developer

**Next steps:** Execute this plan using the executing-plans or subagent-driven-development skill.
