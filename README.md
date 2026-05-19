<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <b>config-lib</b>
    <br>
    <sub>
        <sup>RUST CONFIGURATION LIBRARY</sup>
    </sub>
</h1>

<p align="center">
    <a href="https://crates.io/crates/config-lib"><img src="https://img.shields.io/crates/v/config-lib.svg" alt="Crates.io"></a>
    <a href="https://crates.io/crates/config-lib" alt="Download"><img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/config-lib?color=%230099ff"></a>
    <a href="https://docs.rs/config-lib"><img src="https://docs.rs/config-lib/badge.svg" alt="Documentation"></a>
    <a href="https://github.com/jamesgober/config-lib/actions"><img alt="GitHub CI" src="https://github.com/jamesgober/config-lib/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md" title="MSRV"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.82%2B-blue"></a>
</p>

<p align="center">
    <b>Enterprise-Grade Multi-Format Configuration Library</b>
    <br>
    <i>High-performance configuration management with advanced caching, validation, and format preservation</i>
</p>

<br>

<p>
    <strong>config-lib</strong> is a high-performance, enterprise-grade Rust configuration library that provides unified access to 8 different configuration formats through a single, consistent API. Built from the ground up for production environments, it offers advanced features like sub-50ns cached access, hot reloading, audit logging, and comprehensive type safety.
</p>

<p>
    Unlike traditional configuration libraries that focus on a single format, <b>config-lib</b> provides seamless multi-format support with automatic format detection, allowing you to mix and match configuration sources (<b>JSON</b> for APIs, <b>TOML</b> for Rust projects, <b>HCL</b> for DevOps, <b>XML</b> for enterprise systems) while maintaining a unified programming interface. The library automatically handles type conversions, validation, and format preservation, making it ideal for complex applications that need to integrate with multiple configuration ecosystems.
</p>

<p>
    With its enterprise-focused architecture, <b>config-lib</b> delivers production-grade features including comprehensive error handling with zero unsafe code, multi-tier caching with lock-free read paths, environment variable overrides with smart prefix matching, and built-in audit logging for compliance requirements. The library is designed for high-throughput applications, targeting sub-50ns cached access times on hot paths and supporting <b>hot reloading</b> without service interruption.
</p>

<hr>


## Features

### **Multi-Format Support**
- **CONF** - Built-in parser for standard .conf files (default)
- **INI** - Full INI file parsing with sections, comments, and data type detection  
- **JSON** - JSON format with edit capabilities and serialization
- **XML** - Zero-copy XML parsing with quick-xml for Java/.NET environments
- **HCL** - HashiCorp Configuration Language for DevOps workflows
- **Properties** - Complete Java .properties file parsing with Unicode and escaping
- **NOML** - Advanced configuration with dynamic features (feature: `noml`)
- **TOML** - TOML format with format preservation (feature: `toml`)

### **Enterprise Performance**
- **Sub-50ns Cache Access Target** - Multi-tier caching designed for sub-50ns reads on hot paths
- **Zero-Copy Parsing** - Minimized allocations and string operations where possible
- **Lock-Free Read Paths** - Poison-resistant locking with graceful failure recovery
- **Hot Value Cache** - Ultra-fast access for frequently used values
- **Cache Hit Ratio Tracking** - Built-in performance statistics and monitoring

> **Note on benchmark numbers:** detailed criterion-backed benchmark numbers will land with **v1.0.0**. Until then, performance claims should be treated as targets, not guarantees.

<br>

### **Production Features** 
- **Configuration Hot Reloading** - File watching with thread-safe Arc swapping
- **Audit Logging System** - Structured event logging with multiple sinks and severity filtering
- **Environment Variable Overrides** - Smart caching with prefix matching and type conversion
- **Schema Validation** - Trait-based validation system with feature gates
- **Format Preservation** - Maintains comments, whitespace, and original formatting
- **Async Native** - Full async/await support throughout the API

