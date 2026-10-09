//! # TOML Format Parser
//!
//! TOML parser with format preservation capabilities.
//! Uses the NOML library's TOML compatibility for round-trip editing.

use crate::error::Result;
use crate::value::Value;
use std::collections::BTreeMap;

// This module is compiled only with the `toml` feature, which always brings in
// the `noml` crate, so it does not depend on the separate `noml` feature.

/// Parse TOML format configuration
pub fn parse(source: &str) -> Result<Value> {
    let document = noml::parse_raw(source)?;
    resolve_as_toml(&document)
}

/// Parse TOML with format preservation for round-trip editing
pub fn parse_with_preservation(source: &str) -> Result<(Value, noml::Document)> {
    // Parse to get the AST document for format preservation
    let document = noml::parse_string(source, None)?;
    let value = resolve_as_toml(&document)?;
    Ok((value, document))
}

/// Resolve a parsed document with TOML semantics.
///
/// TOML strings are plain text, but NOML expands `${...}` in double-quoted
/// strings. A TOML file such as `password = "${DB_PASSWORD}"` must keep
/// that text, so `${` is escaped as `$${` before resolving.
fn resolve_as_toml(document: &noml::Document) -> Result<Value> {
    let mut literal = document.clone();
    escape_templates(&mut literal.root);
    let resolved = noml::Resolver::new().resolve(&literal)?;
    convert_noml_value(resolved)
}

/// Escape `${` in every double-quoted string so NOML keeps it as text
fn escape_templates(node: &mut noml::AstNode) {
    use noml::parser::ast::{AstValue, StringStyle};
    match &mut node.value {
        AstValue::String { value, style, .. } => {
            if matches!(style, StringStyle::Double | StringStyle::TripleDouble)
                && value.contains("${")
            {
                *value = value.replace("${", "$${");
            }
        }
        AstValue::Array { elements, .. } => elements.iter_mut().for_each(escape_templates),
        AstValue::Table { entries, .. } => {
            for entry in entries {
                escape_templates(&mut entry.value);
            }
        }
        AstValue::FunctionCall { args, .. } | AstValue::Native { args, .. } => {
            args.iter_mut().for_each(escape_templates);
        }
        _ => {}
    }
}

/// Convert NOML Value to config-lib Value
fn convert_noml_value(noml_value: noml::Value) -> Result<Value> {
    match noml_value {
        noml::Value::Null => Ok(Value::Null),
        noml::Value::Bool(b) => Ok(Value::Bool(b)),
        noml::Value::Integer(i) => Ok(Value::Integer(i)),
        noml::Value::Float(f) => Ok(Value::Float(f)),
        noml::Value::String(s) => Ok(Value::String(s)),
        noml::Value::Array(arr) => {
            let converted: Result<Vec<Value>> = arr.into_iter().map(convert_noml_value).collect();
            Ok(Value::Array(converted?))
        }
        noml::Value::Table(table) => {
            let mut converted = BTreeMap::new();
            for (key, value) in table {
                converted.insert(key, convert_noml_value(value)?);
            }
            Ok(Value::Table(converted))
        }
        #[cfg(feature = "chrono")]
        noml::Value::DateTime(dt) => Ok(Value::DateTime(dt)),
        // Handle NOML-specific types by converting to basic types
        noml::Value::Binary(_) => Ok(Value::String("binary_data".to_string())),
        noml::Value::Size(size) => Ok(Value::Integer(size as i64)),
        noml::Value::Duration(duration) => Ok(Value::Float(duration)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_toml() {
        let config = parse(
            r#"
            name = "test"
            port = 8080
            debug = true
            
            [database]
            host = "localhost"
            max_connections = 100
        "#,
        )
        .unwrap();

        assert_eq!(config.get("name").unwrap().as_string().unwrap(), "test");
        assert_eq!(config.get("port").unwrap().as_integer().unwrap(), 8080);
        assert_eq!(
            config.get("database.host").unwrap().as_string().unwrap(),
            "localhost"
        );
    }

    #[test]
    fn test_toml_arrays() {
        let config = parse(
            r#"
            servers = ["alpha", "beta", "gamma"]
            ports = [8001, 8002, 8003]
        "#,
        )
        .unwrap();

        let servers = config.get("servers").unwrap().as_array().unwrap();
        assert_eq!(servers.len(), 3);
        assert_eq!(servers[0].as_string().unwrap(), "alpha");

        let ports = config.get("ports").unwrap().as_array().unwrap();
        assert_eq!(ports[0].as_integer().unwrap(), 8001);
    }

    #[test]
    fn toml_strings_keep_dollar_braces() {
        let value =
            parse("password = \"${DB_PASSWORD}\"\nliteral = '${x}'\nescaped = \"$${y}\"").unwrap();
        assert_eq!(
            value.get("password"),
            Some(&Value::string("${DB_PASSWORD}"))
        );
        assert_eq!(value.get("literal"), Some(&Value::string("${x}")));
        assert_eq!(value.get("escaped"), Some(&Value::string("$${y}")));
    }
}
