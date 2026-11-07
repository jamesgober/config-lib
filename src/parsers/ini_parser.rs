//! INI format parser implementation
//!
//! Supports standard INI format with:
//! - Sections: \[section_name\]
//! - Key-value pairs: key=value or key:value
//! - Comments: ; comment or # comment
//! - Escape sequences: \n, \t, \\, etc.
//! - Quoted values with spaces
//! - Case-sensitive keys and sections

use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::BTreeMap;

/// Parse INI format configuration
pub fn parse(source: &str) -> Result<Value> {
    parse_ini(source)
}

/// Parse INI format string into a Value::Table
pub fn parse_ini(content: &str) -> Result<Value> {
    let mut parser = IniParser::new(content);
    parser.parse()
}

/// INI parser state.
///
/// `position` is a byte offset into `content` and always sits on a UTF-8
/// character boundary: it only moves by the encoded length of the
/// character under it.
struct IniParser<'a> {
    content: &'a str,
    position: usize,
    line: usize,
    current_section: Option<String>,
    result: BTreeMap<String, Value>,
}

impl<'a> IniParser<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            content,
            position: 0,
            line: 1,
            current_section: None,
            result: BTreeMap::new(),
        }
    }

    fn parse(&mut self) -> Result<Value> {
        loop {
            self.skip_whitespace_and_comments()?;

            match self.peek() {
                None => break,
                Some('[') => self.parse_section()?,
                Some(_) => self.parse_key_value()?,
            }
        }

        Ok(Value::Table(std::mem::take(&mut self.result)))
    }

    /// The character at the current position, or `None` at the end.
    ///
    /// O(1): `position` is a byte offset on a character boundary, so this
    /// decodes one character instead of counting from the start.
    fn peek(&self) -> Option<char> {
        self.content.get(self.position..)?.chars().next()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.peek() {
            self.position += ch.len_utf8();
        }
    }

    /// The input from byte offset `start` to the current position.
    ///
    /// Both ends are character boundaries, because positions only move by
    /// whole characters; the empty fallback is never taken.
    fn since(&self, start: usize) -> &'a str {
        self.content.get(start..self.position).unwrap_or("")
    }

    /// Skip spaces and tabs on the current line
    fn skip_spaces(&mut self) {
        while let Some(' ' | '\t') = self.peek() {
            self.advance();
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<()> {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r') => self.advance(),
                Some(';' | '#') => {
                    // Skip comment until end of line
                    while !matches!(self.peek(), None | Some('\n')) {
                        self.advance();
                    }
                }
                Some('\n') => {
                    self.advance();
                    self.line += 1;
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_section(&mut self) -> Result<()> {
        self.advance(); // Skip '['
        let start = self.position;

        // Find closing bracket
        while !matches!(self.peek(), None | Some(']')) {
            if self.peek() == Some('\n') {
                return Err(Error::Parse {
                    message: "Unterminated section".to_string(),
                    line: self.line,
                    column: 1,
                    file: None,
                });
            }
            self.advance();
        }

        if self.peek() != Some(']') {
            return Err(Error::Parse {
                message: "Missing closing bracket for section".to_string(),
                line: self.line,
                column: 1,
                file: None,
            });
        }

        let section_name = self.since(start).trim().to_string();
        self.advance(); // Skip ']'

        if section_name.is_empty() {
            return Err(Error::Parse {
                message: "Empty section name".to_string(),
                line: self.line,
                column: 1,
                file: None,
            });
        }

        self.current_section = Some(section_name);
        Ok(())
    }

    fn parse_key_value(&mut self) -> Result<()> {
        let key = self.parse_key()?;

        if key.is_empty() {
            // A line that starts with '=' or ':'. Returning without
            // consuming anything used to loop forever here.
            return Err(Error::Parse {
                message: "Expected a key name".to_string(),
                line: self.line,
                column: 1,
                file: None,
            });
        }

        // The separator and the value are on the key's line: an empty
        // value must not take the next line as its text.
        self.skip_spaces();

        if !matches!(self.peek(), Some('=' | ':')) {
            return Err(Error::Parse {
                message: format!("Expected '=' or ':' after key '{key}'"),
                line: self.line,
                column: 1,
                file: None,
            });
        }

        self.advance(); // Skip separator
        self.skip_spaces();

        let value = self.parse_value()?;

        // Store the key-value pair
        let full_key = match &self.current_section {
            Some(section) => format!("{section}.{key}"),
            None => key,
        };

        self.result.insert(full_key, value);
        Ok(())
    }

    fn parse_key(&mut self) -> Result<String> {
        let start = self.position;

        while let Some(ch) = self.peek() {
            match ch {
                '=' | ':' | '\n' | '\r' => break,
                ';' | '#' => break, // Comment starts
                _ => self.advance(),
            }
        }

        let key = self.since(start).trim();
        Ok(key.to_string())
    }

    fn parse_value(&mut self) -> Result<Value> {
        let mut value_chars = Vec::new();
        let mut in_quotes = false;
        let mut quote_char = '\0';

        while let Some(ch) = self.peek() {
            match ch {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                    self.advance();
                    // Don't include the opening quote
                }
                '\\' if in_quotes => {
                    // Handle escape sequences within quotes
                    self.advance(); // Skip backslash
                    if let Some(escaped_char) = self.peek() {
                        match escaped_char {
                            'n' => value_chars.push('\n'),
                            't' => value_chars.push('\t'),
                            'r' => value_chars.push('\r'),
                            '\\' => value_chars.push('\\'),
                            '"' => value_chars.push('"'),
                            '\'' => value_chars.push('\''),
                            _ => {
                                value_chars.push('\\');
                                value_chars.push(escaped_char);
                            }
                        }
                        self.advance();
                    }
                }
                ch if in_quotes && ch == quote_char => {
                    in_quotes = false;
                    self.advance();
                    // Don't include the closing quote
                    break;
                }
                '\n' | '\r' if !in_quotes => break,
                ';' | '#' if !in_quotes => break, // Comment starts
                _ => {
                    if ch == '\n' {
                        // A line end inside quotes
                        self.line += 1;
                    }
                    value_chars.push(ch);
                    self.advance();
                }
            }
        }

        // If we're not in quotes, trim whitespace from the end
        let value_str = if !in_quotes {
            value_chars
                .iter()
                .collect::<String>()
                .trim_end()
                .to_string()
        } else {
            value_chars.iter().collect::<String>()
        };

        // For unquoted values, still process escape sequences
        let processed_value = if in_quotes {
            value_str // Already processed during parsing
        } else {
            self.process_escape_sequences(&value_str)
        };

        // Try to parse as different types
        self.parse_typed_value(&processed_value)
    }

    fn process_escape_sequences(&self, value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.peek() {
                    Some('n') => {
                        chars.next();
                        result.push('\n');
                    }
                    Some('t') => {
                        chars.next();
                        result.push('\t');
                    }
                    Some('r') => {
                        chars.next();
                        result.push('\r');
                    }
                    Some('\\') => {
                        chars.next();
                        result.push('\\');
                    }
                    Some('"') => {
                        chars.next();
                        result.push('"');
                    }
                    Some('\'') => {
                        chars.next();
                        result.push('\'');
                    }
                    _ => result.push(ch),
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn parse_typed_value(&self, value: &str) -> Result<Value> {
        if value.is_empty() {
            return Ok(Value::String(String::new()));
        }

        // Try boolean
        match value.to_lowercase().as_str() {
            "true" | "yes" | "on" | "1" => return Ok(Value::Bool(true)),
            "false" | "no" | "off" | "0" => return Ok(Value::Bool(false)),
            _ => {}
        }

        // Try integer
        if let Ok(int_val) = value.parse::<i64>() {
            return Ok(Value::Integer(int_val));
        }

        // Try float
        if let Ok(float_val) = value.parse::<f64>() {
            return Ok(Value::Float(float_val));
        }

        // Default to string
        Ok(Value::String(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_ini() {
        let content = r#"
key1=value1
key2=value2
        "#;

        let result = parse_ini(content).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(map.get("key1").unwrap().as_string().unwrap(), "value1");
            assert_eq!(map.get("key2").unwrap().as_string().unwrap(), "value2");
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_sections() -> crate::Result<()> {
        let content = r#"
[section1]
key1=value1

[section2]
key2=value2
        "#;

        let result = parse_ini(content)?;
        if let Value::Table(map) = result {
            let key1 = map
                .get("section1.key1")
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: "section1.key1".to_string(),
                    available: map.keys().cloned().collect(),
                })?;
            assert_eq!(key1.as_string()?, "value1");

            let key2 = map
                .get("section2.key2")
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: "section2.key2".to_string(),
                    available: map.keys().cloned().collect(),
                })?;
            assert_eq!(key2.as_string()?, "value2");
        } else {
            return Err(crate::Error::Parse {
                message: "Expected table".to_string(),
                line: 0,
                column: 0,
                file: None,
            });
        }
        Ok(())
    }

    #[test]
    fn test_comments() {
        let content = r#"
; This is a comment
key1=value1  ; Inline comment
# Hash comment
key2=value2
        "#;

        let result = parse_ini(content).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(map.get("key1").unwrap().as_string().unwrap(), "value1");
            assert_eq!(map.get("key2").unwrap().as_string().unwrap(), "value2");
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_quoted_values() {
        let content = r#"
key1="quoted value"
key2='single quoted'
key3="value with spaces"
        "#;

        let result = parse_ini(content).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(
                map.get("key1").unwrap().as_string().unwrap(),
                "quoted value"
            );
            assert_eq!(
                map.get("key2").unwrap().as_string().unwrap(),
                "single quoted"
            );
            assert_eq!(
                map.get("key3").unwrap().as_string().unwrap(),
                "value with spaces"
            );
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_escape_sequences() {
        let content = r#"
key1="line1\nline2"
key2="tab\there"
key3="quote\"here"
        "#;

        let result = parse_ini(content).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(
                map.get("key1").unwrap().as_string().unwrap(),
                "line1\nline2"
            );
            assert_eq!(map.get("key2").unwrap().as_string().unwrap(), "tab\there");
            assert_eq!(map.get("key3").unwrap().as_string().unwrap(), "quote\"here");
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_data_types() {
        let content = r#"
string_val=hello
int_val=42
float_val=1.234
bool_true=true
bool_false=false
bool_yes=yes
bool_no=no
        "#;

        let result = parse_ini(content).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(map.get("string_val").unwrap().as_string().unwrap(), "hello");
            assert_eq!(map.get("int_val").unwrap().as_integer().unwrap(), 42);
            assert_eq!(map.get("float_val").unwrap().as_float().unwrap(), 1.234);
            assert!(map.get("bool_true").unwrap().as_bool().unwrap());
            assert!(!map.get("bool_false").unwrap().as_bool().unwrap());
            assert!(map.get("bool_yes").unwrap().as_bool().unwrap());
            assert!(!map.get("bool_no").unwrap().as_bool().unwrap());
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_colon_separator() {
        let content = r#"
key1:value1
key2:value2
        "#;

        let result = parse_ini(content).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(map.get("key1").unwrap().as_string().unwrap(), "value1");
            assert_eq!(map.get("key2").unwrap().as_string().unwrap(), "value2");
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn non_ascii_text_parses() {
        let content = "; caf\u{e9}\n[se\u{e7}\u{e3}o]\nn\u{e4}me = J\u{fc}rgen ; \u{2713}\n\
                       quoted = \"\u{3b1} \u{3b2}\"\n";
        let Value::Table(map) = parse_ini(content).unwrap() else {
            panic!("Expected table");
        };
        assert_eq!(
            map.get("se\u{e7}\u{e3}o.n\u{e4}me")
                .unwrap()
                .as_string()
                .unwrap(),
            "J\u{fc}rgen"
        );
        assert_eq!(
            map.get("se\u{e7}\u{e3}o.quoted")
                .unwrap()
                .as_string()
                .unwrap(),
            "\u{3b1} \u{3b2}"
        );
    }

    #[test]
    fn line_starting_with_separator_is_an_error() {
        // The first three used to loop forever; a NUL used to read as the end
        for content in ["=x\n", ":x\n", "a=1\n= 2\n", "\0"] {
            assert!(parse_ini(content).is_err(), "{content:?}");
        }
    }

    #[test]
    fn empty_value_does_not_take_the_next_line() {
        let Value::Table(map) = parse_ini("a=\nb=2\n").unwrap() else {
            panic!("Expected table");
        };
        assert_eq!(map.get("a").unwrap().as_string().unwrap(), "");
        assert_eq!(map.get("b").unwrap().as_integer().unwrap(), 2);
    }

    #[test]
    fn crlf_counts_one_line() {
        match parse_ini("a=1\r\nb=2\r\n[open\r\n").unwrap_err() {
            Error::Parse { line, .. } => assert_eq!(line, 3),
            other => panic!("expected a parse error, got {other:?}"),
        }
    }

    #[test]
    fn test_error_handling() {
        // Test unterminated section
        let content = "[section";
        assert!(parse_ini(content).is_err());

        // Test invalid key-value
        let content = "key_without_value";
        assert!(parse_ini(content).is_err());
    }
}
