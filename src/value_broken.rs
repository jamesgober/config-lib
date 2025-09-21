//! # Value System
//!
//! Core value types and operations for configuration data representation.
//! This module provides the [`Value`] enum and all associated functionality
//! for working with configuration data in a type-safe manner.
//!
//! ## Value Types
//!
//! Supports all standard configuration data types:
//!
//! - **Primitives**: `null`, `bool`, `i64`, `f64`, `String`
//! - **Collections**: `Array<Value>`, `Table<String, Value>`
//! - **Optional**: `DateTime` (with `chrono` feature)
//!
//! ## Type Conversions
//!
//! The [`Value`] type provides safe conversions with comprehensive error handling:
//!
//! ```rust
//! use config_lib::Value;
//!
//! let value = Value::string("42");
//!
//! // Safe conversions with error handling
//! let as_int = value.as_integer()?;  // Ok(42)
//! let as_float = value.as_float()?;  // Ok(42.0)
//! let as_bool = value.as_bool();     // Err(type mismatch)
//!
//! # Ok::<(), config_lib::Error>(())
//! ```

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

/// A configuration value - the fundamental unit of data in configuration documents.
///
/// Values are designed to be lightweight, cloneable, and convertible
/// to/from Rust native types with zero-copy operations where possible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// Null/empty value
    Null,

    /// Boolean value (true/false)
    Bool(bool),

    /// Integer value (64-bit signed)
    Integer(i64),

    /// Floating-point value (64-bit)
    Float(f64),

    /// String value (UTF-8)
    String(String),

    /// Array of values
    Array(Vec<Value>),

    /// Table/object with string keys
    Table(BTreeMap<String, Value>),

    /// Date/time value (optional feature)
    #[cfg(feature = "chrono")]
    #[serde(with = "chrono::serde::ts_seconds")]
    DateTime(DateTime<Utc>),
}

impl Value {
    /// Create a new null value
    #[inline(always)]
    pub fn null() -> Self {
        Value::Null
    }

    /// Create a new boolean value
    #[inline(always)]
    pub fn bool(value: bool) -> Self {
        Value::Bool(value)
    }

    /// Create a new integer value
    #[inline(always)]
    pub fn integer(value: i64) -> Self {
        Value::Integer(value)
    }

    /// Create a new float value
    #[inline(always)]
    pub fn float(value: f64) -> Self {
        Value::Float(value)
    }

    /// Create a new string value
    #[inline(always)]
    pub fn string(value: impl Into<String>) -> Self {
        Value::String(value.into())
    }

    /// Create a new array value
    #[inline(always)]
    pub fn array(value: Vec<Value>) -> Self {
        Value::Array(value)
    }

    /// Create a new table value
    #[inline(always)]
    pub fn table(value: BTreeMap<String, Value>) -> Self {
        Value::Table(value)
    }

