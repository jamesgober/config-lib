use crate::{Error, Result, Value};use crate::{Error, Result, Value};

use std::collections::BTreeMap;use std::collections::BTreeMap;



/// Represents a span in the source text for error reporting/// Zero-copy token representing a slice of the input

#[derive(Debug, Clone, PartialEq)]#[derive(Debug, Clone, Copy, PartialEq)]

pub struct Span {pub enum Token<'a> {

    pub start: usize,    // Structural tokens

    pub end: usize,    LeftBracket,      // [

    pub line: usize,    RightBracket,     // ]

    pub column: usize,    Equals,           // =

}    Newline,          // \n

    Eof,              // End of input

impl Span {    

    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {    // Value tokens (zero-copy slices)

        Self { start, end, line, column }    Identifier(&'a str),      // key names, unquoted values

    }    String(&'a str),          // "quoted string" content only

}    Integer(&'a str),         // raw number text

    Float(&'a str),           // raw float text

/// Zero-copy tokens for maximum performance    Boolean(&'a str),         // true/false

#[derive(Debug, Clone, PartialEq)]    

pub enum Token<'a> {    // Whitespace and comments (skipped in parsing)

    // Value tokens    Whitespace(&'a str),

    String(&'a str),    Comment(&'a str),

    Integer(&'a str),}

    Float(&'a str),

    Boolean(&'a str),/// Zero-copy lexer with position tracking

    Identifier(&'a str),pub struct Lexer<'a> {

        input: &'a str,

    // Structural tokens    bytes: &'a [u8],

    LeftBracket,   // [    position: usize,

    RightBracket,  // ]    line: usize,

    Equals,        // =    column: usize,

    Newline,}

    Comment(&'a str),

    /// AST node with source position for error reporting

    // Special#[derive(Debug, Clone)]

    Eof,pub struct AstNode {

}    pub value: Box<AstValue>,  // Box to break recursion

    pub span: Span,

/// Zero-copy lexer for enterprise performance}

pub struct Lexer<'a> {

    input: &'a str,#[derive(Debug, Clone)]

    position: usize,pub struct Span {

    pub line: usize,    pub start: usize,

    pub column: usize,    pub end: usize,

}    pub line: usize,

    pub column: usize,

impl<'a> Lexer<'a> {}

    pub fn new(input: &'a str) -> Self {

        Self {#[derive(Debug, Clone)]

            input,pub enum AstValue {

            position: 0,    Document(BTreeMap<String, AstNode>),

            line: 1,    Section { name: String, entries: BTreeMap<String, AstNode> },

            column: 1,    KeyValue { key: String, value: Box<AstNode> },  // Box to break recursion

        }    String(String),

    }    Integer(i64),

        Float(f64),

    /// Get next token - zero allocation when possible    Boolean(bool),

    pub fn next_token(&mut self) -> Result<Token<'a>> {    Array(Vec<AstNode>),

        self.skip_whitespace();    Null,

        }

        if self.position >= self.input.len() {

            return Ok(Token::Eof);impl<'a> Lexer<'a> {

        }    #[inline(always)]

            pub fn new(input: &'a str) -> Self {

        let start = self.position;        Self {

        let ch = self.current_char();            input,

                    bytes: input.as_bytes(),

        match ch {            position: 0,

            '[' => {            line: 1,

                self.advance();            column: 1,

                Ok(Token::LeftBracket)        }

            }    }

            ']' => {    

                self.advance();    /// Zero-copy tokenization - returns slices into original input

                Ok(Token::RightBracket)    #[inline(always)]

            }    pub fn next_token(&mut self) -> Result<Token<'a>> {

            '=' => {        self.skip_whitespace();

                self.advance();        

                Ok(Token::Equals)        if self.is_at_end() {

            }            return Ok(Token::Eof);

            '\n' => {        }

                self.advance();        

                self.line += 1;        let start = self.position;

                self.column = 1;        let ch = self.current_byte();

                Ok(Token::Newline)        

            }        match ch {

            '#' | ';' => {            b'[' => {

                // Comment until end of line                self.advance();

                let comment_start = self.position;                Ok(Token::LeftBracket)

                while self.position < self.input.len() && self.current_char() != '\n' {            }

                    self.advance();            b']' => {

                }                self.advance();

                Ok(Token::Comment(&self.input[comment_start..self.position]))                Ok(Token::RightBracket)

            }            }

            '"' => {            b'=' => {

                // Quoted string                self.advance();

                self.advance(); // Skip opening quote                Ok(Token::Equals)

                let string_start = self.position;            }

                            b'\n' => {

                while self.position < self.input.len() && self.current_char() != '"' {                self.advance();

                    if self.current_char() == '\\' {                Ok(Token::Newline)

                        self.advance(); // Skip escape char            }

                        if self.position < self.input.len() {            b'"' => self.lex_quoted_string(),

                            self.advance(); // Skip escaped char            b'#' => self.lex_comment(),

                        }            b'0'..=b'9' | b'-' | b'+' => self.lex_number(),

                    } else {            _ => self.lex_identifier(),

                        self.advance();        }

                    }    }

                }    

                    #[inline(always)]

                if self.position >= self.input.len() {    fn current_byte(&self) -> u8 {

                    return Err(Error::parse("Unterminated string", self.line, self.column));        self.bytes[self.position]

                }    }

                    

                let string_end = self.position;    #[inline(always)]

                self.advance(); // Skip closing quote    fn peek_byte(&self, offset: usize) -> Option<u8> {

                Ok(Token::String(&self.input[string_start..string_end]))        self.bytes.get(self.position + offset).copied()

            }    }

            _ if ch.is_ascii_digit() || ch == '-' => {    

                // Number (integer or float)    #[inline(always)]

                let num_start = self.position;    fn advance(&mut self) {

                        if self.position < self.bytes.len() {

                if ch == '-' {            if self.bytes[self.position] == b'\n' {

                    self.advance();                self.line += 1;

                }                self.column = 1;

                            } else {

                // Parse digits                self.column += 1;

                while self.position < self.input.len() && self.current_char().is_ascii_digit() {            }

                    self.advance();            self.position += 1;

                }        }

                    }

                // Check for decimal point    

                if self.position < self.input.len() && self.current_char() == '.' {    #[inline(always)]

                    self.advance();    fn advance_by(&mut self, count: usize) {

                    while self.position < self.input.len() && self.current_char().is_ascii_digit() {        for _ in 0..count {

                        self.advance();            self.advance();

                    }        }

                    Ok(Token::Float(&self.input[num_start..self.position]))    }

                } else {    

                    Ok(Token::Integer(&self.input[num_start..self.position]))    #[inline(always)]

                }    fn is_at_end(&self) -> bool {

            }        self.position >= self.bytes.len()

            _ if ch.is_ascii_alphabetic() || ch == '_' => {    }

                // Identifier or boolean    

                let ident_start = self.position;    #[inline(always)]

                    fn skip_whitespace(&mut self) {

                while self.position < self.input.len() {        while !self.is_at_end() {

                    let c = self.current_char();            match self.current_byte() {

                    if c.is_ascii_alphanumeric() || c == '_' {                b' ' | b'\t' | b'\r' => self.advance(),

                        self.advance();                _ => break,

                    } else {            }

                        break;        }

                    }    }

                }    

                    /// Zero-copy quoted string lexing

                let ident = &self.input[ident_start..self.position];    fn lex_quoted_string(&mut self) -> Result<Token<'a>> {

                        let start_pos = self.position;

                // Check for boolean values        self.advance(); // Skip opening quote

                match ident {        

                    "true" | "false" | "yes" | "no" | "on" | "off" | "1" | "0" => {        let content_start = self.position;

                        Ok(Token::Boolean(ident))        

                    }        while !self.is_at_end() && self.current_byte() != b'"' {

                    _ => Ok(Token::Identifier(ident))            if self.current_byte() == b'\\' {

                }                self.advance(); // Skip escape char

            }                if !self.is_at_end() {

            _ => {                    self.advance(); // Skip escaped char

                // Unrecognized character, treat as identifier for now                }

                let start = self.position;            } else {

                self.advance();                self.advance();

                while self.position < self.input.len() && !self.current_char().is_whitespace() {            }

                    let c = self.current_char();        }

                    if c == '=' || c == '[' || c == ']' || c == '#' || c == ';' {        

                        break;        if self.is_at_end() {

                    }            return Err(Error::parse(

                    self.advance();                "Unterminated string",

                }                self.line,

                Ok(Token::Identifier(&self.input[start..self.position]))                self.column,

            }            ));

        }        }

    }        

            let content_end = self.position;

    fn current_char(&self) -> char {        self.advance(); // Skip closing quote

        self.input.chars().nth(self.position).unwrap_or('\0')        

    }        // Return zero-copy slice of string content

            let content = &self.input[content_start..content_end];

    fn advance(&mut self) {        Ok(Token::String(content))

        if self.position < self.input.len() {    }

            self.position += 1;    

            self.column += 1;    /// Zero-copy comment lexing

        }    fn lex_comment(&mut self) -> Result<Token<'a>> {

    }        let start = self.position;

            

    fn skip_whitespace(&mut self) {        while !self.is_at_end() && self.current_byte() != b'\n' {

        while self.position < self.input.len() {            self.advance();

            let ch = self.current_char();        }

            if ch.is_whitespace() && ch != '\n' {        

                self.advance();        let content = &self.input[start..self.position];

            } else {        Ok(Token::Comment(content))

                break;    }

            }    

        }    /// Zero-copy number lexing with type detection

    }    fn lex_number(&mut self) -> Result<Token<'a>> {

}        let start = self.position;

        let mut has_dot = false;

/// AST node with source location        

#[derive(Debug, Clone)]        // Handle sign

pub struct AstNode {        if matches!(self.current_byte(), b'-' | b'+') {

    pub value: Box<AstValue>,            self.advance();

    pub span: Span,        }

}        

        // Consume digits and optional decimal point

/// AST value types for zero-copy parsing        while !self.is_at_end() {

#[derive(Debug, Clone)]            match self.current_byte() {

pub enum AstValue {                b'0'..=b'9' => self.advance(),

    Document(BTreeMap<String, AstNode>),                b'.' if !has_dot => {

    Section { name: String, entries: BTreeMap<String, AstNode> },                    has_dot = true;

    KeyValue { key: String, value: Box<AstNode> },                    self.advance();

    String(String),                }

    Integer(i64),                _ => break,

    Float(f64),            }

    Boolean(bool),        }

    Array(Vec<AstNode>),        

    Null,        let text = &self.input[start..self.position];

}        

        if has_dot {

/// Zero-copy recursive descent parser            Ok(Token::Float(text))

pub struct Parser<'a> {        } else {

    lexer: Lexer<'a>,            Ok(Token::Integer(text))

    current_token: Token<'a>,        }

    position: usize,    }

}    

    /// Zero-copy identifier/keyword lexing

impl<'a> Parser<'a> {    fn lex_identifier(&mut self) -> Result<Token<'a>> {

    pub fn new(mut lexer: Lexer<'a>) -> Result<Self> {        let start = self.position;

        let current_token = lexer.next_token()?;        

        Ok(Self {        while !self.is_at_end() {

            lexer,            match self.current_byte() {

            current_token,                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.' => {

            position: 0,                    self.advance();

        })                }

    }                _ => break,

                }

    pub fn parse(&mut self) -> Result<AstNode> {        }

        let start = self.position;        

        let document = self.parse_document()?;        if start == self.position {

                    return Err(Error::parse(

        let span = Span::new(start, self.position, 1, 1);                "Expected identifier",

        Ok(AstNode {                self.line,

            value: Box::new(AstValue::Document(document)),                self.column,

            span,            ));

        })        }

    }        

            let text = &self.input[start..self.position];

    fn parse_document(&mut self) -> Result<BTreeMap<String, AstNode>> {        

        let mut document = BTreeMap::new();        // Check for boolean keywords

        let mut current_section: Option<String> = None;        match text {

                    "true" | "false" | "yes" | "no" | "on" | "off" => Ok(Token::Boolean(text)),

        while !matches!(self.current_token, Token::Eof) {            "null" | "nil" => Ok(Token::Identifier(text)), // Will be parsed as null value

            match &self.current_token {            _ => Ok(Token::Identifier(text)),

                Token::Comment(_) | Token::Newline => {        }

                    self.advance_token()?;    }

                }    

                Token::LeftBracket => {    #[inline(always)]

                    // Parse section header like [section_name]    fn current_span(&self) -> Span {

                    self.advance_token()?; // consume '['        Span {

                                start: self.position,

                    if let Token::Identifier(section_name) = &self.current_token {            end: self.position,

                        let section_name = section_name.to_string();            line: self.line,

                        self.advance_token()?; // consume section name            column: self.column,

                                }

                        if matches!(self.current_token, Token::RightBracket) {    }

                            self.advance_token()?; // consume ']'}

                            

                            // Create section if it doesn't exist/// Zero-copy AST parser - builds minimal tree structure

                            if !document.contains_key(&section_name) {pub struct Parser<'a> {

                                let span = Span::new(self.position, self.position, self.lexer.line, self.lexer.column);    lexer: Lexer<'a>,

                                document.insert(    current_token: Token<'a>,

                                    section_name.clone(),}

                                    AstNode {

                                        value: Box::new(AstValue::Section {impl<'a> Parser<'a> {

                                            name: section_name.clone(),    pub fn new(input: &'a str) -> Result<Self> {

                                            entries: BTreeMap::new(),        let mut lexer = Lexer::new(input);

                                        }),        let current_token = lexer.next_token()?;

                                        span,        

                                    },        Ok(Self {

                                );            lexer,

                            }            current_token,

                            current_section = Some(section_name);        })

                        } else {    }

                            return Err(Error::parse("Expected ']' after section name", self.lexer.line, self.lexer.column));    

                        }    /// Parse the entire configuration into an AST

                    } else {    pub fn parse(&mut self) -> Result<AstNode> {

                        return Err(Error::parse("Expected section name after '['", self.lexer.line, self.lexer.column));        let mut document = BTreeMap::new();

                    }        let mut current_section: Option<String> = None;

                }        let start_span = self.lexer.current_span();

                Token::Identifier(key) => {        

                    // Parse key-value pair        while !matches!(self.current_token, Token::Eof) {

                    let key = key.to_string();            self.skip_newlines();

                    self.advance_token()?;            

                                if matches!(self.current_token, Token::Eof) {

                    if matches!(self.current_token, Token::Equals) {                break;

                        self.advance_token()?; // consume '='            }

                        let value = self.parse_value()?;            

                                    match &self.current_token {

                        // Add to current section or top-level                Token::LeftBracket => {

                        if let Some(ref section_name) = current_section {                    // Section header

                            if let Some(section) = document.get_mut(section_name) {                    current_section = Some(self.parse_section_header()?);

                                if let AstValue::Section { entries, .. } = section.value.as_mut() {                }

                                    entries.insert(key, value);                Token::Identifier(_) => {

                                }                    // Key-value pair

                            }                    let (key, value) = self.parse_key_value()?;

                        } else {                    

                            document.insert(key, value);                    match &current_section {

                        }                        Some(section_name) => {

                    } else {                            // Add to section

                        return Err(Error::parse("Expected '=' after key", self.lexer.line, self.lexer.column));                            let section = document.entry(section_name.clone())

                    }                                .or_insert_with(|| AstNode {

                }                                    value: Box::new(AstValue::Section {

                _ => {                                        name: section_name.clone(),

                    return Err(Error::parse("Unexpected token in document", self.lexer.line, self.lexer.column));                                        entries: BTreeMap::new(),

                }                                    }),

            }                                    span: start_span.clone(),

        }                                });

                                    

        Ok(document)                            if let AstValue::Section { entries, .. } = section.value.as_mut() {

    }                                entries.insert(key, value);

                                }

    fn advance_token(&mut self) -> Result<()> {                        }

        self.position += 1;                        None => {

        self.current_token = self.lexer.next_token()?;                            // Add to root

        Ok(())                            document.insert(key, value);

    }                        }

                        }

    fn parse_value(&mut self) -> Result<AstNode> {                }

        let start = self.position;                Token::Comment(_) => {

        let span = Span::new(start, start, self.lexer.line, self.lexer.column);                    // Skip comments

                            self.advance_token()?;

        match &self.current_token {                }

            Token::String(s) => {                _ => {

                let value = AstNode {                    return Err(Error::parse(

                    value: Box::new(AstValue::String(s.to_string())),                        "Unexpected token",

                    span,                        self.lexer.line,

                };                        self.lexer.column,

                self.advance_token()?;                    ));

                Ok(value)                }

            }            }

            Token::Integer(s) => {        }

                let int_val = s.parse::<i64>()        

                    .map_err(|_| Error::parse("Invalid integer", self.lexer.line, self.lexer.column))?;        let end_span = self.lexer.current_span();

                let value = AstNode {        Ok(AstNode {

                    value: Box::new(AstValue::Integer(int_val)),            value: Box::new(AstValue::Document(document)),

                    span,            span: Span {

                };                start: start_span.start,

                self.advance_token()?;                end: end_span.end,

                Ok(value)                line: start_span.line,

            }                column: start_span.column,

            Token::Float(s) => {            },

                let float_val = s.parse::<f64>()        })

                    .map_err(|_| Error::parse("Invalid float", self.lexer.line, self.lexer.column))?;    }

                let value = AstNode {    

                    value: Box::new(AstValue::Float(float_val)),    fn parse_section_header(&mut self) -> Result<String> {

                    span,        self.expect_token(Token::LeftBracket)?;

                };        

                self.advance_token()?;        if let Token::Identifier(name) = self.current_token {

                Ok(value)            let section_name = name.to_string();

            }            self.advance_token()?;

            Token::Boolean(s) => {            self.expect_token(Token::RightBracket)?;

                let bool_val = matches!(s, &"true" | &"yes" | &"on" | &"1");            Ok(section_name)

                let value = AstNode {        } else {

                    value: Box::new(AstValue::Boolean(bool_val)),            Err(Error::parse(

                    span,                "Expected section name",

                };                self.lexer.line,

                self.advance_token()?;                self.lexer.column,

                Ok(value)            ))

            }        }

            Token::Identifier(s) => {    }

                if matches!(s, &"null" | &"nil") {    

                    let value = AstNode {    fn parse_key_value(&mut self) -> Result<(String, AstNode)> {

                        value: Box::new(AstValue::Null),        let key = if let Token::Identifier(k) = self.current_token {

                        span,            k.to_string()

                    };        } else {

                    self.advance_token()?;            return Err(Error::parse(

                    Ok(value)                "Expected key name",

                } else {                self.lexer.line,

                    // Treat unknown identifier as string                self.lexer.column,

                    let value = AstNode {            ));

                        value: Box::new(AstValue::String(s.to_string())),        };

                        span,        

                    };        self.advance_token()?;

                    self.advance_token()?;        self.expect_token(Token::Equals)?;

                    Ok(value)        

                }        let value = self.parse_value()?;

            }        Ok((key, value))

            Token::LeftBracket => {    }

                // Parse array    

                self.advance_token()?; // consume '['    /// Parse value with potential array detection

                let mut elements = Vec::new();    fn parse_value(&mut self) -> Result<AstNode> {

                        let span = self.lexer.current_span();

                while !matches!(self.current_token, Token::RightBracket | Token::Eof) {        

                    if matches!(self.current_token, Token::Comment(_) | Token::Newline) {        match &self.current_token {

                        self.advance_token()?;            Token::String(s) => {

                        continue;                let value = AstNode {

                    }                    value: Box::new(AstValue::String(s.to_string())),

                                        span,

                    let element = self.parse_value()?;                };

                    elements.push(element);                self.advance_token()?;

                                    Ok(value)

                    // Skip whitespace and newlines            }

                    while matches!(self.current_token, Token::Comment(_) | Token::Newline) {            Token::Integer(s) => {

                        self.advance_token()?;                let int_val = s.parse::<i64>()

                    }                    .map_err(|_| Error::parse("Invalid integer", self.lexer.line, self.lexer.column))?;

                }                let value = AstNode {

                                    value: Box::new(AstValue::Integer(int_val)),

                if matches!(self.current_token, Token::RightBracket) {                    span,

                    self.advance_token()?; // consume ']'                };

                    Ok(AstNode {                self.advance_token()?;

                        value: Box::new(AstValue::Array(elements)),                Ok(value)

                        span,            }

                    })            Token::Float(s) => {

                } else {                let float_val = s.parse::<f64>()

                    Err(Error::parse("Expected ']' to close array", self.lexer.line, self.lexer.column))                    .map_err(|_| Error::parse("Invalid float", self.lexer.line, self.lexer.column))?;

                }                let value = AstNode {

            }                    value: Box::new(AstValue::Float(float_val)),

            _ => Err(Error::parse("Expected value", self.lexer.line, self.lexer.column)),                    span,

        }                };

    }                self.advance_token()?;

}                Ok(value)

            }

/// Convert AST to Value for runtime use            Token::Boolean(s) => {

impl AstNode {                let bool_val = matches!(s, &"true" | &"yes" | &"on" | &"1");

    pub fn to_value(&self) -> Value {                let value = AstNode {

        match self.value.as_ref() {                    value: Box::new(AstValue::Boolean(bool_val),

            AstValue::String(s) => Value::string(s.clone()),                    span,

            AstValue::Integer(i) => Value::integer(*i),                };

            AstValue::Float(f) => Value::float(*f),                self.advance_token()?;

            AstValue::Boolean(b) => Value::bool(*b),                Ok(value)

            AstValue::Null => Value::null(),            }

            AstValue::Array(elements) => {            Token::Identifier(s) => {

                let values: Vec<Value> = elements.iter().map(|el| el.to_value()).collect();                if matches!(s, &"null" | &"nil") {

                Value::array(values)                    let value = AstNode {

            }                        value: Box::new(AstValue::Null,

            AstValue::Document(map) | AstValue::Section { entries: map, .. } => {                        span,

                let mut table = BTreeMap::new();                    };

                for (key, node) in map {                    self.advance_token()?;

                    table.insert(key.clone(), node.to_value());                    return Ok(value);

                }                }

                Value::table(table)                

            }                // Collect potential array elements until newline

            AstValue::KeyValue { value, .. } => value.to_value(),                let mut elements = Vec::new();

        }                

    }                // First element (current identifier)

}                elements.push(AstNode {

                    value: Box::new(AstValue::String(s.to_string()),

/// Main parsing function for enterprise performance                    span: span.clone(),

pub fn parse(input: &str) -> Result<Value> {                });

    let lexer = Lexer::new(input);                self.advance_token()?;

    let mut parser = Parser::new(lexer)?;                

    let ast = parser.parse()?;                // Look for more elements

    Ok(ast.to_value())                while !matches!(self.current_token, Token::Newline | Token::Eof) {

}                    match &self.current_token {

                        Token::String(s) => {

#[cfg(test)]                            elements.push(AstNode {

mod tests {                                value: Box::new(AstValue::String(s.to_string()),

    use super::*;                                span: self.lexer.current_span(),

                            });

    #[test]                            self.advance_token()?;

    fn test_basic_key_value() {                        }

        let input = "key = value";                        Token::Integer(s) => {

        let result = parse(input).unwrap();                            let int_val = s.parse::<i64>()

                                        .map_err(|_| Error::parse("Invalid integer", self.lexer.line, self.lexer.column))?;

        if let Value::Table(table) = result {                            elements.push(AstNode {

            assert_eq!(table.get("key").unwrap().as_string().unwrap(), "value");                                value: Box::new(AstValue::Integer(int_val),

        } else {                                span: self.lexer.current_span(),

            panic!("Expected table");                            });

        }                            self.advance_token()?;

    }                        }

                            Token::Float(s) => {

    #[test]                            let float_val = s.parse::<f64>()

    fn test_section() {                                .map_err(|_| Error::parse("Invalid float", self.lexer.line, self.lexer.column))?;

        let input = r#"                            elements.push(AstNode {

[section]                                value: Box::new(AstValue::Float(float_val),

key = "value"                                span: self.lexer.current_span(),

        "#;                            });

        let result = parse(input).unwrap();                            self.advance_token()?;

                                }

        if let Value::Table(table) = result {                        Token::Boolean(s) => {

            let section = table.get("section").unwrap();                            let bool_val = matches!(s, &"true" | &"yes" | &"on" | &"1");

            if let Value::Table(section_table) = section {                            elements.push(AstNode {

                assert_eq!(section_table.get("key").unwrap().as_string().unwrap(), "value");                                value: Box::new(AstValue::Boolean(bool_val),

            } else {                                span: self.lexer.current_span(),

                panic!("Expected section table");                            });

            }                            self.advance_token()?;

        } else {                        }

            panic!("Expected table");                        Token::Identifier(s) => {

        }                            elements.push(AstNode {

    }                                value: Box::new(AstValue::String(s.to_string()),

                                    span: self.lexer.current_span(),

    #[test]                            });

    fn test_types() {                            self.advance_token()?;

        let input = r#"                        }

string = "hello"                        _ => break,

integer = 42                    }

float = 3.14                }

boolean = true                

null_val = null                if elements.len() > 1 {

        "#;                    Ok(AstNode {

        let result = parse(input).unwrap();                        value: Box::new(AstValue::Array(elements),

                                span,

        if let Value::Table(table) = result {                    })

            assert_eq!(table.get("string").unwrap().as_string().unwrap(), "hello");                } else {

            assert_eq!(table.get("integer").unwrap().as_integer().unwrap(), 42);                    Ok(elements.into_iter().next().unwrap())

            assert_eq!(table.get("float").unwrap().as_float().unwrap(), 3.14);                }

            assert_eq!(table.get("boolean").unwrap().as_bool().unwrap(), true);            }

            assert!(table.get("null_val").unwrap().is_null());            _ => Err(Error::parse(

        } else {                "Expected value",

            panic!("Expected table");                self.lexer.line,

        }                self.lexer.column,

    }            )),

            }

    #[test]    }

    fn test_array() {    

        let input = "items = [1 2 3]";    #[inline(always)]

        let result = parse(input).unwrap();    fn advance_token(&mut self) -> Result<()> {

                self.current_token = self.lexer.next_token()?;

        if let Value::Table(table) = result {        Ok(())

            let array = table.get("items").unwrap();    }

            if let Value::Array(arr) = array {    

                assert_eq!(arr.len(), 3);    #[inline(always)]

                assert_eq!(arr[0].as_integer().unwrap(), 1);    fn expect_token(&mut self, expected: Token<'a>) -> Result<()> {

                assert_eq!(arr[1].as_integer().unwrap(), 2);        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {

                assert_eq!(arr[2].as_integer().unwrap(), 3);            self.advance_token()

            } else {        } else {

                panic!("Expected array");            Err(Error::parse(

            }                format!("Expected token, found {:?}", self.current_token),

        } else {                self.lexer.line,

            panic!("Expected table");                self.lexer.column,

        }            ))

    }        }

}    }
    
    #[inline(always)]
    fn skip_newlines(&mut self) {
        while matches!(self.current_token, Token::Newline) {
            let _ = self.advance_token();
        }
    }
}

/// Convert AST to Value for runtime use
impl AstNode {
    pub fn to_value(&self) -> Value {
        match self.value.as_ref() {  // Use as_ref to access Box contents
            AstValue::String(s) => Value::string(s.clone()),
            AstValue::Integer(i) => Value::integer(*i),
            AstValue::Float(f) => Value::float(*f),
            AstValue::Boolean(b) => Value::bool(*b),
            AstValue::Null => Value::null(),
            AstValue::Array(elements) => {
                let values: Vec<Value> = elements.iter().map(|el| el.to_value()).collect();
                Value::array(values)
            }
            AstValue::Document(map) | AstValue::Section { entries: map, .. } => {
                let mut table = BTreeMap::new();
                for (key, node) in map {
                    table.insert(key.clone(), node.to_value());
                }
                Value::table(table)
            }
            AstValue::KeyValue { value, .. } => value.to_value(),
        }
    }
}

/// High-performance zero-copy CONF parser entry point
#[inline(always)]
pub fn parse(input: &str) -> Result<Value> {
    let mut parser = Parser::new(input)?;
    let ast = parser.parse()?;
    Ok(ast.to_value())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_lexer() {
        let input = r#"key = "value""#;
        let mut lexer = Lexer::new(input);
        
        assert!(matches!(lexer.next_token().unwrap(), Token::Identifier("key")));
        assert!(matches!(lexer.next_token().unwrap(), Token::Equals));
        assert!(matches!(lexer.next_token().unwrap(), Token::String("value")));
        assert!(matches!(lexer.next_token().unwrap(), Token::Eof));
    }
    
    #[test]
    fn test_space_separated_arrays() {
        let config = parse("ports = 8001 8002 8003").unwrap();
        let ports = config.get("ports").unwrap().as_array().unwrap();
        assert_eq!(ports.len(), 3);
        assert_eq!(ports[0].as_integer().unwrap(), 8001);
    }
}