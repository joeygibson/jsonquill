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
        let mut segments = Vec::new();

        self.skip_whitespace();

        // Expect root ($)
        if self.peek() != Some('$') {
            return Err(JsonPathError::InvalidSyntax {
                message: "JSONPath must start with '$'".to_string(),
            });
        }
        self.next();
        segments.push(PathSegment::Root);

        // Parse remaining segments
        while !self.is_eof() {
            self.skip_whitespace();
            match self.peek() {
                Some('.') => {
                    self.next();
                    if self.peek() == Some('.') {
                        segments.push(self.parse_recursive_descent()?);
                    } else if self.peek() == Some('*') {
                        self.next();
                        segments.push(PathSegment::Wildcard);
                    } else {
                        let name = self.parse_identifier()?;
                        segments.push(PathSegment::Child(name));
                    }
                }
                Some('[') => {
                    segments.push(self.parse_bracket_expression()?);
                }
                _ => break,
            }
        }

        Ok(JsonPath::new(segments))
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
        let pos = self.position; // Save position before advancing
        match self.next() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(JsonPathError::UnexpectedToken {
                position: pos, // Use saved position
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
        self.skip_whitespace();
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

    /// Parses recursive descent (..)
    fn parse_recursive_descent(&mut self) -> Result<PathSegment, JsonPathError> {
        self.expect('.')?;
        if self.peek() == Some('[') {
            Ok(PathSegment::RecursiveDescent(None))
        } else if self.peek() == Some('*') {
            self.next();
            Ok(PathSegment::RecursiveDescent(None))
        } else {
            let name = self.parse_identifier()?;
            Ok(PathSegment::RecursiveDescent(Some(name)))
        }
    }

    /// Parses bracket expression: [index], [start:end], ['key'], [*]
    fn parse_bracket_expression(&mut self) -> Result<PathSegment, JsonPathError> {
        self.expect('[')?;
        self.skip_whitespace();

        let segment = match self.peek() {
            Some('*') => {
                self.next();
                self.skip_whitespace();
                self.expect(']')?;
                PathSegment::Wildcard
            }
            Some('\'') | Some('"') => {
                let properties = self.parse_bracket_string()?;
                self.skip_whitespace();
                self.expect(']')?;
                if properties.len() == 1 {
                    PathSegment::Child(properties.into_iter().next().unwrap())
                } else {
                    PathSegment::MultiProperty(properties)
                }
            }
            Some('-') | Some('0'..='9') => {
                // Check if this looks like a slice by peeking ahead for ':'
                let saved_pos = self.position;
                let mut looks_like_slice = false;
                while !self.is_eof() {
                    match self.peek() {
                        Some(':') => {
                            looks_like_slice = true;
                            break;
                        }
                        Some(']') => break,
                        Some(_) => {
                            self.next();
                        }
                        None => break,
                    }
                }
                self.position = saved_pos; // restore position

                if looks_like_slice {
                    self.parse_slice()?
                } else {
                    let idx = self.parse_bracket_number()?;
                    self.skip_whitespace();
                    self.expect(']')?;
                    PathSegment::Index(idx)
                }
            }
            Some(':') => self.parse_slice()?,
            _ => {
                return Err(JsonPathError::InvalidSyntax {
                    message: "Invalid bracket expression".to_string(),
                })
            }
        };

        Ok(segment)
    }

    /// Parses string(s) inside brackets: ['key'] or ['key1','key2']
    fn parse_bracket_string(&mut self) -> Result<Vec<String>, JsonPathError> {
        let mut properties = Vec::new();
        loop {
            self.skip_whitespace();
            let quote = match self.peek() {
                Some('\'') | Some('"') => self.next().unwrap(),
                _ => break,
            };

            let mut value = String::new();
            loop {
                match self.next() {
                    Some(ch) if ch == quote => break,
                    Some('\\') => match self.next() {
                        Some('n') => value.push('\n'),
                        Some('t') => value.push('\t'),
                        Some('r') => value.push('\r'),
                        Some('\\') => value.push('\\'),
                        Some('\'') => value.push('\''),
                        Some('"') => value.push('"'),
                        Some(_) | None => {
                            return Err(JsonPathError::InvalidSyntax {
                                message: "Invalid escape sequence".to_string(),
                            })
                        }
                    },
                    Some(ch) => value.push(ch),
                    None => {
                        return Err(JsonPathError::UnexpectedEnd {
                            expected: format!("closing quote '{}'", quote),
                        })
                    }
                }
            }
            properties.push(value);

            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.next();
            } else {
                break;
            }
        }
        Ok(properties)
    }

    /// Parses a number inside brackets
    fn parse_bracket_number(&mut self) -> Result<isize, JsonPathError> {
        let num_str = self.parse_number_string()?;
        num_str
            .parse::<isize>()
            .map_err(|_| JsonPathError::InvalidSyntax {
                message: format!("Invalid number: {}", num_str),
            })
    }

    /// Parses a number as a string
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

    /// Parses array slice: [start:end], [start:], [:end], [:]
    fn parse_slice(&mut self) -> Result<PathSegment, JsonPathError> {
        let start = if self.peek() == Some(':') {
            None
        } else {
            Some(self.parse_bracket_number()?)
        };

        self.skip_whitespace();
        self.expect(':')?;
        self.skip_whitespace();

        let end = if self.peek() == Some(']') {
            None
        } else {
            Some(self.parse_bracket_number()?)
        };

        self.skip_whitespace();
        self.expect(']')?;

        // Validate slice bounds
        if let (Some(s), Some(e)) = (start, end) {
            if s >= 0 && e >= 0 && s > e {
                return Err(JsonPathError::InvalidSyntax {
                    message: format!("Invalid slice: start ({}) > end ({})", s, e),
                });
            }
        }

        Ok(PathSegment::Slice(start, end))
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

    #[test]
    fn test_parse_negative_index() {
        let result = Parser::parse("$.items[-1]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Index(-1));
    }

    #[test]
    fn test_parse_slice_full() {
        let result = Parser::parse("$.items[1:3]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Slice(Some(1), Some(3)));
    }

    #[test]
    fn test_parse_slice_start_only() {
        let result = Parser::parse("$.items[2:]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Slice(Some(2), None));
    }

    #[test]
    fn test_parse_slice_end_only() {
        let result = Parser::parse("$.items[:5]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments[2], PathSegment::Slice(None, Some(5)));
    }

    #[test]
    fn test_parse_multi_property() {
        let result = Parser::parse("$.store['book','music']");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(
            path.segments[2],
            PathSegment::MultiProperty(vec!["book".to_string(), "music".to_string()])
        );
    }

    #[test]
    fn test_parse_bracket_notation() {
        let result = Parser::parse("$['store']['book']");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[1], PathSegment::Child("store".to_string()));
        assert_eq!(path.segments[2], PathSegment::Child("book".to_string()));
    }

    #[test]
    fn test_parse_recursive_descent_wildcard() {
        let result = Parser::parse("$..*");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[1], PathSegment::RecursiveDescent(None));
    }

    #[test]
    fn test_parse_complex_path() {
        let result = Parser::parse("$.store..price[0:2]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 4);
        assert_eq!(path.segments[1], PathSegment::Child("store".to_string()));
        assert_eq!(
            path.segments[2],
            PathSegment::RecursiveDescent(Some("price".to_string()))
        );
        assert_eq!(path.segments[3], PathSegment::Slice(Some(0), Some(2)));
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let result = Parser::parse("$ . store [ 0 ]");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[1], PathSegment::Child("store".to_string()));
        assert_eq!(path.segments[2], PathSegment::Index(0));
    }
}
