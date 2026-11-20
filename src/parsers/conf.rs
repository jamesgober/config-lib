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

/// Split a section name at its dots: `[a.b]` is the table `b` inside `a`
/// (since v1.4.0; before, it was one key named `a.b`).
fn section_path(name: &str, line: usize, column: usize) -> Result<Vec<String>> {
    let parts: Vec<String> = name.split('.').map(|p| p.trim().to_string()).collect();
    if parts.iter().any(String::is_empty) {
        return Err(Error::parse(
            "A section name has an empty part",
            line,
            column,
        ));
    }
    Ok(parts)
}

fn section_clash(line: usize, column: usize) -> Error {
    Error::parse("A section has the name of a key above it", line, column)
}

fn table_is_array(line: usize, column: usize) -> Error {
    Error::parse(
        "A [section] has the name of a [[list]] of tables above it",
        line,
        column,
    )
}

fn array_is_table(line: usize, column: usize) -> Error {
    Error::parse(
        "A [[list]] of tables has the name of a [section] or key above it",
        line,
        column,
    )
}

/// The tables an `[[a]]` header builds: an array whose items are all
/// tables. CONF has no inline tables, so only `[[...]]` makes one.
fn is_table_array(items: &[Value]) -> bool {
    !items.is_empty() && items.iter().all(|item| matches!(item, Value::Table(_)))
}

/// The table one step into `value` for a path through it: a table itself,
/// or the last table of an `[[a]]` list, as in TOML (`[a.b]` after
/// `[[a]]` belongs to the last `a`). `None` for anything else.
fn step_into(value: &mut Value) -> Option<&mut BTreeMap<String, Value>> {
    match value {
        Value::Table(inner) => Some(inner),
        Value::Array(items) if is_table_array(items) => match items.last_mut() {
            Some(Value::Table(inner)) => Some(inner),
            _ => None,
        },
        _ => None,
    }
}

/// What a header opens: a `[section]` or a new item of a `[[list]]`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Header {
    Table,
    ArrayItem,
}

/// Take the table at `path` out of `root` so a section can add to it, or
/// start an empty one. For `[[a]]` it is always a new, empty item, after
/// a check that `a` is a list of tables or nothing yet. A non-table in the
/// way is refused, and so is mixing `[a]` and `[[a]]`.
fn open_section(
    root: &mut BTreeMap<String, Value>,
    path: &[String],
    header: Header,
    line: usize,
    column: usize,
) -> Result<BTreeMap<String, Value>> {
    let (last, parents) = match path.split_last() {
        Some(split) => split,
        None => return Ok(BTreeMap::new()),
    };
    let mut table = root;
    for part in parents {
        table = match table.get_mut(part.as_str()) {
            None => return Ok(BTreeMap::new()),
            Some(value) => step_into(value).ok_or_else(|| section_clash(line, column))?,
        };
    }
    match (header, table.get(last.as_str())) {
        (Header::ArrayItem, None) => Ok(BTreeMap::new()),
        (Header::ArrayItem, Some(Value::Array(items))) if is_table_array(items) => {
            Ok(BTreeMap::new())
        }
        (Header::ArrayItem, Some(_)) => Err(array_is_table(line, column)),
        (Header::Table, Some(Value::Array(items))) if is_table_array(items) => {
            Err(table_is_array(line, column))
        }
        (Header::Table, _) => match table.remove(last.as_str()) {
            None => Ok(BTreeMap::new()),
            Some(Value::Table(inner)) => Ok(inner),
            Some(other) => {
                table.insert(last.clone(), other);
                Err(section_clash(line, column))
            }
        },
    }
}

