//! # High-Level Configuration Management
//!
//! Advanced configuration management API providing intuitive interfaces for
//! loading, modifying, validating, and saving configurations with format preservation.

use crate::error::{Error, Result};
use crate::load::{self, LoadOptions};
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
            cache: new_cache(),
            defaults: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "noml")]
            noml_document: None,
            #[cfg(feature = "validation")]
            validation_rules: None,
        }
    }

    /// Load configuration from a string
    ///
    /// Applies the default [`LoadOptions`]. With no format the text is
    /// read as CONF unless it is clearly XML, JSON or INI (see
    /// [`parsers::detect_format`]); pass the format when you know it.
    pub fn from_string(source: &str, format: Option<&str>) -> Result<Self> {
        Self::from_string_with(source, format, &LoadOptions::default())
    }

    /// Load configuration from a string with the given [`LoadOptions`]
    ///
    /// NOML includes in the text resolve against the current directory and
    /// may only read files inside it (or the directories the options
    /// allow).
    ///
    /// Since v1.4.0.
    pub fn from_string_with(
        source: &str,
        format: Option<&str>,
        options: &LoadOptions,
    ) -> Result<Self> {
        let detected_format = format.unwrap_or_else(|| parsers::detect_format(source));
        let values = parsers::parse_string_with(source, Some(detected_format), None, options)?;
        Ok(Self::from_parsed(source, detected_format, values))
    }

    /// Load configuration from a string without reading anything outside it
    ///
    /// The same as [`Config::from_string`] for every format except NOML
    /// (feature `noml`): `include` and every function call (`env()`
    /// included) are refused with [`Error::Parse`] carrying the line and
    /// column of the first one, and the message does not quote the input.
    /// NOML native types (`@size("10MB")`) and `${...}` references to other
    /// values in the same document still work.
    ///
    /// Use it when the caller has checked the configuration file itself
    /// (owner, mode, size, file type) before handing its text over: an
    /// include would read a second file without those checks, and `env()`
    /// would copy an environment variable into the configuration.
    ///
    /// TOML, JSON, CONF and the other formats read only the text in both
    /// entry points (TOML refuses NOML's `include` and `env()` everywhere
    /// since v1.3.0). With no format hint the text is never read as NOML
    /// (since v1.4.0).
    ///
    /// Since v1.3.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_lib::Config;
    ///
    /// let config = Config::from_string_strict("port = 8080", Some("conf"))?;
    /// assert_eq!(config.get("port").map(|v| v.as_integer()).transpose()?, Some(8080));
    /// # Ok::<(), config_lib::Error>(())
    /// ```
    pub fn from_string_strict(source: &str, format: Option<&str>) -> Result<Self> {
        Self::from_string_with(source, format, &LoadOptions::strict_defaults())
    }

    /// Build a `Config` from values already parsed out of `source`
    fn from_parsed(source: &str, detected_format: &str, values: Value) -> Self {
        #[cfg(not(feature = "noml"))]
        let _ = source;

        #[cfg(feature = "noml")]
        let mut config = Self {
            values,
            file_path: None,
            format: detected_format.to_string(),
            modified: false,
            options: ConfigOptions::default(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache: new_cache(),
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
            cache: new_cache(),
            defaults: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "validation")]
            validation_rules: None,
        };

        // Store format-specific preservation data (parsed only, never
        // resolved, so nothing outside `source` is read here)
        #[cfg(feature = "noml")]
        if detected_format == "noml" || detected_format == "toml" {
            if let Ok(document) = noml::parse_string(source, None) {
                config.noml_document = Some(document);
            }
        }

        config
    }

    /// Load configuration from a file
    ///
    /// The format comes from the file extension (an unknown extension is
    /// CONF; a file with none is detected from its text). Applies the
    /// default [`LoadOptions`]: the file must be a regular file of at most
    /// 16 MiB, and NOML includes may only read files inside the file's own
    /// directory.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file_with(path, &LoadOptions::default())
    }

    /// Load configuration from a file with strict NOML
    ///
    /// The file counterpart of [`Config::from_string_strict`]: a NOML file
    /// may not use `include` or call `env()`. Other formats load as with
    /// [`Config::from_file`].
    ///
    /// Since v1.4.0.
    pub fn from_file_strict<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file_with(path, &LoadOptions::strict_defaults())
    }

    /// Load configuration from a file with the given [`LoadOptions`]
    ///
    /// Since v1.4.0.
    pub fn from_file_with<P: AsRef<Path>>(path: P, options: &LoadOptions) -> Result<Self> {
        let path = path.as_ref();
        let content = load::read_file(path, options.max_file_size)?;
        Self::from_file_content(path, &content, options)
    }

    fn from_file_content(path: &Path, content: &str, options: &LoadOptions) -> Result<Self> {
        let format = parsers::file_format(path, content);
        let values = parsers::parse_string_with(content, Some(format), Some(path), options)?;
        let mut config = Self::from_parsed(content, format, values);
        config.file_path = Some(path.to_path_buf());
        Ok(config)
    }

    /// Async version of from_file
    #[cfg(feature = "async")]
    pub async fn from_file_async<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let options = LoadOptions::default();
        let content = load::read_file_async(path, options.max_file_size).await?;
        Self::from_file_content(path, &content, &options)
    }

    /// Get a value by path
    pub fn get(&self, path: &str) -> Option<&Value> {
        self.values.get(path)
    }

    /// Get a mutable reference to a value by path
    ///
    /// Counts as a change: refused on a read-only configuration (since
    /// v1.4.0), and clears the resolved-path cache so [`Config::get_arc`]
    /// does not return the old value.
    pub fn get_mut(&mut self, path: &str) -> Result<&mut Value> {
        self.ensure_writable()?;
        let value = self.values.get_mut_nested(path)?;
        self.modified = true;
        self.cache.clear();
        Ok(value)
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
    ///
    /// Since v1.4.0 the file is replaced atomically: the text is written to
    /// a temporary file in the same directory, flushed to disk, and renamed
    /// over the target, so a reader (a hot-reload watcher, for one) never
    /// sees a half-written file. The new file keeps the old one's
    /// permissions. When `path` is a symbolic link, the file it points to is
    /// replaced and the link is kept.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let serialized = self.serialize()?;
        atomic_write(path.as_ref(), serialized.as_bytes())
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
        let path = path.as_ref().to_path_buf();
        tokio::task::spawn_blocking(move || atomic_write(&path, serialized.as_bytes()))
            .await
            .map_err(|_| Error::internal("save task failed"))?
    }

    /// Serialize the configuration to string format
    pub fn serialize(&self) -> Result<String> {
        match self.format.as_str() {
            "json" => {
                #[cfg(any(feature = "json", feature = "jsonx"))]
                return crate::parsers::json_parser::serialize(&self.values);
                #[cfg(not(any(feature = "json", feature = "jsonx")))]
                return Err(Error::feature_not_enabled("json"));
            }
            "toml" => {
                #[cfg(feature = "toml")]
                {
                    // The preserved document keeps comments and formatting,
                    // but only matches the values until they are changed
                    #[cfg(feature = "noml")]
                    if !self.modified {
                        if let Some(ref document) = self.noml_document {
                            return Ok(noml::serialize_document(document)?);
                        }
                    }
                    self.serialize_as_toml(false)
                }
                #[cfg(not(feature = "toml"))]
                return Err(Error::feature_not_enabled("toml"));
            }
            "noml" => {
                #[cfg(feature = "noml")]
                {
                    if !self.modified {
                        if let Some(ref document) = self.noml_document {
                            return Ok(noml::serialize_document(document)?);
                        }
                    }
                    self.serialize_as_toml(true)
                }
                #[cfg(not(feature = "noml"))]
                return Err(Error::feature_not_enabled("noml"));
            }
            "conf" => self.serialize_as_conf(),
            _ => Err(Error::unknown_format(&self.format)),
        }
    }

    /// Serialize as CONF format
    ///
    /// The text reads back to the same values (since v1.4.0): strings are
    /// always quoted and escaped, arrays are written as `[a, b]`, and a
    /// nested table becomes a `[parent.child]` section. Comments and the
    /// original layout are not kept, and an empty table is not written.
    /// Values CONF cannot hold (NaN and infinite floats, tables inside
    /// arrays, keys with characters outside letters, digits, `_`, `-` and
    /// `.`) are an error rather than text that reads back differently.
    fn serialize_as_conf(&self) -> Result<String> {
        let mut output = String::new();
        if let Value::Table(table) = &self.values {
            conf_writer::write_table(&mut output, table, &mut Vec::new())?;
        }
        Ok(output)
    }

    /// Serialize the values as TOML text.
    ///
    /// Writes the current values as TOML. With `noml` set, a literal `${` in
    /// a string is written as `$${` so NOML does not treat it as
    /// interpolation when the file is read back. TOML has no null, so a null
    /// value is an error unless writing NOML.
    #[cfg(any(feature = "toml", feature = "noml"))]
    fn serialize_as_toml(&self, noml: bool) -> Result<String> {
        let mut out = String::new();
        if let Value::Table(table) = &self.values {
            toml_writer::write_table(&mut out, table, &mut Vec::new(), noml)?;
        }
        Ok(out)
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
        merge_in_place(&mut self.values, other);
        Ok(())
    }

    /// Take over the runtime state of `old`: options (read-only mode and
    /// the cache knobs), the shared defaults table, and the validation
    /// rules. Used by hot reload so that a reload changes only the values.
    pub(crate) fn inherit_runtime_state(&mut self, old: &mut Config) {
        self.options = old.options.clone();
        self.defaults = Arc::clone(&old.defaults);
        #[cfg(feature = "validation")]
        {
            self.validation_rules = old.validation_rules.take();
        }
    }

    /// Give the validation rules back to `old` (a reload was refused)
    #[cfg(feature = "validation")]
    pub(crate) fn return_validation_rules(&mut self, old: &mut Config) {
        old.validation_rules = self.validation_rules.take();
    }

    /// True when the configuration holds no values at all
    pub(crate) fn is_empty(&self) -> bool {
        match &self.values {
            Value::Table(table) => table.is_empty(),
            Value::Null => true,
            _ => false,
        }
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

    /// Get a value converted to `V`, or `default` when the path is missing
    /// or the value does not convert.
    ///
    /// `V` can be `bool`, `String`, `f64`, any integer type from `i8`/`u8`
    /// to `i64`/`u64`, `isize`, `usize`, or `Value` itself. Integer reads
    /// are range-checked, so `70000` read as `u16` returns the default. See
    /// the `TryFrom<Value>` impls on [`Value`] for the conversion rules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_lib::Config;
    ///
    /// let config = Config::from_string(
    ///     r#"
    /// [server]
    /// port = 8080
    /// host = "0.0.0.0"
    /// "#,
    ///     Some("conf"),
    /// )?;
    ///
    /// let port: u16 = config.get_or("server.port", 3000);
    /// let host: String = config.get_or("server.host", "localhost".to_string());
    /// let tls = config.get_or("server.tls", false); // missing, so the default
    ///
    /// assert_eq!(port, 8080);
    /// assert_eq!(host, "0.0.0.0");
    /// assert!(!tls);
    /// # Ok::<(), config_lib::Error>(())
    /// ```
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
    ///
    /// Every value is checked, at every level, with its full dotted path
    /// (`server.port`) as the path. Before v1.4.0 only top-level values
    /// were checked, so keys inside sections were never validated.
    #[cfg(feature = "validation")]
    pub fn validate(&mut self) -> Result<Vec<ValidationError>> {
        fn walk(
            rules: &mut ValidationRuleSet,
            table: &BTreeMap<String, Value>,
            prefix: &str,
            errors: &mut Vec<ValidationError>,
        ) {
            for (key, value) in table {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                errors.extend(rules.validate(&path, value));
                if let Value::Table(inner) = value {
                    walk(rules, inner, &path, errors);
                }
            }
        }

        match &mut self.validation_rules {
            Some(rules) => {
                if let Value::Table(table) = &self.values {
                    let mut errors = Vec::new();
                    walk(rules, table, "", &mut errors);
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
    /// The canonical options for the server use case:
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
    load: LoadOptions,
    #[cfg(feature = "validation")]
    validation_rules: Option<ValidationRuleSet>,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder
    pub fn new() -> Self {
        Self {
            format: None,
            load: LoadOptions::default(),
            #[cfg(feature = "validation")]
            validation_rules: None,
        }
    }

    /// Set the [`LoadOptions`] the build reads with (limits, include
    /// policy, strict NOML). Since v1.4.0.
    pub fn load_options(mut self, options: LoadOptions) -> Self {
        self.load = options;
        self
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
        let mut config = Config::from_string_with(source, self.format.as_deref(), &self.load)?;
        #[cfg(not(feature = "validation"))]
        let config = Config::from_string_with(source, self.format.as_deref(), &self.load)?;

        #[cfg(feature = "validation")]
        if let Some(rules) = self.validation_rules {
            config.set_validation_rules(rules);
        }

        Ok(config)
    }

    /// Build Config from file
    pub fn from_file<P: AsRef<Path>>(self, path: P) -> Result<Config> {
        #[cfg(feature = "validation")]
        let mut config = Config::from_file_with(path, &self.load)?;
        #[cfg(not(feature = "validation"))]
        let config = Config::from_file_with(path, &self.load)?;

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
            cache: new_cache(),
            defaults: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "noml")]
            noml_document: None,
            #[cfg(feature = "validation")]
            validation_rules: None,
        }
    }
}

/// Merge `other` into `target`: tables merge key by key at every level,
/// anything else replaces. Works in place, with no temporary `Config`.
fn merge_in_place(target: &mut Value, other: &Value) {
    match (target, other) {
        (Value::Table(target_table), Value::Table(other_table)) => {
            for (key, other_value) in other_table {
                match target_table.get_mut(key) {
                    Some(existing) if existing.is_table() && other_value.is_table() => {
                        merge_in_place(existing, other_value);
                    }
                    Some(existing) => *existing = other_value.clone(),
                    None => {
                        target_table.insert(key.clone(), other_value.clone());
                    }
                }
            }
        }
        (target, other) => *target = other.clone(),
    }
}

/// The resolved-path cache with a fixed, small shard count.
///
/// `DashMap::new` sizes its shard array from the CPU count (about 16 KiB on
/// a 32-thread host) before holding a single key; eight shards keep an
/// empty `Config` small at configuration-scale load.
fn new_cache() -> DashMap<Box<str>, Arc<Value>> {
    DashMap::with_shard_amount(8)
}

/// Replace `path` with `bytes` so that no reader sees a partial file.
///
/// Writes a temporary file next to the target, flushes it to disk, copies
/// the old file's permissions, and renames it over the target. A symbolic
/// link is followed, so the file it points to is replaced.
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::sync::atomic::AtomicUsize;
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    let io_err = |e| Error::io(path.display().to_string(), e);
    let target = match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => std::fs::canonicalize(path).map_err(io_err)?,
        _ => path.to_path_buf(),
    };
    let dir = match target.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config".to_string());
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{name}.{}.{n}.tmp", std::process::id()));

    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        if let Ok(meta) = std::fs::metadata(&target) {
            file.set_permissions(meta.permissions())?;
        }
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, &target)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result.map_err(io_err)
}

