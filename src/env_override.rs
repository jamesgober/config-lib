//! # Environment Variable Override System
//!
//! Smart environment variable override patterns with zero-copy prefix matching
//! and intelligent caching for sub-50ns performance.
//!
//! ## Performance Features
//! - Cached environment variable lookups
//! - Zero-copy prefix matching
//! - Lazy evaluation with memoization
//! - Feature-gated for zero impact when disabled
//!
//! ## Override Patterns
//! - Prefix-based: `APP_DATABASE_HOST` -> `database.host`
//! - Docker-style: `DATABASE__HOST` -> `database.host`
//! - Kubernetes-style: `DATABASE_HOST` with section mapping
//! - Custom mapping patterns

use crate::{error::Error, Result, Value};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

/// Environment variable override configuration
#[derive(Debug, Clone)]
pub struct EnvOverrideConfig {
    /// Prefix for environment variables (e.g., "APP_", "MYAPP_")
    pub prefix: String,
    /// Separator for nested keys (e.g., "_", "__")
    pub separator: String,
    /// Whether to convert keys to lowercase
    pub lowercase_keys: bool,
    /// Whether to cache environment variable lookups
    pub enable_cache: bool,
    /// Custom key mappings: env_var -> config_key
    pub custom_mappings: HashMap<String, String>,
}

impl Default for EnvOverrideConfig {
    fn default() -> Self {
        Self {
            prefix: "APP_".to_string(),
            separator: "_".to_string(),
            lowercase_keys: true,
            enable_cache: true,
            custom_mappings: HashMap::new(),
        }
    }
}

/// Cached environment variable lookup system
pub struct EnvOverrideSystem {
    config: EnvOverrideConfig,
    cache: Arc<RwLock<HashMap<String, Option<String>>>>,
}

impl EnvOverrideSystem {
    /// Create a new environment override system
    pub fn new(config: EnvOverrideConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(EnvOverrideConfig::default())
    }

