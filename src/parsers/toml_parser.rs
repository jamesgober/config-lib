//! # TOML Format Parser
//!
//! TOML parser with format preservation capabilities.
//! Uses the NOML library's TOML compatibility for round-trip editing.

use crate::error::Result;
use crate::value::Value;
use std::collections::BTreeMap;

/// Parse TOML format configuration
#[cfg(feature = "noml")]
pub fn parse(source: &str) -> Result<Value> {
    // Use NOML's TOML parsing capability for format preservation
    let noml_value = noml::parse(source)?;
    convert_noml_value(noml_value)
}

/// Parse TOML format configuration (fallback when NOML is not available)
#[cfg(not(feature = "noml"))]
pub fn parse(_source: &str) -> Result<Value> {
    Err(Error::general(
        "TOML parsing requires either the 'noml' feature or a dedicated TOML parser"
    ))
}

/// Parse TOML with format preservation for round-trip editing
#[cfg(feature = "noml")]
pub fn parse_with_preservation(source: &str) -> Result<(Value, noml::Document)> {
    // Parse to get the AST document for format preservation
    let document = noml::parse_string(source, None)?;
    
    // Resolve to get the actual values
    let mut resolver = noml::Resolver::new();
    let resolved = resolver.resolve(&document)?;
    
    // Convert to config-lib Value
    let value = convert_noml_value(resolved)?;
    
    Ok((value, document))
}

/// Parse TOML with format preservation (fallback when NOML is not available)
#[cfg(not(feature = "noml"))]
pub fn parse_with_preservation(_source: &str) -> Result<(Value, ())> {
    Err(Error::general(
        "TOML format preservation requires the 'noml' feature"
    ))
}

/// Convert NOML Value to config-lib Value
#[cfg(feature = "noml")]
fn convert_noml_value(noml_value: noml::Value) -> Result<Value> {
    match noml_value {
        noml::Value::Null => Ok(Value::Null),
        noml::Value::Bool(b) => Ok(Value::Bool(b)),
        noml::Value::Integer(i) => Ok(Value::Integer(i)),
        noml::Value::Float(f) => Ok(Value::Float(f)),
        noml::Value::String(s) => Ok(Value::String(s)),
        noml::Value::Array(arr) => {
            let converted: Result<Vec<Value>> = arr
                .into_iter()
                .map(convert_noml_value)
                .collect();
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
        #[cfg(not(feature = "chrono"))]
        noml::Value::DateTime(dt) => Ok(Value::String(dt.to_rfc3339())),
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
        let config = parse(r#"
            name = "test"
            port = 8080
            debug = true
            
            [database]
            host = "localhost"
            max_connections = 100
        "#).unwrap();
        
        assert_eq!(config.get("name").unwrap().as_string().unwrap(), "test");
        assert_eq!(config.get("port").unwrap().as_integer().unwrap(), 8080);
        assert_eq!(config.get("database.host").unwrap().as_string().unwrap(), "localhost");
    }

    #[test]
    fn test_toml_arrays() {
        let config = parse(r#"
            servers = ["alpha", "beta", "gamma"]
            ports = [8001, 8002, 8003]
        "#).unwrap();
        
        let servers = config.get("servers").unwrap().as_array().unwrap();
        assert_eq!(servers.len(), 3);
        assert_eq!(servers[0].as_string().unwrap(), "alpha");
        
        let ports = config.get("ports").unwrap().as_array().unwrap();
        assert_eq!(ports[0].as_integer().unwrap(), 8001);
    }
}