/// Put a section's table back at its path, or add it to the end of its
/// `[[list]]`. A section with no keys is kept as an empty table, as TOML
/// and NOML keep one (since v1.5.0), so a caller that checks section names
/// sees it: a misspelled `[auth.passwrd]` with nothing under it is an
/// unknown section, not nothing.
fn close_section(root: &mut BTreeMap<String, Value>, open: OpenSection) -> Result<()> {
    let OpenSection {
        path,
        table: section,
        header,
        line,
        column,
    } = open;
    let (last, parents) = match path.split_last() {
        Some(split) => split,
        None => return Ok(()),
    };
    let mut table = root;
    for part in parents {
        let entry = table
            .entry(part.clone())
            .or_insert_with(|| Value::table(BTreeMap::new()));
        table = step_into(entry).ok_or_else(|| section_clash(line, column))?;
    }
    match header {
        Header::Table => {
            table.insert(last.clone(), Value::table(section));
        }
        Header::ArrayItem => match table
            .entry(last.clone())
            .or_insert_with(|| Value::array(Vec::new()))
        {
            Value::Array(items) => items.push(Value::table(section)),
            _ => return Err(array_is_table(line, column)),
        },
    }
    Ok(())
}

/// A section being read: its path, its table, what opened it, and the
/// header's line and column.
struct OpenSection {
    path: Vec<String>,
    table: BTreeMap<String, Value>,
    header: Header,
    line: usize,
    column: usize,
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
        // The open section's path, table and header position, kept out of
        // `root` while its keys are read: each key goes straight into it,
        // and the name is not copied per key.
        let mut section: Option<OpenSection> = None;

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                break;
            }

            // Check for section header
            if self.peek() == Some('[') {
                let (line, column) = (self.line, self.column);
                let (name, header) = self.parse_section_header()?;
                let path = section_path(&name, line, column)?;
                if let Some(open) = section.take() {
                    close_section(&mut root, open)?;
                }
                // A section seen before continues where it left off. One
                // named like a key above it used to lose its keys
                // silently; it is refused. `[[a]]` starts a new item of
                // the list `a` (since v1.5.0).
                let table = open_section(&mut root, &path, header, line, column)?;
                section = Some(OpenSection {
                    path,
                    table,
                    header,
                    line,
                    column,
                });
                continue;
            }

            // Parse key-value pair
            let (key, value) = self.parse_key_value()?;

            match &mut section {
                Some(open) => {
                    open.table.insert(key, value);
                }
                None => {
                    root.insert(key, value);
                }
            }
        }

        if let Some(open) = section {
            close_section(&mut root, open)?;
        }
        Ok(Value::table(root))
    }

    /// Parse a section header, `[name]` or `[[name]]`
    fn parse_section_header(&mut self) -> Result<(String, Header)> {
        self.expect('[')?;
        let header = if self.peek() == Some('[') {
            self.advance();
            Header::ArrayItem
        } else {
            Header::Table
        };
        let start = self.position;

        // Find the closing bracket
        self.advance_while(|ch| ch != ']' && ch != '\n');
        if self.peek() != Some(']') {
            return Err(Error::parse(
                "Unterminated section header",
                self.line,
                self.column,
            ));
        }

        let section_name = self.since(start).trim().to_string();
        self.expect(']')?;
        if header == Header::ArrayItem && self.peek() != Some(']') {
            return Err(Error::parse(
                "A [[list]] header ends with ]]",
                self.line,
                self.column,
            ));
        }
        if header == Header::ArrayItem {
            self.advance();
        }

        Ok((section_name, header))
    }

    /// Parse a key-value pair
    fn parse_key_value(&mut self) -> Result<(String, Value)> {
        let key = self.parse_key()?;
        self.skip_whitespace();
        self.expect('=')?;
        self.skip_whitespace();
        let value = self.parse_value()?;

        // Only a comment may follow a value on its line, so a line such as
        // `tls = "on" tls = off` is an error, not two assignments
        self.skip_whitespace();
        if !matches!(self.peek(), None | Some('\n' | '\r' | '#')) {
            return Err(Error::parse(
                "Unexpected text after the value",
                self.line,
                self.column,
            ));
        }

        Ok((key, value))
    }

    /// Parse a configuration key
    fn parse_key(&mut self) -> Result<String> {
        let start = self.position;

        self.advance_while(|ch| ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.');

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

        // Read until end of line, comment, or special character. A `#`
        // starts a comment only at the start of the value or after
        // whitespace, so `abc#def` is one value (since v1.4.0).
        let after_space = std::cell::Cell::new(true);
        self.advance_while(|ch| {
            let stop = ch == '\n'
                || ch == '\r'
                || (ch == '#' && after_space.get())
                || (in_array && (ch == ',' || ch == ']'));
            after_space.set(ch == ' ' || ch == '\t');
            !stop
        });

        let raw_value = self.since(start).trim();

        if raw_value.is_empty() {
            return Ok(Value::null());
        }

        // Try to parse as boolean or null. The words are ASCII and no
        // other character lower-cases to one of their letters, so an
        // ASCII comparison matches what `to_lowercase` did, without
        // allocating a lowered copy of every value.
        let is = |word: &str| raw_value.eq_ignore_ascii_case(word);
        if is("true") || is("yes") || is("on") {
            return Ok(Value::bool(true));
        }
        if is("false") || is("no") || is("off") {
            return Ok(Value::bool(false));
        }
        if is("null") || is("nil") {
            return Ok(Value::null());
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

        // Try float: decimal syntax only, so `nan`, `inf` and `infinity`
        // stay strings, and an integer too large for i64 is a string
        // rather than a rounded float
        if let Some(f) = super::decimal_float(value) {
            return Ok(Value::float(f));
        }

        // Default to string
        Ok(Value::string(value.to_string()))
    }

    /// Skip whitespace but not newlines
    fn skip_whitespace(&mut self) {
        self.advance_while(|ch| ch == ' ' || ch == '\t');
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();

            // Skip comments
            if self.peek() == Some('#') {
                self.advance_while(|ch| ch != '\n');
                self.advance();
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

    /// Advance over every character for which `keep` holds.
    ///
    /// An ASCII byte is checked and stepped over directly; anything else
    /// is decoded as a character, so `position` stays on a character
    /// boundary and `line` and `column` count as [`advance`] does.
    ///
    /// [`advance`]: ConfParser::advance
    fn advance_while(&mut self, keep: impl Fn(char) -> bool) {
        let bytes = self.input.as_bytes();
        while let Some(&byte) = bytes.get(self.position) {
            if byte.is_ascii() {
                let ch = char::from(byte);
                if !keep(ch) {
                    break;
                }
                self.position += 1;
                if byte == b'\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
            } else {
                match self.peek() {
                    Some(ch) if keep(ch) => {
                        self.advance();
                    }
                    _ => break,
                }
            }
        }
    }

    /// Expect a specific character
    fn expect(&mut self, expected: char) -> Result<()> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            // The character found is not quoted back
            Some(_) => Err(Error::parse(
                format!("Expected '{expected}'"),
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
    fn test_a_repeated_section_continues_and_an_empty_one_is_kept() {
        let value = parse("[a]\nx = 1\n[empty]\n[b]\ny = 2\n[a]\nz = 3\n").unwrap();
        let root = value.as_table().unwrap();
        assert_eq!(root.len(), 3);
        assert!(root["empty"].as_table().unwrap().is_empty());
        let a = root["a"].as_table().unwrap();
        assert_eq!(a["x"].as_integer().unwrap(), 1);
        assert_eq!(a["z"].as_integer().unwrap(), 3);
        assert_eq!(root["b"].as_table().unwrap()["y"].as_integer().unwrap(), 2);
    }

    #[test]
    fn test_an_empty_dotted_section_is_an_empty_table_like_toml() {
        // v1.5.0: `[auth.passwrd]` with no keys used to vanish, so a check
        // of the section's keys never saw the misspelling.
        let value =
            parse("[auth]\nkey = 1\n[auth.passwrd]\n[content.fields.post.author]\n").unwrap();
        let auth = value.get("auth").unwrap().as_table().unwrap();
        assert_eq!(auth.len(), 2);
        assert!(auth["passwrd"].as_table().unwrap().is_empty());
        let author = value.get("content.fields.post.author").unwrap();
        assert!(author.as_table().unwrap().is_empty());
        // A section after its empty parent adds to the same table.
        let value = parse("[a]\n[a.b]\nx = 1\n").unwrap();
        assert_eq!(value.get("a.b.x").unwrap().as_integer().unwrap(), 1);
        // An empty section named like a key above it is still refused.
        assert!(parse("db = on\n[db]\n").is_err());
    }

    fn items<'v>(value: &'v Value, path: &str) -> &'v [Value] {
        value.get(path).unwrap().as_array().unwrap()
    }

    #[test]
    fn test_lists_of_tables_read_as_toml_does() {
        // v1.5.0: `[[name]]` used to fail on its first line.
        let text = "\
[server]
port = 1

[[upstream]]
host = \"a\"
weight = 2

[[upstream]]
host = \"b\"
[upstream.tls]
verify = true
[[upstream.path]]
prefix = \"/api\"
[[upstream.path]]
prefix = \"/web\"

[limits]
rate = 10
";
        let value = parse(text).unwrap();
        assert_eq!(value.get("server.port").unwrap().as_integer().unwrap(), 1);
        assert_eq!(value.get("limits.rate").unwrap().as_integer().unwrap(), 10);
        let upstream = items(&value, "upstream");
        assert_eq!(upstream.len(), 2);
        let first = upstream[0].as_table().unwrap();
        assert_eq!(first["host"].as_string().unwrap(), "a");
        assert_eq!(first["weight"].as_integer().unwrap(), 2);
        assert!(!first.contains_key("tls"));
        // `[upstream.tls]` and `[[upstream.path]]` belong to the last item.
        let second = upstream[1].as_table().unwrap();
        assert_eq!(second["host"].as_string().unwrap(), "b");
        assert!(second["tls"].as_table().unwrap()["verify"]
            .as_bool()
            .unwrap());
        let paths = second["path"].as_array().unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(
            paths[1].as_table().unwrap()["prefix"].as_string().unwrap(),
            "/web"
        );

        // The same text through TOML gives the same value.
        #[cfg(feature = "toml")]
        assert_eq!(crate::parsers::toml_parser::parse(text).unwrap(), value);
    }

    #[test]
    fn test_an_empty_list_item_is_kept_and_lists_nest_in_sections() {
        let value =
            parse("[[rule]]\n[[rule]]\nname = x\n[site]\n[[site.host]]\nname = a\n").unwrap();
        let rules = items(&value, "rule");
        assert_eq!(rules.len(), 2);
        assert!(rules[0].as_table().unwrap().is_empty());
        assert_eq!(items(&value, "site.host").len(), 1);
    }

    #[test]
    fn test_mixing_a_list_with_a_section_or_key_is_refused_with_its_position() {
        for (text, line, column) in [
            ("[a]\nk = 1\n[[a]]\n", 3, 1),
            ("[[a]]\nk = 1\n[a]\n", 3, 1),
            ("a = 1\n\n[[a]]\n", 3, 1),
            ("a = [1, 2]\n[[a]]\n", 2, 1),
            ("a = [1, 2]\n[a.b]\n", 2, 1),
            ("[[a]\nk = 1\n", 1, 5),
            ("[[a\n", 1, 4),
        ] {
            match parse(text) {
                Err(Error::Parse {
                    line: l, column: c, ..
                }) => assert_eq!((l, c), (line, column), "{text:?}"),
                other => panic!("expected a parse error for {text:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_a_section_named_like_a_key_is_refused() {
        // Its keys used to be dropped without a word.
        match parse("db = on\n[db]\npassword = hunter2\n") {
            Err(Error::Parse { line, message, .. }) => {
                assert_eq!(line, 2);
                assert!(!message.contains("hunter2"));
            }
            other => panic!("expected a parse error, got {other:?}"),
        }
    }

    #[test]
    fn test_words_match_in_any_ascii_case() {
        let value = parse("a = TRUE\nb = Off\nc = NiL\nd = truth\ne = Ünïcode\n").unwrap();
        let root = value.as_table().unwrap();
        assert!(root["a"].as_bool().unwrap());
        assert!(!root["b"].as_bool().unwrap());
        assert!(root["c"].is_null());
        assert_eq!(root["d"].as_string().unwrap(), "truth");
        assert_eq!(root["e"].as_string().unwrap(), "Ünïcode");
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