### **Reliability & Safety**
- **Zero Unsafe Code** - All `unwrap()` calls eliminated, comprehensive error handling
- **Type Safety** - Rich type system with automatic conversions and validation
- **Enterprise Error Handling** - Production-ready error messages with context preservation
- **Comprehensive Testing** - Extensive unit, integration, and doc test coverage

---

<br>

## **Why Choose config-lib?**

### **Unified Multi-Format Support**
Unlike single-format libraries, **config-lib** provides seamless access to 8 configuration formats through one consistent API. No need to learn different libraries for `TOML`, `JSON`, `XML`, and `HCL` - one API handles them all with automatic format detection.

### **Enterprise-Grade Performance**
Multi-tier caching with lock-free read paths is designed to deliver sub-50ns cached access on hot paths. Built for high-throughput applications with minimal performance overhead.

### **Production-Ready Reliability**
Zero unsafe code, comprehensive error handling, and poison-resistant locking ensure your configuration system won't crash your application. Extensive testing coverage validates edge cases and error conditions.

### **Developer Experience First**
Rich type system with automatic conversions, format preservation for round-trip editing, and detailed error messages with source location context. No more cryptic parsing errors or manual type casting.

### **Advanced Enterprise Features**
Hot reloading without service interruption, structured audit logging for compliance, environment variable overrides with smart caching, and schema validation with custom rules - features typically requiring multiple libraries.

---

<br>


## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
config-lib = "0.9"

# For enhanced functionality, enable optional features:
config-lib = { version = "0.9", features = [
    "json",           # JSON format support with serialization
    "xml",            # XML format support with quick-xml backend  
    "hcl",            # HashiCorp Configuration Language support
    "toml",           # TOML format support with preservation
    "noml",           # NOML format support with dynamic features
    "validation",     # Schema validation and type checking
    "async",          # Async operations and hot reloading
    "env-override",   # Environment variable override system
    "audit",          # Audit logging and compliance features
] }
```

**Feature Recommendations**:
- **Minimal**: Use default features for CONF/INI/Properties support
- **Web Applications**: Add `"json"`, `"env-override"`, `"validation"`
- **DevOps Tools**: Add `"hcl"`, `"toml"`, `"async"`, `"audit"`
- **Enterprise Systems**: Add `"xml"`, `"validation"`, `"audit"`, `"env-override"`
- **Full Featured**: Include all features for maximum flexibility

---

<br>

## Quick Start

```rust
use config_lib::Config;

