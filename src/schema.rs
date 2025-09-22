//! # Schema Validation
//!
//! Type-safe schema validation for configuration values.
//! Provides compile-time and runtime type checking with detailed error reporting.

use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::{BTreeMap, HashMap};

/// Configuration schema definition
#[derive(Debug, Clone)]
pub struct Schema {
    fields: HashMap<String, FieldSchema>,
}

/// Schema definition for a single field
#[derive(Debug, Clone, PartialEq)]
pub struct FieldSchema {
    field_type: FieldType,
    required: bool,
    default: Option<Value>,
    description: Option<String>,
}

/// Supported field types for validation
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// Null value
    Null,
    /// Boolean value
    Bool,
    /// Integer value
    Integer,
    /// Float value
    Float,
    /// String value
    String,
    /// Array of specific type
    Array(Box<FieldType>),
    /// Table/object with field schemas
    Table(HashMap<String, FieldSchema>),
    /// Union of multiple types
    Union(Vec<FieldType>),
    /// Any type (no validation)
    Any,
}

/// Builder for creating schemas
pub struct SchemaBuilder {
    fields: HashMap<String, FieldSchema>,
}

impl SchemaBuilder {
    /// Create a new schema builder
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Add a required string field
    pub fn require_string(mut self, name: &str) -> Self {
        self.fields.insert(
            name.to_string(),
            FieldSchema {
                field_type: FieldType::String,
                required: true,
                default: None,
                description: None,
            },
        );
        self
    }

    /// Add a required integer field
    pub fn require_integer(mut self, name: &str) -> Self {
        self.fields.insert(
            name.to_string(),
            FieldSchema {
                field_type: FieldType::Integer,
                required: true,
                default: None,
                description: None,
            },
        );
        self
    }

    /// Add a required boolean field
    pub fn require_bool(mut self, name: &str) -> Self {
        self.fields.insert(
            name.to_string(),
            FieldSchema {
                field_type: FieldType::Bool,
                required: true,
                default: None,
                description: None,
            },
        );
        self
    }

    /// Add an optional string field
    pub fn optional_string(mut self, name: &str) -> Self {
        self.fields.insert(
            name.to_string(),
            FieldSchema {
                field_type: FieldType::String,
                required: false,
                default: None,
                description: None,
            },
        );
        self
    }

    /// Add an optional integer field
    pub fn optional_integer(mut self, name: &str) -> Self {
        self.fields.insert(
            name.to_string(),
            FieldSchema {
                field_type: FieldType::Integer,
                required: false,
                default: None,
                description: None,
            },
        );
        self
    }

    /// Add an optional boolean field
    pub fn optional_bool(mut self, name: &str) -> Self {
        self.fields.insert(
            name.to_string(),
            FieldSchema {
                field_type: FieldType::Bool,
                required: false,
                default: None,
                description: None,
            },
        );
        self
    }

    /// Add a field with custom type
    pub fn field(mut self, name: &str, field_type: FieldType, required: bool) -> Self {
        self.fields.insert(
            name.to_string(),
            FieldSchema {
                field_type,
                required,
                default: None,
                description: None,
            },
        );
        self
    }

    /// Add a field with default value
    pub fn field_with_default(mut self, name: &str, field_type: FieldType, default: Value) -> Self {
        self.fields.insert(
            name.to_string(),
            FieldSchema {
                field_type,
                required: false,
                default: Some(default),
                description: None,
            },
        );
        self
    }

    /// Add description to the last added field
    pub fn with_description(mut self, description: &str) -> Self {
        if let Some((_, field)) = self.fields.iter_mut().last() {
            field.description = Some(description.to_string());
        }
        self
    }

