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

#[cfg(feature = "validation")]
use crate::validation::{ValidationError, ValidationRuleSet};

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

    /// Validation rules for this configuration
    #[cfg(feature = "validation")]
    validation_rules: Option<ValidationRuleSet>,
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
            #[cfg(feature = "validation")]
            validation_rules: None,
        }
    }

    /// Load configuration from a string
    pub fn from_string(source: &str, format: Option<&str>) -> Result<Self> {
        let detected_format = format.unwrap_or_else(|| parsers::detect_format(source));

        let values = parsers::parse_string(source, Some(detected_format))?;

        let mut config = Self {
            values,
            file_path: None,
            format: detected_format.to_string(),
            modified: false,
            #[cfg(feature = "noml")]
            noml_document: None,
            #[cfg(feature = "validation")]
            validation_rules: None,
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
        let content =
            std::fs::read_to_string(path).map_err(|e| Error::io(path.display().to_string(), e))?;

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
            }
            None => Err(Error::internal(
                "Cannot save configuration that wasn't loaded from a file",
            )),
        }
    }

    /// Save the configuration to a specific file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let serialized = self.serialize()?;
        std::fs::write(path, serialized).map_err(|e| Error::io("save".to_string(), e))?;
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
            }
            None => Err(Error::internal(
                "Cannot save configuration that wasn't loaded from a file",
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
                    #[cfg(feature = "noml")]
                    if let Some(ref document) = self.noml_document {
                        return Ok(noml::serialize_document(document)?);
                    }
                    // Fallback to basic serialization
                    self.serialize_as_toml()
                }
                #[cfg(not(feature = "toml"))]
                return Err(Error::feature_not_enabled("toml"));
            }
            "noml" => {
                #[cfg(feature = "noml")]
                {
                    if let Some(ref document) = self.noml_document {
                        Ok(noml::serialize_document(document)?)
                    } else {
                        Err(Error::internal("NOML document not preserved"))
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
                output.push_str(&format!("{key} = {formatted_value}\n"));
            }
        }

        // Second pass: write sections
        for (key, value) in table {
            if let Value::Table(nested_table) = value {
                let section_name = if section_prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{section_prefix}.{key}")
                };

                output.push_str(&format!("\n[{section_name}]\n"));
                self.write_conf_table(output, nested_table, &section_name)?;
            }
        }

        Ok(())
    }

    /// Format a value for CONF output
    #[allow(clippy::only_used_in_recursion)]
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
                let items: Result<Vec<String>> =
                    arr.iter().map(|v| self.format_conf_value(v)).collect();
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
            "Basic TOML serialization not implemented - use NOML library",
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
                            if let (Value::Table(_), Value::Table(_)) = (&*self_value, other_value)
                            {
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

    // =====================================================================
    // Validation Methods (Feature-gated)
    // =====================================================================

    // --- CONVENIENCE METHODS & BUILDER PATTERN ---

    /// Get a value by path with a more ergonomic API
    pub fn key(&self, path: &str) -> ConfigValue {
        ConfigValue::new(self.get(path))
    }

    /// Check if configuration has any value at the given path
    pub fn has(&self, path: &str) -> bool {
        self.contains_key(path)
    }

    /// Get a value with a default fallback
    pub fn get_or<V>(&self, path: &str, default: V) -> V
    where
        V: TryFrom<Value> + Clone,
        V::Error: std::fmt::Debug,
    {
        self.get(path)
            .and_then(|v| V::try_from(v.clone()).ok())
            .unwrap_or(default)
    }

    // --- VALIDATION SUPPORT ---

    /// Set validation rules for this configuration
    #[cfg(feature = "validation")]
    pub fn set_validation_rules(&mut self, rules: ValidationRuleSet) {
        self.validation_rules = Some(rules);
    }

    /// Validate the current configuration against all registered rules
    #[cfg(feature = "validation")]
    pub fn validate(&mut self) -> Result<Vec<ValidationError>> {
        match &mut self.validation_rules {
            Some(rules) => {
                if let Value::Table(table) = &self.values {
                    let mut errors = Vec::new();

                    // Validate each key-value pair
                    for (key, value) in table {
                        errors.extend(rules.validate(key, value));
                    }

                    // Also validate for required keys (if any RequiredKeyValidator exists)
                    // This is handled by individual rule implementations

                    Ok(errors)
                } else {
                    Err(Error::validation(
                        "Configuration root must be a table for validation",
                    ))
                }
            }
            None => Ok(Vec::new()), // No rules = no errors
        }
    }

    /// Validate and return only critical errors
    #[cfg(feature = "validation")]
    pub fn validate_critical_only(&mut self) -> Result<Vec<ValidationError>> {
        let all_errors = self.validate()?;
        Ok(all_errors
            .into_iter()
            .filter(|e| e.severity == crate::validation::ValidationSeverity::Critical)
            .collect())
    }

    /// Check if configuration is valid (has no critical errors)
    #[cfg(feature = "validation")]
    pub fn is_valid(&mut self) -> Result<bool> {
        let critical_errors = self.validate_critical_only()?;
        Ok(critical_errors.is_empty())
    }

    /// Validate a specific value at a path
    #[cfg(feature = "validation")]
    pub fn validate_path(&mut self, path: &str) -> Result<Vec<ValidationError>> {
        // Get the value first to avoid borrowing conflicts, clone to own it
        let value = self
            .get(path)
            .ok_or_else(|| Error::key_not_found(path))?
            .clone();

        match &mut self.validation_rules {
            Some(rules) => Ok(rules.validate(path, &value)),
            None => Ok(Vec::new()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Ergonomic wrapper for accessing configuration values
pub struct ConfigValue<'a> {
    value: Option<&'a Value>,
}

impl<'a> ConfigValue<'a> {
    fn new(value: Option<&'a Value>) -> Self {
        Self { value }
    }

    /// Get as string with default fallback
    pub fn as_string(&self) -> Result<String> {
        match self.value {
            Some(v) => v.as_string().map(|s| s.to_string()),
            None => Err(Error::key_not_found("value not found")),
        }
    }

    /// Get as string with custom default
    pub fn as_string_or(&self, default: &str) -> String {
        self.value
            .and_then(|v| v.as_string().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }

    /// Get as integer with default fallback
    pub fn as_integer(&self) -> Result<i64> {
        match self.value {
            Some(v) => v.as_integer(),
            None => Err(Error::key_not_found("value not found")),
        }
    }

    /// Get as integer with custom default
    pub fn as_integer_or(&self, default: i64) -> i64 {
        self.value
            .and_then(|v| v.as_integer().ok())
            .unwrap_or(default)
    }

    /// Get as boolean with default fallback
    pub fn as_bool(&self) -> Result<bool> {
        match self.value {
            Some(v) => v.as_bool(),
            None => Err(Error::key_not_found("value not found")),
        }
    }

    /// Get as boolean with custom default
    pub fn as_bool_or(&self, default: bool) -> bool {
        self.value.and_then(|v| v.as_bool().ok()).unwrap_or(default)
    }

    /// Check if the value exists
    pub fn exists(&self) -> bool {
        self.value.is_some()
    }

    /// Get the underlying Value reference if it exists
    pub fn value(&self) -> Option<&'a Value> {
        self.value
    }
}

/// Builder pattern for Config creation
pub struct ConfigBuilder {
    format: Option<String>,
    #[cfg(feature = "validation")]
    validation_rules: Option<ValidationRuleSet>,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder
    pub fn new() -> Self {
        Self {
            format: None,
            #[cfg(feature = "validation")]
            validation_rules: None,
        }
    }

    /// Set the configuration format
    pub fn format<S: Into<String>>(mut self, format: S) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set validation rules
    #[cfg(feature = "validation")]
    pub fn validation_rules(mut self, rules: ValidationRuleSet) -> Self {
        self.validation_rules = Some(rules);
        self
    }

    /// Build Config from string
    pub fn from_string(self, source: &str) -> Result<Config> {
        #[cfg(feature = "validation")]
        let mut config = Config::from_string(source, self.format.as_deref())?;
        #[cfg(not(feature = "validation"))]
        let config = Config::from_string(source, self.format.as_deref())?;

        #[cfg(feature = "validation")]
        if let Some(rules) = self.validation_rules {
            config.set_validation_rules(rules);
        }

        Ok(config)
    }

    /// Build Config from file
    pub fn from_file<P: AsRef<Path>>(self, path: P) -> Result<Config> {
        #[cfg(feature = "validation")]
        let mut config = Config::from_file(path)?;
        #[cfg(not(feature = "validation"))]
        let config = Config::from_file(path)?;

        #[cfg(feature = "validation")]
        if let Some(rules) = self.validation_rules {
            config.set_validation_rules(rules);
        }

        Ok(config)
    }
}

impl Default for ConfigBuilder {
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
            #[cfg(feature = "validation")]
            validation_rules: None,
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
        let config = Config::from_string("key = value\nport = 8080", Some("conf")).unwrap();

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
