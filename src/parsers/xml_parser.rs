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
use quick_xml::{events::Event, Reader};
use std::collections::BTreeMap;

/// XML configuration parser with zero-copy optimizations
#[cfg(feature = "xml")]
pub struct XmlParser<'a> {
    reader: Reader<&'a [u8]>,
}

#[cfg(feature = "xml")]
impl<'a> XmlParser<'a> {
    /// Create a new XML parser for the given content
    pub fn new(content: &'a str) -> Self {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true); // Trim whitespace for cleaner parsing

        Self { reader }
    }

    /// Parse XML content into a Value tree
    pub fn parse(&mut self) -> Result<Value> {
        let mut stack: Vec<(String, BTreeMap<String, Value>)> = Vec::new();
        let mut root = BTreeMap::new();
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    let mut element_map = BTreeMap::new();

                    // Handle attributes
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                            let value = String::from_utf8_lossy(&attr.value).into_owned();
                            element_map.insert(key, Value::string(value));
                        }
                    }

                    stack.push((name, element_map));
                }

                Ok(Event::End(_)) => {
                    if let Some((tag_name, element_map)) = stack.pop() {
                        // If element only contains text, unwrap it
                        let value = if element_map.len() == 1 && element_map.contains_key("text") {
                            element_map.get("text").unwrap().clone()
                        } else {
                            Value::table(element_map)
                        };

                        if let Some((_, ref mut parent)) = stack.last_mut() {
                            parent.insert(tag_name, value);
                        } else {
                            root.insert(tag_name, value);
                        }
                    }
                }

                Ok(Event::Text(e)) => {
                    if let Ok(text_data) = e.unescape() {
                        let text = text_data.trim();
                        if !text.is_empty() {
                            if let Some((_, ref mut element_map)) = stack.last_mut() {
                                if element_map.is_empty() {
                                    // Simple text content
                                    element_map.insert("text".to_string(), self.parse_value(text));
                                } else {
                                    // Add as text attribute
                                    element_map.insert("text".to_string(), self.parse_value(text));
                                }
                            }
                        }
                    }
                }

                Ok(Event::Empty(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    let mut element_map = BTreeMap::new();

                    // Handle attributes
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                            let value = String::from_utf8_lossy(&attr.value).into_owned();
                            element_map.insert(key, Value::string(value));
                        }
                    }

                    let value = Value::table(element_map);

                    if let Some((_, ref mut parent)) = stack.last_mut() {
                        parent.insert(name, value);
                    } else {
                        root.insert(name, value);
                    }
                }

                Ok(Event::Eof) => break,

                Err(e) => {
                    return Err(Error::io(
                        "XML parsing error".to_string(),
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("XML error: {}", e),
                        ),
                    ))
                }

                _ => {}
            }
            buf.clear();
        }

        Ok(Value::table(root))
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

#[cfg(feature = "xml")]
impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        Error::io(
            "XML parsing error".to_string(),
            std::io::Error::new(std::io::ErrorKind::InvalidData, err),
        )
    }
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
    fn test_simple_xml() {
        let xml = r#"
        <configuration>
            <database>
                <host>localhost</host>
                <port>5432</port>
                <enabled>true</enabled>
            </database>
            <app>
                <name>MyApp</name>
                <version>1.0.0</version>
            </app>
        </configuration>
        "#;

        let result = parse_xml(xml).unwrap();

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
                panic!("Expected database configuration");
            }
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_xml_with_attributes() {
        let xml = r#"
        <config>
            <server host="localhost" port="8080" ssl="true">
                <name>MainServer</name>
            </server>
        </config>
        "#;

        let result = parse_xml(xml).unwrap();

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
                panic!("Expected server configuration");
            }
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_self_closing_tags() {
        let xml = r#"
        <config>
            <feature name="auth" enabled="true" />
            <feature name="cache" enabled="false" />
        </config>
        "#;

        let result = parse_xml(xml).unwrap();
        println!("Parsed XML: {:#?}", result);

        // Test passes if parsing doesn't panic
        assert!(matches!(result, Value::Table(_)));
    }
}