    /// Build the schema
    pub fn build(self) -> Schema {
        Schema {
            fields: self.fields,
        }
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Schema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Create a schema from a builder
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    /// Validate a value against this schema
    pub fn validate(&self, value: &Value) -> Result<()> {
        match value {
            Value::Table(table) => self.validate_table(table, ""),
            _ => Err(Error::schema("", "Root value must be a table")),
        }
    }

    /// Validate a table against the schema
    fn validate_table(&self, table: &BTreeMap<String, Value>, path: &str) -> Result<()> {
        // Check required fields
        for (field_name, field_schema) in &self.fields {
            let field_path = if path.is_empty() {
                field_name.clone()
            } else {
                format!("{}.{}", path, field_name)
            };

            match table.get(field_name) {
                Some(value) => {
                    self.validate_field(value, field_schema, &field_path)?;
                }
                None => {
                    if field_schema.required {
                        return Err(Error::schema(
                            field_path,
                            format!("Required field '{}' is missing", field_name),
                        ));
                    }
                }
            }
        }

        // Check for unknown fields (optional - could be configurable)
        for field_name in table.keys() {
            if !self.fields.contains_key(field_name) {
                // For now, we allow unknown fields
                // Could add strict mode later
            }
        }

        Ok(())
    }

    /// Validate a single field
    fn validate_field(&self, value: &Value, schema: &FieldSchema, path: &str) -> Result<()> {
        self.validate_type(value, &schema.field_type, path)
    }

    /// Validate a value against a type
    fn validate_type(&self, value: &Value, field_type: &FieldType, path: &str) -> Result<()> {
        match (value, field_type) {
            (Value::Null, FieldType::Null) => Ok(()),
            (Value::Bool(_), FieldType::Bool) => Ok(()),
            (Value::Integer(_), FieldType::Integer) => Ok(()),
            (Value::Float(_), FieldType::Float) => Ok(()),
            (Value::String(_), FieldType::String) => Ok(()),

            // Allow integer to float conversion
            (Value::Integer(_), FieldType::Float) => Ok(()),

            // Array validation
            (Value::Array(arr), FieldType::Array(element_type)) => {
                for (i, element) in arr.iter().enumerate() {
                    let element_path = format!("{}[{}]", path, i);
                    self.validate_type(element, element_type, &element_path)?;
                }
                Ok(())
            }

            // Table validation
            (Value::Table(table), FieldType::Table(table_schema)) => {
                // Create a temporary schema for nested validation
                let nested_schema = Schema {
                    fields: table_schema.clone(),
                };
                nested_schema.validate_table(table, path)
            }

            // Union type validation
            (value, FieldType::Union(types)) => {
                for union_type in types {
                    if self.validate_type(value, union_type, path).is_ok() {
                        return Ok(());
                    }
                }
                Err(Error::schema(
                    path.to_string(),
                    format!("Value does not match any of the union types: {:?}", types),
                ))
            }

            // Any type always validates
            (_, FieldType::Any) => Ok(()),

            // Type mismatch
            _ => Err(Error::schema(
                path.to_string(),
                format!("Expected {:?}, found {}", field_type, value.type_name()),
            )),
        }
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_simple_schema() {
        let schema = SchemaBuilder::new()
            .require_string("name")
            .require_integer("port")
            .optional_bool("debug")
            .build();

        // Valid config
        let mut config = BTreeMap::new();
        config.insert("name".to_string(), Value::string("test"));
        config.insert("port".to_string(), Value::integer(8080));
        let config = Value::table(config);

        assert!(schema.validate(&config).is_ok());

        // Missing required field
        let mut config = BTreeMap::new();
        config.insert("name".to_string(), Value::string("test"));
        let config = Value::table(config);

        assert!(schema.validate(&config).is_err());

        // Wrong type
        let mut config = BTreeMap::new();
        config.insert("name".to_string(), Value::string("test"));
        config.insert("port".to_string(), Value::string("not a number"));
        let config = Value::table(config);

        assert!(schema.validate(&config).is_err());
    }

    #[test]
    fn test_array_schema() {
        let schema = SchemaBuilder::new()
            .field("items", FieldType::Array(Box::new(FieldType::String)), true)
            .build();

        // Valid array
        let mut config = BTreeMap::new();
        config.insert(
            "items".to_string(),
            Value::array(vec![
                Value::string("a"),
                Value::string("b"),
                Value::string("c"),
            ]),
        );
        let config = Value::table(config);

        assert!(schema.validate(&config).is_ok());

        // Invalid array element
        let mut config = BTreeMap::new();
        config.insert(
            "items".to_string(),
            Value::array(vec![
                Value::string("a"),
                Value::integer(123), // Wrong type
                Value::string("c"),
            ]),
        );
        let config = Value::table(config);

        assert!(schema.validate(&config).is_err());
    }

    #[test]
    fn test_union_type() {
        let schema = SchemaBuilder::new()
            .field(
                "value",
                FieldType::Union(vec![FieldType::String, FieldType::Integer]),
                true,
            )
            .build();

        // String value
        let mut config = BTreeMap::new();
        config.insert("value".to_string(), Value::string("test"));
        let config = Value::table(config);
        assert!(schema.validate(&config).is_ok());

        // Integer value
        let mut config = BTreeMap::new();
        config.insert("value".to_string(), Value::integer(42));
        let config = Value::table(config);
        assert!(schema.validate(&config).is_ok());

        // Invalid type
        let mut config = BTreeMap::new();
        config.insert("value".to_string(), Value::bool(true));
        let config = Value::table(config);
        assert!(schema.validate(&config).is_err());
    }
}
