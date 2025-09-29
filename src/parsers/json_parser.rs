//! # JSON Format Parser
//!
//! JSON parser with potential for edit capabilities through structured preservation.

use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::BTreeMap;

/// Parse JSON format configuration  
pub fn parse(source: &str) -> Result<Value> {
    let json_value: serde_json::Value = serde_json::from_str(source)
        .map_err(|e| Error::parse(format!("JSON parse error: {e}"), e.line(), e.column()))?;

    convert_json_value(json_value)
}

/// Convert serde_json::Value to config-lib Value
fn convert_json_value(json_value: serde_json::Value) -> Result<Value> {
    match json_value {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(Error::parse(
                    format!("Invalid number: {n}"),
                    1,
                    1, // JSON parser doesn't give us position info
                ))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s)),
        serde_json::Value::Array(arr) => {
            let converted: Result<Vec<Value>> = arr.into_iter().map(convert_json_value).collect();
            Ok(Value::Array(converted?))
        }
        serde_json::Value::Object(obj) => {
            let mut converted = BTreeMap::new();
            for (key, value) in obj {
                converted.insert(key, convert_json_value(value)?);
            }
            Ok(Value::Table(converted))
        }
    }
}

/// Serialize config-lib Value back to JSON
pub fn serialize(value: &Value) -> Result<String> {
    let json_value = convert_to_json_value(value)?;
    serde_json::to_string_pretty(&json_value)
        .map_err(|e| Error::internal(format!("JSON serialization error: {e}")))
}

/// Convert config-lib Value to serde_json::Value
fn convert_to_json_value(value: &Value) -> Result<serde_json::Value> {
    match value {
        Value::Null => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                Ok(serde_json::Value::Number(n))
            } else {
                Err(Error::type_error(
                    f.to_string(),
                    "valid JSON number",
                    "float",
                ))
            }
        }
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Array(arr) => {
            let converted: Result<Vec<serde_json::Value>> =
                arr.iter().map(convert_to_json_value).collect();
            Ok(serde_json::Value::Array(converted?))
        }
        Value::Table(table) => {
            let mut converted = serde_json::Map::new();
            for (key, value) in table {
                converted.insert(key.clone(), convert_to_json_value(value)?);
            }
            Ok(serde_json::Value::Object(converted))
        }
        #[cfg(feature = "chrono")]
        Value::DateTime(dt) => Ok(serde_json::Value::String(dt.to_rfc3339())),
    }
}

/// Convert serde_json::Value to config-lib Value (alias for enterprise usage)
pub fn from_json_value(json_value: serde_json::Value) -> Result<Value> {
    convert_json_value(json_value)
}

/// Convert config-lib Value to serde_json::Value for enterprise usage  
pub fn to_json_value(value: &Value) -> Result<serde_json::Value> {
    match value {
        Value::Null => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(*i))),
        Value::Float(f) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(*f)
                .ok_or_else(|| Error::serialize("Invalid float value".to_string()))?,
        )),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Array(arr) => {
            let converted: Result<Vec<serde_json::Value>> = arr.iter().map(to_json_value).collect();
            Ok(serde_json::Value::Array(converted?))
        }
        Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (key, value) in table {
                map.insert(key.clone(), to_json_value(value)?);
            }
            Ok(serde_json::Value::Object(map))
        }
        #[cfg(feature = "chrono")]
        Value::DateTime(dt) => Ok(serde_json::Value::String(dt.to_rfc3339())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_json() -> crate::Result<()> {
        let config = parse(
            r#"
        {
            "name": "test",
            "port": 8080,
            "database": {
                "host": "localhost"
            }
        }
        "#,
        )?;

        assert_eq!(
            config
                .get("name")
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: "name".to_string(),
                    available: vec![
                        "name".to_string(),
                        "port".to_string(),
                        "database".to_string()
                    ],
                })?
                .as_string()?,
            "test"
        );
        assert_eq!(
            config
                .get("port")
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: "port".to_string(),
                    available: vec![
                        "name".to_string(),
                        "port".to_string(),
                        "database".to_string()
                    ],
                })?
                .as_integer()?,
            8080
        );
        assert_eq!(
            config
                .get("database.host")
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: "database.host".to_string(),
                    available: vec![
                        "name".to_string(),
                        "port".to_string(),
                        "database".to_string()
                    ],
                })?
                .as_string()?,
            "localhost"
        );
        Ok(())
    }

    #[test]
    fn test_json_arrays() -> crate::Result<()> {
        let config = parse(
            r#"
        {
            "servers": ["alpha", "beta", "gamma"],
            "ports": [8001, 8002, 8003]
        }
        "#,
        )?;

        let servers = config
            .get("servers")
            .ok_or_else(|| crate::Error::KeyNotFound {
                key: "servers".to_string(),
                available: vec!["servers".to_string(), "ports".to_string()],
            })?
            .as_array()?;
        assert_eq!(servers.len(), 3);
        assert_eq!(servers[0].as_string()?, "alpha");

        let ports = config
            .get("ports")
            .ok_or_else(|| crate::Error::KeyNotFound {
                key: "ports".to_string(),
                available: vec!["servers".to_string(), "ports".to_string()],
            })?
            .as_array()?;
        assert_eq!(ports[0].as_integer()?, 8001);
        Ok(())
    }

    #[test]
    fn test_json_serialization() -> crate::Result<()> {
        let mut table = BTreeMap::new();
        table.insert("name".to_string(), Value::String("test".to_string()));
        table.insert("port".to_string(), Value::Integer(8080));
        let config = Value::Table(table);

        let json = serialize(&config)?;
        assert!(json.contains("\"name\": \"test\""));
        assert!(json.contains("\"port\": 8080"));
        Ok(())
    }
}
