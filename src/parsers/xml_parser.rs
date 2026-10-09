//! # XML Configuration Parser
//!
//! High-performance XML configuration parser using zero-copy parsing.
//! Supports common XML configuration patterns used in enterprise Java/.NET environments.
//!
//! ## Performance Features
//! - Zero-copy parsing using `quick-xml`
//! - Streaming parser for memory efficiency
//! - Feature-gated to ensure zero impact when disabled
//!
//! ## Supported XML Patterns
//! - Spring Boot application.xml
//! - ASP.NET Core appsettings.xml
//! - Maven/Gradle configuration XML
//! - Generic key-value XML structures

use crate::{error::Error, Result, Value};
#[cfg(feature = "xml")]
use quick_xml::{
    escape::resolve_predefined_entity,
    events::{BytesStart, Event},
    Reader, XmlVersion,
};
use std::collections::BTreeMap;

/// XML configuration parser with zero-copy optimizations
#[cfg(feature = "xml")]
pub struct XmlParser<'a> {
    reader: Reader<&'a [u8]>,
}

/// Deepest element nesting accepted. Deeper input is rejected so that
/// building and dropping the value tree cannot overflow the stack.
#[cfg(feature = "xml")]
const MAX_DEPTH: usize = 128;

/// An element whose end tag has not been seen yet
#[cfg(feature = "xml")]
struct OpenElement {
    name: String,
    children: BTreeMap<String, Value>,
    text: String,
}

#[cfg(feature = "xml")]
impl<'a> XmlParser<'a> {
    /// Create a new XML parser for the given content
    pub fn new(content: &'a str) -> Self {
        let reader = Reader::from_str(content);
        Self { reader }
    }

    /// Parse XML content into a Value tree.
    ///
    /// Elements become tables keyed by tag name and attributes become string
    /// entries. An element holding only text becomes that text, converted to a
    /// bool, integer or float when it parses as one; text next to child
    /// elements or attributes is stored under `"text"`. Repeated sibling
    /// elements with the same name are collected into an array in document
    /// order. Entity and character references (`&amp;`, `&#x41;`) and CDATA
    /// sections are resolved into the text.
    pub fn parse(&mut self) -> Result<Value> {
        let mut stack: Vec<OpenElement> = Vec::new();
        let mut root = BTreeMap::new();
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    if stack.len() >= MAX_DEPTH {
                        return Err(xml_error(format!(
                            "elements are nested deeper than {MAX_DEPTH} levels"
                        )));
                    }
                    let (name, children) = Self::element(&e)?;
                    stack.push(OpenElement {
                        name,
                        children,
                        text: String::new(),
                    });
                }

                Ok(Event::End(_)) => {
                    if let Some(element) = stack.pop() {
                        let value = self.finish(element.children, element.text.trim());
                        let parent = match stack.last_mut() {
                            Some(parent) => &mut parent.children,
                            None => &mut root,
                        };
                        insert_child(parent, element.name, value);
                    }
                }

                Ok(Event::Empty(e)) => {
                    let (name, children) = Self::element(&e)?;
                    let parent = match stack.last_mut() {
                        Some(parent) => &mut parent.children,
                        None => &mut root,
                    };
                    insert_child(parent, name, Value::table(children));
                }

                Ok(Event::Text(e)) => {
                    if let Some(element) = stack.last_mut() {
                        element
                            .text
                            .push_str(&e.xml10_content().map_err(xml_error)?);
                    }
                }

                Ok(Event::CData(e)) => {
                    if let Some(element) = stack.last_mut() {
                        element
                            .text
                            .push_str(&e.xml10_content().map_err(xml_error)?);
                    }
                }

                Ok(Event::GeneralRef(e)) => {
                    if let Some(element) = stack.last_mut() {
                        if let Some(ch) = e.resolve_char_ref().map_err(xml_error)? {
                            element.text.push(ch);
                        } else {
                            let name = e.decode().map_err(xml_error)?;
                            match resolve_predefined_entity(&name) {
                                Some(text) => element.text.push_str(text),
                                None => {
                                    return Err(xml_error(format!(
                                        "unknown entity reference `&{name};`"
                                    )))
                                }
                            }
                        }
                    }
                }

