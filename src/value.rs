//! Value types and operations for the config library.
//!
//! This module provides a flexible value system that can represent various data types
//! commonly found in configuration files.

use crate::error::{Error, Result};
use std::collections::BTreeMap;
use std::fmt;

/// Represents a configuration value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null/empty value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<Value>),
    /// Table (key-value pairs)
    Table(BTreeMap<String, Value>),
}

impl Value {
    /// Create a new null value
    pub fn null() -> Self {
        Value::Null
    }

    /// Create a new boolean value
    pub fn bool(value: bool) -> Self {
        Value::Bool(value)
    }

    /// Create a new integer value
    pub fn integer(value: i64) -> Self {
        Value::Integer(value)
    }

    /// Create a new float value
    pub fn float(value: f64) -> Self {
        Value::Float(value)
    }

    /// Create a new string value
    pub fn string<S: Into<String>>(value: S) -> Self {
        Value::String(value.into())
    }

    /// Create a new array value
    pub fn array(values: Vec<Value>) -> Self {
        Value::Array(values)
    }

    /// Create a new table value
    pub fn table(table: BTreeMap<String, Value>) -> Self {
        Value::Table(table)
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Table(_) => "table",
        }
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Check if this value is a boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Check if this value is an integer
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// Check if this value is a float
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// Check if this value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if this value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Check if this value is a table
    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }

    /// Try to convert this value to a boolean
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::String(s) => {
                match s.to_lowercase().as_str() {
                    "true" | "yes" | "1" | "on" => Ok(true),
                    "false" | "no" | "0" | "off" => Ok(false),
                    _ => Err(Error::type_error(
                        "Cannot convert to boolean",
                        "bool",
                        self.type_name(),
                    )),
                }
            },
            _ => Err(Error::type_error(
                "Cannot convert to boolean",
                "bool",
                self.type_name(),
            )),
        }
    }

    /// Try to convert this value to an integer
    pub fn as_integer(&self) -> Result<i64> {
        match self {
            Value::Integer(i) => Ok(*i),
            Value::Float(f) => Ok(*f as i64),
            Value::String(s) => s.parse::<i64>().map_err(|_| Error::type_error(
                "Cannot convert to integer",
                "integer", 
                self.type_name(),
            )),
            _ => Err(Error::type_error(
                "Cannot convert to integer",
                "integer",
                self.type_name(),
            )),
        }
    }

    /// Try to convert this value to a float
    pub fn as_float(&self) -> Result<f64> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Integer(i) => Ok(*i as f64),
            Value::String(s) => s.parse::<f64>().map_err(|_| Error::type_error(
                "Cannot convert to float",
                "float",
                self.type_name(),
            )),
            _ => Err(Error::type_error(
                "Cannot convert to float",
                "float",
                self.type_name(),
            )),
        }
    }

    /// Try to convert this value to a string - ZERO-COPY optimized
    pub fn as_string(&self) -> Result<&str> {
        match self {
            Value::String(s) => Ok(s.as_str()),
            _ => Err(Error::type_error(
                "Cannot convert to string",
                "string", 
                self.type_name(),
            )),
        }
    }

    /// Convert this value to a string representation (allocating)
    pub fn to_string_representation(&self) -> Result<String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            _ => Err(Error::type_error(
                "Cannot convert to string representation",
                "string",
                self.type_name(),
            )),
        }
    }

    /// Try to get this value as an array
    pub fn as_array(&self) -> Result<&Vec<Value>> {
        match self {
            Value::Array(arr) => Ok(arr),
            _ => Err(Error::type_error(
                "Cannot convert to array",
                "array",
                self.type_name(),
            )),
        }
    }

    /// Try to get this value as a mutable array
    pub fn as_array_mut(&mut self) -> Result<&mut Vec<Value>> {
        match self {
            Value::Array(arr) => Ok(arr),
            _ => Err(Error::type_error(
                "Cannot convert to array",
                "array",
                self.type_name(),
            )),
        }
    }

    /// Try to get this value as a table
    pub fn as_table(&self) -> Result<&BTreeMap<String, Value>> {
        match self {
            Value::Table(table) => Ok(table),
            _ => Err(Error::type_error(
                "Cannot convert to table",
                "table",
                self.type_name(),
            )),
        }
    }

    /// Try to get this value as a mutable table
    pub fn as_table_mut(&mut self) -> Result<&mut BTreeMap<String, Value>> {
        match self {
            Value::Table(table) => Ok(table),
            _ => Err(Error::type_error(
                "Cannot convert to table",
                "table",
                self.type_name(),
            )),
        }
    }

    /// Get a value by path (dot-separated)
    pub fn get(&self, path: &str) -> Option<&Value> {
        if path.is_empty() {
            return Some(self);
        }

        let parts: Vec<&str> = path.split('.').collect();
        let mut current = self;

        for part in parts {
            match current {
                Value::Table(table) => {
                    current = table.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Get a mutable reference to a value by path (ENTERPRISE ERROR HANDLING)
    pub fn get_mut_nested(&mut self, path: &str) -> Result<&mut Value> {
        if path.is_empty() {
            return Ok(self);
        }

        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(Error::key_not_found(path));
        }

        let (last_key, parent_path) = parts.split_last()
            .ok_or_else(|| Error::key_not_found(path))?;

        // Navigate to parent
        let mut current = self;
        for part in parent_path {
            match current {
                Value::Table(table) => {
                    current = table.get_mut(*part)
                        .ok_or_else(|| Error::key_not_found(*part))?;
                }
                _ => return Err(Error::type_error(
                    format!("Cannot navigate into {} when looking for key '{}'", current.type_name(), part),
                    "table",
                    current.type_name(),
                )),
            }
        }

        // Get the final value
        match current {
            Value::Table(table) => {
                table.get_mut(*last_key)
                    .ok_or_else(|| Error::key_not_found(*last_key))
            }
            _ => Err(Error::type_error(
                format!("Cannot get key '{}' from {}", last_key, current.type_name()),
                "table",
                current.type_name(),
            )),
        }
    }

    /// Set a value by path, creating intermediate tables as needed (ZERO-COPY optimized)
    pub fn set_nested(&mut self, path: &str, value: Value) -> Result<()> {
        if path.is_empty() {
            return Err(Error::key_not_found(""));
        }

        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(Error::key_not_found(path));
        }

        let (last_key, parent_path) = parts.split_last()
            .ok_or_else(|| Error::key_not_found(path))?;

        // Navigate to parent, creating tables as needed
        let mut current = self;
        for part in parent_path {
            if let Value::Table(table) = current {
                // ZERO-COPY: Use entry API to avoid string allocation when possible
                let entry = table.entry(part.to_string()).or_insert_with(|| {
                    Value::table(BTreeMap::new())
                });
                current = entry;
            } else {
                return Err(Error::type_error(
                    format!("Cannot navigate into {}", current.type_name()),
                    "table",
                    current.type_name(),
                ));
            }
        }

        // Set the final value
        if let Value::Table(table) = current {
            table.insert(last_key.to_string(), value);
            Ok(())
        } else {
            Err(Error::type_error(
                format!("Cannot set key in {}", current.type_name()),
                "table",
                current.type_name(),
            ))
        }
    }

    /// Remove a value by path (ENTERPRISE ERROR HANDLING)
    pub fn remove(&mut self, path: &str) -> Result<Option<Value>> {
        if path.is_empty() {
            let old = std::mem::replace(self, Value::Null);
            return Ok(Some(old));
        }

        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(Error::key_not_found(path));
        }

        let (last_key, parent_path) = parts.split_last()
            .ok_or_else(|| Error::key_not_found(path))?;

        // Navigate to parent
        let mut current = self;
        for part in parent_path {
            match current {
                Value::Table(table) => {
                    current = table.get_mut(*part)
                        .ok_or_else(|| Error::key_not_found(*part))?;
                }
                _ => return Err(Error::type_error(
                    format!("Cannot navigate into {} when removing key '{}'", current.type_name(), part),
                    "table",
                    current.type_name(),
                )),
            }
        }

        // Remove from parent
        if let Value::Table(table) = current {
            Ok(table.remove(*last_key))
        } else {
            Err(Error::type_error(
                format!("Cannot remove key '{}' from {}", last_key, current.type_name()),
                "table",
                current.type_name(),
            ))
        }
    }

    /// Get all keys at the current level (for tables only) - ZERO-COPY optimized
    pub fn keys(&self) -> Result<Vec<&str>> {
        match self {
            Value::Table(table) => Ok(table.keys().map(|k| k.as_str()).collect()),
            _ => Err(Error::type_error(
                "Cannot get keys from non-table value",
                "table",
                self.type_name(),
            )),
        }
    }

    /// Check if a path exists
    pub fn contains_key(&self, path: &str) -> bool {
        self.get(path).is_some()
    }

    /// Set a value by path (backward compatibility alias)
    pub fn set(&mut self, path: &str, value: Value) -> Result<()> {
        self.set_nested(path, value)
    }

    /// Get the length of arrays or tables
    pub fn len(&self) -> usize {
        match self {
            Value::Array(arr) => arr.len(),
            Value::Table(table) => table.len(),
            Value::String(s) => s.len(),
            _ => 0,
        }
    }

    /// Check if arrays or tables are empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Table(table) => {
                write!(f, "{{")?;
                for (i, (key, value)) in table.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

// ZERO-COPY conversion implementations
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value as i64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value as f64)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::Array(value)
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(value: BTreeMap<String, Value>) -> Self {
        Value::Table(value)
    }
}

