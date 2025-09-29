//! # CONF Format Parser
//!
//! High-performance parser for standard .conf configuration files.
//!
//! Supports the common configuration format used by many Unix/Linux applications:
//!
//! ```conf
//! # Comments start with #
//! key = value
//! quoted_value = "string with spaces"
//! number = 42
//! float = 3.14
//! boolean = true
//!
//! # Sections
//! [section]
//! nested_key = value
//!
//! # Arrays (space or comma separated)
//! array = item1 item2 item3
//! comma_array = item1, item2, item3
//! ```

use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::BTreeMap;

/// Parse CONF format configuration
pub fn parse(source: &str) -> Result<Value> {
    let mut parser = ConfParser::new(source);
    parser.parse()
}

/// High-performance CONF parser with zero-allocation lexing
/// CONF parser state
struct ConfParser<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> ConfParser<'a> {
    /// Create a new parser
    fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Parse the entire configuration
    fn parse(&mut self) -> Result<Value> {
        let mut root = BTreeMap::new();
        let mut current_section = None;

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                break;
            }

            // Check for section header
            if self.peek() == Some('[') {
                current_section = Some(self.parse_section_header()?);
                continue;
            }

            // Parse key-value pair
            let (key, value) = self.parse_key_value()?;

            match &current_section {
                Some(section) => {
                    // Add to section
                    let section_table = root
                        .entry(section.clone())
                        .or_insert_with(|| Value::table(BTreeMap::new()));

                    if let Value::Table(table) = section_table {
                        table.insert(key, value);
                    }
                }
                None => {
                    // Add to root
                    root.insert(key, value);
                }
            }
        }

        Ok(Value::table(root))
    }

    /// Parse a section header like [section_name]
    fn parse_section_header(&mut self) -> Result<String> {
        self.expect('[')?;
        let start = self.position;

        // Find the closing bracket
        while let Some(ch) = self.peek() {
            if ch == ']' {
                break;
            }
            if ch == '\n' {
                return Err(Error::parse(
                    "Unterminated section header",
                    self.line,
                    self.column,
                ));
            }
            self.advance();
        }

        let section_name = self.input[start..self.position].trim().to_string();
        self.expect(']')?;

        Ok(section_name)
    }

    /// Parse a key-value pair
    fn parse_key_value(&mut self) -> Result<(String, Value)> {
        let key = self.parse_key()?;
        self.skip_whitespace();
        self.expect('=')?;
        self.skip_whitespace();
        let value = self.parse_value()?;

        Ok((key, value))
    }

    /// Parse a configuration key
    fn parse_key(&mut self) -> Result<String> {
        let start = self.position;

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
                self.advance();
            } else {
                break;
            }
        }

        if start == self.position {
            return Err(Error::parse("Expected key name", self.line, self.column));
        }

        Ok(self.input[start..self.position].to_string())
    }

    /// Parse a configuration value
    fn parse_value(&mut self) -> Result<Value> {
        self.skip_whitespace();

        match self.peek() {
            Some('"') => self.parse_quoted_string(),
            Some('\'') => self.parse_single_quoted_string(),
            Some('[') => self.parse_array(),
            _ => {
                // For all other cases (including numbers), use unquoted value parsing
                // which handles space-separated arrays
                self.parse_unquoted_value()
            }
        }
    }

    /// Parse a quoted string
    fn parse_quoted_string(&mut self) -> Result<Value> {
        self.expect('"')?;
        let _start = self.position;
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                break;
            }
            if ch == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => {
                        return Err(Error::parse(
                            "Unterminated escape sequence",
                            self.line,
                            self.column,
                        ))
                    }
                }
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }

        self.expect('"')?;
        Ok(Value::string(result))
    }

    /// Parse a single-quoted string (no escape sequences)
    fn parse_single_quoted_string(&mut self) -> Result<Value> {
        self.expect('\'')?;
        let start = self.position;

        while let Some(ch) = self.peek() {
            if ch == '\'' {
                break;
            }
            self.advance();
        }

        let content = self.input[start..self.position].to_string();
        self.expect('\'')?;
        Ok(Value::string(content))
    }

    /// Parse an array [item1, item2, item3]
    fn parse_array(&mut self) -> Result<Value> {
        self.expect('[')?;
        let mut items = Vec::new();

        self.skip_whitespace();

        if self.peek() == Some(']') {
            self.advance();
            return Ok(Value::array(items));
        }

        loop {
            items.push(self.parse_value()?);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(Error::parse(
                        "Expected ',' or ']' in array",
                        self.line,
                        self.column,
                    ))
                }
            }
        }

        Ok(Value::array(items))
    }

    /// Parse a number (integer or float)
    #[allow(dead_code)]
    fn parse_number(&mut self) -> Result<Value> {
        let start = self.position;
        let mut has_dot = false;

        // Handle sign
        if self.peek() == Some('-') || self.peek() == Some('+') {
            self.advance();
        }

        // Parse digits and optional decimal point
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let number_str = &self.input[start..self.position];

        if has_dot {
            number_str.parse::<f64>().map(Value::float).map_err(|_| {
                Error::parse(
                    format!("Invalid float: {number_str}"),
                    self.line,
                    self.column,
                )
            })
        } else {
            number_str.parse::<i64>().map(Value::integer).map_err(|_| {
                Error::parse(
                    format!("Invalid integer: {number_str}"),
                    self.line,
                    self.column,
                )
            })
        }
    }

    /// Parse an unquoted value (string, boolean, or array)
    fn parse_unquoted_value(&mut self) -> Result<Value> {
        let start = self.position;

        // Read until end of line, comment, or special character
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' || ch == '#' {
                break;
            }
            self.advance();
        }

        let raw_value = self.input[start..self.position].trim();

        if raw_value.is_empty() {
            return Ok(Value::null());
        }

        // Try to parse as boolean
        match raw_value.to_lowercase().as_str() {
            "true" | "yes" | "on" => return Ok(Value::bool(true)),
            "false" | "no" | "off" => return Ok(Value::bool(false)),
            "null" | "nil" | "" => return Ok(Value::null()),
            _ => {}
        }

        // Check if it's a space or comma separated array
        if raw_value.contains(' ') || raw_value.contains(',') {
            let items: Vec<Value> = raw_value
                .split([' ', ','])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| self.parse_simple_value(s))
                .collect::<Result<Vec<_>>>()?;

            if items.len() > 1 {
                return Ok(Value::array(items));
            }
        }

        // Parse as simple value
        self.parse_simple_value(raw_value)
    }

    /// Parse a simple value (no arrays or complex types)
    fn parse_simple_value(&self, value: &str) -> Result<Value> {
        // Try integer
        if let Ok(i) = value.parse::<i64>() {
            return Ok(Value::integer(i));
        }

        // Try float
        if let Ok(f) = value.parse::<f64>() {
            return Ok(Value::float(f));
        }

        // Default to string
        Ok(Value::string(value.to_string()))
    }

    /// Skip whitespace but not newlines
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();

            // Skip comments
            if self.peek() == Some('#') {
                while let Some(ch) = self.peek() {
                    self.advance();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }

            // Skip newlines
            if self.peek() == Some('\n') || self.peek() == Some('\r') {
                self.advance();
                continue;
            }

            break;
        }
    }

    /// Peek at the current character
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /// Advance to the next character
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    /// Expect a specific character
    fn expect(&mut self, expected: char) -> Result<()> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(Error::parse(
                format!("Expected '{expected}', found '{ch}'"),
                self.line,
                self.column,
            )),
            None => Err(Error::parse(
                format!("Expected '{expected}', found end of input"),
                self.line,
                self.column,
            )),
        }
    }

    /// Check if we're at the end of input
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_key_value() {
        let config = parse("key = value").unwrap();
        assert_eq!(config.get("key").unwrap().as_string().unwrap(), "value");
    }

    #[test]
    fn test_numbers() {
        let config = parse("int = 42\nfloat = 3.14").unwrap();
        assert_eq!(config.get("int").unwrap().as_integer().unwrap(), 42);
        assert_eq!(config.get("float").unwrap().as_float().unwrap(), 3.14);
    }

    #[test]
    fn test_booleans() {
        let config = parse("bool1 = true\nbool2 = false").unwrap();
        assert!(config.get("bool1").unwrap().as_bool().unwrap());
        assert!(!config.get("bool2").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_quoted_strings() {
        let config = parse(r#"quoted = "hello world""#).unwrap();
        assert_eq!(
            config.get("quoted").unwrap().as_string().unwrap(),
            "hello world"
        );
    }

    #[test]
    fn test_sections() {
        let config = parse("[section]\nkey = value").unwrap();
        assert_eq!(
            config.get("section.key").unwrap().as_string().unwrap(),
            "value"
        );
    }

    #[test]
    fn test_arrays() {
        let config = parse("arr = item1 item2 item3").unwrap();
        let arr = config.get("arr").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_string().unwrap(), "item1");
    }

    #[test]
    fn test_comments() {
        let config = parse("# This is a comment\nkey = value # inline comment").unwrap();
        assert_eq!(config.get("key").unwrap().as_string().unwrap(), "value");
    }
}