/// CONF text output that reads back to the same values
mod conf_writer {
    use crate::error::{Error, Result};
    use crate::value::Value;
    use std::collections::BTreeMap;
    use std::fmt::Write;

    fn key_ok(key: &str) -> bool {
        !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
    }

    fn bad_key() -> Error {
        Error::serialize("a key holds characters CONF cannot write")
    }

    /// Plain keys first, then one `[a.b]` section per nested table
    pub(super) fn write_table(
        out: &mut String,
        table: &BTreeMap<String, Value>,
        path: &mut Vec<String>,
    ) -> Result<()> {
        let has_plain = table.values().any(|v| !v.is_table());
        if !path.is_empty() && (has_plain || table.is_empty()) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push('[');
            out.push_str(&path.join("."));
            out.push_str("]\n");
        }
        for (key, value) in table {
            if value.is_table() {
                continue;
            }
            if !key_ok(key) {
                return Err(bad_key());
            }
            out.push_str(key);
            out.push_str(" = ");
            write_value(out, value)?;
            out.push('\n');
        }
        for (key, value) in table {
            if let Value::Table(inner) = value {
                // A section name is split at dots when read back
                if !key_ok(key) || key.contains('.') {
                    return Err(bad_key());
                }
                path.push(key.clone());
                write_table(out, inner, path)?;
                path.pop();
            }
        }
        Ok(())
    }

    fn write_value(out: &mut String, value: &Value) -> Result<()> {
        match value {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => {
                let _ = write!(out, "{b}");
            }
            Value::Integer(i) => {
                let _ = write!(out, "{i}");
            }
            Value::Float(f) if f.is_finite() => {
                // Debug keeps a '.' or an exponent, so it reads back as a float
                let _ = write!(out, "{f:?}");
            }
            Value::Float(_) => {
                return Err(Error::serialize("CONF cannot hold NaN or infinity"));
            }
            Value::String(s) => write_string(out, s),
            Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    if item.is_table() {
                        return Err(Error::serialize("CONF cannot hold a table in an array"));
                    }
                    write_value(out, item)?;
                }
                out.push(']');
            }
            Value::Table(_) => {
                return Err(Error::serialize("CONF cannot hold a table as a value"));
            }
            #[cfg(feature = "chrono")]
            Value::DateTime(dt) => write_string(out, &dt.to_rfc3339()),
        }
        Ok(())
    }

    fn write_string(out: &mut String, s: &str) {
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                c => out.push(c),
            }
        }
        out.push('"');
    }
}