// Parse any supported format automatically
let mut config = Config::from_string(r#"
[database]
host = "localhost"
port = 5432
username = "admin"

[app]
name = "MyApp"
debug = true
"#, None)?;

// Access values with type safety
let host = config.get("database.host")?.as_string()?;
let port = config.get("database.port")?.as_integer()?;
let debug = config.get("app.debug")?.as_bool()?;

// Modify configuration (preserves format and comments)
config.set("database.port", 5433)?;
config.set("app.version", "1.0.0")?;

println!("Connecting to {}:{}", host, port);
```

### **Multi-Format Support**

```rust
use config_lib::Config;

// All 8 formats now fully operational
let config = Config::from_string(r#"
[server]
port = 8080
host = "localhost"
"#, Some("toml"))?;

// Consistent API patterns across all parsers
let port = config.get("server.port")?.as_integer()?;
let timeout = config.get("server.timeout")
    .and_then(|v| v.as_integer().ok())
    .unwrap_or(30);
let name = config.get("app.name")
    .and_then(|v| v.as_string().ok())
    .unwrap_or_else(|| "DefaultApp".to_string());

// Check existence
if config.contains_key("server.ssl") {
    println!("SSL configuration found");
}
```

### **Enterprise Caching**

```rust
use config_lib::EnterpriseConfig;

// High-performance cached configuration
let config = EnterpriseConfig::from_string(r#"
database.host = "localhost"
server.port = 8080
"#, Some("conf"))?;

// Sub-50ns target for cached access (verified at v1.0.0)
let cached_value = config.get("database.host");

// View cache performance stats
let (hits, misses, ratio) = config.cache_stats();
println!("Cache hit ratio: {:.2}%", ratio * 100.0);
```

### **Default Configuration Settings**

**config-lib** provides multiple powerful methods for setting default/preset values that serve as fallbacks when keys are missing from configuration files. This ensures your application always has sensible defaults while allowing configuration files to override specific values.

#### **Method 1: ConfigBuilder with Presets (Recommended)**

```rust
use config_lib::{ConfigBuilder, Value};
use std::collections::HashMap;

// Set up comprehensive default values before loading config files
let mut defaults = HashMap::new();
defaults.insert("app.name".to_string(), Value::String("MyApplication".to_string()));
defaults.insert("app.version".to_string(), Value::String("1.0.0".to_string()));
defaults.insert("app.debug".to_string(), Value::Bool(false));

// Server defaults
defaults.insert("server.host".to_string(), Value::String("localhost".to_string()));
defaults.insert("server.port".to_string(), Value::Integer(8080));
defaults.insert("server.timeout".to_string(), Value::Integer(30));
defaults.insert("server.workers".to_string(), Value::Integer(4));

// Database defaults
defaults.insert("database.host".to_string(), Value::String("localhost".to_string()));
defaults.insert("database.port".to_string(), Value::Integer(5432));
defaults.insert("database.pool_size".to_string(), Value::Integer(10));
defaults.insert("database.timeout".to_string(), Value::Integer(60));

// Logging defaults
defaults.insert("logging.level".to_string(), Value::String("info".to_string()));
defaults.insert("logging.file".to_string(), Value::String("app.log".to_string()));
defaults.insert("logging.max_size".to_string(), Value::String("10MB".to_string()));

// Create config with defaults, then load from file
let config = ConfigBuilder::new()
    .with_defaults(defaults)         // Apply presets first
    .from_file("app.conf")?          // File values override presets
    .build()?;

// All values are guaranteed to exist (from file or defaults)
let app_name = config.get("app.name")?.as_string()?;           // File value or "MyApplication"
let port = config.get("server.port")?.as_integer()?;           // File value or 8080
let pool_size = config.get("database.pool_size")?.as_integer()?; // File value or 10
let log_level = config.get("logging.level")?.as_string()?;      // File value or "info"
```

#### **Method 2: Enterprise Config with Default Tables**

```rust
use config_lib::enterprise::EnterpriseConfig;

// Create enterprise config with default support
let mut config = EnterpriseConfig::new();

// Set comprehensive defaults that serve as fallbacks
config.set_default("app.name", Value::String("Enterprise App".to_string()));
config.set_default("app.environment", Value::String("development".to_string()));
config.set_default("server.port", Value::Integer(8080));
config.set_default("server.host", Value::String("0.0.0.0".to_string()));
config.set_default("database.connection_timeout", Value::Integer(30));
config.set_default("cache.enabled", Value::Bool(true));
config.set_default("cache.ttl", Value::Integer(3600));

// Load configuration from file (with ultra-fast caching)
config = EnterpriseConfig::from_file("production.conf")?;

// Automatically falls back to defaults for missing keys (sub-50ns access)
let environment = config.get_or_default("app.environment");     // File value or "development"
let cache_ttl = config.get_or_default("cache.ttl");             // File value or 3600
let workers = config.get_or_default("server.workers");          // File value or default if set

// Enterprise features: performance monitoring
let (hits, misses, ratio) = config.cache_stats();
println!("Cache performance: {:.2}% hit ratio", ratio * 100.0);
```

#### **Method 3: Inline Defaults with get_or()**

```rust
use config_lib::Config;

// Load configuration file
let config = Config::from_file("app.conf")?;

// Provide defaults inline when accessing values
let app_config = AppConfig {
    name: config.get_or("app.name", "DefaultApp".to_string()),
    port: config.get_or("server.port", 8080),
    debug: config.get_or("app.debug", false),
    timeout: config.get_or("server.timeout", 30),
    
    // Database configuration with sensible defaults
    db_host: config.get_or("database.host", "localhost".to_string()),
    db_port: config.get_or("database.port", 5432),
    db_pool_size: config.get_or("database.pool_size", 10),
    
    // Feature flags with conservative defaults
    analytics_enabled: config.get_or("features.analytics", false),
    cache_enabled: config.get_or("features.cache", true),
    monitoring_enabled: config.get_or("features.monitoring", false),
};
```

#### **Best Practices for Default Configuration**

1. **Use Sensible Defaults**: Choose defaults that work for most use cases
2. **Document Defaults**: Keep defaults in sync with documentation
3. **Environment-Specific**: Use different defaults for dev/staging/production
4. **Type Safety**: Ensure defaults match expected types
5. **Validation**: Validate both defaults and overrides
6. **Performance**: Use Enterprise config for high-performance scenarios

```rust
// Example: Production-ready defaults with validation
use config_lib::{ConfigBuilder, Value, validation::Validator};

// Production defaults (secure and performant)
let defaults = HashMap::from([
    ("server.host".to_string(), Value::String("127.0.0.1".to_string())), // Secure default
    ("server.port".to_string(), Value::Integer(8443)),                     // HTTPS default
    ("server.ssl_enabled".to_string(), Value::Bool(true)),                // Secure by default
    ("database.ssl_mode".to_string(), Value::String("require".to_string())), // Secure DB
    ("logging.level".to_string(), Value::String("warn".to_string())),      // Production logging
    ("features.debug".to_string(), Value::Bool(false)),                   // Disable debug
]);

// Validation rules for all values (including defaults)
let validator = Validator::new()
    .add_rule("server.port", ValidationRule::IntegerRange(1, 65535))
    .add_rule("server.host", ValidationRule::Required)
    .add_rule("logging.level", ValidationRule::OneOf(vec!["error", "warn", "info", "debug"]));

let config = ConfigBuilder::new()
    .with_defaults(defaults)
    .from_file("production.conf")?
    .with_validation(validator)         // Validate defaults + file values
    .build()?;
```

### **Environment Variable Integration**

```rust
use config_lib::{Config, env_override::EnvOverride};

// Load configuration with environment variable overrides
let mut config = Config::from_file("app.conf")?;

// Enable environment variable overrides with prefix
let env_override = EnvOverride::new()
    .with_prefix("MYAPP_")  // Maps MYAPP_DATABASE_HOST to database.host
    .with_separator("_")    // Use underscore as path separator
    .case_insensitive();    // MYAPP_database_HOST also works

config.apply_env_overrides(&env_override)?;

// Now environment variables override file values:
// MYAPP_DATABASE_HOST=prod.db.com overrides database.host from file
// MYAPP_SERVER_PORT=9090 overrides server.port from file
let host = config.get("database.host")?.as_string()?;  // From env or file
let port = config.get("server.port")?.as_integer()?;   // From env or file
```

### **Configuration Validation & Type Safety**

```rust
use config_lib::{Config, validation::{Validator, ValidationRule}};

// Define validation rules for configuration
let validator = Validator::new()
    .add_rule("server.port", ValidationRule::IntegerRange(1, 65535))
    .add_rule("database.host", ValidationRule::Required)
    .add_rule("app.name", ValidationRule::StringPattern(r"^[a-zA-Z][a-zA-Z0-9_-]*$"))
    .add_rule("logging.level", ValidationRule::OneOf(vec!["debug", "info", "warn", "error"]));

let mut config = Config::from_file("app.conf")?;

// Validate configuration before use
validator.validate(&config)?;  // Fails fast with detailed error messages

// Safe access with automatic type conversion
let port = config.get("server.port")?.as_integer()?;  // Guaranteed valid range
let log_level = config.get("logging.level")?.as_string()?;  // Guaranteed valid value
```

### **Hot Reloading & Audit Logging**

```rust
use config_lib::{Config, audit};

// Enable audit logging
let audit_logger = audit::AuditLogger::new()
    .with_console_sink(audit::Severity::Info)
    .with_file_sink("config_audit.log", audit::Severity::Warning)?;

// Hot reloading configuration
let config = Config::from_file_with_hot_reload("app.conf", move |new_config| {
    println!("Configuration reloaded!");
    audit_logger.log_reload("app.conf", "admin");
})?;
```

### **Advanced Multi-File Configuration**

```rust
use config_lib::{ConfigBuilder, ConfigMergeStrategy};

// Load and merge multiple configuration files
let config = ConfigBuilder::new()
    .from_file("default.conf")?     // Base configuration
    .merge_file("environment.json", ConfigMergeStrategy::Override)?  // Environment overrides
    .merge_file("local.toml", ConfigMergeStrategy::Additive)?        // Local additions
    .merge_file("secrets.hcl", ConfigMergeStrategy::SecureOverride)? // Secure values
    .build()?;

// Access merged configuration
let database_url = config.get("database.url")?.as_string()?;  // From secrets.hcl
let app_name = config.get("app.name")?.as_string()?;          // From default.conf
let debug_mode = config.get("debug")?.as_bool()?;             // From environment.json
```

---

<br>

## **Status: Late Beta — On the Path to v1.0.0**

**Current Version:** `0.9.x` — late beta, API stabilizing, structurally mature.
**Target:** `1.0.0` stable, scheduled per the roadmap in `.dev/ROADMAP.md`.

**What's complete**:
- **Universal Format Support** - All 8 configuration formats with consistent API (CONF, INI, JSON, XML, HCL, Properties, NOML, TOML)
- **Multi-Tier Caching** - Designed for sub-50ns cached reads on hot paths
- **Production Safety** - Zero unsafe code, comprehensive error handling, poison-resistant locking
- **Advanced Features** - Hot reloading, audit logging, environment overrides, schema validation
- **Developer Experience** - Rich type system, format preservation (NOML/TOML), automatic type conversion
- **Quality Assurance** - Comprehensive test suite, zero clippy warnings

**What's planned for v1.0.0**:
- Unified `Config` API (consolidating current `Config` + `EnterpriseConfig`)
- Lock-free cached reads with verified sub-50ns benchmarks
- Event-driven hot reload via `notify` (inotify / FSEvents / ReadDirectoryChangesW)
- Fuzz-tested parsers for every format
- Stability contract: API frozen for the lifetime of v1.x

**Performance targets** (to be verified by committed benchmarks before v1.0.0):
- **<50ns** cached value access on hot paths
- **<500ns** cached write
- **<100ms** hot-reload detection latency (event-driven)
- **Zero-allocation** hot-path reads

**Note on API stability**: The public API is **not yet frozen**. Expect refinements through `0.9.x` releases. The `v1.0.0` release will lock down the stability contract documented in `docs/STABILITY-1.0.md`.

---

<br>

## **Documentation & Resources**

### **Documentation**
- **[Documentation Index](docs/README.md)** - Complete documentation hub and navigation
- **[API Reference](docs/API.md)** - Comprehensive API documentation with examples
- **[Valid Formats](docs/FORMATS.md)** - Detailed format specifications and examples
- **[Code Guidelines](docs/GUIDELINES.md)** - Development standards and best practices

### **External Resources**
- **[API Documentation](https://docs.rs/config-lib)** - Complete API reference with examples
- **[Crate Registry](https://crates.io/crates/config-lib)** - Official crate distribution
- **[Examples Directory](examples/)** - 20+ comprehensive examples covering all features
- **[Performance Benchmarks](benches/)** - Detailed performance analysis and comparisons
- **[NOML Language](https://github.com/noml-lang/noml)** - NOML language specification and usage.

### **Getting Started Guides**
- **[Quick Start Guide](examples/basic.rs)** - Basic configuration loading and access
- **[Multi-Format Demo](examples/multi_format.rs)** - Working with different configuration formats
- **[Enterprise Features](examples/enterprise_demo.rs)** - Advanced caching and performance
- **[Hot Reloading](examples/hot_reload_demo.rs)** - Dynamic configuration updates
- **[Validation System](examples/validation_demo.rs)** - Schema validation and type checking

### **Common Use Cases**
- **Web Applications**: Environment overrides, JSON/TOML configs
- **DevOps Tools**: [HCL integration](examples/hcl_demo.rs), audit logging, hot reloading
- **Enterprise Systems**: [XML support](examples/xml_demo.rs), validation, caching
- **Microservices**: Multi-format support, environment-based configuration

<br>

---

### **Troubleshooting**

<details>
  <summary><b>"Parser not found" error when using TOML, NOML,or JSON?</b></summary>
    <p>✅ Enable the corresponding feature flag in your <b>Cargo.toml</b>: <code>features = ["toml", "noml", "json"]</code>.</p>
</details>

<details>
  <summary><b>Poor performance with large configuration files?</b></summary>
    <p>✅ Enable caching with <code>EnterpriseConfig</code> for sub-50ns target access times on frequently accessed values.</p>
</details>

<details>
  <summary><b>Configuration changes not reflected in running application?</b></summary>
    <p>✅ Use <code>Config::from_file_with_hot_reload()</code> for automatic configuration updates without restart.</p>
</details>

<details>
  <summary><b>Type conversion errors during value access?</b></summary>
    <p>✅ Use the safe accessor methods like <code>as_string_or("default")</code> or enable validation with custom rules.</p>
</details>

---

<br>

### **Version Compatibility**
- **Rust**: 1.82+ (currently); MSRV will be lowered to **1.75** in `v1.0.0` per portfolio standard
- **Edition**: 2021 (currently); will move to **2024** in `v1.0.0`
- **MSRV Policy**: Once `v1.0.0` ships, MSRV is guaranteed within the last 12 stable Rust releases
- **API Stability**: Pre-1.0 — expect refinements. `v1.0.0` will freeze the public API.
- **Feature Flags**: All optional features maintain independent compatibility


### **Development Setup**
```bash
git clone https://github.com/jamesgober/config-lib.git
cd config-lib
cargo test --all-features  # Run comprehensive test suite
cargo bench               # Performance benchmarks
cargo clippy              # Lint checks (should show zero warnings)
```


## **Contributing**
We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details.

<hr>
<br>

<!-- LICENSE
############################################# -->
<div id="license">
    <h2>License</h2>
    <p><b>config-lib</b> is <b>dual-licensed</b> under either of:</p>
    <ul>
        <li><b>Apache License, Version 2.0</b> — see <a href="./LICENSE-APACHE" title="Apache-2.0 License">LICENSE-APACHE</a> or <a href="http://www.apache.org/licenses/LICENSE-2.0" title="Apache-2.0 License" target="_blank">apache.org/licenses/LICENSE-2.0</a></li>
        <li><b>MIT License</b> — see <a href="./LICENSE-MIT" title="MIT License">LICENSE-MIT</a> or <a href="https://opensource.org/licenses/MIT" title="MIT License" target="_blank">opensource.org/licenses/MIT</a></li>
    </ul>
    <p>at your option.</p>
    <br>
    <h3>Contribution</h3>
    <p>Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in <b>config-lib</b> by you, as defined in the <b>Apache-2.0 license</b>, shall be <b>dual-licensed</b> as above, without any additional terms or conditions.</p>
    <p>Unless required by applicable law or agreed to in writing, software distributed under the Licenses is distributed on an <b>"AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND</b>, either express or implied.</p>
</div>

<br>

<!-- FOOT COPYRIGHT
################################################# -->
<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>JAMES GOBER.</strong></sup>
</div>