                Ok(Event::Eof) => {
                    if let Some(element) = stack.last() {
                        return Err(xml_error(format!(
                            "element `<{}>` is not closed",
                            element.name
                        )));
                    }
                    break;
                }

                Err(e) => return Err(xml_error(e)),

                _ => {}
            }
            buf.clear();
        }

        Ok(Value::table(root))
    }

    /// Tag name and attributes of a start or empty tag
    fn element(e: &BytesStart<'_>) -> Result<(String, BTreeMap<String, Value>)> {
        let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
        let mut attributes = BTreeMap::new();
        for attr in e.attributes() {
            let attr = attr.map_err(xml_error)?;
            let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
            let value = attr
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(xml_error)?
                .into_owned();
            attributes.insert(key, Value::string(value));
        }
        Ok((name, attributes))
    }

    /// Value of a closed element from its children/attributes and text
    fn finish(&self, mut children: BTreeMap<String, Value>, text: &str) -> Value {
        if text.is_empty() {
            Value::table(children)
        } else if children.is_empty() {
            self.parse_value(text)
        } else {
            children.insert("text".to_string(), self.parse_value(text));
            Value::table(children)
        }
    }

    /// Parse a text value into appropriate type
    fn parse_value(&self, text: &str) -> Value {
        // Try parsing as different types
        if let Ok(bool_val) = text.parse::<bool>() {
            Value::bool(bool_val)
        } else if let Ok(int_val) = text.parse::<i64>() {
            Value::integer(int_val)
        } else if let Ok(float_val) = text.parse::<f64>() {
            Value::float(float_val)
        } else {
            Value::string(text)
        }
    }
}

/// Store a child element, turning repeated names into an array
#[cfg(feature = "xml")]
fn insert_child(parent: &mut BTreeMap<String, Value>, name: String, value: Value) {
    match parent.get_mut(&name) {
        // Element values are never arrays themselves, so an array here was
        // built from earlier siblings with the same name
        Some(Value::Array(items)) => items.push(value),
        Some(existing) => {
            let first = std::mem::replace(existing, Value::Null);
            *existing = Value::array(vec![first, value]);
        }
        None => {
            parent.insert(name, value);
        }
    }
}

/// Wrap an XML problem in the crate's error type
#[cfg(feature = "xml")]
fn xml_error(err: impl std::fmt::Display) -> Error {
    Error::io(
        "XML parsing error".to_string(),
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("XML error: {err}")),
    )
}

#[cfg(feature = "xml")]
impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        Error::io(
            "XML parsing error".to_string(),
            std::io::Error::new(std::io::ErrorKind::InvalidData, err),
        )
    }
}

/// Parse XML format configuration
#[cfg(feature = "xml")]
pub fn parse(source: &str) -> Result<Value> {
    parse_xml(source)
}

/// Parse XML format configuration (fallback when XML is not available)
#[cfg(not(feature = "xml"))]
pub fn parse(_source: &str) -> Result<Value> {
    Err(crate::error::Error::feature_not_enabled("xml"))
}

/// Parse XML configuration from string
#[cfg(feature = "xml")]
pub fn parse_xml(content: &str) -> Result<Value> {
    let mut parser = XmlParser::new(content);
    parser.parse()
}

/// Placeholder when XML feature is disabled
#[cfg(not(feature = "xml"))]
pub fn parse_xml(_content: &str) -> Result<Value> {
    Err(crate::error::Error::feature_not_enabled("xml"))
}

#[cfg(all(test, feature = "xml"))]
mod tests {
    use super::*;

