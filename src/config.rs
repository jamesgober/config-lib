//! # High-Level Configuration Management
//!
//! Advanced configuration management API providing intuitive interfaces for
//! loading, modifying, validating, and saving configurations with format preservation.

use crate::error::{Error, Result};
use crate::parsers;
use crate::value::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[cfg(feature = "schema")]
use crate::schema::Schema;

/// High-level configuration manager with format preservation and change tracking
///
/// [`Config`] provides a comprehensive API for managing configurations
/// throughout their lifecycle. It maintains both the resolved values (for fast access)
/// and format-specific preservation data (for round-trip editing).
///
/// ## Key Features
///
/// - **Format Preservation**: Maintains comments, whitespace, and original formatting
/// - **Change Tracking**: Automatic detection of modifications
/// - **Type Safety**: Rich type conversion with comprehensive error handling
/// - **Path-based Access**: Dot notation for nested value access
/// - **Multi-format Support**: CONF, TOML, JSON, NOML formats
/// - **Schema Validation**: Optional schema validation and enforcement
/// - **Async Support**: Non-blocking file operations (with feature flag)
///
/// ## Examples
///
/// ```rust
/// use config_lib::Config;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Load from string
/// let mut config = Config::from_string("port = 8080\nname = \"MyApp\"", None)?;
///
/// // Access values
/// let port = config.get("port").unwrap().as_integer()?;
/// let name = config.get("name").unwrap().as_string()?;
///
/// // Modify values
/// config.set("port", 9000)?;
///
/// # Ok(())
/// # }
/// ```
pub struct Config {
    /// The resolved configuration values
    values: Value,
    
    /// Path to the source file (if loaded from file)
    file_path: Option<PathBuf>,
    
    /// Detected or specified format
    format: String,
    
    /// Change tracking - has the config been modified?
    modified: bool,
    