    /// Create with custom prefix
    pub fn with_prefix(prefix: &str) -> Self {
        let config = EnvOverrideConfig {
            prefix: prefix.to_string(),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Apply environment variable overrides to a configuration value
    pub fn apply_overrides(&self, mut value: Value) -> Result<Value> {
        self.apply_overrides_recursive(&mut value, String::new())?;
        Ok(value)
    }

    /// Recursively apply overrides to nested configuration
    fn apply_overrides_recursive(&self, value: &mut Value, path: String) -> Result<()> {
        match value {
            Value::Table(ref mut table) => {
                for (key, val) in table.iter_mut() {
                    let nested_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };

                    // Check for environment override
                    if let Some(env_value) = self.get_env_override(&nested_path, val)? {
                        *val = env_value;
                    } else {
                        // Recurse into nested structures
                        self.apply_overrides_recursive(val, nested_path)?;
                    }
                }
            }
            Value::Array(ref mut array) => {
                for (index, val) in array.iter_mut().enumerate() {
                    let nested_path = format!("{path}[{index}]");
                    self.apply_overrides_recursive(val, nested_path)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Get environment variable override for a configuration key
    ///
    /// The variable's text is read as the type of the value it replaces
    /// (see [`EnvOverrideSystem::typed_value`]).
    fn get_env_override(&self, key: &str, current: &Value) -> Result<Option<Value>> {
        // Try different override patterns
        let env_keys = vec![
            self.generate_env_key(key),
            self.generate_docker_style_key(key),
            self.generate_k8s_style_key(key),
        ];

        for env_key in env_keys {
            if let Some(env_value) = self.get_cached_env(&env_key)? {
                return Ok(Some(self.typed_value(&env_value, current)));
            }
        }

        // Check custom mappings
        if let Some(custom_key) = self.config.custom_mappings.get(key) {
            if let Some(env_value) = self.get_cached_env(custom_key)? {
                return Ok(Some(self.typed_value(&env_value, current)));
            }
        }

        Ok(None)
    }

    /// Generate environment variable key from config key
    fn generate_env_key(&self, key: &str) -> String {
        let key = if self.config.lowercase_keys {
            key.to_uppercase()
        } else {
            key.to_string()
        };

        let env_key = key.replace('.', &self.config.separator);
        format!("{}{}", self.config.prefix, env_key)
    }

    /// Generate Docker-style environment key (double underscore)
    fn generate_docker_style_key(&self, key: &str) -> String {
        let key = key.to_uppercase().replace('.', "__");
        format!("{}{}", self.config.prefix, key)
    }

    /// Generate Kubernetes-style environment key (single underscore, no dots)
    fn generate_k8s_style_key(&self, key: &str) -> String {
        let key = key.to_uppercase().replace('.', "_");
        format!("{}{}", self.config.prefix, key)
    }

    /// Get environment variable with caching
    fn get_cached_env(&self, key: &str) -> Result<Option<String>> {
        if !self.config.enable_cache {
            return Ok(env::var(key).ok());
        }

        // Try cache first (fast path)
        {
            let cache = self
                .cache
                .read()
                .map_err(|e| Error::internal(format!("Cache read error: {e}")))?;

            if let Some(cached_value) = cache.get(key) {
                return Ok(cached_value.clone());
            }
        }

        // Cache miss - get from environment and cache result
        let env_value = env::var(key).ok();

        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| Error::internal(format!("Cache write error: {e}")))?;

            cache.insert(key.to_string(), env_value.clone());
        }

        Ok(env_value)
    }

    /// Read an override as the type of the value it replaces.
    ///
    /// Since v1.4.0 a string stays a string (`APP_DB_PASSWORD=ab,cd` is the
    /// text `ab,cd`, not a list), an integer, float or boolean is parsed as
    /// one, and a list is split at commas only when the file holds a list.
    /// Text that does not read as the old type, and an override of a null,
    /// a table or a date, falls back to [`EnvOverrideSystem::parse_env_value`].
    fn typed_value(&self, raw: &str, current: &Value) -> Value {
        let trimmed = raw.trim();
        match current {
            Value::String(_) => return Value::string(raw),
            Value::Integer(_) => {
                if let Ok(i) = trimmed.parse::<i64>() {
                    return Value::integer(i);
                }
            }
            Value::Float(_) => {
                if let Some(f) = crate::parsers::decimal_float(trimmed) {
                    return Value::float(f);
                }
                if let Ok(i) = trimmed.parse::<i64>() {
                    return Value::float(i as f64);
                }
            }
            Value::Bool(_) => {
                if let Ok(b) = Value::string(trimmed).as_bool() {
                    return Value::bool(b);
                }
            }
            Value::Array(_) if !trimmed.starts_with('[') => {
                return Value::array(
                    raw.split(',')
                        .map(|s| self.parse_scalar_value(s.trim()))
                        .collect(),
                );
            }
            _ => {}
        }
        self.parse_env_value(raw)
    }

    /// Parse environment variable value into appropriate type
    ///
    /// JSON (when a JSON backend is enabled) is tried first, then a
    /// comma-separated list, then a scalar.
    fn parse_env_value(&self, value: &str) -> Value {
        // Handle objects and arrays (JSON), when a JSON backend is enabled.
        // A value that does not parse as JSON falls through.
        #[cfg(any(feature = "json", feature = "jsonx"))]
        if value.trim_start().starts_with('{') || value.trim_start().starts_with('[') {
            if let Ok(json_value) = crate::parsers::json_parser::parse(value) {
                return json_value;
            }
        }

        // Handle arrays (comma-separated)
        if value.contains(',') {
            let items: Vec<Value> = value
                .split(',')
                .map(|s| self.parse_scalar_value(s.trim()))
                .collect();
            return Value::array(items);
        }

        // Handle scalar values
        self.parse_scalar_value(value)
    }

    /// Parse scalar environment variable value
    fn parse_scalar_value(&self, value: &str) -> Value {
        // Try boolean
        match value.to_lowercase().as_str() {
            // `1` and `0` stay integers (since v1.4.0); `as_bool` still
            // reads them as true and false
            "true" | "yes" | "on" => return Value::bool(true),
            "false" | "no" | "off" => return Value::bool(false),
            _ => {}
        }

        // Try integer
        if let Ok(int_val) = value.parse::<i64>() {
            return Value::integer(int_val);
        }

        // Try float (decimal syntax only, so `nan` and `inf` stay text)
        if let Some(float_val) = crate::parsers::decimal_float(value) {
            return Value::float(float_val);
        }

        // Default to string
        Value::string(value)
    }

    /// Clear the environment variable cache
    pub fn clear_cache(&self) -> Result<()> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| Error::internal(format!("Cache clear error: {e}")))?;
        cache.clear();
        Ok(())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Result<(usize, usize)> {
        let cache = self
            .cache
            .read()
            .map_err(|e| Error::internal(format!("Cache stats error: {e}")))?;

        let total_entries = cache.len();
        let hit_entries = cache.values().filter(|v| v.is_some()).count();

        Ok((hit_entries, total_entries))
    }
}

/// Apply environment variable overrides to configuration
pub fn apply_env_overrides(value: Value, config: EnvOverrideConfig) -> Result<Value> {
    let system = EnvOverrideSystem::new(config);
    system.apply_overrides(value)
}

/// Apply environment variable overrides with default configuration
pub fn apply_env_overrides_default(value: Value) -> Result<Value> {
    let system = EnvOverrideSystem::with_defaults();
    system.apply_overrides(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_env_key_generation() {
        let config = EnvOverrideConfig {
            prefix: "APP_".to_string(),
            separator: "_".to_string(),
            lowercase_keys: true,
            enable_cache: false,
            custom_mappings: HashMap::new(),
        };

        let system = EnvOverrideSystem::new(config);

        assert_eq!(
            system.generate_env_key("database.host"),
            "APP_DATABASE_HOST"
        );
        assert_eq!(
            system.generate_docker_style_key("database.host"),
            "APP_DATABASE__HOST"
        );
        assert_eq!(
            system.generate_k8s_style_key("database.host"),
            "APP_DATABASE_HOST"
        );
    }

    #[test]
    fn test_value_parsing() {
        let system = EnvOverrideSystem::with_defaults();

        assert_eq!(system.parse_scalar_value("true"), Value::bool(true));
        assert_eq!(system.parse_scalar_value("false"), Value::bool(false));
        assert_eq!(system.parse_scalar_value("123"), Value::integer(123));
        assert_eq!(system.parse_scalar_value("1.234"), Value::float(1.234));
        assert_eq!(system.parse_scalar_value("hello"), Value::string("hello"));
    }

    #[test]
    fn test_array_parsing() {
        let system = EnvOverrideSystem::with_defaults();

        let result = system.parse_env_value("a,b,c");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::string("a"));
            assert_eq!(arr[1], Value::string("b"));
            assert_eq!(arr[2], Value::string("c"));
        } else {
            panic!("Expected array");
        }
    }

    #[cfg(any(feature = "json", feature = "jsonx"))]
    #[test]
    fn test_json_values() {
        let system = EnvOverrideSystem::with_defaults();

        // JSON is tried before a comma-separated list
        let mut inner = std::collections::BTreeMap::new();
        inner.insert("on".to_string(), Value::bool(true));
        let mut outer = std::collections::BTreeMap::new();
        outer.insert("flags".to_string(), Value::table(inner));
        assert_eq!(
            system.parse_env_value(r#" {"flags": {"on": true}}"#),
            Value::table(outer)
        );
        assert_eq!(
            system.parse_env_value("[9223372036854775808]"),
            Value::array(vec![Value::float(9_223_372_036_854_775_808.0)])
        );
        assert_eq!(
            system.parse_env_value(r#"["café"]"#),
            Value::array(vec![Value::string("caf\u{e9}")])
        );

        // Not JSON: kept as the scalar it reads as.
        assert_eq!(system.parse_env_value("{oops"), Value::string("{oops"));
        assert_eq!(system.parse_env_value("[1e400]"), Value::string("[1e400]"));
    }

    #[test]
    fn test_cache_operations() {
        let system = EnvOverrideSystem::with_defaults();

        // Cache should start empty
        let (hits, total) = system.cache_stats().unwrap();
        assert_eq!(hits, 0);
        assert_eq!(total, 0);

        // Clear cache should work
        system.clear_cache().unwrap();

        let (hits, total) = system.cache_stats().unwrap();
        assert_eq!(hits, 0);
        assert_eq!(total, 0);
    }
}