    #[test]
    fn test_basic_xml_parsing() -> crate::Result<()> {
        let xml = r#"
        <configuration>
            <database>
                <host>localhost</host>
                <port>5432</port>
                <enabled>true</enabled>
            </database>
            <app>
                <name>TestApp</name>
                <version>1.0</version>
            </app>
        </configuration>
        "#;

        let result = parse_xml(xml)?;

        if let Value::Table(config) = result {
            if let Some(Value::Table(db)) = config.get("configuration").and_then(|v| {
                if let Value::Table(t) = v {
                    t.get("database")
                } else {
                    None
                }
            }) {
                assert_eq!(db.get("host"), Some(&Value::string("localhost")));
                assert_eq!(db.get("port"), Some(&Value::integer(5432)));
                assert_eq!(db.get("enabled"), Some(&Value::bool(true)));
            } else {
                return Err(crate::Error::Parse {
                    message: "Expected database configuration".to_string(),
                    line: 0,
                    column: 0,
                    file: None,
                });
            }
        } else {
            return Err(crate::Error::Parse {
                message: "Expected table result".to_string(),
                line: 0,
                column: 0,
                file: None,
            });
        }
        Ok(())
    }

    #[test]
    fn test_xml_with_attributes() -> crate::Result<()> {
        let xml = r#"
        <config>
            <server host="localhost" port="8080" ssl="true">
                <name>MainServer</name>
            </server>
        </config>
        "#;

        let result = parse_xml(xml)?;

        if let Value::Table(config) = result {
            if let Some(Value::Table(server_config)) = config.get("config").and_then(|v| {
                if let Value::Table(t) = v {
                    t.get("server")
                } else {
                    None
                }
            }) {
                assert_eq!(server_config.get("host"), Some(&Value::string("localhost")));
                assert_eq!(server_config.get("port"), Some(&Value::string("8080")));
                assert_eq!(server_config.get("ssl"), Some(&Value::string("true")));
                assert_eq!(
                    server_config.get("name"),
                    Some(&Value::string("MainServer"))
                );
            } else {
                return Err(crate::Error::Parse {
                    message: "Expected server configuration".to_string(),
                    line: 0,
                    column: 0,
                    file: None,
                });
            }
        } else {
            return Err(crate::Error::Parse {
                message: "Expected table result".to_string(),
                line: 0,
                column: 0,
                file: None,
            });
        }
        Ok(())
    }

    #[test]
    fn test_self_closing_tags() -> crate::Result<()> {
        let xml = r#"
        <config>
            <feature name="auth" enabled="true" />
            <feature name="cache" enabled="false" />
        </config>
        "#;

        let result = parse_xml(xml)?;

        // Test passes if parsing doesn't panic
        assert!(matches!(result, Value::Table(_)));
        Ok(())
    }

    #[test]
    fn test_entities_cdata_and_repeated_elements() -> crate::Result<()> {
        let xml = r#"<config>
            <title>Tom &amp; Jerry &#x41;</title>
            <query><![CDATA[a < b && c]]></query>
            <server name="a &amp; b" />
            <host>one</host>
            <host>two</host>
            <host>three</host>
        </config>"#;
        let value = parse_xml(xml)?;
        let config = value.get("config").and_then(|v| v.as_table().ok()).cloned();
        let config = config.expect("config table");
        assert_eq!(config.get("title"), Some(&Value::string("Tom & Jerry A")));
        assert_eq!(config.get("query"), Some(&Value::string("a < b && c")));
        assert_eq!(
            config
                .get("server")
                .and_then(|s| s.as_table().ok())
                .and_then(|t| t.get("name")),
            Some(&Value::string("a & b"))
        );
        assert_eq!(
            config.get("host"),
            Some(&Value::array(vec![
                Value::string("one"),
                Value::string("two"),
                Value::string("three"),
            ]))
        );
        Ok(())
    }

    #[test]
    fn test_malformed_xml_is_an_error() {
        assert!(parse_xml("<a><b>1</b>").is_err(), "unclosed element");
        assert!(parse_xml("<a></b>").is_err(), "mismatched end tag");
        assert!(parse_xml("<a>&bogus;</a>").is_err(), "unknown entity");
        let deep = format!("{}1{}", "<a>".repeat(100_000), "</a>".repeat(100_000));
        assert!(parse_xml(&deep).is_err(), "nesting limit");
        let ok = format!("{}1{}", "<a>".repeat(128), "</a>".repeat(128));
        assert!(parse_xml(&ok).is_ok());
    }
}