    /// Format-specific preservation data
    #[cfg(feature = "noml")]
    noml_document: Option<noml::Document>,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            values: Value::table(BTreeMap::new()),
            file_path: None,
            format: "conf".to_string(),
            modified: false,
            #[cfg(feature = "noml")]
            noml_document: None,
        }
    }

    /// Load configuration from a string
    pub fn from_string(source: &str, format: Option<&str>) -> Result<Self> {
        let detected_format = format.unwrap_or_else(|| {
            parsers::detect_format(source)
        });

        let values = parsers::parse_string(source, Some(detected_format))?;
        
        let config = Self {
            values,
            file_path: None,
            format: detected_format.to_string(),
            modified: false,
            #[cfg(feature = "noml")]
            noml_document: None,
        };

        // Store format-specific preservation data
        #[cfg(feature = "noml")]
        if detected_format == "noml" || detected_format == "toml" {
            if let Ok(document) = noml::parse_string(source, None) {
                config.noml_document = Some(document);
            }
        }

        Ok(config)
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::io(path.display().to_string(), e))?;

        let format = parsers::detect_format_from_path(path)
            .unwrap_or_else(|| parsers::detect_format(&content));

        let mut config = Self::from_string(&content, Some(format))?;
        config.file_path = Some(path.to_path_buf());
        
        Ok(config)
    }

    /// Async version of from_file
    #[cfg(feature = "async")]
    pub async fn from_file_async<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| Error::io(path.display().to_string(), e))?;

        let format = parsers::detect_format_from_path(path)
            .unwrap_or_else(|| parsers::detect_format(&content));

        let mut config = Self::from_string(&content, Some(format))?;
        config.file_path = Some(path.to_path_buf());
        
        Ok(config)
    }

    /// Get a value by path
    pub fn get(&self, path: &str) -> Option<&Value> {
        self.values.get(path)
    }

    /// Get a mutable reference to a value by path
    pub fn get_mut(&mut self, path: &str) -> Result<&mut Value> {
        self.values.get_mut_nested(path)
    }

    /// Set a value by path
    pub fn set<V: Into<Value>>(&mut self, path: &str, value: V) -> Result<()> {
        self.values.set_nested(path, value.into())?;
        self.modified = true;
        Ok(())
    }

    /// Remove a value by path  
    pub fn remove(&mut self, path: &str) -> Result<Option<Value>> {
        let result = self.values.remove(path)?;
        if result.is_some() {
            self.modified = true;
        }
        Ok(result)
    }

    /// Check if a path exists
    pub fn contains_key(&self, path: &str) -> bool {
        self.values.contains_key(path)
    }

    /// Get all keys in the configuration
    pub fn keys(&self) -> Result<Vec<&str>> {
        self.values.keys()
    }

    /// Check if the configuration has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark the configuration as unmodified
    pub fn mark_clean(&mut self) {
        self.modified = false;
    }

    /// Get the configuration format
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Get the file path (if loaded from file)
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// Save the configuration to its original file
    pub fn save(&mut self) -> Result<()> {
        match &self.file_path {
            Some(path) => {
                self.save_to_file(path.clone())?;
                self.modified = false;
                Ok(())
            },
            None => Err(Error::internal(
                "Cannot save configuration that wasn't loaded from a file"
            )),
        }
    }

    /// Save the configuration to a specific file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let serialized = self.serialize()?;
        std::fs::write(path, serialized)
            .map_err(|e| Error::io("save".to_string(), e))?;
        Ok(())
    }

    /// Async version of save
    #[cfg(feature = "async")]
    pub async fn save_async(&mut self) -> Result<()> {
        match &self.file_path {
            Some(path) => {
                self.save_to_file_async(path.clone()).await?;
                self.modified = false;
                Ok(())
            },
            None => Err(Error::internal(
                "Cannot save configuration that wasn't loaded from a file"
            )),
        }
    }

    /// Async version of save_to_file
    #[cfg(feature = "async")]
    pub async fn save_to_file_async<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let serialized = self.serialize()?;
        tokio::fs::write(path, serialized)
            .await
            .map_err(|e| Error::io("save".to_string(), e))?;
        Ok(())
    }

    /// Serialize the configuration to string format
    pub fn serialize(&self) -> Result<String> {
        match self.format.as_str() {
            "json" => {
                #[cfg(feature = "json")]
                return crate::parsers::json_parser::serialize(&self.values);
                #[cfg(not(feature = "json"))]
                return Err(Error::feature_not_enabled("json"));
            }
            "toml" => {
                #[cfg(feature = "toml")]
                {
                    // Use NOML's serializer for format preservation
                    if let Some(ref document) = self.noml_document {
                        return Ok(noml::serialize_document(document));
                    } else {
                        // Fallback to basic serialization
                        return self.serialize_as_toml();
                    }
                }
                #[cfg(not(feature = "toml"))]
                return Err(Error::feature_not_enabled("toml"));
            }
            "noml" => {
                #[cfg(feature = "noml")]
                {
                    if let Some(ref document) = self.noml_document {
                        return Ok(noml::serialize_document(document));
                    } else {
                        return Err(Error::internal("NOML document not preserved"));
                    }
                }
                #[cfg(not(feature = "noml"))]
                return Err(Error::feature_not_enabled("noml"));
            }
            "conf" => self.serialize_as_conf(),
            _ => Err(Error::unknown_format(&self.format)),
        }
    }

    /// Serialize as CONF format
    fn serialize_as_conf(&self) -> Result<String> {
        let mut output = String::new();
        if let Value::Table(table) = &self.values {
            self.write_conf_table(&mut output, table, "")?;
        }
        Ok(output)
    }

    /// Helper to write CONF format table
    fn write_conf_table(
        &self,
        output: &mut String,
        table: &BTreeMap<String, Value>,
        section_prefix: &str,
    ) -> Result<()> {
        // First pass: write simple key-value pairs
        for (key, value) in table {
            if !value.is_table() {
                let formatted_value = self.format_conf_value(value)?;
                output.push_str(&format!("{} = {}\n", key, formatted_value));
            }
        }

        // Second pass: write sections
        for (key, value) in table {
            if let Value::Table(nested_table) = value {
                let section_name = if section_prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", section_prefix, key)
                };
                
                output.push_str(&format!("\n[{}]\n", section_name));
                self.write_conf_table(output, nested_table, &section_name)?;
            }
        }
        
        Ok(())
    }

    /// Format a value for CONF output
    fn format_conf_value(&self, value: &Value) -> Result<String> {
        match value {
            Value::Null => Ok("null".to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::String(s) => {
                if s.contains(' ') || s.contains('\t') || s.contains('\n') {
                    Ok(format!("\"{}\"", s.replace('"', "\\\"")))
                } else {
                    Ok(s.clone())
                }
            }
            Value::Array(arr) => {
                let items: Result<Vec<String>> = arr
                    .iter()
                    .map(|v| self.format_conf_value(v))
                    .collect();
                Ok(items?.join(" "))
            }
            Value::Table(_) => Err(Error::type_error(
                "Cannot serialize nested table as value",
                "primitive",
                "table",
            )),
            #[cfg(feature = "chrono")]
            Value::DateTime(dt) => Ok(dt.to_rfc3339()),
        }
    }

    /// Serialize as TOML format (basic implementation)
    #[cfg(feature = "toml")]
    fn serialize_as_toml(&self) -> Result<String> {
        // This is a simplified TOML serializer
        // In practice, you'd use the NOML library for proper TOML serialization
        Err(Error::internal(
            "Basic TOML serialization not implemented - use NOML library"
        ))
    }

    /// Validate the configuration against a schema
    #[cfg(feature = "schema")]
    pub fn validate_schema(&self, schema: &Schema) -> Result<()> {
        schema.validate(&self.values)
    }

    /// Get the underlying Value
    pub fn as_value(&self) -> &Value {
        &self.values
    }

    /// Merge another configuration into this one
    pub fn merge(&mut self, other: &Config) -> Result<()> {
        self.merge_value(&other.values)?;
        self.modified = true;
        Ok(())
    }

    /// Helper to merge values recursively
    fn merge_value(&mut self, other: &Value) -> Result<()> {
        match (&mut self.values, other) {
            (Value::Table(self_table), Value::Table(other_table)) => {
                for (key, other_value) in other_table {
                    match self_table.get_mut(key) {
                        Some(self_value) => {
                            if let (Value::Table(_), Value::Table(_)) = (&*self_value, other_value) {
                                // Create a temporary config for recursive merging
                                let mut temp_config = Config::new();
                                temp_config.values = self_value.clone();
                                temp_config.merge_value(other_value)?;
                                *self_value = temp_config.values;
                            } else {
                                // Replace value
                                *self_value = other_value.clone();
                            }
                        }
                        None => {
                            // Insert new value
                            self_table.insert(key.clone(), other_value.clone());
                        }
                    }
                }
            }
            _ => {
                // Replace entire value
                self.values = other.clone();
            }
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Value to Config
impl From<Value> for Config {
    fn from(value: Value) -> Self {
        Self {
            values: value,
            file_path: None,
            format: "conf".to_string(),
            modified: false,
            #[cfg(feature = "noml")]
            noml_document: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(!config.is_modified());
        assert_eq!(config.format(), "conf");
    }

    #[test]
    fn test_config_from_string() {
        let config = Config::from_string(
            "key = value\nport = 8080",
            Some("conf")
        ).unwrap();
        
        assert_eq!(config.get("key").unwrap().as_string().unwrap(), "value");
        assert_eq!(config.get("port").unwrap().as_integer().unwrap(), 8080);
    }

    #[test]
    fn test_config_modification() {
        let mut config = Config::new();
        assert!(!config.is_modified());
        
        config.set("key", "value").unwrap();
        assert!(config.is_modified());
        
        config.mark_clean();
        assert!(!config.is_modified());
    }

    #[test]
    fn test_config_merge() {
        let mut config1 = Config::new();
        config1.set("a", 1).unwrap();
        config1.set("b.x", 2).unwrap();
        
        let mut config2 = Config::new();
        config2.set("b.y", 3).unwrap();
        config2.set("c", 4).unwrap();
        
        config1.merge(&config2).unwrap();
        
        assert_eq!(config1.get("a").unwrap().as_integer().unwrap(), 1);
        assert_eq!(config1.get("b.x").unwrap().as_integer().unwrap(), 2);
        assert_eq!(config1.get("b.y").unwrap().as_integer().unwrap(), 3);
        assert_eq!(config1.get("c").unwrap().as_integer().unwrap(), 4);
    }
}