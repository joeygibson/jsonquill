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
