//! # High-Level Configuration Management
//!
//! Advanced configuration management API providing intuitive interfaces for
//! loading, modifying, validating, and saving configurations with format preservation.

use crate::error::{Error, Result};
use crate::parsers;
use crate::value::Value;
use dashmap::DashMap;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

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
#[derive(Debug)]
pub struct Config {
    /// The resolved configuration values
    values: Value,

    /// Path to the source file (if loaded from file)
    file_path: Option<PathBuf>,

    /// Detected or specified format
    format: String,

    /// Change tracking - has the config been modified?
    modified: bool,

    /// Opt-out behavior knobs (read-only, cache sizing, etc.).
    /// See [`ConfigOptions`].
    options: ConfigOptions,

    /// Cache hit counter (loaded via `Ordering::Relaxed` from
    /// [`Config::cache_stats`]). Wired in v0.9.9 — populated by
    /// [`Config::get_arc`] when the cache layer is enabled.
    cache_hits: AtomicU64,

    /// Cache miss counter. See [`Config::cache_stats`].
    cache_misses: AtomicU64,

    /// Resolved-path cache. `DashMap` is sharded so reads hold a
    /// short shard-local lock rather than a global lock; under
    /// typical config workloads (read-mostly, infrequent writes)
    /// this provides effectively-lock-free reads. Storing
    /// `Arc<Value>` keeps clones cheap on hit (refcount bump only).
    ///
    /// Population is lazy — only paths actually requested through
    /// [`Config::get_arc`] enter the cache. `set` / `remove` /
    /// `merge` clear the cache entirely (writes are rare in the
    /// design envelope; a per-prefix invalidation strategy is
    /// post-1.0 backlog).
    cache: DashMap<Box<str>, Arc<Value>>,