    /// Create a new datetime value
    #[cfg(feature = "chrono")]
    #[inline(always)]
    pub fn datetime(value: DateTime<Utc>) -> Self {
        Value::DateTime(value)
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
            #[cfg(feature = "chrono")]
            Value::DateTime(_) => "datetime",
        }
    }

    /// Check if this value is null
    #[inline(always)]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Check if this value is a boolean
    #[inline(always)]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Check if this value is an integer
    #[inline(always)]
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// Check if this value is a float
    #[inline(always)]
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// Check if this value is a string
    #[inline(always)]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if this value is an array
    #[inline(always)]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Check if this value is a table
    #[inline(always)]
    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }

    /// Convert to boolean with intelligent string parsing
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::String(s) => {
                match s.to_lowercase().as_str() {
                    "true" | "yes" | "on" | "1" => Ok(true),
                    "false" | "no" | "off" | "0" => Ok(false),
                    _ => Err(Error::type_error(s, "bool", self.type_name())),
                }
            }
            Value::Integer(i) => Ok(*i != 0),
            _ => Err(Error::type_error(
                format!("{:?}", self),
                "bool",
                self.type_name(),
            )),
        }
    }

    /// Convert to integer with string parsing
    pub fn as_integer(&self) -> Result<i64> {
        match self {
            Value::Integer(i) => Ok(*i),
            Value::Float(f) => {
                if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                    Ok(*f as i64)
                } else {
                    Err(Error::type_error(
                        f.to_string(),
                        "integer",
                        self.type_name(),
                    ))
                }
            }
            Value::String(s) => s.parse::<i64>().map_err(|_| {
                Error::type_error(s, "integer", self.type_name())
            }),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(Error::type_error(
                format!("{:?}", self),
                "integer",
                self.type_name(),
            )),
        }
    }

    /// Convert to float with string parsing
    pub fn as_float(&self) -> Result<f64> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Integer(i) => Ok(*i as f64),
            Value::String(s) => s.parse::<f64>().map_err(|_| {
                Error::type_error(s, "float", self.type_name())
            }),
            _ => Err(Error::type_error(
                format!("{:?}", self),
                "float",
                self.type_name(),
            )),
        }
    }

    /// Convert to string
    pub fn as_string(&self) -> Result<&str> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(Error::type_error(
                format!("{:?}", self),
                "string",
                self.type_name(),
            )),
        }
    }

    /// Convert to string (owned)
    pub fn as_string_owned(&self) -> Result<String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            _ => Err(Error::type_error(
                format!("{:?}", self),
                "string",
                self.type_name(),
            )),
        }
    }

    /// Convert to array
    pub fn as_array(&self) -> Result<&Vec<Value>> {
        match self {
            Value::Array(arr) => Ok(arr),
            _ => Err(Error::type_error(
                format!("{:?}", self),
                "array",
                self.type_name(),
            )),
        }
    }

    /// Convert to table
    pub fn as_table(&self) -> Result<&BTreeMap<String, Value>> {
        match self {
            Value::Table(table) => Ok(table),
            _ => Err(Error::type_error(
                format!("{:?}", self),
                "table",
                self.type_name(),
            )),
        }
    }

    /// Convert to datetime
    #[cfg(feature = "chrono")]
    pub fn as_datetime(&self) -> Result<&DateTime<Utc>> {
        match self {
            Value::DateTime(dt) => Ok(dt),
            _ => Err(Error::type_error(
                format!("{:?}", self),
                "datetime",
                self.type_name(),
            )),
        }
    }

    /// Get a value by path (dot-separated keys)
    ///
    /// Examples:
    /// - `"key"` - top-level key
    /// - `"section.key"` - nested key
    /// - `"section.subsection.key"` - deeply nested key
    pub fn get(&self, path: &str) -> Option<&Value> {
        if path.is_empty() {
            return Some(self);
        }

        let mut current = self;
        for part in path.split('.') {
            match current {
                Value::Table(table) => {
                    current = table.get(part)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    /// Get a mutable reference to a value by path
    pub fn get_mut_nested(&mut self, path: &str) -> Result<&mut Value> {
        if path.is_empty() {
            return Err(Error::key_not_found(""));
    }

    /// Set a value by path, creating intermediate tables as needed
    pub fn set_nested(&mut self, path: &str, value: Value) -> Result<()> {
        if path.is_empty() {
            return Err(Error::key_not_found(""));
        }

        let parts: Vec<&str> = path.split('.').collect();
        let (last_key, parent_path) = parts.split_last()
            .ok_or_else(|| Error::key_not_found(path))?;

        // Navigate to parent, creating tables as needed
        let mut current = self;
        for part in parent_path {
            if let Value::Table(table) = current {
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

    /// Remove a value by path
    pub fn remove(&mut self, path: &str) -> Option<Value> {
        if path.is_empty() {
            let old = std::mem::replace(self, Value::Null);
            return Some(old);
        }

        let parts: Vec<&str> = path.split('.').collect();
        let (last_key, parent_path) = parts.split_last().unwrap();

        // Navigate to parent
        let mut current = self;
        for part in parent_path {
            match current {
                Value::Table(table) => {
                    current = table.get_mut(&part.to_string())?;
                }
                _ => return None,
            }
        }

        // Remove from parent
        if let Value::Table(table) = current {
            table.remove(*last_key)
        } else {
            None
        }
    }

    /// Get all keys at the current level (for tables only)
    pub fn keys(&self) -> Vec<String> {
        match self {
            Value::Table(table) => table.keys().cloned().collect(),
            _ => Vec::new(),
        }
    }

    /// Check if a path exists
    pub fn contains_key(&self, path: &str) -> bool {
        self.get(path).is_some()
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
            #[cfg(feature = "chrono")]
            Value::DateTime(dt) => write!(f, "{}", dt.to_rfc3339()),
        }
    }
}

/// Convert from basic Rust types
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value as i64)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value as f64)
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

#[cfg(feature = "chrono")]
impl From<DateTime<Utc>> for Value {
    fn from(value: DateTime<Utc>) -> Self {
        Value::DateTime(value)
    }
}