/// TOML text output for TOML and modified NOML configurations
#[cfg(any(feature = "toml", feature = "noml"))]
mod toml_writer {
    use crate::error::{Error, Result};
    use crate::value::Value;
    use std::collections::BTreeMap;
    use std::fmt::Write;

    fn is_table_array(value: &Value) -> bool {
        matches!(value, Value::Array(items)
            if !items.is_empty() && items.iter().all(|v| matches!(v, Value::Table(_))))
    }

    /// Write plain keys first, then `[table]` and `[[array]]` sections
    pub(super) fn write_table(
        out: &mut String,
        table: &BTreeMap<String, Value>,
        path: &mut Vec<String>,
        noml: bool,
    ) -> Result<()> {
        for (key, value) in table {
            if !matches!(value, Value::Table(_)) && !is_table_array(value) {
                write_key(out, key);
                out.push_str(" = ");
                write_value(out, value, path, key, noml)?;
                out.push('\n');
            }
        }
        for (key, value) in table {
            path.push(key.clone());
            match value {
                Value::Table(inner) => {
                    header(out, path, false);
                    write_table(out, inner, path, noml)?;
                }
                Value::Array(items) if is_table_array(value) => {
                    for item in items {
                        header(out, path, true);
                        if let Value::Table(inner) = item {
                            write_table(out, inner, path, noml)?;
                        }
                    }
                }
                _ => {}
            }
            path.pop();
        }
        Ok(())
    }

