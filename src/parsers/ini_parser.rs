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
        while self.position < self.content.len() {
            self.skip_whitespace_and_comments()?;

            if self.position >= self.content.len() {
                break;
            }

            let ch = self.current_char();

            match ch {
                '[' => self.parse_section()?,
                '\n' | '\r' => {
                    self.advance();
                    self.line += 1;
                }
                _ => self.parse_key_value()?,
            }
        }

        Ok(Value::Table(self.result.clone()))
    }

    fn current_char(&self) -> char {
        self.content.chars().nth(self.position).unwrap_or('\0')
    }

    fn advance(&mut self) {
        if self.position < self.content.len() {
            self.position += 1;
        }
    }

    // Commented out to avoid unused warnings - could be useful for future enhancements
    // fn peek_char(&self, offset: usize) -> char {
    //     self.content.chars().nth(self.position + offset).unwrap_or('\0')
    // }

    fn skip_whitespace_and_comments(&mut self) -> Result<()> {
        loop {
            let ch = self.current_char();

            match ch {
                ' ' | '\t' => self.advance(),
                ';' | '#' => {
                    // Skip comment until end of line
                    while self.current_char() != '\n' && self.current_char() != '\0' {
                        self.advance();
                    }
                }
                '\n' | '\r' => {
                    self.advance();
                    self.line += 1;
                }
                '\0' => break,
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_section(&mut self) -> Result<()> {
        self.advance(); // Skip '['
        let start = self.position;

        // Find closing bracket
        while self.current_char() != ']' && self.current_char() != '\0' {
            if self.current_char() == '\n' {
                return Err(Error::Parse {
                    message: "Unterminated section".to_string(),
                    line: self.line,
                    column: 1,
                    file: None,
                });
            }
            self.advance();
        }

        if self.current_char() != ']' {
            return Err(Error::Parse {
                message: "Missing closing bracket for section".to_string(),
                line: self.line,
                column: 1,
                file: None,
            });
        }

        let section_name = self.content[start..self.position].trim().to_string();
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
            return Ok(()); // Skip empty lines
        }

        self.skip_whitespace_and_comments()?;

        let ch = self.current_char();
        if ch != '=' && ch != ':' {
            return Err(Error::Parse {
                message: format!("Expected '=' or ':' after key '{key}'"),
                line: self.line,
                column: 1,
                file: None,
            });
        }

        self.advance(); // Skip separator
        self.skip_whitespace_and_comments()?;

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

        while self.position < self.content.len() {
            let ch = self.current_char();
            match ch {
                '=' | ':' | '\n' | '\r' | '\0' => break,
                ';' | '#' => break, // Comment starts
                _ => self.advance(),
            }
        }

        let key = self.content[start..self.position].trim();
        Ok(key.to_string())
    }

    fn parse_value(&mut self) -> Result<Value> {
        let mut value_chars = Vec::new();
        let mut in_quotes = false;
        let mut quote_char = '\0';

        while self.position < self.content.len() {
            let ch = self.current_char();

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
                    if self.position < self.content.len() {
                        let escaped_char = self.current_char();
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
                '\n' | '\r' | '\0' if !in_quotes => break,
                ';' | '#' if !in_quotes => break, // Comment starts
                _ => {
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
    fn test_sections() {
        let content = r#"
[section1]
key1=value1

[section2]
key2=value2
        "#;

        let result = parse_ini(content).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(
                map.get("section1.key1").unwrap().as_string().unwrap(),
                "value1"
            );
            assert_eq!(
                map.get("section2.key2").unwrap().as_string().unwrap(),
                "value2"
            );
        } else {
            panic!("Expected table");
        }
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
    fn test_error_handling() {
        // Test unterminated section
        let content = "[section";
        assert!(parse_ini(content).is_err());

        // Test invalid key-value
        let content = "key_without_value";
        assert!(parse_ini(content).is_err());
    }
}
