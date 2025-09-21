//! # NOML Format Parser
//!
//! Integration with the NOML library for advanced configuration features.
//! This parser leverages the full NOML library to provide:
//! 
//! - Environment variable resolution
//! - File includes  
//! - Variable interpolation
//! - Native types (@size, @duration, etc.)
//! - Format preservation for round-trip editing

use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::BTreeMap;

/// Parse NOML format configuration using the noml library
pub fn parse(source: &str) -> Result<Value> {
    let noml_value = noml::parse(source)?;
    convert_noml_value(noml_value)
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
        noml::Value::Binary(data) => {
            // Convert binary data to base64 string for compatibility
            Ok(Value::String(base64::encode(data)))
        }
        noml::Value::Size(size) => {
            // Convert size to integer (bytes)
            Ok(Value::Integer(size as i64))
        }
        noml::Value::Duration(duration) => {
            // Convert duration to float (seconds)
            Ok(Value::Float(duration))
        }
    }
}

/// Parse NOML with format preservation for round-trip editing
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_noml() {
        let config = parse(r#"
            name = "test"
            port = 8080
            debug = true
        "#).unwrap();
        
        assert_eq!(config.get("name").unwrap().as_string().unwrap(), "test");
        assert_eq!(config.get("port").unwrap().as_integer().unwrap(), 8080);
        assert_eq!(config.get("debug").unwrap().as_bool().unwrap(), true);
    }

    #[test] 
    fn test_noml_features() {
        std::env::set_var("TEST_VAR", "hello");
        
        let config = parse(r#"
            greeting = env("TEST_VAR", "world")
            size = @size("10MB")
            timeout = @duration("30s")
        "#).unwrap();
        
        assert_eq!(config.get("greeting").unwrap().as_string().unwrap(), "hello");
        // Size converted to bytes
        assert_eq!(config.get("size").unwrap().as_integer().unwrap(), 10485760);
        // Duration converted to seconds
        assert_eq!(config.get("timeout").unwrap().as_float().unwrap(), 30.0);
    }
}