    fn header(out: &mut String, path: &[String], array: bool) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(if array { "[[" } else { "[" });
        for (i, key) in path.iter().enumerate() {
            if i > 0 {
                out.push('.');
            }
            write_key(out, key);
        }
        out.push_str(if array { "]]\n" } else { "]\n" });
    }

    fn write_key(out: &mut String, key: &str) {
        let bare = key.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
        if bare {
            out.push_str(key);
        } else {
            write_string(out, key, false);
        }
    }

    fn write_string(out: &mut String, s: &str, noml: bool) {
        out.push('"');
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                '$' if noml && chars.peek() == Some(&'{') => out.push_str("$$"),
                c if c.is_control() => {
                    let _ = write!(out, "\\u{:04X}", c as u32);
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }

    fn write_value(
        out: &mut String,
        value: &Value,
        path: &[String],
        key: &str,
        noml: bool,
    ) -> Result<()> {
        match value {
            Value::Null if noml => out.push_str("null"),
            Value::Null => {
                let mut full = path.join(".");
                if !full.is_empty() {
                    full.push('.');
                }
                full.push_str(key);
                return Err(Error::serialize(format!(
                    "TOML has no null value; cannot write '{full}'"
                )));
            }
            Value::Bool(b) => {
                let _ = write!(out, "{b}");
            }
            Value::Integer(i) => {
                let _ = write!(out, "{i}");
            }
            Value::Float(f) => {
                if f.is_nan() {
                    out.push_str("nan");
                } else if f.is_infinite() {
                    out.push_str(if *f > 0.0 { "inf" } else { "-inf" });
                } else {
                    // Debug always keeps a '.' or exponent, so it reads back as a float
                    let _ = write!(out, "{f:?}");
                }
            }
            Value::String(s) => write_string(out, s, noml),
            Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    write_value(out, item, path, key, noml)?;
                }
                out.push(']');
            }
            Value::Table(table) => {
                out.push('{');
                for (i, (k, v)) in table.iter().enumerate() {
                    out.push_str(if i > 0 { ", " } else { " " });
                    write_key(out, k);
                    out.push_str(" = ");
                    write_value(out, v, path, key, noml)?;
                }
                out.push_str(if table.is_empty() { "}" } else { " }" });
            }
            #[cfg(feature = "chrono")]
            Value::DateTime(dt) => write_string(out, &dt.to_rfc3339(), noml),
        }
        Ok(())
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

    #[cfg(feature = "toml")]
    #[test]
    fn modified_toml_serializes_current_values() {
        let source = "# comment\nname = \"app\"\n\n[server]\nport = 80\n";
        let mut config = Config::from_string(source, Some("toml")).unwrap();

        // Unchanged: the original text is kept when the document is preserved
        #[cfg(feature = "noml")]
        assert_eq!(config.serialize().unwrap(), source);

        config.set("server.port", 8080).unwrap();
        config.set("path", "C:\\dir \"q\"").unwrap();
        let text = config.serialize().unwrap();
        let reloaded = Config::from_string(&text, Some("toml")).unwrap();
        assert_eq!(reloaded.as_value(), config.as_value(), "{text}");

        config.set("nothing", Value::Null).unwrap();
        assert!(config.serialize().is_err(), "TOML has no null");
    }

    #[cfg(feature = "noml")]
    #[test]
    fn modified_noml_serializes_current_values() {
        let source = "name = \"app\"\nlog = \"/var/log/${name}.log\"\n";
        let mut config = Config::from_string(source, Some("noml")).unwrap();
        assert_eq!(
            config.get("log").unwrap().as_string().unwrap(),
            "/var/log/app.log"
        );

        config.set("template", "${not_a_reference}").unwrap();
        config.set("nothing", Value::Null).unwrap();
        let text = config.serialize().unwrap();
        let reloaded = Config::from_string(&text, Some("noml")).unwrap();
        assert_eq!(reloaded.as_value(), config.as_value(), "{text}");
    }
}
