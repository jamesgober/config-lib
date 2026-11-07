//! # NOML Format Parser
//!
//! Integration with the NOML library for advanced configuration features.
//! [`parse`] uses the full NOML library to provide:
//!
//! - Environment variable resolution (`env("NAME")`)
//! - File includes (`include "path"`)
//! - Variable interpolation
//! - Native types (@size, @duration, etc.)
//! - Format preservation for round-trip editing
//!
//! [`parse_strict`] (since v1.3.0) reads only the text it is given: it
//! refuses `include` and every function call, `env()` included, with
//! `Error::Parse` and the line and column, before anything is resolved.
//! Native types and `${...}` references to other values in the same
//! document still work.

#[cfg(feature = "noml")]
use super::noml_guard::{self, Policy};
#[cfg(not(feature = "noml"))]
use crate::error::Error;
use crate::error::Result;
use crate::value::Value;
use std::collections::BTreeMap;

/// Parse NOML format configuration using the noml library
#[cfg(feature = "noml")]
pub fn parse(source: &str) -> Result<Value> {
    // Parse the document
    let document = noml::parse_string(source, None)?;

    // Resolve dynamic features (env vars, includes, etc.)
    let mut resolver = noml::Resolver::new();
    let resolved = resolver.resolve(&document)?;

    convert_noml_value(resolved)
}

/// Parse NOML without reading anything outside `source`.
///
/// Same as [`parse`], except that `include` and every function call
/// (`env()` included) are refused with `Error::Parse` carrying the line and
/// column of the first one. The message does not quote the input. Native
/// types (`@size("10MB")`) and `${...}` references to other values in the
/// document are allowed, since they read only the document.
///
/// Use this when the caller has checked the configuration file itself
/// (owner, mode, size, file type): an include would read a second file
/// without those checks, and `env()` would copy any environment variable
/// into the configuration.
#[cfg(feature = "noml")]
pub fn parse_strict(source: &str) -> Result<Value> {
    let document = noml::parse_string(source, None)?;
    noml_guard::check(&document, Policy::StrictNoml)?;
    let resolved = noml::Resolver::new().resolve(&document)?;
    convert_noml_value(resolved)
}

/// Parse NOML format configuration (fallback when NOML is not available)
#[cfg(not(feature = "noml"))]
pub fn parse(_source: &str) -> Result<Value> {
    Err(Error::general(
        "NOML parsing requires the 'noml' feature to be enabled",
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
        noml::Value::DateTime(_) => Ok(Value::String("datetime_value".to_string())),
        // Handle NOML-specific types by converting to basic types.
        //
        // Binary values render as the literal `"<binary data>"` placeholder
        // string. A previous version of this module proposed feature-gated
        // base64 encoding for binary values, but the `base64` Cargo feature
        // was never defined — the code path was dead and the optional
        // `base64` dependency was unused. Removed for v0.9.7.
        noml::Value::Binary(_) => Ok(Value::String("<binary data>".to_string())),
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
        let config = parse(
            r#"
            name = "test"
            port = 8080
            debug = true
        "#,
        )
        .unwrap();

        assert_eq!(config.get("name").unwrap().as_string().unwrap(), "test");
        assert_eq!(config.get("port").unwrap().as_integer().unwrap(), 8080);
        assert!(config.get("debug").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_noml_features() {
        std::env::set_var("TEST_VAR", "hello");

        let config = parse(
            r#"
            greeting = env("TEST_VAR", "world")
            size = @size("10MB")
            timeout = @duration("30s")
        "#,
        )
        .unwrap();

        assert_eq!(
            config.get("greeting").unwrap().as_string().unwrap(),
            "hello"
        );
        // Size converted to bytes
        assert_eq!(config.get("size").unwrap().as_integer().unwrap(), 10485760);
        // Duration converted to seconds
        assert_eq!(config.get("timeout").unwrap().as_float().unwrap(), 30.0);
    }
}