    /// Per-path defaults table consulted by [`Config::get_or_default`]
    /// when the requested path is missing from `values`. Wrapped in
    /// `Arc<RwLock<_>>` so defaults can be updated independently of
    /// the main value tree, including under `read_only` mode (defaults
    /// are operator-provided fallbacks, not user-supplied data).
    defaults: Arc<RwLock<BTreeMap<String, Value>>>,

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
            options: ConfigOptions::default(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache: DashMap::new(),
            defaults: Arc::new(RwLock::new(BTreeMap::new())),
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

        #[cfg(feature = "noml")]
        let mut config = Self {
            values,
            file_path: None,
            format: detected_format.to_string(),
            modified: false,
            options: ConfigOptions::default(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache: DashMap::new(),
            defaults: Arc::new(RwLock::new(BTreeMap::new())),
            noml_document: None,
            #[cfg(feature = "validation")]
            validation_rules: None,
        };

        #[cfg(not(feature = "noml"))]
        let config = Self {
            values,
            file_path: None,
            format: detected_format.to_string(),
            modified: false,
            options: ConfigOptions::default(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache: DashMap::new(),
            defaults: Arc::new(RwLock::new(BTreeMap::new())),
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
    ///
    /// Invalidates the entire resolved-path cache (writes are rare in
    /// the design envelope; a per-prefix invalidation strategy is
    /// post-1.0 backlog).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration was constructed with [`ConfigOptions::read_only`]
    /// - The path is invalid (e.g. attempts to insert into a non-table value)
    pub fn set<V: Into<Value>>(&mut self, path: &str, value: V) -> Result<()> {
        self.ensure_writable()?;
        self.values.set_nested(path, value.into())?;
        self.modified = true;
        self.cache.clear();
        Ok(())
    }

    /// Remove a value by path
    ///
    /// Invalidates the entire resolved-path cache on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration was constructed with [`ConfigOptions::read_only`]
    /// - The path is malformed
    pub fn remove(&mut self, path: &str) -> Result<Option<Value>> {
        self.ensure_writable()?;
        let result = self.values.remove(path)?;
        if result.is_some() {
            self.modified = true;
            self.cache.clear();
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
    ///
    /// Invalidates the entire resolved-path cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration was constructed with
    /// [`ConfigOptions::read_only`].
    pub fn merge(&mut self, other: &Config) -> Result<()> {
        self.ensure_writable()?;
        self.merge_value(&other.values)?;
        self.modified = true;
        self.cache.clear();
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
    pub fn key(&self, path: &str) -> ConfigValue<'_> {
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

// =========================================================================
// ConfigOptions — opt-out behavior knobs (foundation for v0.9.5)
// =========================================================================

/// Opt-out behavior knobs for [`Config`].
///
/// `ConfigOptions` carries the small set of toggles that should not be
/// enabled by default: making a `Config` read-only, sizing the cache,
/// and so on. The struct is `#[non_exhaustive]` so v0.9.x can add new
/// knobs without breaking SemVer. Users construct it via
/// [`ConfigOptions::new`] or [`ConfigOptions::default`] and apply
/// individual toggles through the consuming builder methods.
///
/// The field set lays the groundwork for the lock-free caching work
/// landing in v0.9.5. In v0.9.4 the only knob that has runtime effect
/// is [`ConfigOptions::read_only`]; the cache-related knobs are
/// accepted today so that the public API surface does not change
/// again when v0.9.5 switches the cache on.
///
/// # Examples
///
/// ```rust
/// use config_lib::{Config, ConfigOptions};
///
/// // Default options — caching on, writes allowed.
/// let _cfg = Config::with_options(ConfigOptions::default());
///
/// // Read-only configuration for a hot path that must never be mutated.
/// let opts = ConfigOptions::new().read_only(true);
/// let _cfg = Config::with_options(opts);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConfigOptions {
    /// Reject every `set` / `remove` / `merge` call with
    /// [`Error::general`] instead of mutating. Useful for once-loaded
    /// configurations that must never change at runtime.
    pub read_only: bool,

    /// Whether the internal caching layer is active. **Reserved** — the
    /// caching layer ships in v0.9.5; in v0.9.4 this field is accepted
    /// for forward compatibility but does not yet change runtime
    /// behavior (every `get` already hits an in-memory `Value`).
    pub cache_enabled: bool,

    /// Maximum number of resolved-key entries the cache will hold
    /// before evicting. **Reserved for v0.9.5.**
    pub cache_capacity: usize,
}

impl Default for ConfigOptions {
    /// The canonical options for the Hive DB use case:
    /// caching enabled, writes allowed, capacity tuned for typical
    /// server config size.
    fn default() -> Self {
        Self {
            read_only: false,
            cache_enabled: true,
            cache_capacity: 1024,
        }
    }
}

impl ConfigOptions {
    /// Construct a `ConfigOptions` with default values
    /// ([`ConfigOptions::default`]).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the read-only flag. See [`ConfigOptions::read_only`].
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Toggle the caching layer. **Reserved for v0.9.5** — currently
    /// a no-op at runtime; the setter exists so call-sites compile
    /// against the same shape they will use after the cache lands.
    pub fn cache_enabled(mut self, cache_enabled: bool) -> Self {
        self.cache_enabled = cache_enabled;
        self
    }

    /// Set the cache capacity. **Reserved for v0.9.5** — see
    /// [`ConfigOptions::cache_enabled`].
    pub fn cache_capacity(mut self, cache_capacity: usize) -> Self {
        self.cache_capacity = cache_capacity;
        self
    }
}

impl Config {
    /// Construct a new empty [`Config`] with the supplied
    /// [`ConfigOptions`].
    ///
    /// This is the explicit opt-out constructor. For the canonical
    /// defaults (caching on, writes allowed), prefer [`Config::new`].
    pub fn with_options(options: ConfigOptions) -> Self {
        let mut config = Self::new();
        config.options = options;
        config
    }

    /// Return the [`ConfigOptions`] currently in effect on this config.
    pub fn options(&self) -> &ConfigOptions {
        &self.options
    }

    /// Returns `true` if this configuration was constructed read-only
    /// (see [`ConfigOptions::read_only`]).
    pub fn is_read_only(&self) -> bool {
        self.options.read_only
    }

    /// Helper used by mutating methods to short-circuit when the
    /// configuration is in read-only mode.
    fn ensure_writable(&self) -> Result<()> {
        if self.options.read_only {
            Err(Error::general("Configuration is read-only"))
        } else {
            Ok(())
        }
    }

    /// Return a snapshot of the cache-hit / cache-miss counters.
    ///
    /// Counters are loaded with `Ordering::Relaxed` — the values are
    /// statistics, not synchronisation primitives, and the hit/miss
    /// classification is best-effort under concurrent reads.
    pub fn cache_stats(&self) -> CacheStats {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits.saturating_add(misses);
        let hit_ratio = if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        };
        CacheStats {
            hits,
            misses,
            hit_ratio,
        }
    }

    /// Resolve a dotted path to a cached `Arc<Value>`.
    ///
    /// `get_arc` is the cache-backed thread-safe accessor introduced
    /// in v0.9.9. The first lookup of a given path walks the value
    /// tree, allocates an `Arc<Value>` containing a clone of the
    /// resolved node, and inserts it into the resolved-path cache.
    /// Subsequent lookups of the same path hit the cache and return
    /// a cheap refcount-bump clone of the `Arc<Value>`.
    ///
    /// This is the recommended accessor for:
    ///
    /// - **Multi-threaded reads.** `Arc<Value>` is `Send + Sync` and
    ///   independent of `&self`'s lifetime, so the value can be
    ///   handed off to other threads.
    /// - **Hot loops.** Cache hits avoid the tree walk; the
    ///   refcount bump is a single relaxed atomic.
    ///
    /// The existing `Config::get(&self, path) -> Option<&Value>` is
    /// still the right choice for single-threaded reads that just
    /// peek and drop the borrow — no clone, no Arc bump.
    ///
    /// The cache is invalidated wholesale on any `set` / `remove` /
    /// `merge`. The runtime knob [`ConfigOptions::cache_enabled`]
    /// (default `true`) toggles the cache layer; with it disabled,
    /// every `get_arc` call walks the tree and allocates a fresh
    /// `Arc<Value>`.
    pub fn get_arc(&self, path: &str) -> Option<Arc<Value>> {
        if self.options.cache_enabled {
            if let Some(entry) = self.cache.get(path) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Some(Arc::clone(entry.value()));
            }
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
            let resolved = self.values.get(path)?.clone();
            let arc = Arc::new(resolved);
            self.cache.insert(path.into(), Arc::clone(&arc));
            Some(arc)
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
            Some(Arc::new(self.values.get(path)?.clone()))
        }
    }

    /// Clear the resolved-path cache.
    ///
    /// Idempotent; safe to call from any thread. Useful when an
    /// out-of-band actor (e.g. a hot-reload watcher) has changed
    /// the underlying value tree and the cache snapshot is now stale.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Set a default value for the given path.
    ///
    /// Defaults are an operator-supplied fallback table consulted by
    /// [`Config::get_or_default`] when the requested path is missing
    /// from the main value tree. Defaults are independent of the
    /// `read_only` flag — a read-only configuration can still have
    /// its defaults table populated (defaults are deployment-time
    /// declarations, not user-supplied data).
    ///
    /// Calls to `set_default` do **not** invalidate the cache; the
    /// cache is keyed on observed value-tree resolutions, not on
    /// defaults-table membership.
    ///
    /// # Errors
    ///
    /// Returns an error if the defaults-table lock has been poisoned.
    pub fn set_default<V: Into<Value>>(&self, path: &str, value: V) -> Result<()> {
        let mut defaults = self
            .defaults
            .write()
            .map_err(|_| Error::concurrency("defaults table lock poisoned"))?;
        defaults.insert(path.to_string(), value.into());
        Ok(())
    }

    /// Resolve a path against the main value tree, falling back to the
    /// defaults table when not found.
    ///
    /// Returns `None` only when neither the value tree nor the defaults
    /// table contains an entry for `path`. The return type is owned
    /// (`Option<Value>`) because the defaults table sits behind an
    /// `Arc<RwLock<_>>` and producing a borrowed reference would
    /// require holding the lock guard across the caller's use.
    pub fn get_or_default(&self, path: &str) -> Option<Value> {
        if let Some(v) = self.values.get(path) {
            return Some(v.clone());
        }
        let defaults = self.defaults.read().ok()?;
        defaults.get(path).cloned()
    }

    /// Toggle this configuration into read-only mode at runtime.
    ///
    /// Equivalent to constructing with
    /// `ConfigOptions::new().read_only(true)` but available as a
    /// post-construction switch. Subsequent calls to `set` / `remove`
    /// / `merge` return `Err(Error::general("Configuration is
    /// read-only"))`.
    ///
    /// The toggle is **one-way** by design — there is no
    /// `make_writable()` companion. A configuration that has been
    /// declared immutable should not become mutable again; if you
    /// need that flexibility, keep two `Config` handles instead.
    pub fn make_read_only(&mut self) {
        self.options.read_only = true;
    }
}

// =========================================================================
// CacheStats — read-only snapshot of cache performance counters
// =========================================================================

/// Snapshot of a [`Config`]'s cache-hit / cache-miss counters.
///
/// Returned by [`Config::cache_stats`]. The struct is `#[non_exhaustive]`
/// so the v1.x SemVer contract can add new counter fields (e.g.
/// `evictions`, `insertions`, per-shard breakdowns) in MINOR releases
/// without breaking user code.
///
/// # Stability note for v0.9.5
///
/// In v0.9.5 every `CacheStats` returned by [`Config::cache_stats`]
/// has `hits = 0`, `misses = 0`, `hit_ratio = 0.0`. The lock-free
/// cache layer that populates these counters lands in a follow-up
/// v0.9.5 implementation release. The struct is shipping now so the
/// API surface is locked in ahead of the implementation.
///
/// # Examples
///
/// ```rust
/// use config_lib::Config;
///
/// let cfg = Config::new();
/// let stats = cfg.cache_stats();
/// assert_eq!(stats.hits, 0);
/// assert_eq!(stats.misses, 0);
/// assert_eq!(stats.hit_ratio, 0.0);
/// ```
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct CacheStats {
    /// Number of cache lookups that resolved to a cached value.
    pub hits: u64,
    /// Number of cache lookups that did not find a cached value and
    /// fell through to the canonical storage.
    pub misses: u64,
    /// `hits / (hits + misses)` as an f64 in `[0.0, 1.0]`. Returns
    /// `0.0` when no lookups have happened yet.
    pub hit_ratio: f64,
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
            options: ConfigOptions::default(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache: DashMap::new(),
            defaults: Arc::new(RwLock::new(BTreeMap::new())),
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