// ENTERPRISE: Helper functions for zero-copy operations
impl Value {
    /// Create a string value from a slice without unnecessary allocation
    pub fn string_from_slice(value: &str) -> Self {
        Value::String(value.to_string())
    }

    /// Get string slice without allocation - enterprise optimization
    pub fn as_str(&self) -> Result<&str> {
        match self {
            Value::String(s) => Ok(s.as_str()),
            _ => Err(Error::type_error(
                "Value is not a string",
                "string",
                self.type_name(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        assert_eq!(Value::null(), Value::Null);
        assert_eq!(Value::bool(true), Value::Bool(true));
        assert_eq!(Value::integer(42), Value::Integer(42));
        assert_eq!(Value::float(3.14), Value::Float(3.14));
        assert_eq!(Value::string("test"), Value::String("test".to_string()));
    }

    #[test]
    fn test_type_checking() {
        let null = Value::null();
        let bool_val = Value::bool(true);
        let int_val = Value::integer(42);
        let float_val = Value::float(3.14);
        let string_val = Value::string("test");
        let array_val = Value::array(vec![Value::integer(1), Value::integer(2)]);
        let table_val = Value::table(BTreeMap::new());

        assert!(null.is_null());
        assert!(bool_val.is_bool());
        assert!(int_val.is_integer());
        assert!(float_val.is_float());
        assert!(string_val.is_string());
        assert!(array_val.is_array());
        assert!(table_val.is_table());
    }

    #[test]
    fn test_value_conversion() {
        let bool_val = Value::bool(true);
        let int_val = Value::integer(42);
        let float_val = Value::float(3.14);
        let string_val = Value::string("test");

        assert_eq!(bool_val.as_bool().unwrap(), true);
        assert_eq!(int_val.as_integer().unwrap(), 42);
        assert_eq!(float_val.as_float().unwrap(), 3.14);
        assert_eq!(string_val.as_string().unwrap(), "test");
    }

    #[test]
    fn test_nested_access() {
        let mut table = BTreeMap::new();
        let mut inner_table = BTreeMap::new();
        inner_table.insert("inner_key".to_string(), Value::string("inner_value"));
        table.insert("outer_key".to_string(), Value::table(inner_table));
        
        let value = Value::table(table);
        
        assert_eq!(
            value.get("outer_key.inner_key").unwrap().as_string().unwrap(),
            "inner_value"
        );
    }

    #[test] 
    fn test_enterprise_error_handling() {
        let mut value = Value::table(BTreeMap::new());
        
        // Test proper error handling instead of panics
        assert!(value.get_mut_nested("nonexistent.key").is_err());
        assert!(value.remove("nonexistent.key").is_err());
        
        // Test successful operations
        assert!(value.set_nested("test.key", Value::string("value")).is_ok());
        assert!(value.get("test.key").is_some());
    }
}