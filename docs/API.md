<h1 id="top" align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br><b>config-lib</b><br>
    <sub><sup>API REFERENCE — v1.0.0</sup></sub>
</h1>
<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./README.md" title="Documentation"><b>DOCS</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>API</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="./FORMATS.md" title="Formats"><b>FORMATS</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./ARCHITECTURE.md" title="Architecture"><b>ARCHITECTURE</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./PERFORMANCE.md" title="Performance"><b>PERFORMANCE</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./PLATFORM-NOTES.md" title="Platform Notes"><b>PLATFORM</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./SECURITY.md" title="Security"><b>SECURITY</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./STABILITY-1.0.md" title="Stability Contract"><b>STABILITY</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./GUIDELINES.md" title="Developer Guidelines"><b>GUIDELINES</b></a>
    </sup>
</div>

<br>

> Canonical API reference for `config-lib v1.0.0`. Every public type and free function is documented here with description, key methods, parameter notes, and at least one runnable code example. The full v1.x stability contract for these items is in [`STABILITY-1.0.md`](./STABILITY-1.0.md).

---

## Table of Contents

### Getting Started
- [Installation](#installation)
- [Feature Flags](#feature-flags)

### Top-Level Free Functions
- [`parse`](#parse)
- [`parse_file`](#parse_file)
- [`parse_file_async`](#parse_file_async)
- [`validate`](#validate)

### Core Types
- [`Error` / `Result`](#error)
- [`Value`](#value)
- [`Config`](#config)
- [`ConfigOptions`](#configoptions)
- [`ConfigBuilder`](#configbuilder)
- [`ConfigValue`](#configvalue)
- [`CacheStats`](#cachestats)

### Hot Reload (`hot_reload` module)
- [`HotReloadConfig`](#hotreloadconfig)
- [`HotReloadHandle`](#hotreloadhandle)
- [`Subscription`](#subscription)
- [`ConfigChangeEvent`](#configchangeevent)

### Multi-Instance
- [`ConfigManager`](#configmanager)

### Schema Validation (feature: `schema`)
- [`Schema`](#schema)
- [`SchemaBuilder`](#schemabuilder)
- [`FieldType`](#fieldtype)
- [`FieldSchema`](#fieldschema)

### Rule-Based Validation (feature: `validation`)
- [`ValidationRule`](#validationrule)
- [`ValidationRuleSet`](#validationruleset)
- [`ValidationError`](#validationerror)
- [`ValidationResult`](#validationresult)
- [`ValidationSeverity`](#validationseverity)
- [`ValueType`](#valuetype)
- [`TypeValidator` / `RangeValidator` / `RequiredKeyValidator`](#built-in-validators)

### Audit Logging (`audit` module, always compiled)
- [`AuditLogger`](#auditlogger)
- [`AuditEvent`](#auditevent)
- [`AuditEventType`](#auditeventtype)
- [`AuditSeverity`](#auditseverity)
- [`AuditSink` (trait)](#auditsink-trait)
- [`ConsoleSink` / `FileSink`](#audit-sinks)
- [`init_audit_logger` / `get_audit_logger` / `audit_log`](#audit-free-functions)

### Environment Variable Overrides (feature: `env-override`)
- [`EnvOverrideConfig`](#envoverrideconfig)
- [`EnvOverrideSystem`](#envoverridesystem)
- [`apply_env_overrides` / `apply_env_overrides_default`](#apply_env_overrides)

### Parser Submodules (`parsers::*`)
- [Top-level dispatch: `parse_string` / `parse_file` / `detect_format`](#parsers-top-level)
- [Per-format parsers](#parsers-per-format)

### Deprecated APIs
- [`EnterpriseConfig`](#enterpriseconfig-deprecated)
- [`enterprise::direct::*`](#enterprise-direct-deprecated)
- [`HotReloadConfig::with_change_notifications`](#with_change_notifications-deprecated)

---

<h2 id="installation">Installation</h2>

```toml
[dependencies]
config-lib = "1.0"

# Common feature additions:
config-lib = { version = "1.0", features = ["json", "validation", "env-override"] }

# Everything on:
config-lib = { version = "1.0", features = [
    "json", "xml", "hcl", "noml", "toml",
    "validation", "schema", "async", "chrono",
    "env-override",
] }

# Slim build (CONF only, no hot reload, no extras):
config-lib = { version = "1.0", default-features = false, features = ["conf"] }
```

**MSRV:** Rust `1.75+` for the default feature set; Rust `1.82+` if `noml` or `toml` features are enabled (upstream `noml = "=0.9.0"` declares 1.82).

<h2 id="feature-flags">Feature Flags</h2>

| Feature        | Default? | Purpose                                                                                |
|----------------|----------|----------------------------------------------------------------------------------------|
| `conf`         | yes      | Built-in CONF format parser                                                            |
| `hot-reload`   | yes      | Event-driven file watching via `notify` (inotify/FSEvents/RDCW)                        |
| `json`         | no       | JSON parsing via `serde_json`                                                          |
| `xml`          | no       | XML parsing via `quick-xml`                                                            |
| `hcl`          | no       | HashiCorp Configuration Language (built-in parser)                                     |
| `noml`         | no       | NOML parsing via the upstream `noml` crate (pinned `=0.9.0`)                           |
| `toml`         | no       | TOML parsing via the `noml` crate for format preservation                              |
| `validation`   | no       | Rule-based validation framework (`regex`-backed)                                       |
| `schema`       | no       | Schema validation framework                                                            |
| `async`        | no       | Async file I/O via `tokio`                                                             |
| `chrono`       | no       | DateTime support via `chrono`                                                          |
| `env-override` | no       | Environment-variable override system                                                   |

Feature names and their effects are part of the v1.x stability contract — see [`STABILITY-1.0.md`](./STABILITY-1.0.md) §4.

---

# Top-Level Free Functions

The crate root re-exports four free functions for users who only need parse-once / validate-once semantics.

<h2 id="parse"><code>parse</code></h2>

```rust
pub fn parse(source: &str, format: Option<&str>) -> Result<Value>
```

Parse configuration data from an in-memory string and return the resulting [`Value`](#value) tree. Auto-detects the format when `format` is `None`.

**Parameters:**

| Name     | Type             | Description                                                                  |
|----------|------------------|------------------------------------------------------------------------------|
| `source` | `&str`           | The configuration text                                                       |
| `format` | `Option<&str>`   | Format hint: `"conf"`, `"ini"`, `"properties"`, `"json"`, `"xml"`, `"hcl"`, `"noml"`, `"toml"`. `None` triggers content-based auto-detection |

**Errors:** Returns [`Error::Parse`](#error) on syntax errors, [`Error::UnknownFormat`](#error) when detection fails, or [`Error::FeatureNotEnabled`](#error) when the format requires a Cargo feature that isn't enabled.

**Example — explicit format:**

```rust
use config_lib::parse;

let value = parse("port = 8080\nname = \"app\"", Some("conf"))?;
assert_eq!(value.get("port").unwrap().as_integer()?, 8080);
# Ok::<(), config_lib::Error>(())
```

**Example — auto-detection:**

```rust
use config_lib::parse;

// Auto-detects JSON from the leading `{`.
let value = parse(r#"{"port": 8080}"#, None)?;
assert_eq!(value.get("port").unwrap().as_integer()?, 8080);
# Ok::<(), config_lib::Error>(())
```

<h2 id="parse_file"><code>parse_file</code></h2>

```rust
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Value>
```

Read a configuration file from disk and parse it. Format is detected from the file extension first (`.conf`, `.ini`, `.json`, `.xml`, `.hcl`, `.toml`, `.noml`, `.properties`); falls back to content-based detection if the extension isn't recognized.

**Errors:** Returns [`Error::Io`](#error) on filesystem errors, plus all errors documented for [`parse`](#parse).

**Example:**

```rust
use config_lib::parse_file;

let value = parse_file("app.conf")?;
let port = value
    .get("server.port")
    .ok_or_else(|| config_lib::Error::key_not_found("server.port"))?
    .as_integer()?;
# Ok::<(), config_lib::Error>(())
```

<h2 id="parse_file_async"><code>parse_file_async</code></h2>

```rust
#[cfg(feature = "async")]
pub async fn parse_file_async<P: AsRef<Path>>(path: P) -> Result<Value>
```

Async variant of [`parse_file`](#parse_file). Reads via `tokio::fs::read_to_string`. Requires the `async` feature.

The async file I/O is worker-thread-pool-backed on every platform — see [`PLATFORM-NOTES.md`](./PLATFORM-NOTES.md). Use this when you don't want to block the current async runtime's executor thread.

**Example:**

```rust,no_run
# #[cfg(feature = "async")]
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use config_lib::parse_file_async;

let value = parse_file_async("app.conf").await?;
let port = value.get("server.port").unwrap().as_integer()?;
# Ok(())
# }
```

<h2 id="validate"><code>validate</code></h2>

```rust
#[cfg(feature = "schema")]
pub fn validate(config: &Value, schema: &Schema) -> Result<()>
```

Validate a [`Value`](#value) tree against a [`Schema`](#schema). Returns `Ok(())` on success; returns [`Error::Schema`](#error) (or accumulates [`ValidationError`](#validationerror) details inside the error) on failure. Requires the `schema` feature.

**Example:**

```rust
# #[cfg(feature = "schema")]
# {
use config_lib::{parse, SchemaBuilder, validate};

let value = parse(r#"name = "my-app"
port = 8080"#, Some("conf"))?;

let schema = SchemaBuilder::new()
    .require_string("name")
    .require_integer("port")
    .build();

validate(&value, &schema)?;
# }
# Ok::<(), config_lib::Error>(())
```

---

# Core Types

<h2 id="error"><code>Error</code> / <code>Result</code></h2>

```rust
pub type Result<T> = std::result::Result<T, Error>;

#[non_exhaustive]
pub enum Error {
    Parse { message: String, line: usize, column: usize, file: Option<String> },
    UnknownFormat { format: String },
    KeyNotFound { key: String, available: Vec<String> },
    Type { value: String, expected_type: String, actual_type: String },
    Io { path: String, source: std::io::Error },
    Schema { path: String, message: String, expected: Option<String> },  // feature: schema
    Validation { message: String },
    General { message: String },
    FeatureNotEnabled { feature: String },
    Concurrency { message: String },
    Noml { source: noml::NomlError },                                     // feature: noml
    Internal { message: String, context: Option<String> },
    // ... `#[non_exhaustive]` — new variants may be added in MINOR releases
}
```

`Error` is `#[non_exhaustive]` — match on it with a wildcard arm.

**Constructor helpers:** prefer these over struct-literal construction.

| Method                                          | Description                                                |
|-------------------------------------------------|------------------------------------------------------------|
| `Error::parse(msg, line, column)`               | Syntax error at a known position                           |
| `Error::parse_with_file(msg, line, col, file)`  | Same, plus the file path                                   |
| `Error::key_not_found(key)`                     | Path not found in the configuration                        |
| `Error::key_not_found_with_suggestions(k, vs)`  | Same, with suggested neighbors                             |
| `Error::type_error(value, expected, actual)`    | Type conversion failed                                     |
| `Error::io(path, source)`                       | Wrap a `std::io::Error` with file context                  |
| `Error::unknown_format(format)`                 | Format detection / dispatch failed                         |
| `Error::feature_not_enabled(feature)`           | Operation requires a Cargo feature that isn't enabled      |
| `Error::concurrency(msg)`                       | Lock poisoning / thread coordination failure               |
| `Error::serialize(msg)`                         | Serialization error                                        |
| `Error::schema(path, msg)` *(schema feature)*   | Schema validation failure                                  |
| `Error::validation(msg)`                        | Validation rule failed                                     |
| `Error::general(msg)`                           | Generic error with message                                 |
| `Error::internal(msg)`                          | Internal invariant violation (should never happen)         |

`Error` implements `std::error::Error` (via `thiserror`), `Debug`, and `Display`.

**Example:**

```rust
use config_lib::{parse, Error};

let value = parse("port = 8080", Some("conf"))?;
match value.get("missing_key") {
    Some(v) => println!("found: {v:?}"),
    None => println!("not found"),
}

// Propagating with `?`:
fn require_port(v: &config_lib::Value) -> Result<i64, Error> {
    v.get("port")
        .ok_or_else(|| Error::key_not_found("port"))?
        .as_integer()
}
# Ok::<(), Error>(())
```

<h2 id="value"><code>Value</code></h2>

```rust
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Table(BTreeMap<String, Value>),
    #[cfg(feature = "chrono")]
    DateTime(chrono::DateTime<chrono::Utc>),
}
```

The variant-data type that every parser produces and every accessor returns. `Value` is **not** `#[non_exhaustive]` — exhaustive pattern matching against every variant is a deliberate feature for users writing format converters and type dispatchers.

### Construction

| Method                              | Returns                                       |
|-------------------------------------|-----------------------------------------------|
| `Value::null()`                     | `Value::Null`                                 |
| `Value::bool(b)`                    | `Value::Bool(b)`                              |
| `Value::integer(n)`                 | `Value::Integer(n)`                           |
| `Value::float(f)`                   | `Value::Float(f)`                             |
| `Value::string(s)` *(`impl Into<String>`)* | `Value::String(s.into())`              |
| `Value::array(vec)`                 | `Value::Array(vec)`                           |
| `Value::table(map)`                 | `Value::Table(map)`                           |
| `Value::datetime(dt)` *(chrono)*    | `Value::DateTime(dt)`                         |

### Type Inspection

`type_name()`, `is_null()`, `is_bool()`, `is_integer()`, `is_float()`, `is_string()`, `is_array()`, `is_table()` — all return `bool` or `&'static str` and do not allocate.

### Type Conversion (fallible)

```rust
pub fn as_bool(&self) -> Result<bool>;
pub fn as_integer(&self) -> Result<i64>;
pub fn as_float(&self) -> Result<f64>;
pub fn as_string(&self) -> Result<&str>;
pub fn as_array(&self) -> Result<&Vec<Value>>;
pub fn as_array_mut(&mut self) -> Result<&mut Vec<Value>>;
pub fn as_table(&self) -> Result<&BTreeMap<String, Value>>;
pub fn as_table_mut(&mut self) -> Result<&mut BTreeMap<String, Value>>;
pub fn to_string_representation(&self) -> Result<String>;
```

Each `as_*` returns `Ok(...)` on the matching variant and `Err(Error::Type { ... })` otherwise. Numeric conversions are strict: `as_integer` on a `Value::Float` returns `Err`, not silent truncation.

### Path Access

```rust
pub fn get(&self, path: &str) -> Option<&Value>;
pub fn get_mut_nested(&mut self, path: &str) -> Result<&mut Value>;
pub fn set_nested(&mut self, path: &str, value: Value) -> Result<()>;
pub fn remove(&mut self, path: &str) -> Result<Option<Value>>;
pub fn contains_key(&self, path: &str) -> bool;
pub fn keys(&self) -> Result<Vec<&str>>;
pub fn len(&self) -> usize;
pub fn is_empty(&self) -> bool;
```

Paths use dot notation: `"server.database.host"`. All return `Err(Error::Type)` when applied to a non-table `Value`.

**Example — construction + traversal:**

```rust
use config_lib::Value;
use std::collections::BTreeMap;

let mut tree = BTreeMap::new();
tree.insert("port".to_string(), Value::integer(8080));
tree.insert("name".to_string(), Value::string("my-app"));

let v = Value::table(tree);
assert_eq!(v.get("port").unwrap().as_integer()?, 8080);
assert_eq!(v.get("name").unwrap().as_string()?, "my-app");
assert!(v.contains_key("port"));
assert!(!v.contains_key("missing"));
# Ok::<(), config_lib::Error>(())
```

<h2 id="config"><code>Config</code></h2>

```rust
#[derive(Debug)]
pub struct Config { /* ... */ }
```

The primary user-facing configuration type. Owns a [`Value`](#value) tree, tracks the source file path + format, exposes the cache + defaults table + read-only mode introduced in v0.9.5–v0.9.9.

`Config: Send + Sync` (every field is `Send + Sync`). Multi-thread sharing patterns are documented in [`ARCHITECTURE.md`](./ARCHITECTURE.md) §5.

### Construction

| Constructor                                | Description                                                                |
|--------------------------------------------|----------------------------------------------------------------------------|
| `Config::new()`                            | Empty configuration (no values, no file path)                              |
| `Config::from_string(source, format)`      | Parse from in-memory string with optional format hint                      |
| `Config::from_file(path)`                  | Read + parse from disk; format detected by extension or content            |
| `Config::from_file_async(path)` *(async)*  | Async variant; requires the `async` feature                                |
| `Config::with_options(opts)`               | Empty `Config` with non-default [`ConfigOptions`](#configoptions)          |
| `Config::from(value)`                      | Construct from an existing [`Value`](#value) tree                          |

### Value Access (read)

```rust
pub fn get(&self, path: &str) -> Option<&Value>;
pub fn get_arc(&self, path: &str) -> Option<Arc<Value>>;
pub fn key(&self, path: &str) -> ConfigValue<'_>;
pub fn contains_key(&self, path: &str) -> bool;
pub fn has(&self, path: &str) -> bool;  // alias for contains_key
pub fn keys(&self) -> Result<Vec<&str>>;
pub fn get_or<V>(&self, path: &str, default: V) -> V
    where V: TryFrom<Value> + Clone;
pub fn as_value(&self) -> &Value;
```

| Accessor      | Returns               | Use when                                                                  |
|---------------|-----------------------|---------------------------------------------------------------------------|
| `get`         | `Option<&Value>`      | Single-threaded peek-and-drop. Zero allocation.                           |
| `get_arc`     | `Option<Arc<Value>>`  | Multi-threaded reads, hot loops. Cache-backed (v1.0.0+).                  |
| `key`         | `ConfigValue<'_>`     | Fluent-style chained accessors with defaults                              |
| `contains_key`| `bool`                | Existence check without resolving the value                               |

### Value Access (mutate)

```rust
pub fn get_mut(&mut self, path: &str) -> Result<&mut Value>;
pub fn set<V: Into<Value>>(&mut self, path: &str, value: V) -> Result<()>;
pub fn remove(&mut self, path: &str) -> Result<Option<Value>>;
pub fn merge(&mut self, other: &Config) -> Result<()>;
```

All three mutating methods (`set`, `remove`, `merge`) invalidate the resolved-path cache wholesale on success. They return `Err(Error::general("Configuration is read-only"))` if the `Config` was constructed with `ConfigOptions::read_only = true` or had [`make_read_only`](#config-make-read-only) called on it.

### Cache Management

```rust
pub fn cache_stats(&self) -> CacheStats;
pub fn clear_cache(&self);
```

`cache_stats` is a relaxed-atomic snapshot — see [`CacheStats`](#cachestats). `clear_cache` is the explicit invalidation hook for out-of-band mutations.

### Defaults Table

```rust
pub fn set_default<V: Into<Value>>(&self, path: &str, value: V) -> Result<()>;
pub fn get_or_default(&self, path: &str) -> Option<Value>;
```

Per-path fallback table consulted when the main value tree doesn't have a key. **Independent of `read_only`** — defaults are deployment-time declarations, not user-supplied data. Note the `&self` receiver: defaults can be set on a `Config` that you don't have `&mut` access to.

### File I/O

```rust
pub fn save(&mut self) -> Result<()>;
pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()>;
pub fn save_async(&mut self) -> Result<()>;             // feature: async
pub fn save_to_file_async<P: AsRef<Path>>(&self, path: P) -> Result<()>;  // feature: async
pub fn serialize(&self) -> Result<String>;
pub fn format(&self) -> &str;
pub fn file_path(&self) -> Option<&Path>;
```

`save` writes back to the original file path (errors if `Config::new()` was used and no path was set). `save_to_file` accepts any path. `serialize` returns the on-disk representation in the configured format.

### Modification Tracking

```rust
pub fn is_modified(&self) -> bool;
pub fn mark_clean(&mut self);
```

Set automatically on any `set` / `remove` / `merge`. Reset by `save` and explicitly by `mark_clean`.

### Validation Integration (`validation` feature)

```rust
#[cfg(feature = "validation")]
pub fn set_validation_rules(&mut self, rules: ValidationRuleSet);
pub fn validate(&mut self) -> Result<Vec<ValidationError>>;
pub fn validate_critical_only(&mut self) -> Result<Vec<ValidationError>>;
pub fn is_valid(&mut self) -> Result<bool>;
pub fn validate_path(&mut self, path: &str) -> Result<Vec<ValidationError>>;
```

Attach a [`ValidationRuleSet`](#validationruleset) and call `validate()` to collect violations; `is_valid()` is the "any critical errors?" boolean.

### Schema Integration (`schema` feature)

```rust
#[cfg(feature = "schema")]
pub fn validate_schema(&self, schema: &Schema) -> Result<()>;
```

### Options + Read-Only Mode

<a id="config-make-read-only"></a>

```rust
pub fn options(&self) -> &ConfigOptions;
pub fn is_read_only(&self) -> bool;
pub fn make_read_only(&mut self);
```

`make_read_only` is a one-way switch — no `make_writable` companion. See [`ConfigOptions`](#configoptions) for construction-time configuration.

**Example — load + access + modify:**

```rust
use config_lib::Config;

let mut config = Config::from_string(r#"
[server]
port = 8080
host = "localhost"
"#, Some("conf"))?;

// Read
let port = config.get("server.port").unwrap().as_integer()?;
assert_eq!(port, 8080);

// Write (invalidates cache)
config.set("server.port", 9090)?;
config.set("server.workers", 4i64)?;

// Defaults
config.set_default("server.timeout", 30i64)?;
let timeout = config.get_or_default("server.timeout").unwrap().as_integer()?;
assert_eq!(timeout, 30);

// Modification tracking
assert!(config.is_modified());
# Ok::<(), config_lib::Error>(())
```

**Example — thread-safe cached access via `get_arc`:**

```rust
use config_lib::Config;
use std::sync::Arc;

let mut config = Config::new();
config.set("port", 8080i64)?;
let shared = Arc::new(config);

let handles: Vec<_> = (0..4).map(|_| {
    let cfg = Arc::clone(&shared);
    std::thread::spawn(move || {
        // First call walks the tree + populates the cache.
        // Subsequent calls hit the DashMap-backed cache.
        let port = cfg.get_arc("port").unwrap();
        port.as_integer().unwrap()
    })
}).collect();

for h in handles {
    assert_eq!(h.join().unwrap(), 8080);
}

let stats = shared.cache_stats();
assert!(stats.hits + stats.misses >= 4);
# Ok::<(), config_lib::Error>(())
```

<h2 id="configoptions"><code>ConfigOptions</code></h2>

```rust
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConfigOptions {
    pub read_only: bool,
    pub cache_enabled: bool,
    pub cache_capacity: usize,
}
```

Opt-out behavior knobs for [`Config`](#config). `#[non_exhaustive]` so v1.x MINOR releases can add new knobs without breaking SemVer; callers go through the consuming builder methods rather than struct literals.

**Fields:**

| Field              | Default | Effect                                                                       |
|--------------------|---------|------------------------------------------------------------------------------|
| `read_only`        | `false` | Reject `set` / `remove` / `merge` with `Err(Error::general(...))`            |
| `cache_enabled`    | `true`  | Toggle the `get_arc` resolved-path cache (write-heavy workloads may disable) |
| `cache_capacity`   | `1024`  | Maximum cached entries before eviction (reserved; not yet enforced)          |

### Builder Methods

```rust
pub fn new() -> Self;                                      // == default()
pub fn read_only(self, read_only: bool) -> Self;
pub fn cache_enabled(self, cache_enabled: bool) -> Self;
pub fn cache_capacity(self, cache_capacity: usize) -> Self;
```

All builder methods consume `self` and return `Self` for fluent chaining.

**Example:**

```rust
use config_lib::{Config, ConfigOptions};

// Default options: caching on, writes allowed
let _cfg = Config::with_options(ConfigOptions::default());

// Read-only configuration for a hot path
let opts = ConfigOptions::new().read_only(true);
let mut locked = Config::with_options(opts);
assert!(locked.set("foo", "bar").is_err());

// Write-heavy workload: disable cache
let opts = ConfigOptions::new().cache_enabled(false);
let _cfg = Config::with_options(opts);
```

<h2 id="configbuilder"><code>ConfigBuilder</code></h2>

```rust
pub struct ConfigBuilder { /* ... */ }
```

Fluent builder for `Config` instances. Useful when you want to compose format hints and validation rules before parsing.

| Method                                         | Description                          |
|------------------------------------------------|--------------------------------------|
| `ConfigBuilder::new()`                         | New empty builder                    |
| `.format(fmt)`                                 | Set the format hint                  |
| `.validation_rules(rules)` *(validation)*      | Attach a `ValidationRuleSet`         |
| `.from_string(source)`                         | Parse from an in-memory string       |
| `.from_file(path)`                             | Parse from a file                    |

**Example:**

```rust
use config_lib::ConfigBuilder;

let config = ConfigBuilder::new()
    .format("conf")
    .from_string("port = 8080\n")?;

assert_eq!(config.get("port").unwrap().as_integer()?, 8080);
# Ok::<(), config_lib::Error>(())
```

<h2 id="configvalue"><code>ConfigValue</code></h2>

```rust
pub struct ConfigValue<'a> { /* ... */ }
```

Ergonomic accessor wrapper returned by [`Config::key`](#config-value-access-read). Provides fluent-style access with default fallbacks.

| Method                          | Returns          | Notes                                          |
|---------------------------------|------------------|------------------------------------------------|
| `.as_string()`                  | `Result<String>` | Errors if missing or non-string                |
| `.as_string_or(default)`        | `String`         | Returns `default.to_string()` on missing/wrong-type |
| `.as_integer()`                 | `Result<i64>`    | Errors if missing or non-integer               |
| `.as_integer_or(default)`       | `i64`            | Returns `default` on missing/wrong-type        |
| `.as_bool()`                    | `Result<bool>`   | Errors if missing or non-bool                  |
| `.as_bool_or(default)`          | `bool`           | Returns `default` on missing/wrong-type        |
| `.exists()`                     | `bool`           | Whether the key was found                      |
| `.value()`                      | `Option<&Value>` | Borrow the raw `Value` if present              |

**Example:**

```rust
use config_lib::Config;

let config = Config::from_string("port = 8080\nname = \"app\"", Some("conf"))?;

let port = config.key("port").as_integer_or(8080);  // 8080 (file value)
let host = config.key("host").as_string_or("localhost");  // "localhost" (default)
let debug = config.key("debug").as_bool_or(false);  // false (default)

assert_eq!(port, 8080);
assert_eq!(host, "localhost");
assert!(!debug);
# Ok::<(), config_lib::Error>(())
```

<h2 id="cachestats"><code>CacheStats</code></h2>

```rust
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_ratio: f64,  // hits / (hits + misses), 0.0 when total == 0
}
```

Snapshot of [`Config::get_arc`](#config-value-access-read)'s cache counters. Counters are loaded with `Ordering::Relaxed` — values are statistics, not synchronization primitives.

**Example:**

```rust
use config_lib::Config;

let mut config = Config::new();
config.set("port", 8080i64)?;

let _ = config.get_arc("port"); // miss → populates cache
let _ = config.get_arc("port"); // hit
let _ = config.get_arc("port"); // hit

let stats = config.cache_stats();
assert_eq!(stats.hits, 2);
assert_eq!(stats.misses, 1);
assert!(stats.hit_ratio > 0.5);
# Ok::<(), config_lib::Error>(())
```

---

# Hot Reload (`hot_reload` module)

The `hot_reload` module ships the event-driven file-watching subsystem and the v1.0.0 lock-free notification dispatch (`on_change`). See [`ARCHITECTURE.md`](./ARCHITECTURE.md) §3a + §4 for the full design.

Available when the `hot-reload` Cargo feature is enabled (default in v0.9.6+).

<h2 id="hotreloadconfig"><code>HotReloadConfig</code></h2>

```rust
pub struct HotReloadConfig { /* ... */ }
```

Entry point. Wraps a `Config` with file-watching, debouncing, and notification dispatch.

### Construction + builder methods

```rust
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
pub fn with_poll_interval(self, interval: Duration) -> Self;
pub fn with_debounce(self, debounce: Duration) -> Self;
pub fn with_polling_fallback(self) -> Self;
```

| Method                       | Default                  | Effect                                                                                  |
|------------------------------|--------------------------|-----------------------------------------------------------------------------------------|
| `with_poll_interval(d)`      | `Duration::from_secs(1)` | Polling cadence (primary when `hot-reload` feature off; watchdog when on)               |
| `with_debounce(d)`           | `Duration::from_millis(100)` | Collapses bursts of kernel events (atomic-rename saves emit 3-4 events per save)    |
| `with_polling_fallback()`    | off                      | Run a polling watchdog alongside the event-driven watcher (NFS-safe)                    |

### Notification API (v1.0.0+ — lock-free)

```rust
pub fn on_change<F>(&self, handler: F) -> Subscription
where F: Fn(&ConfigChangeEvent) + Send + Sync + 'static;
```

Register a handler. Returns a [`Subscription`](#subscription) RAII guard whose `Drop` impl unregisters the handler. See [`Subscription`](#subscription) for lifetime management.

### Snapshot / manual reload / start watcher

```rust
pub fn config(&self) -> Arc<RwLock<Config>>;
pub fn snapshot(&self) -> Result<Config>;
pub fn reload(&mut self) -> Result<bool>;
pub fn start_watching(self) -> HotReloadHandle;
pub fn file_path(&self) -> &Path;
pub fn last_modified(&self) -> SystemTime;
```

- `config()` — share the `Arc<RwLock<Config>>` with other threads; the reloader thread atomically swaps the inner `Config` on each successful reload.
- `snapshot()` — re-parse the file from disk and return a fresh `Config` (does not affect the watcher's shared state).
- `reload()` — manually trigger a reload check; returns `Ok(true)` if a reload happened, `Ok(false)` if mtime was unchanged.
- `start_watching()` — consume self, spawn the background worker, return a [`HotReloadHandle`](#hotreloadhandle).

### Deprecated bridge

<a id="with_change_notifications-deprecated"></a>

```rust
#[deprecated(since = "1.0.0")]
pub fn with_change_notifications(self) -> (Self, Receiver<ConfigChangeEvent>);
```

Returns a `(HotReloadConfig, Receiver)` pair like the v0.9.x API. Internally routes through `on_change`, so it shares the same dispatch path; it just adds an `mpsc::send` per event. Kept for source compatibility through v1.x. New code should use [`on_change`](#hotreloadconfig).

**Example — `on_change` + start_watching:**

```rust,no_run
use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};
use std::time::Duration;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let hot = HotReloadConfig::from_file("app.conf")?
    .with_debounce(Duration::from_millis(50));

// Register a handler. Stored in `_sub` so it lives for this scope.
let _sub = hot.on_change(|event: &ConfigChangeEvent| {
    if let ConfigChangeEvent::Reloaded { path, .. } = event {
        println!("config reloaded from {}", path.display());
    }
});

let handle = hot.start_watching();
// ... application runs; the handler fires inline on each reload ...
handle.stop()?;
# Ok(())
# }
```

<h2 id="hotreloadhandle"><code>HotReloadHandle</code></h2>

```rust
pub struct HotReloadHandle { /* ... */ }
```

Background-worker handle returned by [`HotReloadConfig::start_watching`](#hotreloadconfig). Dropping the handle (or calling [`HotReloadHandle::stop`](#hotreloadhandle-stop)) tears down the watcher.

### Register handlers after `start_watching`

```rust
pub fn on_change<F>(&self, handler: F) -> Subscription
where F: Fn(&ConfigChangeEvent) + Send + Sync + 'static;
```

Same semantics as [`HotReloadConfig::on_change`](#hotreloadconfig). Use this when the consumer of the handle is a different component from whoever called `start_watching`.

<a id="hotreloadhandle-stop"></a>

### Lifecycle

```rust
pub fn stop(self) -> Result<()>;
```

Signals the worker to exit and joins the thread. Drop also runs `stop()` semantics if the handle is dropped without explicit stop.

**Example:**

```rust,no_run
use config_lib::hot_reload::HotReloadConfig;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let hot = HotReloadConfig::from_file("app.conf")?;
let handle = hot.start_watching();

// Different component registers its own handler via the handle:
let _component_sub = handle.on_change(|event| {
    // ... handle event ...
});

// Later:
handle.stop()?;
# Ok(())
# }
```

<h2 id="subscription"><code>Subscription</code></h2>

```rust
#[must_use = "..."]
pub struct Subscription { /* ... */ }

impl Drop for Subscription {
    fn drop(&mut self) {
        // unregisters the handler from the watcher's handler list
    }
}
```

RAII handle for a registered change-notification handler. The `#[must_use]` attribute ensures unused `Subscription`s emit a compiler warning — dropping immediately would unregister immediately, which is almost never what the caller wants.

### Lifetime control

```rust
pub fn forget(self);
```

Detach the drop-based unregistration hook. The handler stays in the list for the lifetime of the underlying `HotReloadConfig` / `HotReloadHandle`. Use for process-lifetime handlers where you have no convenient owning scope.

**Idiomatic patterns:**

```rust,no_run
# use config_lib::hot_reload::{HotReloadConfig, ConfigChangeEvent};
# fn run() -> Result<(), Box<dyn std::error::Error>> {
# let hot = HotReloadConfig::from_file("app.conf")?;

// Pattern 1: scope-bound subscription.
let _sub = hot.on_change(|_e: &ConfigChangeEvent| { /* ... */ });
// Handler runs for the rest of the surrounding scope; drops at end.

// Pattern 2: process-lifetime subscription.
hot.on_change(|_e: &ConfigChangeEvent| { /* ... */ }).forget();
// Handler runs until the watcher itself is dropped.

// Pattern 3: explicit early-drop.
let sub = hot.on_change(|_e: &ConfigChangeEvent| { /* ... */ });
// ... later:
drop(sub);  // handler unregistered immediately
# Ok(())
# }
```

<h2 id="configchangeevent"><code>ConfigChangeEvent</code></h2>

```rust
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ConfigChangeEvent {
    Reloaded     { path: PathBuf, timestamp: SystemTime },
    ReloadFailed { path: PathBuf, error: String, timestamp: SystemTime },
    FileModified { path: PathBuf, timestamp: SystemTime },
    FileDeleted  { path: PathBuf, timestamp: SystemTime },
    // #[non_exhaustive] — match with a wildcard arm
}
```

The event variant delivered to `on_change` handlers (and to the deprecated `Receiver<ConfigChangeEvent>` bridge).

| Variant         | When it fires                                                                |
|-----------------|-------------------------------------------------------------------------------|
| `FileModified`  | Kernel event for the watched file arrived; reload about to be attempted       |
| `Reloaded`      | Reload succeeded; the shared `Config` has been atomically swapped             |
| `ReloadFailed`  | Reload attempt failed (parse error, permissions, etc.); last-known-good kept  |
| `FileDeleted`   | Watched file no longer exists; last-known-good `Config` preserved             |

`#[non_exhaustive]` so v1.x MINOR releases can add new variants (e.g. `Renamed`, `PermissionDenied`) without breaking SemVer.

**Example — exhaustive matching with wildcard arm:**

```rust,no_run
use config_lib::hot_reload::ConfigChangeEvent;

fn handle_event(event: &ConfigChangeEvent) {
    match event {
        ConfigChangeEvent::Reloaded { path, .. } => {
            println!("reloaded {}", path.display());
        }
        ConfigChangeEvent::ReloadFailed { path, error, .. } => {
            eprintln!("reload of {} failed: {}", path.display(), error);
        }
        ConfigChangeEvent::FileModified { .. } => {
            // typically not actioned — the Reloaded event that follows is the one to handle
        }
        ConfigChangeEvent::FileDeleted { path, .. } => {
            eprintln!("config file {} deleted", path.display());
        }
        // Required: `ConfigChangeEvent` is `#[non_exhaustive]`.
        _ => {}
    }
}
```

---

<h2 id="configmanager"><code>ConfigManager</code></h2>

```rust
#[derive(Debug, Default)]
pub struct ConfigManager { /* ... */ }
```

Multi-instance primitive — name-indexed map of `Arc<RwLock<Config>>`. Useful when one process maintains several independent configurations (e.g. one per database, one per service, plus a global). All loaded `Config`s are independently swappable; cloned `Arc`s share the same underlying `Config` so writes through one handle are visible to all the others.

### API

```rust
pub fn new() -> Self;
pub fn load<P: AsRef<Path>>(&self, name: &str, path: P) -> Result<()>;
pub fn get(&self, name: &str) -> Option<Arc<RwLock<Config>>>;
pub fn list(&self) -> Vec<String>;
pub fn remove(&self, name: &str) -> bool;
```

| Method     | Receiver | Effect                                                                            |
|------------|----------|-----------------------------------------------------------------------------------|
| `load`     | `&self`  | Parse a file and insert under `name`; replaces any existing entry under that name |
| `get`      | `&self`  | Return `Arc<RwLock<Config>>` shared with previous callers of `get(name)`          |
| `list`     | `&self`  | Names of all currently-loaded configurations                                      |
| `remove`   | `&self`  | Drop the name → config mapping; existing `Arc` holders keep their handle          |

**Example:**

```rust,no_run
use config_lib::ConfigManager;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let manager = ConfigManager::new();
manager.load("app", "app.conf")?;
manager.load("db", "database.conf")?;

let app_handle = manager.get("app").unwrap();
let db_handle = manager.get("db").unwrap();

// Read from one:
{
    let app = app_handle.read().unwrap();
    println!("app name: {:?}", app.get("name"));
}

// Write to the other:
{
    let mut db = db_handle.write().unwrap();
    db.set("max_connections", 200i64)?;
}

println!("loaded configs: {:?}", manager.list());
# Ok(())
# }
```

---

# Schema Validation (`schema` feature)

The `schema` feature adds a declarative schema layer for validating `Value` trees against an expected shape. Re-exported at the crate root: `Schema`, `SchemaBuilder`.

<h2 id="schema"><code>Schema</code></h2>

```rust
pub struct Schema { /* ... */ }
```

A compiled schema. Construct via [`SchemaBuilder`](#schemabuilder). Validate values via [`validate`](#validate) or [`Config::validate_schema`](#schema-integration-schema-feature).

<h2 id="schemabuilder"><code>SchemaBuilder</code></h2>

```rust
pub struct SchemaBuilder { /* ... */ }
```

Fluent builder for `Schema` instances.

| Method                                | Effect                                          |
|---------------------------------------|-------------------------------------------------|
| `SchemaBuilder::new()`                | New empty builder                               |
| `.require_string(key)`                | Field must exist and be a `Value::String`       |
| `.require_integer(key)`               | Field must exist and be a `Value::Integer`      |
| `.require_bool(key)`                  | Field must exist and be a `Value::Bool`         |
| `.optional_string(key)`               | Field may exist; if present, must be a string   |
| `.build()`                            | Finalize into a `Schema`                        |

(Additional builder methods are exposed for other types and for default-value declarations; see rustdoc for the complete list.)

<h2 id="fieldtype"><code>FieldType</code></h2>

```rust
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum FieldType {
    Null,
    Bool,
    Integer,
    Float,
    String,
    Array(Box<FieldType>),
    Table(HashMap<String, FieldSchema>),
    Union(Vec<FieldType>),
    Any,
}
```

The type-shape a `Schema` field can declare. `#[non_exhaustive]`.

<h2 id="fieldschema"><code>FieldSchema</code></h2>

A single field's schema (field type + required-or-not + default + description). Used inside `FieldType::Table`.

**Example — assemble a schema + validate:**

```rust
# #[cfg(feature = "schema")]
# {
use config_lib::{parse, SchemaBuilder, validate};

let value = parse(r#"
name = "my-app"
port = 8080
"#, Some("conf"))?;

let schema = SchemaBuilder::new()
    .require_string("name")
    .require_integer("port")
    .build();

validate(&value, &schema)?;
# }
# Ok::<(), config_lib::Error>(())
```

---

# Rule-Based Validation (`validation` feature)

The `validation` feature adds an extensible rule engine that complements the schema layer. Re-exported at the crate root: `ValidationError`, `ValidationResult`, `ValidationRule`, `ValidationRuleSet`, `ValidationSeverity`.

<h2 id="validationrule"><code>ValidationRule</code></h2>

```rust
pub trait ValidationRule: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, path: &str, value: &Value) -> ValidationResult;
    fn priority(&self) -> u8 { 50 }  // default — lower = higher priority
}
```

Implement this trait for custom validation logic. Three built-in implementations ship in the crate:

- [`TypeValidator`](#built-in-validators) — fail if the value isn't a specified type
- [`RangeValidator`](#built-in-validators) — fail if a numeric value is outside `[min, max]`
- [`RequiredKeyValidator`](#built-in-validators) — fail if a key is missing from a `Value::Table`

<h2 id="validationruleset"><code>ValidationRuleSet</code></h2>

```rust
#[derive(Default)]
pub struct ValidationRuleSet { /* ... */ }
```

A collection of rules. Implements `Debug` (lists each rule by name).

| Method                              | Effect                                          |
|-------------------------------------|-------------------------------------------------|
| `ValidationRuleSet::new()`          | Empty rule set                                  |
| `.add_rule::<R: ValidationRule + 'static>(rule)` | Add a rule, returns `Self` for chaining |
| `.validate(path, value)`            | Run every rule against a `(path, value)` pair, returns `Vec<ValidationError>` |

<h2 id="validationerror"><code>ValidationError</code></h2>

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub path: String,
    pub rule: String,
    pub message: String,
    pub severity: ValidationSeverity,
}
```

Single validation failure. Implements `Display` for human-readable diagnostics.

<h2 id="validationresult"><code>ValidationResult</code></h2>

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(ValidationError),
}
```

<h2 id="validationseverity"><code>ValidationSeverity</code></h2>

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[non_exhaustive]
pub enum ValidationSeverity {
    Critical = 4,
    Error = 3,    // default
    Warning = 2,
    Info = 1,
}
```

`#[non_exhaustive]`. Implements `Ord` for severity comparisons. `Config::is_valid()` returns `false` only when any error has severity `Critical`.

<h2 id="valuetype"><code>ValueType</code></h2>

```rust
pub enum ValueType {
    Null, Bool, Integer, Float, String, Array, Table, DateTime, Any,
}
```

Used by `TypeValidator` to declare the expected type of a `Value`.

<h2 id="built-in-validators">Built-in Validators</h2>

```rust
pub struct TypeValidator { /* ... */ }
pub struct RangeValidator { /* ... */ }
pub struct RequiredKeyValidator { /* ... */ }
```

Each implements [`ValidationRule`](#validationrule) and exposes a constructor:

- `TypeValidator::new(path, expected_type)` — requires the value at `path` to have type `expected_type`
- `RangeValidator::new(path, min, max)` — requires a numeric value at `path` to satisfy `min ≤ v ≤ max`
- `RequiredKeyValidator::new(required_keys)` — requires each name in `required_keys` to be present in the root table

**Example — apply rules to a `Config`:**

```rust
# #[cfg(feature = "validation")]
# {
use config_lib::{
    Config, ValidationRuleSet,
    validation::{RangeValidator, RequiredKeyValidator, TypeValidator, ValueType},
};

let mut config = Config::from_string(r#"
name = "my-app"
port = 8080
"#, Some("conf"))?;

let rules = ValidationRuleSet::new()
    .add_rule(RequiredKeyValidator::new(vec!["name".into(), "port".into()]))
    .add_rule(TypeValidator::new("name", ValueType::String))
    .add_rule(RangeValidator::new("port", 1.0, 65535.0));

config.set_validation_rules(rules);
let errors = config.validate()?;
assert!(errors.is_empty());
# }
# Ok::<(), config_lib::Error>(())
```

---

# Audit Logging (`audit` module, always compiled)

Structured-event audit logging with pluggable sinks. The module is always compiled (no feature gate) so compliance-grade environments don't have to feature-flag-juggle to enable it.

<h2 id="auditlogger"><code>AuditLogger</code></h2>

```rust
pub struct AuditLogger { /* ... */ }
```

Owner of audit-event sinks. Construct one and add sinks; call `log_event(...)` to dispatch events through every registered sink.

| Method                                      | Effect                                                       |
|---------------------------------------------|--------------------------------------------------------------|
| `AuditLogger::new()`                        | Empty logger                                                 |
| `.with_console_sink(min_severity)`          | Add a `ConsoleSink` writing to stdout                        |
| `.with_file_sink(path, min_severity)`       | Add a `FileSink` writing to the given path                   |
| `.add_sink(Box<dyn AuditSink>)`             | Add a custom sink                                            |
| `.enabled(bool)`                            | Toggle the whole logger on/off                               |
| `.log_event(event)`                         | Dispatch an event through every sink (fire-and-forget)       |
| `.flush()`                                  | Flush every sink                                             |

<h2 id="auditevent"><code>AuditEvent</code></h2>

```rust
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: SystemTime,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub key: Option<String>,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub user_context: Option<String>,
    pub metadata: HashMap<String, String>,
    pub error_message: Option<String>,
}
```

Structured audit record. Implements `Display` for human-readable output.

<h2 id="auditeventtype"><code>AuditEventType</code></h2>

```rust
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AuditEventType {
    Access,
    Modification,
    ValidationFailure,
    Reload,
    Load,
    Save,
}
```

<h2 id="auditseverity"><code>AuditSeverity</code></h2>

```rust
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AuditSeverity {
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}
```

`#[non_exhaustive]`. Implements `PartialOrd` for severity-threshold filtering.

<h2 id="auditsink-trait"><code>AuditSink</code> (trait)</h2>

```rust
pub trait AuditSink: Send + Sync {
    fn write_event(&self, event: &AuditEvent) -> Result<(), String>;
    fn flush(&self) -> Result<(), String>;
}
```

Implement this trait to plug in custom audit destinations (syslog, structured-log servers, message brokers, etc.).

<h2 id="audit-sinks">Built-in Sinks</h2>

```rust
pub struct ConsoleSink { /* ... */ }
pub struct FileSink { /* ... */ }
```

| Constructor                              | Behavior                                                              |
|------------------------------------------|-----------------------------------------------------------------------|
| `ConsoleSink::new(min_severity)`         | Writes events at `severity ≥ min_severity` to stdout as `AUDIT: <event display>` |
| `FileSink::new(path, min_severity)`      | Appends events at `severity ≥ min_severity` to the given file         |

<h2 id="audit-free-functions">Process-Global Logger Helpers</h2>

```rust
pub fn init_audit_logger(logger: AuditLogger);
pub fn get_audit_logger() -> Option<Arc<AuditLogger>>;
pub fn audit_log(event: AuditEvent);
```

Optional convenience layer for users who want one process-global audit destination. Initialize once at startup with `init_audit_logger`; subsequent code calls `audit_log(event)` to dispatch through the global logger.

**Example:**

```rust
use config_lib::audit::{
    AuditEvent, AuditEventType, AuditLogger, AuditSeverity,
    init_audit_logger, audit_log,
};
use std::collections::HashMap;
use std::time::SystemTime;

let logger = AuditLogger::new()
    .with_console_sink(AuditSeverity::Info);
init_audit_logger(logger);

audit_log(AuditEvent {
    id: "evt-001".to_string(),
    timestamp: SystemTime::now(),
    event_type: AuditEventType::Load,
    severity: AuditSeverity::Info,
    key: None,
    old_value: None,
    new_value: None,
    user_context: Some("admin".to_string()),
    metadata: HashMap::new(),
    error_message: None,
});
```

---

# Environment Variable Overrides (`env-override` feature)

Override-by-environment-variable, with prefix matching and type-aware parsing. Re-exported at the crate root via `env_override::*`.

<h2 id="envoverrideconfig"><code>EnvOverrideConfig</code></h2>

```rust
pub struct EnvOverrideConfig { /* ... */ }
```

Configuration knobs for the env-override system: prefix, separator, case sensitivity.

| Builder method                  | Effect                                                        |
|---------------------------------|---------------------------------------------------------------|
| `EnvOverrideConfig::new()`      | Empty config (no prefix, default separator)                   |
| `.with_prefix(prefix)`          | E.g. `"MYAPP_"` — only env vars with this prefix are considered |
| `.with_separator(sep)`          | E.g. `"_"` for `MYAPP_DATABASE_HOST` → `database.host`         |
| `.case_insensitive()`           | `MYAPP_database_HOST` works too                                |

<h2 id="envoverridesystem"><code>EnvOverrideSystem</code></h2>

```rust
pub struct EnvOverrideSystem { /* ... */ }
```

Stateful override resolver with internal caching. Construct once per process; reuse across multiple `apply_overrides` calls.

<h2 id="apply_env_overrides"><code>apply_env_overrides</code> / <code>apply_env_overrides_default</code></h2>

```rust
pub fn apply_env_overrides(value: Value, config: EnvOverrideConfig) -> Result<Value>;
pub fn apply_env_overrides_default(value: Value) -> Result<Value>;
```

Returns a new `Value` with environment-variable overrides applied. `apply_env_overrides_default` uses sensible defaults (no prefix; underscore separator).

**Example:**

```rust
# #[cfg(feature = "env-override")]
# {
use config_lib::{parse, env_override::{apply_env_overrides, EnvOverrideConfig}};

let value = parse("port = 8080", Some("conf"))?;

// At this point, if MYAPP_PORT=9090 is in the environment:
let value = apply_env_overrides(
    value,
    EnvOverrideConfig::new().with_prefix("MYAPP_").with_separator("_"),
)?;
// `value.get("port").unwrap().as_integer()?` would be 9090 (env) or 8080 (no env)
# }
# Ok::<(), config_lib::Error>(())
```

---

# Parser Submodules (`parsers::*`)

The `parsers` module is `pub` so advanced users can call format-specific parsers directly (bypassing format detection). The top-level [`parse`](#parse) and [`parse_file`](#parse_file) functions are usually what you want.

<h2 id="parsers-top-level">Top-Level Dispatch</h2>

```rust
pub fn parse_string(source: &str, format: Option<&str>) -> Result<Value>;
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Value>;
pub fn detect_format(content: &str) -> &'static str;
pub fn detect_format_from_path(path: &Path) -> Option<&'static str>;
```

Same dispatch logic used by `crate::parse` and `crate::parse_file`. Exposed for callers who want the format-detection helpers directly.

<h2 id="parsers-per-format">Per-Format Parsers</h2>

Each submodule exposes a `parse(source: &str) -> Result<Value>` and (in most cases) one additional named variant:

| Module                                | Function(s)                                          | Feature   |
|---------------------------------------|------------------------------------------------------|-----------|
| `parsers::conf`                       | `parse`                                              | `conf` (default) |
| `parsers::ini_parser`                 | `parse`, `parse_ini`                                 | always    |
| `parsers::properties_parser`          | `parse`, `PropertiesParser` struct                   | always    |
| `parsers::json_parser`                | `parse`, `serialize`, `from_json_value`, `to_json_value` | `json`  |
| `parsers::xml_parser`                 | `parse`, `parse_xml`, `XmlParser`                    | `xml`     |
| `parsers::hcl_parser`                 | `parse`, `parse_hcl`, `HclParser`                    | `hcl`     |
| `parsers::noml_parser`                | `parse`, `parse_with_preservation`                   | `noml`    |
| `parsers::toml_parser`                | `parse`, `parse_with_preservation`                   | `toml`    |

When the corresponding Cargo feature is disabled, the module's `parse` function still exists but returns `Err(Error::feature_not_enabled(...))`.

**Example — bypass detection, call the JSON parser directly:**

```rust
# #[cfg(feature = "json")]
# {
use config_lib::parsers::json_parser;

let value = json_parser::parse(r#"{"port": 8080}"#)?;
assert_eq!(value.get("port").unwrap().as_integer()?, 8080);
# }
# Ok::<(), config_lib::Error>(())
```

**Format preservation (NOML/TOML only):**

```rust
# #[cfg(feature = "noml")]
# {
use config_lib::parsers::noml_parser;

let source = r#"
# This comment is preserved
port = 8080
"#;

let (value, document) = noml_parser::parse_with_preservation(source)?;
// `value` is the runtime `Value` tree.
// `document` is the upstream `noml::Document` with format-preservation
// information, suitable for round-trip editing.
let _ = (value, document);
# }
# Ok::<(), config_lib::Error>(())
```

For full format details, see [`FORMATS.md`](./FORMATS.md).

---

# Deprecated APIs

These items continue to compile and work through the v1.x line per the deprecation policy in [`STABILITY-1.0.md`](./STABILITY-1.0.md) §7. Removal is scheduled for v2.0.

<h2 id="enterpriseconfig-deprecated"><code>EnterpriseConfig</code> (deprecated since v0.9.4)</h2>

The pre-v0.9.4 cached-and-thread-safe configuration type. **Every operation it exposed is now on the unified [`Config`](#config):**

| Old (`EnterpriseConfig`)           | New (`Config`)                                  |
|------------------------------------|-------------------------------------------------|
| `EnterpriseConfig::new()`          | `Config::new()`                                 |
| `EnterpriseConfig::from_string`    | `Config::from_string`                           |
| `EnterpriseConfig::from_file`      | `Config::from_file`                             |
| `cfg.get(k)` *(owned)*             | `cfg.get_arc(k)`                                |
| `cfg.set(k, v)`                    | `cfg.set(k, v)`                                 |
| `cfg.exists(k)`                    | `cfg.contains_key(k)`                           |
| `cfg.set_default(k, v)`            | `cfg.set_default(k, v)`                         |
| `cfg.get_or_default(k)`            | `cfg.get_or_default(k)`                         |
| `cfg.cache_stats()`                | `cfg.cache_stats()`                             |
| `cfg.make_read_only()`             | `cfg.make_read_only()`                          |
| `cfg.clear()`                      | `cfg.clear_cache()`                             |

See [`examples/enterprise_demo.rs`](../examples/enterprise_demo.rs) for a runnable side-by-side migration table.

<h2 id="enterprise-direct-deprecated"><code>enterprise::direct::*</code> (deprecated since v0.9.4)</h2>

`enterprise::direct::parse_string` and `enterprise::direct::parse_file` are thin wrappers around the top-level [`parse`](#parse) and [`parse_file`](#parse_file). They exist for v0.9.x source compatibility. New code should call the top-level functions.

<h2 id="with-change-notifications-deprecated"><code>HotReloadConfig::with_change_notifications</code> (deprecated since v1.0.0)</h2>

The pre-v1.0.0 channel-based notification API. Returns `(HotReloadConfig, Receiver<ConfigChangeEvent>)`. Internally bridges to [`on_change`](#hotreloadconfig) — same dispatch path, plus one `mpsc::send` per event. See [`Subscription`](#subscription) for the recommended replacement.

---

# See Also

- [`README.md`](../README.md) — overview, quick start, feature highlights
- [`STABILITY-1.0.md`](./STABILITY-1.0.md) — v1.0 SemVer contract
- [`ARCHITECTURE.md`](./ARCHITECTURE.md) — internal design, decision logs
- [`PERFORMANCE.md`](./PERFORMANCE.md) — performance contract + benchmark methodology
- [`PLATFORM-NOTES.md`](./PLATFORM-NOTES.md) — Linux / macOS / Windows behavior
- [`SECURITY.md`](./SECURITY.md) — threat model, fuzz methodology, disclosure
- [`FORMATS.md`](./FORMATS.md) — per-format specifications
- [`GUIDELINES.md`](./GUIDELINES.md) — contributor / development standards
- rustdoc on [docs.rs](https://docs.rs/config-lib) — auto-generated, machine-readable

---

<sub>Last reviewed: 2026-05-19 (v1.0.0).</sub>
