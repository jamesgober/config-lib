<h1 id="top" align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>config-lib</b><br>
    <sub><sup>API REFERENCE</sup></sub>
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
        <a href="./GUIDELINES.md" title="Developer Guidelines"><b>GUIDELINES</b></a>
    </sup>
</div>

<br>

## Table of Contents

### **Core Library**
- **[Installation](#installation)**
- **[Feature Flags](#feature-flags)**
- **[Core Functions](#core-functions)**
  - [`parse()`](#parse)
  - [`parse_file()`](#parse_file)
  - [`validate()`](#validate)
  - [`parse_file_async()`](#parse_file_async)
- **[Error Handling](#error-handling)**

### **Configuration Management**
- **[Config API](#config-api)**
  - [Creation & Loading](#config-creation)
  - [Value Access](#config-access)
  - [Modification & Persistence](#config-modification)
  - [Validation & Schema](#schema-api)
- **[ConfigBuilder API](#configbuilder-api)**
  - [Default Settings](#default-settings)
  - [Multi-Source Loading](#multi-source-loading)
  - [Environment Integration](#environment-integration)
- **[Value API](#value-api)**
  - [Type Construction](#value-construction)
  - [Type Checking](#value-checking)
  - [Type Conversion](#value-conversion)

### **Enterprise Features**
- **[EnterpriseConfig API](#enterpriseconfig-api)**
  - [Performance & Caching](#enterprise-performance)
  - [Thread Safety](#enterprise-threading)
  - [Statistics & Monitoring](#enterprise-monitoring)

### **Advanced Features**
- **[Hot Reload API](#hot-reload-api)**
- **[Audit Logging API](#audit-api)**
- **[Environment Override API](#env-override-api)**
- **[Schema Validation API](#schema-api)**
- **[Async Operations API](#async-api)**

<hr>
<br>

<h2 id="installation">Installation</h2>


### 📋 Install Manually
```toml
[dependencies]
config-lib  = "0.9.0"
```
> Add this to your `Cargo.toml`:


#### Install Features
```toml
[dependencies]

# Single feature
config-lib = { version = "0.9.0", features = ["async"] }

# Multiple features
config-lib = { version = "0.9.0", features = ["async, noml"] }

# Disable Default
config-lib = { version = "0.9.0", features = ["async"] }
```
> **[Features](#feature-flags)**

<br>


### 📋 Install via Terminal
```bash
# Basic installation
cargo add config-lib

# Enable a feature
cargo add config-lib --features async

# Enable multiple features
cargo add config-lib --features async,noml

# Disable Default
cargo add config-lib --features async
```




<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="feature-flags">Feature Flags</h2>

### **Format Support**

| Feature               | Default | Description |
|-----------------------|:-------:|---------------------------------------------------------------|
| `conf`                |  ✅     | CONF file support (built-in parser) - always available        |
| `ini`                 |  ✅     | INI file support (built-in parser) - always available         |
| `properties`          |  ✅     | Java Properties file support (built-in parser)                |
| `json`                |  ❌     | JSON format support with serde_json backend                   |
| `xml`                 |  ❌     | XML format support with quick-xml backend                     |
| `hcl`                 |  ❌     | HashiCorp Configuration Language support                      |
| `toml`                |  ❌     | TOML format support with format preservation                  |
| `noml`                |  ❌     | NOML format support with dynamic features                     |

### **Enterprise & Advanced Features**

| Feature               | Default | Description |
|-----------------------|:-------:|---------------------------------------------------------------|
| `async`               |  ❌     | Async file operations and hot reloading                       |
| `validation`          |  ❌     | Schema validation and type checking system                    |
| `env-override`        |  ❌     | Environment variable override system                          |
| `audit`               |  ❌     | Comprehensive audit logging for compliance                    |
| `hot-reload`          |  ❌     | Zero-downtime configuration hot reloading                     |
| `enterprise`          |  ❌     | Enterprise caching and performance optimizations              |

### **Optional Integrations**

| Feature               | Default | Description |
|-----------------------|:-------:|---------------------------------------------------------------|
| `chrono`              |  ❌     | DateTime support with chrono integration                      |
| `serde`               |  ❌     | Serde serialization/deserialization support                  |

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="core-functions">Core Functions</h2>

The core library provides simple, standalone functions for parsing and validating configuration data without requiring complex setup.

<br>

<h3 id="parse">parse()</h3>

Parse configuration from a string with optional format hint.

**Signature:** `pub fn parse(source: &str, format: Option<&str>) -> Result<Value>`

**Parameters:**
- `source` - The configuration data as a string
- `format` - Optional format hint ("conf", "ini", "properties", "json", "xml", "hcl", "toml", "noml")

**Returns:** `Result<Value>` containing the parsed configuration data

**Errors:**
- Format is unknown or unsupported
- Input contains syntax errors  
- Required features are not enabled for the detected format

**Examples:**

```rust
use config_lib::parse;

// Parse with automatic format detection
let config = parse(r#"
port = 8080
name = "MyApp"
debug = true
"#, None)?;

// Parse with explicit format
let config = parse(r#"
{
  "server": {
    "port": 8080,
    "host": "localhost"
  }
}
"#, Some("json"))?;

// Access parsed values
let port = config.get("port")?.as_integer()?;
let name = config.get("name")?.as_string()?;
let host = config.get("server.host")?.as_string()?;

# Ok::<(), config_lib::Error>(())
```

<br>

<h3 id="parse_file">parse_file()</h3>

Parse configuration from a file, auto-detecting format from extension.

**Signature:** `pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Value>`

**Parameters:**
- `path` - Path to the configuration file

**Returns:** `Result<Value>` containing the parsed configuration data

**Errors:**
- File cannot be read (I/O error)
- File format cannot be detected from extension
- File contains syntax errors
- Required features are not enabled for the detected format

**Supported Extensions:**
- `.conf`, `.cfg` → CONF format
- `.ini` → INI format  
- `.properties` → Properties format
- `.json` → JSON format (requires `json` feature)
- `.xml` → XML format (requires `xml` feature)
- `.hcl` → HCL format (requires `hcl` feature)
- `.toml` → TOML format (requires `toml` feature)
- `.noml` → NOML format (requires `noml` feature)

**Examples:**

```rust
use config_lib::parse_file;

// Parse different file formats
let app_config = parse_file("app.conf")?;
let database_config = parse_file("database.ini")?;  
let api_config = parse_file("api.json")?;

// Access values from any format
let app_port = app_config.get("server.port")?.as_integer()?;
let db_host = database_config.get("connection.host")?.as_string()?;
let api_key = api_config.get("auth.key")?.as_string()?;

# Ok::<(), config_lib::Error>(())
```

<br>

<h3 id="validate">validate() <span style="color: #888">[requires: validation]</span></h3>

Validate configuration against a schema definition.

**Signature:** `pub fn validate(config: &Value, schema: &Schema) -> Result<()>`

**Parameters:**
- `config` - Configuration value to validate
- `schema` - Schema definition for validation

**Returns:** `Result<()>` - Ok if validation passes, Error with details if fails

**Requires:** `validation` feature flag

**Examples:**

```rust
# #[cfg(feature = "validation")]
# {
use config_lib::{parse, SchemaBuilder};

let config = parse(r#"
    app_name = "my-service"
    server_port = 8080
    debug_mode = true
    allowed_hosts = ["localhost", "127.0.0.1"]
"#, None)?;

// Define validation schema
let schema = SchemaBuilder::new()
    .require_string("app_name")
    .require_integer_range("server_port", 1, 65535)
    .require_bool("debug_mode")
    .require_array("allowed_hosts")
    .build();

// Validate configuration
config_lib::validate(&config, &schema)?;
println!("Configuration is valid!");
# }

# Ok::<(), config_lib::Error>(())
```

<br>

<h3 id="parse_file_async">parse_file_async() <span style="color: #888">[requires: async]</span></h3>

Async version of parse_file for non-blocking file operations.

**Signature:** `pub async fn parse_file_async<P: AsRef<Path>>(path: P) -> Result<Value>`

**Parameters:**
- `path` - Path to the configuration file

**Returns:** `Result<Value>` containing the parsed configuration data

**Requires:** `async` feature flag

**Examples:**

```rust
# #[cfg(feature = "async")]
# {
use config_lib::parse_file_async;

async fn load_config() -> Result<(), config_lib::Error> {
    // Non-blocking file parsing
    let config = parse_file_async("large-config.toml").await?;
    
    let server_config = config.get("server")?;
    println!("Server configuration loaded: {:?}", server_config);
    
    Ok(())
}

// Usage in async context
# tokio_test::block_on(async {
let config = parse_file_async("app.conf").await?;
let port = config.get("port")?.as_integer()?;
# Ok::<(), config_lib::Error>(())
# })?;
# }

# Ok::<(), config_lib::Error>(())
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="error-handling">Error Handling</h2>

All functions return `Result<T, Error>` for comprehensive error handling.

### **Error Types**

```rust
use config_lib::{Error, Result};

// Common error patterns
match parse("invalid config", None) {
    Ok(config) => println!("Parsed successfully"),
    Err(Error::ParseError { message, line, column }) => {
        eprintln!("Parse error at line {}, column {}: {}", line, column, message);
    },
    Err(Error::IoError(io_err)) => {
        eprintln!("File I/O error: {}", io_err);
    },
    Err(Error::UnsupportedFormat(format)) => {
        eprintln!("Format '{}' not supported or feature not enabled", format);
    },
    Err(err) => {
        eprintln!("Other error: {}", err);
    }
}
```

### **Error Categories**

- **`ParseError`** - Syntax errors in configuration files
- **`IoError`** - File system access errors
- **`UnsupportedFormat`** - Format not supported or feature not enabled
- **`ValidationError`** - Schema validation failures
- **`TypeError`** - Type conversion errors
- **`PathError`** - Invalid configuration paths

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="config-api">Config API</h2>

<h3 id="config-creation">Creation & Loading</h3>

#### **Config::new()**

Create a new empty configuration.

**Signature:** `pub fn new() -> Self`

**Returns:** Empty Config instance with CONF format

**Example:**
```rust
use config_lib::Config;

let mut config = Config::new();
config.set("app.name", "MyApp")?;
config.set("server.port", 8080)?;

# Ok::<(), config_lib::Error>(())
```

#### **Config::from_string()**

Load configuration from a string with optional format hint.

**Signature:** `pub fn from_string(source: &str, format: Option<&str>) -> Result<Self>`

**Parameters:**
- `source` - Configuration data as string
- `format` - Optional format hint

**Returns:** `Result<Config>` with loaded configuration

**Examples:**
```rust
use config_lib::Config;

// Auto-detect format
let config = Config::from_string(r#"
server_port = 8080
app_name = "my-service"
debug = true
"#, None)?;

// Explicit format
let json_config = Config::from_string(r#"
{
  "database": {
    "host": "localhost",
    "port": 5432
  }
}
"#, Some("json"))?;

# Ok::<(), config_lib::Error>(())
```

#### **Config::from_file()**

Load configuration from a file with automatic format detection.

**Signature:** `pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>`

**Parameters:**
- `path` - Path to configuration file

**Returns:** `Result<Config>` with loaded configuration

**Examples:**
```rust
use config_lib::Config;

// Load different formats
let app_config = Config::from_file("app.conf")?;
let db_config = Config::from_file("database.toml")?;
let api_config = Config::from_file("api.json")?;

# Ok::<(), config_lib::Error>(())
```

#### **Config::from_file_async()** <span style="color: #888">[requires: async]</span>

Async version of from_file for non-blocking file operations.

**Signature:** `pub async fn from_file_async<P: AsRef<Path>>(path: P) -> Result<Self>`

**Examples:**
```rust
# #[cfg(feature = "async")]
# {
use config_lib::Config;

async fn load_configs() -> Result<(), config_lib::Error> {
    let config = Config::from_file_async("large-config.toml").await?;
    println!("Config loaded asynchronously");
    Ok(())
}
# }

# Ok::<(), config_lib::Error>(())
```

<br>

<h3 id="config-access">Value Access</h3>

#### **Config::get()**

Get a value by path using dot notation.

**Signature:** `pub fn get(&self, path: &str) -> Option<&Value>`

**Parameters:**
- `path` - Dot-separated path to the value

**Returns:** `Option<&Value>` - Some if found, None if missing

**Examples:**
```rust
use config_lib::Config;

let config = Config::from_string(r#"
[server]
port = 8080
host = "localhost"

[database]
url = "postgres://localhost/mydb"
"#, Some("toml"))?;

// Access nested values
let port = config.get("server.port")?.as_integer()?;
let host = config.get("server.host")?.as_string()?;
let db_url = config.get("database.url")?.as_string()?;

// Safe access with defaults
let timeout = config.get("server.timeout")
    .and_then(|v| v.as_integer().ok())
    .unwrap_or(30);

# Ok::<(), config_lib::Error>(())
```

#### **Config::contains_key()**

Check if a path exists in the configuration.

**Signature:** `pub fn contains_key(&self, path: &str) -> bool`

**Examples:**
```rust
use config_lib::Config;

let config = Config::from_string("app.name = \"MyApp\"", None)?;

if config.contains_key("app.name") {
    println!("App name is configured");
}

if !config.contains_key("app.version") {
    println!("Version not set, using default");
}

# Ok::<(), config_lib::Error>(())
```

#### **Config::keys()**

Get all available keys in the configuration.

**Signature:** `pub fn keys(&self) -> Result<Vec<&str>>`

**Examples:**
```rust
use config_lib::Config;

let config = Config::from_string(r#"
app.name = "MyApp"
server.port = 8080
database.host = "localhost"
"#, None)?;

let keys = config.keys()?;
for key in keys {
    println!("Available key: {}", key);
}
// Output: app.name, server.port, database.host

# Ok::<(), config_lib::Error>(())
```

<br>

<h3 id="config-modification">Modification & Persistence</h3>

#### **Config::set()**

Set a value by path, creating nested structure as needed.

**Signature:** `pub fn set<V: Into<Value>>(&mut self, path: &str, value: V) -> Result<()>`

**Parameters:**
- `path` - Dot-separated path to set
- `value` - Value to set (any type implementing Into<Value>)

**Examples:**
```rust
use config_lib::Config;

let mut config = Config::new();

// Set various types
config.set("app.name", "MyApp")?;
config.set("server.port", 8080)?;
config.set("features.debug", true)?;
config.set("allowed_hosts", vec!["localhost", "127.0.0.1"])?;

// Nested structure created automatically
config.set("database.connection.pool_size", 10)?;

# Ok::<(), config_lib::Error>(())
```

#### **Config::remove()**

Remove a value by path.

**Signature:** `pub fn remove(&mut self, path: &str) -> Result<Option<Value>>`

**Returns:** The removed value if it existed

**Examples:**
```rust
use config_lib::Config;

let mut config = Config::from_string(r#"
app.name = "MyApp"
server.port = 8080
debug = true
"#, None)?;

// Remove a value
let old_port = config.remove("server.port")?;
println!("Removed port: {:?}", old_port);

// Try to remove non-existent value
let missing = config.remove("nonexistent")?;
assert!(missing.is_none());

# Ok::<(), config_lib::Error>(())
```

#### **Config::is_modified()**

Check if the configuration has been modified since loading.

**Signature:** `pub fn is_modified(&self) -> bool`

**Examples:**
```rust
use config_lib::Config;

let mut config = Config::from_file("app.conf")?;
assert!(!config.is_modified()); // Clean after loading

config.set("new_setting", "value")?;
assert!(config.is_modified()); // Modified after change

# Ok::<(), config_lib::Error>(())
```

#### **Config::save()**

Save configuration back to file, preserving format and comments.

**Signature:** `pub fn save(&self) -> Result<()>`

**Requires:** Must have been loaded from a file

**Examples:**
```rust
use config_lib::Config;

let mut config = Config::from_file("app.conf")?;
config.set("server.port", 9090)?;
config.set("app.version", "1.2.0")?;

// Save preserves format and comments
config.save()?;
println!("Configuration saved with format preservation");

# Ok::<(), config_lib::Error>(())
```

#### **Config::save_to_file()**

Save configuration to a specific file path.

**Signature:** `pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()>`

**Examples:**
```rust
use config_lib::Config;

let config = Config::from_string(r#"
app.name = "MyApp"
server.port = 8080
"#, None)?;

// Save to different file
config.save_to_file("backup.conf")?;
config.save_to_file("configs/app.conf")?;

# Ok::<(), config_lib::Error>(())
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="configbuilder-api">ConfigBuilder API</h2>

Fluent API for building configurations with advanced options and multiple sources.

#### **ConfigBuilder::new()**

Create a new configuration builder.

**Signature:** `pub fn new() -> Self`

**Example:**
```rust
use config_lib::ConfigBuilder;

let config = ConfigBuilder::new()
    .from_file("base.conf")?
    .merge_file("override.conf")?
    .build()?;

# Ok::<(), config_lib::Error>(())
```

#### **ConfigBuilder::from_file()**

Load initial configuration from a file.

**Signature:** `pub fn from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self>`

**Example:**
```rust
use config_lib::ConfigBuilder;

let builder = ConfigBuilder::new()
    .from_file("app.conf")?;

# Ok::<(), config_lib::Error>(())
```

#### **ConfigBuilder::from_string()**

Load initial configuration from a string.

**Signature:** `pub fn from_string(mut self, source: &str, format: Option<&str>) -> Result<Self>`

**Example:**
```rust
use config_lib::ConfigBuilder;

let builder = ConfigBuilder::new()
    .from_string(r#"
server.port = 8080
app.name = "MyApp"
"#, Some("conf"))?;

# Ok::<(), config_lib::Error>(())
```

#### **ConfigBuilder::merge_file()**

Merge additional configuration from a file.

**Signature:** `pub fn merge_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self>`

**Example:**
```rust
use config_lib::ConfigBuilder;

// Layer multiple configuration files
let config = ConfigBuilder::new()
    .from_file("defaults.conf")?      // Base configuration
    .merge_file("environment.json")?  // Environment-specific overrides
    .merge_file("local.toml")?        // Local development overrides
    .build()?;

# Ok::<(), config_lib::Error>(())
```

#### **ConfigBuilder::with_env_overrides()**

Enable environment variable overrides.

**Signature:** `pub fn with_env_overrides(mut self, prefix: &str) -> Self`

**Example:**
```rust
use config_lib::ConfigBuilder;

let config = ConfigBuilder::new()
    .from_file("app.conf")?
    .with_env_overrides("MYAPP_")  // MYAPP_SERVER_PORT overrides server.port
    .build()?;

# Ok::<(), config_lib::Error>(())
```

#### **ConfigBuilder::with_defaults()**

Set default values that are used when keys are missing from the loaded configuration. This is the primary method for implementing preset/fallback configuration values.

**Signature:** `pub fn with_defaults(mut self, defaults: HashMap<String, Value>) -> Self`

**Parameters:**
- `defaults` - HashMap of key-value pairs to use as defaults

**Returns:**
- `Self` - Builder for method chaining

**Description:**
Defaults are applied first, then configuration files override them. This ensures your application always has sensible values while allowing customization through configuration files. Supports nested keys using dot notation (e.g., `"database.host"`).

**Examples:**

**Basic Usage:**
```rust
use config_lib::{ConfigBuilder, Value};
use std::collections::HashMap;

// Set up essential application defaults
let mut defaults = HashMap::new();
defaults.insert("server.port".to_string(), Value::Integer(8080));
defaults.insert("app.debug".to_string(), Value::Bool(false));
defaults.insert("database.timeout".to_string(), Value::Integer(30));
defaults.insert("app.name".to_string(), Value::String("MyApp".to_string()));

let config = ConfigBuilder::new()
    .with_defaults(defaults)
    .from_file("app.conf")?  // File values override defaults
    .build()?;

// Defaults are available even if not in configuration file
let port = config.get("server.port")?.as_integer()?;      // From file or 8080
let timeout = config.get("database.timeout")?.as_integer()?; // From file or 30
let debug = config.get("app.debug")?.as_bool()?;          // From file or false

# Ok::<(), config_lib::Error>(())
```

**Comprehensive Production Defaults:**
```rust
use config_lib::{ConfigBuilder, Value};
use std::collections::HashMap;

// Production-ready defaults with security and performance in mind
let defaults = HashMap::from([
    // Application defaults
    ("app.name".to_string(), Value::String("ProductionApp".to_string())),
    ("app.version".to_string(), Value::String("1.0.0".to_string())),
    ("app.environment".to_string(), Value::String("production".to_string())),
    ("app.debug".to_string(), Value::Bool(false)),
    
    // Server defaults (secure by default)
    ("server.host".to_string(), Value::String("127.0.0.1".to_string())),
    ("server.port".to_string(), Value::Integer(8443)),
    ("server.ssl_enabled".to_string(), Value::Bool(true)),
    ("server.timeout".to_string(), Value::Integer(30)),
    ("server.workers".to_string(), Value::Integer(4)),
    
    // Database defaults
    ("database.host".to_string(), Value::String("localhost".to_string())),
    ("database.port".to_string(), Value::Integer(5432)),
    ("database.pool_size".to_string(), Value::Integer(10)),
    ("database.connection_timeout".to_string(), Value::Integer(60)),
    ("database.ssl_mode".to_string(), Value::String("require".to_string())),
    
    // Caching defaults
    ("cache.enabled".to_string(), Value::Bool(true)),
    ("cache.ttl".to_string(), Value::Integer(3600)),
    ("cache.max_size".to_string(), Value::String("128MB".to_string())),
    
    // Logging defaults
    ("logging.level".to_string(), Value::String("warn".to_string())),
    ("logging.file".to_string(), Value::String("/var/log/app.log".to_string())),
    ("logging.max_size".to_string(), Value::String("100MB".to_string())),
    ("logging.rotate".to_string(), Value::Bool(true)),
    
    // Feature flags (conservative defaults)
    ("features.analytics".to_string(), Value::Bool(false)),
    ("features.monitoring".to_string(), Value::Bool(true)),
    ("features.api_versioning".to_string(), Value::Bool(true)),
]);

let config = ConfigBuilder::new()
    .with_defaults(defaults)
    .from_file("production.conf")?
    .build()?;

// All configuration guaranteed to have values
let app_name = config.get("app.name")?.as_string()?;
let ssl_enabled = config.get("server.ssl_enabled")?.as_bool()?;
let pool_size = config.get("database.pool_size")?.as_integer()?;

# Ok::<(), config_lib::Error>(())
```

**Type-Safe Default Patterns:**
```rust
use config_lib::{ConfigBuilder, Value};
use std::collections::HashMap;

// Helper function for creating typed defaults
fn create_server_defaults() -> HashMap<String, Value> {
    HashMap::from([
        ("server.host".to_string(), Value::String("localhost".to_string())),
        ("server.port".to_string(), Value::Integer(8080)),
        ("server.ssl_enabled".to_string(), Value::Bool(false)),
        ("server.timeout".to_string(), Value::Integer(30)),
    ])
}

fn create_database_defaults() -> HashMap<String, Value> {
    HashMap::from([
        ("database.host".to_string(), Value::String("localhost".to_string())),
        ("database.port".to_string(), Value::Integer(5432)),
        ("database.pool_size".to_string(), Value::Integer(10)),
        ("database.ssl_mode".to_string(), Value::String("prefer".to_string())),
    ])
}

// Combine defaults from multiple sources
let mut all_defaults = create_server_defaults();
all_defaults.extend(create_database_defaults());

let config = ConfigBuilder::new()
    .with_defaults(all_defaults)
    .from_file("app.conf")?
    .build()?;

# Ok::<(), config_lib::Error>(())
```

#### **ConfigBuilder::with_validation()**

Enable schema validation during build.

**Signature:** `pub fn with_validation(mut self, schema: Schema) -> Self`

**Example:**
```rust
# #[cfg(feature = "validation")]
# {
use config_lib::{ConfigBuilder, SchemaBuilder};

let schema = SchemaBuilder::new()
    .require_string("app.name")
    .require_integer_range("server.port", 1, 65535)
    .build();

let config = ConfigBuilder::new()
    .from_file("app.conf")?
    .with_validation(schema)  // Validates during build()
    .build()?;  // Will fail if validation errors
# }

# Ok::<(), config_lib::Error>(())
```

#### **ConfigBuilder::build()**

Finalize the configuration builder and create a Config instance.

**Signature:** `pub fn build(self) -> Result<Config>`

**Example:**
```rust
use config_lib::ConfigBuilder;

// Complete configuration building process
let config = ConfigBuilder::new()
    .from_file("base.conf")?
    .merge_file("overrides.json")?
    .with_env_overrides("APP_")
    .build()?;

// Now use the built configuration
let port = config.get("server.port")?.as_integer()?;
let name = config.get("app.name")?.as_string()?;

# Ok::<(), config_lib::Error>(())
```

#### **Advanced Builder Pattern Example**

```rust
use config_lib::{ConfigBuilder, ConfigValue};
use std::collections::HashMap;

// Complex configuration setup
let mut defaults = HashMap::new();
defaults.insert("server.port".to_string(), ConfigValue::Integer(8080));
defaults.insert("server.host".to_string(), ConfigValue::String("localhost".to_string()));
defaults.insert("app.debug".to_string(), ConfigValue::Bool(false));
defaults.insert("database.pool_size".to_string(), ConfigValue::Integer(10));

let config = ConfigBuilder::new()
    // 1. Start with sensible defaults
    .with_defaults(defaults)
    // 2. Load base configuration
    .from_file("config/default.conf")?
    // 3. Layer environment-specific settings
    .merge_file("config/production.toml")?
    // 4. Add local overrides if they exist
    .merge_file_optional("config/local.json")?
    // 5. Allow environment variable overrides
    .with_env_overrides("MYAPP_")
    // 6. Enable validation
    # #[cfg(feature = "validation")]
    .with_validation(create_app_schema())
    // 7. Build final configuration
    .build()?;

# #[cfg(feature = "validation")]
fn create_app_schema() -> config_lib::Schema {
    config_lib::SchemaBuilder::new()
        .require_string("app.name")
        .require_integer_range("server.port", 1, 65535)
        .build()
}

# Ok::<(), config_lib::Error>(())
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

---

<h2 id="default-settings">Default Settings</h2>

config-lib provides comprehensive support for default/preset configuration values that serve as fallbacks when keys are missing from configuration files. This ensures applications always have sensible defaults while allowing configuration files to override specific values.

### **Default Setting Strategies**

#### **1. ConfigBuilder Defaults (Recommended)**

The primary method for setting defaults using the builder pattern:

```rust
use config_lib::{ConfigBuilder, Value};
use std::collections::HashMap;

let defaults = HashMap::from([
    ("server.port".to_string(), Value::Integer(8080)),
    ("app.debug".to_string(), Value::Bool(false)),
    ("database.timeout".to_string(), Value::Integer(30)),
]);

let config = ConfigBuilder::new()
    .with_defaults(defaults)     // Defaults applied first
    .from_file("app.conf")?      // File overrides defaults
    .build()?;
```

#### **2. Enterprise Config Defaults**

High-performance defaults with caching for enterprise applications:

```rust
use config_lib::enterprise::EnterpriseConfig;

let mut config = EnterpriseConfig::new();

// Set cached defaults
config.set_default("server.port", Value::Integer(8080));
config.set_default("cache.enabled", Value::Bool(true));

// Load file with defaults as fallback
config = EnterpriseConfig::from_file("production.conf")?;

// Sub-50ns access with automatic fallback to defaults
let port = config.get_or_default("server.port");
```

#### **3. Inline Defaults with get_or()**

Provide defaults at access time for maximum flexibility:

```rust
use config_lib::Config;

let config = Config::from_file("app.conf")?;

// Inline defaults (type-safe)
let port = config.get_or("server.port", 8080);
let debug = config.get_or("app.debug", false);
let name = config.get_or("app.name", "DefaultApp".to_string());
```

#### **4. Configuration Merging**

Merge multiple configuration sources with priority ordering:

```rust
use config_lib::Config;

// Base configuration with all defaults
let mut base = Config::from_string(r#"
server.port = 8080
app.debug = false
database.timeout = 30
"#, Some("conf"))?;

// User configuration overrides defaults
let user = Config::from_file("user.conf")?;
base.merge(&user)?;

// Environment-specific overrides
let env = Config::from_file("production.conf")?;
base.merge(&env)?;

// Final priority: defaults < user.conf < production.conf
```

### **Best Practices**

#### **Security-First Defaults**
```rust
// Secure defaults for production
let secure_defaults = HashMap::from([
    ("server.host".to_string(), Value::String("127.0.0.1".to_string())), // Localhost only
    ("server.ssl_enabled".to_string(), Value::Bool(true)),                // SSL required
    ("database.ssl_mode".to_string(), Value::String("require".to_string())), // DB SSL required
    ("logging.level".to_string(), Value::String("warn".to_string())),      // Minimal logging
    ("features.debug".to_string(), Value::Bool(false)),                   // No debug info
]);
```

#### **Environment-Specific Defaults**
```rust
// Different defaults per environment
let defaults = match env::var("ENVIRONMENT").as_deref() {
    Ok("production") => HashMap::from([
        ("logging.level".to_string(), Value::String("error".to_string())),
        ("database.pool_size".to_string(), Value::Integer(20)),
        ("cache.enabled".to_string(), Value::Bool(true)),
    ]),
    Ok("development") => HashMap::from([
        ("logging.level".to_string(), Value::String("debug".to_string())),
        ("database.pool_size".to_string(), Value::Integer(5)),
        ("cache.enabled".to_string(), Value::Bool(false)),
    ]),
    _ => HashMap::from([
        ("logging.level".to_string(), Value::String("info".to_string())),
        ("database.pool_size".to_string(), Value::Integer(10)),
        ("cache.enabled".to_string(), Value::Bool(true)),
    ]),
};
```

#### **Validation with Defaults**
```rust
# #[cfg(feature = "validation")]
# {
use config_lib::validation::{Validator, ValidationRule};

// Ensure defaults meet validation requirements
let defaults = HashMap::from([
    ("server.port".to_string(), Value::Integer(8080)),  // Valid port range
    ("app.name".to_string(), Value::String("ValidApp".to_string())), // Valid pattern
]);

let validator = Validator::new()
    .add_rule("server.port", ValidationRule::IntegerRange(1, 65535))
    .add_rule("app.name", ValidationRule::StringPattern(r"^[a-zA-Z][a-zA-Z0-9_-]*$"));

let config = ConfigBuilder::new()
    .with_defaults(defaults)        // Validated defaults
    .from_file("app.conf")?
    .with_validation(validator)     // Validates defaults + file
    .build()?;
# }
```

### **Performance Considerations**

- **Enterprise Config**: Use for high-frequency access (sub-50ns cached defaults)
- **ConfigBuilder**: Best for application startup with comprehensive defaults
- **Inline get_or()**: Lowest overhead for occasional access
- **Merging**: Good for complex multi-source configurations

### **Common Patterns**

```rust
// Pattern 1: Complete application configuration
struct AppConfig {
    server_port: i64,
    debug_mode: bool,
    db_pool_size: i64,
}

impl AppConfig {
    fn from_config_with_defaults(config: &Config) -> Result<Self> {
        Ok(AppConfig {
            server_port: config.get_or("server.port", 8080),
            debug_mode: config.get_or("app.debug", false),
            db_pool_size: config.get_or("database.pool_size", 10),
        })
    }
}

// Pattern 2: Feature flag defaults
let feature_defaults = HashMap::from([
    ("features.analytics".to_string(), Value::Bool(false)),     // Opt-in
    ("features.caching".to_string(), Value::Bool(true)),        // Opt-out
    ("features.monitoring".to_string(), Value::Bool(true)),     // Always on
]);

// Pattern 3: Tiered default priority
let config = ConfigBuilder::new()
    .with_defaults(system_defaults)      // Lowest priority
    .merge_defaults(app_defaults)        // Medium priority
    .merge_defaults(user_defaults)       // High priority
    .from_file("config.conf")?           // Highest priority
    .build()?;
```

---

<h2 id="value-api">Value API</h2>

The `Value` enum represents all possible configuration data types with comprehensive conversion and validation methods.

<br>

<h3 id="value-construction">Type Construction</h3>

#### **Creating Values**

```rust
use config_lib::Value;

// Basic types
let null = Value::null();
let boolean = Value::bool(true);
let integer = Value::integer(42);
let float = Value::float(3.14);
let string = Value::string("hello");

// Collections
let array = Value::array(vec![
    Value::string("first"),
    Value::integer(123),
    Value::bool(false)
]);

use std::collections::BTreeMap;
let mut table = BTreeMap::new();
table.insert("name".to_string(), Value::string("MyApp"));
table.insert("port".to_string(), Value::integer(8080));
let table_value = Value::table(table);

// DateTime (requires chrono feature)
# #[cfg(feature = "chrono")]
# {
let datetime = Value::datetime(chrono::Utc::now());
# }
```

<br>

<h3 id="value-checking">Type Checking</h3>

```rust
use config_lib::Value;

let value = Value::integer(42);

// Type checking methods
assert!(value.is_integer());
assert!(!value.is_string());
assert!(!value.is_null());

// Get type name
assert_eq!(value.type_name(), "integer");

// Pattern matching
match value {
    Value::Integer(n) => println!("Number: {}", n),
    Value::String(s) => println!("Text: {}", s),
    Value::Bool(b) => println!("Boolean: {}", b),
    _ => println!("Other type: {}", value.type_name()),
}
```

<br>

<h3 id="value-conversion">Type Conversion</h3>

#### **Safe Conversion Methods**

```rust
use config_lib::Value;

let config = config_lib::parse(r#"
port = 8080
name = "MyApp"
debug = true
version = 1.5
tags = ["web", "api"]
"#, None)?;

// Safe conversions with error handling
let port = config.get("port")?.as_integer()?;
let name = config.get("name")?.as_string()?;
let debug = config.get("debug")?.as_bool()?;
let version = config.get("version")?.as_float()?;
let tags = config.get("tags")?.as_array()?;

// Conversion with defaults
let timeout = config.get("timeout")
    .and_then(|v| v.as_integer().ok())
    .unwrap_or(30);

let log_level = config.get("log_level")
    .and_then(|v| v.as_string().ok())
    .unwrap_or_else(|| "info".to_string());

# Ok::<(), config_lib::Error>(())
```

#### **Conversion Methods Reference**

| Method | Returns | Description |
|--------|---------|-------------|
| `as_bool()` | `Result<bool>` | Convert to boolean |
| `as_integer()` | `Result<i64>` | Convert to 64-bit integer |
| `as_float()` | `Result<f64>` | Convert to 64-bit float |
| `as_string()` | `Result<String>` | Convert to string |
| `as_array()` | `Result<&Vec<Value>>` | Get array reference |
| `as_table()` | `Result<&BTreeMap<String, Value>>` | Get table reference |
| `as_datetime()` | `Result<chrono::DateTime<Utc>>` | Convert to datetime (requires chrono) |

#### **Nested Value Access**

```rust
use config_lib::Value;

let config = config_lib::parse(r#"
[server]
host = "localhost"
port = 8080

[database]
connections = 10
timeout = 30
"#, Some("toml"))?;

// Navigate nested structures
let server_section = config.get("server")?.as_table()?;
let host = server_section.get("host").unwrap().as_string()?;

// Direct path access (preferred)
let port = config.get("server.port")?.as_integer()?;
let db_timeout = config.get("database.timeout")?.as_integer()?;

# Ok::<(), config_lib::Error>(())
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="enterpriseconfig-api">EnterpriseConfig API</h2>

Enterprise-grade configuration management with multi-tier caching, thread safety, and performance optimizations.

<br>

<h3 id="enterprise-performance">Performance & Caching</h3>

#### **EnterpriseConfig::new()**

Create a new enterprise configuration instance with optimized caching.

**Signature:** `pub fn new() -> Self`

**Performance:** Initializes multi-tier cache system

**Example:**
```rust
use config_lib::EnterpriseConfig;

let mut config = EnterpriseConfig::new();
config.load_from_string(r#"
server.port = 8080
database.pool_size = 20
cache.ttl = 300
"#, None)?;

# Ok::<(), config_lib::Error>(())
```

#### **EnterpriseConfig::from_file()**

Load configuration with automatic caching optimization.

**Signature:** `pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>`

**Performance:** ~3µs first access, ~457ns cached access

**Example:**
```rust
use config_lib::EnterpriseConfig;

// Load with automatic performance optimization
let config = EnterpriseConfig::from_file("production.conf")?;

// First access: populates cache (~3µs)
let port = config.get("server.port");

// Subsequent access: hits fast cache (~457ns)
let port_again = config.get("server.port");

# Ok::<(), config_lib::Error>(())
```

#### **EnterpriseConfig::get_cached()**

Ultra-fast cached value retrieval.

**Signature:** `pub fn get_cached(&self, path: &str) -> Option<&Value>`

**Performance:** 24.9ns average (50% better than 50ns target)

**Example:**
```rust
use config_lib::EnterpriseConfig;

let config = EnterpriseConfig::from_file("app.conf")?;

// Ultra-fast cached access
let port = config.get_cached("server.port");
let host = config.get_cached("server.host");
let timeout = config.get_cached("database.timeout");

// Perfect for high-frequency access in hot paths
for _ in 0..1_000_000 {
    let _port = config.get_cached("server.port"); // ~25ns each
}

# Ok::<(), config_lib::Error>(())
```

<br>

<h3 id="enterprise-monitoring">Statistics & Monitoring</h3>

#### **EnterpriseConfig::cache_stats()**

Get detailed cache performance statistics.

**Signature:** `pub fn cache_stats(&self) -> (u64, u64, f64)`

**Returns:** `(hits, misses, hit_ratio)`

**Example:**
```rust
use config_lib::EnterpriseConfig;

let config = EnterpriseConfig::from_file("app.conf")?;

// Generate some cache activity
for _ in 0..100 {
    let _port = config.get_cached("server.port");
    let _host = config.get_cached("server.host");
}

// Check performance metrics
let (hits, misses, ratio) = config.cache_stats();
println!("Cache hits: {}", hits);
println!("Cache misses: {}", misses);
println!("Hit ratio: {:.2}%", ratio * 100.0);

// Expected output for repeated access:
// Cache hits: 200
// Cache misses: 2  
// Hit ratio: 99.01%

# Ok::<(), config_lib::Error>(())
```

<br>

<h3 id="enterprise-threading">Thread Safety</h3>

All EnterpriseConfig operations are thread-safe with optimized concurrent access patterns.

**Example:**
```rust
use config_lib::EnterpriseConfig;
use std::sync::Arc;
use std::thread;

let config = Arc::new(EnterpriseConfig::from_file("shared.conf")?);

// Spawn multiple threads for concurrent access
let handles: Vec<_> = (0..4).map(|i| {
    let config = Arc::clone(&config);
    thread::spawn(move || {
        for _ in 0..1000 {
            // Thread-safe cached access
            let _port = config.get_cached("server.port");
            let _timeout = config.get_cached(&format!("worker_{}.timeout", i));
        }
    })
}).collect();

// Wait for all threads
for handle in handles {
    handle.join().unwrap();
}

// Check final statistics
let (hits, misses, ratio) = config.cache_stats();
println!("Concurrent access completed:");
println!("  Threads: 4");
println!("  Total requests: {}", hits + misses);
println!("  Cache hit ratio: {:.2}%", ratio * 100.0);

# Ok::<(), config_lib::Error>(())
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="hot-reload-api">Hot Reload API <span style="color: #888">[requires: hot-reload]</span></h2>

Zero-downtime configuration updates with file watching and automatic reloading.

#### **Config::from_file_with_hot_reload()**

Load configuration with automatic hot reloading enabled.

**Signature:** `pub fn from_file_with_hot_reload<P, F>(path: P, callback: F) -> Result<HotReloadConfig>` where `F: Fn(&Config) + Send + 'static`

**Example:**
```rust
# #[cfg(feature = "hot-reload")]
# {
use config_lib::{Config, hot_reload::ConfigChangeEvent};

// Set up hot reloading with callback
let config = Config::from_file_with_hot_reload("app.conf", |new_config| {
    println!("Configuration reloaded!");
    let port = new_config.get("server.port").unwrap().as_integer().unwrap();
    println!("New port: {}", port);
})?;

// Configuration updates automatically when file changes
// Access is always thread-safe and uses latest version
let current_port = config.get("server.port")?.as_integer()?;
# }

# Ok::<(), config_lib::Error>(())
```

#### **HotReloadConfig::stop_watching()**

Stop the file watcher and hot reload system.

**Example:**
```rust
# #[cfg(feature = "hot-reload")]
# {
use config_lib::Config;

let config = Config::from_file_with_hot_reload("app.conf", |_| {})?;

// Stop hot reloading when no longer needed
config.stop_watching();
# }

# Ok::<(), config_lib::Error>(())
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="audit-api">Audit Logging API <span style="color: #888">[requires: audit]</span></h2>

Comprehensive audit logging for configuration operations and compliance.

#### **AuditLogger::new()**

Create a new audit logger with configurable sinks and severity levels.

**Example:**
```rust
# #[cfg(feature = "audit")]
# {
use config_lib::audit::{AuditLogger, Severity};

let audit = AuditLogger::new()
    .with_console_sink(Severity::Info)
    .with_file_sink("config_audit.log", Severity::Warning)?
    .with_json_sink("audit.jsonl", Severity::Debug)?;

// Log configuration operations
audit.log_config_load("app.conf", "admin_user");
audit.log_config_change("server.port", "8080", "9090", "admin_user");
audit.log_config_save("app.conf", "admin_user");
# }

# Ok::<(), config_lib::Error>(())
```

#### **Audit Event Types**

- **`log_config_load()`** - Configuration file loaded
- **`log_config_save()`** - Configuration file saved  
- **`log_config_change()`** - Configuration value changed
- **`log_access_denied()`** - Unauthorized access attempt
- **`log_validation_error()`** - Schema validation failure
- **`log_reload()`** - Hot reload event

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="env-override-api">Environment Override API <span style="color: #888">[requires: env-override]</span></h2>

Smart environment variable override system with caching and prefix matching.

#### **EnvOverride::new()**

Create environment variable override system.

**Example:**
```rust
# #[cfg(feature = "env-override")]
# {
use config_lib::{Config, env_override::EnvOverride};

let mut config = Config::from_file("app.conf")?;

// Set up environment overrides
let env_override = EnvOverride::new()
    .with_prefix("MYAPP_")           // Maps MYAPP_SERVER_PORT to server.port
    .with_separator("_")             // Use underscore for path separation
    .case_insensitive()              // MYAPP_server_port also works
    .with_cache_ttl(300);            // Cache env vars for 5 minutes

// Apply environment overrides
config.apply_env_overrides(&env_override)?;

// Now environment variables override file values:
// MYAPP_SERVER_PORT=9090 overrides server.port
// MYAPP_DATABASE_HOST=prod.db.com overrides database.host
let port = config.get("server.port")?.as_integer()?; // From env or file
# }

# Ok::<(), config_lib::Error>(())
```

#### **Automatic Type Conversion**

Environment variables are automatically converted to appropriate types:

```rust
# #[cfg(feature = "env-override")]
# {
use std::env;

// Set environment variables (typically done by deployment system)
env::set_var("MYAPP_SERVER_PORT", "8080");           // → integer
env::set_var("MYAPP_DEBUG_MODE", "true");            // → boolean  
env::set_var("MYAPP_APP_NAME", "production-app");    // → string
env::set_var("MYAPP_WORKER_THREADS", "4");           // → integer
env::set_var("MYAPP_CACHE_TTL", "300.5");            // → float

// Values are automatically converted during override application
# }

# Ok::<(), config_lib::Error>(())
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="schema-api">Schema Validation API <span style="color: #888">[requires: validation]</span></h2>

Comprehensive schema validation with custom rules and type checking.

#### **SchemaBuilder::new()**

Build validation schemas with fluent API.

**Example:**
```rust
# #[cfg(feature = "validation")]
# {
use config_lib::{Config, SchemaBuilder, ValidationRule};

let schema = SchemaBuilder::new()
    .require_string("app.name")
    .require_integer_range("server.port", 1, 65535)
    .require_bool("debug_mode")
    .optional_string("log_level", Some("info"))
    .require_array("allowed_hosts")
    .custom_rule("database.url", ValidationRule::Url)
    .custom_rule("admin.email", ValidationRule::Email)
    .build();

let config = Config::from_file("app.conf")?;

// Validate entire configuration
match config.validate(&schema) {
    Ok(()) => println!("Configuration is valid"),
    Err(validation_errors) => {
        for error in validation_errors {
            eprintln!("Validation error: {}", error);
        }
    }
}
# }

# Ok::<(), config_lib::Error>(())
```

#### **Built-in Validation Rules**

| Rule | Description | Example |
|------|-------------|---------|
| `Required` | Value must exist | `.require_string("app.name")` |
| `IntegerRange(min, max)` | Integer within range | `.require_integer_range("port", 1, 65535)` |
| `StringPattern(regex)` | String matches regex | `.string_pattern("version", r"^\d+\.\d+\.\d+$")` |
| `OneOf(options)` | Value in allowed list | `.one_of("level", vec!["debug", "info", "warn"])` |
| `Url` | Valid URL format | `.custom_rule("api_url", ValidationRule::Url)` |
| `Email` | Valid email format | `.custom_rule("contact", ValidationRule::Email)` |
| `MinLength(n)` | String minimum length | `.min_length("password", 8)` |
| `MaxLength(n)` | String maximum length | `.max_length("name", 50)` |

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="async-api">Async Operations API <span style="color: #888">[requires: async]</span></h2>

Non-blocking configuration operations for async applications.

#### **Async File Operations**

```rust
# #[cfg(feature = "async")]
# {
use config_lib::Config;

async fn async_config_operations() -> Result<(), config_lib::Error> {
    // Non-blocking file loading
    let config = Config::from_file_async("large-config.toml").await?;
    
    // Async configuration parsing
    let remote_config = config_lib::parse_file_async("https://api.example.com/config").await?;
    
    // Non-blocking validation
    let schema = config_lib::SchemaBuilder::new()
        .require_string("service.name")
        .build();
    
    config.validate_async(&schema).await?;
    
    Ok(())
}

// Usage in tokio runtime
# tokio_test::block_on(async {
let config = Config::from_file_async("app.conf").await?;
let port = config.get("port")?.as_integer()?;
println!("Loaded port: {}", port);
# Ok::<(), config_lib::Error>(())
# })?;
# }

# Ok::<(), config_lib::Error>(())
```

#### **Async Hot Reloading**

```rust
# #[cfg(all(feature = "async", feature = "hot-reload"))]
# {
use config_lib::Config;

async fn setup_async_hot_reload() -> Result<(), config_lib::Error> {
    let config = Config::from_file_with_hot_reload_async(
        "app.conf",
        |new_config| async move {
            println!("Async configuration reload completed");
            // Perform async operations on configuration change
            notify_services_of_config_change(new_config).await;
        }
    ).await?;
    
    Ok(())
}

async fn notify_services_of_config_change(config: &Config) {
    // Implementation would notify other services
    println!("Notifying services of configuration change");
}
# }

# Ok::<(), config_lib::Error>(())
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>



<!-- FOOT COPYRIGHT
################################################# -->
<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2025 <strong>JAMES GOBER.</strong></sup>
</div>
