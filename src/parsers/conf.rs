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

/// Deepest nesting of `[...]` arrays the parser accepts.
///
/// Arrays are parsed recursively, so without a limit a value such as a
/// long run of `[` would overflow the stack. 128 matches the XML parser's
/// element limit and jsonx-lib's default depth.
const MAX_ARRAY_DEPTH: usize = 128;

/// CONF parser state.
///
/// `position` is a byte offset into `input` and always sits on a UTF-8
/// character boundary: it only moves by the encoded length of the
/// character under it. `line` and `column` count lines and characters
/// (from 1) for error messages.
struct ConfParser<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
    depth: usize,
}

impl<'a> ConfParser<'a> {
    /// Create a new parser
    fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
            depth: 0,
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

        let section_name = self.since(start).trim().to_string();
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

        Ok(self.since(start).to_string())
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

        let content = self.since(start).to_string();
        self.expect('\'')?;
        Ok(Value::string(content))
    }

    /// Parse an array [item1, item2, item3]
    fn parse_array(&mut self) -> Result<Value> {
        if self.depth >= MAX_ARRAY_DEPTH {
            return Err(Error::parse(
                format!("Arrays are nested deeper than the limit of {MAX_ARRAY_DEPTH} levels"),
                self.line,
                self.column,
            ));
        }
        self.depth += 1;
        let result = self.parse_array_items();
        self.depth -= 1;
        result
    }

    /// Parse the brackets and items of one array (see [`Self::parse_array`])
    fn parse_array_items(&mut self) -> Result<Value> {
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

        let number_str = self.since(start);

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

    /// Parse an unquoted value (string, boolean, or array). Inside
    /// brackets an item ends at `,` or `]`, so `[80, 443]` is two items.
    fn parse_unquoted_value(&mut self) -> Result<Value> {
        let start = self.position;
        let in_array = self.depth > 0;

        // Read until end of line, comment, or special character
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' || ch == '#' || (in_array && (ch == ',' || ch == ']')) {
                break;
            }
            self.advance();
        }

        let raw_value = self.since(start).trim();

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

        // Check if it's a space or comma separated array (not inside
        // brackets, where the brackets make the array)
        if !in_array && (raw_value.contains(' ') || raw_value.contains(',')) {
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

    /// Peek at the current character.
    ///
    /// O(1): `position` is a byte offset on a character boundary, so this
    /// decodes one character instead of counting from the start.
    fn peek(&self) -> Option<char> {
        self.input.get(self.position..)?.chars().next()
    }

    /// The input from byte offset `start` to the current position.
    ///
    /// Both ends are character boundaries, because positions only move by
    /// whole characters; the empty fallback is never taken.
    fn since(&self, start: usize) -> &'a str {
        self.input.get(start..self.position).unwrap_or("")
    }

    /// Advance to the next character
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.position += ch.len_utf8();
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
        let config = parse("int = 42\nfloat = 1.234").unwrap();
        assert_eq!(config.get("int").unwrap().as_integer().unwrap(), 42);
        assert_eq!(config.get("float").unwrap().as_float().unwrap(), 1.234);
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
    fn test_bracket_arrays_of_unquoted_items() {
        let config = parse("ports = [80, 443]\nhosts = [a, b c]\nnested = [[1, 2], [3]]").unwrap();
        let ports = config.get("ports").unwrap().as_array().unwrap();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[1].as_integer().unwrap(), 443);
        let hosts = config.get("hosts").unwrap().as_array().unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[1].as_string().unwrap(), "b c");
        let nested = config.get("nested").unwrap().as_array().unwrap();
        assert_eq!(nested.len(), 2);
        assert_eq!(nested[0].as_array().unwrap().len(), 2);
        // Without brackets, spaces and commas still make a list.
        let config = parse("list = 1, 2 3").unwrap();
        assert_eq!(config.get("list").unwrap().as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_comments() {
        let config = parse("# This is a comment\nkey = value # inline comment").unwrap();
        assert_eq!(config.get("key").unwrap().as_string().unwrap(), "value");
    }

    #[test]
    fn non_ascii_comment_before_section_does_not_panic() {
        // Used to slice at a character count instead of a byte offset
        let config = parse("#\u{e9}\u{e9}\u{e9}\n[a]\nk = 1\n").unwrap();
        assert_eq!(config.get("a.k").unwrap().as_integer().unwrap(), 1);
    }

    #[test]
    fn non_ascii_keys_values_and_comments() {
        let source = "# caf\u{e9} \u{4e2d}\u{6587}\n\
                      n\u{e4}me = J\u{fc}rgen # \u{2713}\n\
                      [se\u{e7}\u{e3}o]\n\
                      \u{43a}\u{43b}\u{44e}\u{447} = '\u{1F600} single'\n\
                      quoted = \"\u{3b1}\\t\u{3b2}\"\n\
                      list = [\"\u{e9}\", '\u{fc}']\n";
        let config = parse(source).unwrap();
        assert_eq!(
            config.get("n\u{e4}me").unwrap().as_string().unwrap(),
            "J\u{fc}rgen"
        );
        let section = config.get("se\u{e7}\u{e3}o").unwrap();
        assert_eq!(
            section
                .get("\u{43a}\u{43b}\u{44e}\u{447}")
                .unwrap()
                .as_string()
                .unwrap(),
            "\u{1F600} single"
        );
        assert_eq!(
            section.get("quoted").unwrap().as_string().unwrap(),
            "\u{3b1}\t\u{3b2}"
        );
        let list = section.get("list").unwrap().as_array().unwrap();
        assert_eq!(list[0].as_string().unwrap(), "\u{e9}");
        assert_eq!(list[1].as_string().unwrap(), "\u{fc}");
    }

    #[test]
    fn error_columns_count_characters() {
        // Four two-byte characters and four one-byte characters give the
        // same column: columns count characters, not bytes.
        let wide = parse("\u{43a}\u{43b}\u{44e}\u{447} ? 1").unwrap_err();
        let narrow = parse("abcd ? 1").unwrap_err();
        for err in [wide, narrow] {
            match err {
                Error::Parse { line, column, .. } => assert_eq!((line, column), (1, 7)),
                other => panic!("expected a parse error, got {other:?}"),
            }
        }
        match parse("# \u{e9}\u{e9}\n\u{e9} = 1\n[open\n").unwrap_err() {
            Error::Parse { line, .. } => assert_eq!(line, 3),
            other => panic!("expected a parse error, got {other:?}"),
        }
    }

    #[test]
    fn deep_array_nesting_is_an_error() {
        let ok = format!(
            "k = {}{}",
            "[".repeat(MAX_ARRAY_DEPTH),
            "]".repeat(MAX_ARRAY_DEPTH)
        );
        assert!(parse(&ok).is_ok());
        let deep = format!("k = {}", "[".repeat(MAX_ARRAY_DEPTH + 1));
        assert!(parse(&deep).is_err());
        // A million levels is an error, not a stack overflow
        assert!(parse(&format!("k = {}", "[".repeat(1_000_000))).is_err());
    }
}
