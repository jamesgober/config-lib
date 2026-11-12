<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br><b>config-lib</b>
    <br><sub><sup>RUST CONFIGURATION LIBRARY</sup></sub>
</h1>

<p align="center">
    <a href="https://crates.io/crates/config-lib"><img src="https://img.shields.io/crates/v/config-lib.svg" alt="Crates.io"></a>
    <a href="https://crates.io/crates/config-lib" alt="Download"><img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/config-lib?color=%230099ff"></a>
    <a href="https://docs.rs/config-lib"><img src="https://docs.rs/config-lib/badge.svg" alt="Documentation"></a>
    <a href="https://github.com/jamesgober/config-lib/actions"><img alt="GitHub CI" src="https://github.com/jamesgober/config-lib/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md" title="MSRV"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.75%2B-blue"></a>
</p>

<p align="center">
    <b>Enterprise-Grade Multi-Format Configuration Library</b>
    <br>
    <i>High-performance configuration management with advanced caching, validation, and format preservation</i>
</p>

<p>
    <strong>config-lib</strong> is a high-performance, enterprise-grade Rust configuration library that provides unified access to 8 different configuration formats through a single, consistent API. Built from the ground up for production environments, it offers advanced features like sub-50ns cached access, hot reloading, audit logging, and comprehensive type safety.
</p>

<p>
    Unlike traditional configuration libraries that focus on a single format, <b>config-lib</b> provides seamless multi-format support with automatic format detection, allowing you to mix and match configuration sources (<b>JSON</b> for APIs, <b>TOML</b> for Rust projects, <b>HCL</b> for DevOps, <b>XML</b> for enterprise systems) while maintaining a unified programming interface. The library automatically handles type conversions, validation, and format preservation, making it ideal for complex applications that need to integrate with multiple configuration ecosystems.
</p>

<p>
    With its enterprise-focused architecture, <b>config-lib</b> delivers production-grade features including comprehensive error handling with zero unsafe code, multi-tier caching with lock-free read paths, environment variable overrides with smart prefix matching, and built-in audit logging for compliance requirements. The library is designed for high-throughput applications, targeting sub-50ns cached access times on hot paths and supporting <b>hot reloading</b> without service interruption.
</p>

## Features
### **Multi-Format Support**
- **CONF** - Built-in parser for standard .conf files (default)
- **INI** - Full INI file parsing with sections, comments, and data type detection  
- **JSON** - JSON format with edit capabilities and serialization (feature: `jsonx` on jsonx-lib, strict and dependency-free; or `json` on serde_json)
- **XML** - Zero-copy XML parsing with quick-xml for Java/.NET environments
- **HCL** - The static subset of the HashiCorp Configuration Language (attributes, labeled blocks, lists, objects, comments); expressions are refused
- **Properties** - Complete Java .properties file parsing with Unicode and escaping
- **NOML** - Advanced configuration with dynamic features (feature: `noml`)
- **TOML** - TOML format with format preservation (feature: `toml`)

### Enterprise Performance
- **Sub-50ns Cache Access Target** - Multi-tier caching designed for sub-50ns reads on hot paths
- **Lock-Free Notification Dispatch** - `on_change` handlers fire inline from the reloader thread via `ArcSwap<Vec<Handler>>`; one atomic load + N closure calls, no channel allocation, and no lock held (since v1.4.0), so a handler can read the configuration
- **Zero-Copy Parsing** - Minimized allocations and string operations where possible
- **Lock-Free Read Paths** - Poison-resistant locking with graceful failure recovery
- **Hot Value Cache** - Ultra-fast access for frequently used values
- **Cache Hit Ratio Tracking** - Built-in performance statistics and monitoring

> Performance numbers, methodology, and tuning guidance live in [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md). The committed criterion harnesses in [`benches/`](benches/) produce the measured baselines.

### Production Features
- **Configuration Hot Reloading** - File watching with thread-safe Arc swapping
- **Audit Logging System** - Structured event logging with multiple sinks and severity filtering
- **Environment Variable Overrides** - Smart caching with prefix matching and type conversion
- **Schema Validation** - Trait-based validation system with feature gates
- **Format Preservation** - An unmodified NOML or TOML configuration saves with its comments and layout; CONF saves read back to the same values, without comments
- **Async Native** - Full async/await support throughout the API

### Reliability & Safety
- **Zero Unsafe Code** - All `unwrap()` calls eliminated, comprehensive error handling
- **Type Safety** - Rich type system with automatic conversions and validation
- **Reads only what you give it** - TOML never resolves NOML's `include` or `env()`; `Config::from_string_strict` refuses them in NOML too (since v1.3.0)
- **Bounded loads** - files are regular files of at most 16 MiB, NOML includes stay in the config's directory, `${...}` expansion is capped at 64 MiB, and errors never quote the input; `LoadOptions` sets the limits and strict mode for files and hot reload (since v1.4.0)
- **Typed reads that keep the file's value** - INI and environment values `1` and `0` are integers (they still read as booleans), so `get_or::<u16>` sees them
- **Enterprise Error Handling** - Production-ready error messages with context preservation
- **Comprehensive Testing** - Extensive unit, integration, and doc test coverage

<hr>

## Why Choose config-lib?

### Unified Multi-Format Support
Unlike single-format libraries, **config-lib** provides seamless access to 8 configuration formats through one consistent API. No need to learn different libraries for `TOML`, `JSON`, `XML`, and `HCL` - one API handles them all with automatic format detection.

### Enterprise-Grade Performance
Multi-tier caching with lock-free read paths is designed to deliver sub-50ns cached access on hot paths. Built for high-throughput applications with minimal performance overhead.

### Production-Ready Reliability
Zero unsafe code, comprehensive error handling, and poison-resistant locking ensure your configuration system won't crash your application. Extensive testing coverage validates edge cases and error conditions.

### Developer Experience First
Rich type system with automatic conversions, format preservation for round-trip editing, and detailed error messages with source location context. No more cryptic parsing errors or manual type casting.

### Advanced Enterprise Features
Hot reloading without service interruption, structured audit logging for compliance, environment variable overrides with smart caching, and schema validation with custom rules - features typically requiring multiple libraries.

<hr>

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
config-lib = "1.4"

# For enhanced functionality, enable optional features:
config-lib = { version = "1.4", features = [
    "jsonx",          # JSON format support on jsonx-lib (strict, no dependencies; Rust 1.85+)
    "json",           # JSON format support on serde_json (used when `jsonx` is off)
    "xml",            # XML format support with quick-xml backend  
    "hcl",            # HashiCorp Configuration Language support
    "toml",           # TOML format support with preservation
    "noml",           # NOML format support with dynamic features
    "schema",         # Schema validation (`Schema`, `SchemaBuilder`)
    "validation",     # Validation rules (`validation` module)
    "async",          # Async file loading on tokio
    "env-override",   # Environment variable override system
] }
```

**Feature Recommendations**:
- **Minimal**: Use default features for CONF/INI/Properties support
- **Web Applications**: Add `"jsonx"` (or `"json"`), `"env-override"`, `"validation"`
- **DevOps Tools**: Add `"hcl"`, `"toml"`, `"async"`
- **Enterprise Systems**: Add `"xml"`, `"validation"`, `"env-override"`
- **Full Featured**: Include all features for maximum flexibility

<hr>

## Quick Start

```rust
use config_lib::Config;

// Parse a configuration; name the format. With `None`, text is read as
// CONF unless it clearly starts as XML or JSON or has `;` comments (INI).
let mut config = Config::from_string(r#"
[database]
host = "localhost"
port = 5432
username = "admin"

[app]
name = "MyApp"
debug = true
"#, Some("conf"))?;

// Typed reads with a default: the default's type picks the conversion.
let host: String = config.get_or("database.host", "127.0.0.1".to_string());
let port: u16 = config.get_or("database.port", 5432);
let debug = config.get_or("app.debug", false);
let workers: usize = config.get_or("app.workers", 4); // missing, so 4

// Reads that must succeed: `get` returns `Option<&Value>`.
let name = config
    .get("app.name")
    .ok_or_else(|| config_lib::Error::key_not_found("app.name"))?
    .as_string()?
    .to_owned();

// Modify configuration
config.set("database.port", 5433)?;
config.set("app.version", "1.0.0")?;

println!("{name}: connecting to {host}:{port} (debug {debug}, {workers} workers)");
Ok::<(), config_lib::Error>(())
```

`get_or` works for `bool`, `String`, `f64`, and every integer type from
`i8`/`u8` to `i64`/`u64`, `isize`, and `usize`. Integer reads are
range-checked: a value that does not fit the requested type (say
`70000` read as `u16`, or `8080.5` read as any integer) returns the
default. The same conversions are available directly as
`TryFrom<Value>` and `TryFrom<&Value>`.

### Multi-Format Support

```rust
use config_lib::Config;

// The same calls work for every format; name it.
let config = Config::from_string(r#"
[server]
port = 8080
host = "localhost"
"#, Some("conf"))?;

let port: u16 = config.get_or("server.port", 80);
let timeout: u64 = config.get_or("server.timeout", 30);
let name: String = config.get_or("app.name", "DefaultApp".to_string());
assert_eq!((port, timeout, name.as_str()), (8080, 30, "DefaultApp"));

// Check existence
if config.contains_key("server.ssl") {
    println!("SSL configuration found");
}
Ok::<(), config_lib::Error>(())
```

Format names are `"conf"`, `"ini"`, `"properties"`, `"json"`, `"xml"`,
`"hcl"`, `"toml"`, and `"noml"`; each format other than CONF, INI, and
properties needs its Cargo feature.

### Read-only mode and forward-compatible options

`ConfigOptions` is the structured place to express the small set of
behaviors that should not be enabled by default — making a `Config`
read-only, sizing the cache, etc. The struct is `#[non_exhaustive]`
so v1.x MINOR releases can add new knobs without breaking SemVer;
callers go through the consuming builder methods.

```rust
use config_lib::{Config, ConfigOptions};

// Default options — caching on, writes allowed.
let _cfg = Config::with_options(ConfigOptions::default());

// Read-only configuration for a hot path that must never be mutated.
let opts = ConfigOptions::new().read_only(true);
let mut locked = Config::with_options(opts);
assert!(locked.set("foo", "bar").is_err());   // rejected
```

### Default Configuration Settings

**config-lib** offers three ways to supply defaults for keys that are
missing from a configuration file.

#### Method 1: Inline defaults with `get_or` (recommended)

The default sits at the read, and its type picks the conversion.

```rust,no_run
use config_lib::Config;

struct AppConfig {
    name: String,
    port: u16,
    debug: bool,
    timeout: u64,
    db_host: String,
    db_port: u16,
    db_pool_size: u32,
    analytics_enabled: bool,
    cache_enabled: bool,
}

let config = Config::from_file("app.conf")?;

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
};
Ok::<(), config_lib::Error>(())
```

#### Method 2: A defaults table

`set_default` fills a table that `get_or_default` falls back to when the
main value tree has no entry for the path.

```rust
use config_lib::{Config, Value};

let config = Config::from_string("[server]\nport = 9000", Some("conf"))?;

config.set_default("server.port", 8080)?;
config.set_default("server.workers", 4)?;
config.set_default("logging.level", "info")?;

// File value wins over the default.
assert_eq!(config.get_or_default("server.port"), Some(Value::Integer(9000)));

// Missing from the file, so the default.
assert_eq!(config.get_or_default("logging.level"), Some(Value::from("info")));
Ok::<(), config_lib::Error>(())
```

#### Method 3: The `key` accessor

```rust
use config_lib::Config;

let config = Config::from_string("[cache]\nttl = 600", Some("conf"))?;

let cache_ttl = config.key("cache.ttl").as_integer_or(3600);        // i64
let environment = config.key("app.environment").as_string_or("development");
let verbose = config.key("app.verbose").as_bool_or(false);
assert_eq!((cache_ttl, environment.as_str(), verbose), (600, "development", false));
Ok::<(), config_lib::Error>(())
```

#### Best Practices for Default Configuration

1. **Use Sensible Defaults**: Choose defaults that work for most use cases
2. **Document Defaults**: Keep defaults in sync with documentation
3. **Environment-Specific**: Use different defaults for dev/staging/production
4. **Type Safety**: Read into the narrowest type that fits (`u16` for a port), so an out-of-range value is not accepted
5. **Validation**: Validate both defaults and overrides

### Environment Variable Integration

Needs the `env-override` feature.

```rust,no_run
use config_lib::env_override::EnvOverrideSystem;
use config_lib::Config;

let config = Config::from_file("app.conf")?;

// MYAPP_DATABASE_HOST overrides database.host, MYAPP_SERVER_PORT
// overrides server.port, and so on.
let overrides = EnvOverrideSystem::with_prefix("MYAPP_");
let config = Config::from(overrides.apply_overrides(config.as_value().clone())?);

let host: String = config.get_or("database.host", "localhost".to_string());
let port: u16 = config.get_or("server.port", 8080);
Ok::<(), config_lib::Error>(())
```

### Configuration Validation

Needs the `validation` feature. Rules run against each top-level value.

```rust
use config_lib::validation::{RangeValidator, TypeValidator, ValidationRuleSet, ValueType};
use config_lib::Config;

let mut config = Config::from_string("server_port = 8080\nmax_connections = 100", Some("conf"))?;

config.set_validation_rules(
    ValidationRuleSet::new()
        .add_rule(TypeValidator::new(ValueType::Integer))
        .add_rule(RangeValidator::new(Some(1.0), Some(65535.0))),
);

let errors = config.validate()?;
assert!(errors.is_empty(), "{errors:?}");
Ok::<(), config_lib::Error>(())
```

### Hot Reloading & Audit Logging

```rust,no_run
use config_lib::audit::{self, AuditLogger, AuditSeverity, ConsoleSink};
use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};

// Process-wide audit logger
audit::init_audit_logger(
    AuditLogger::new().add_sink(Box::new(ConsoleSink::new(AuditSeverity::Info))),
);

// Hot reloading: take the shared handle before `start_watching` consumes the watcher.
// A reload that fails to parse, holds no values, or fails the validator keeps
// the current configuration. Handlers run after the lock is released, so they
// may read the configuration.
let hot = HotReloadConfig::from_file("app.conf")?.with_validator(|config| {
    if config.contains_key("server.port") {
        Ok(())
    } else {
        Err(config_lib::Error::validation("server.port is required"))
    }
});
let shared = hot.config();
let reader = hot.config();
let _sub = hot.on_change(move |event| {
    if let ConfigChangeEvent::Reloaded { path, .. } = event {
        let port: u16 = reader.read().map(|c| c.get_or("server.port", 8080)).unwrap_or(8080);
        println!("reloaded {}: port {port}", path.display());
    }
});
let _handle = hot.start_watching();

let port: u16 = shared
    .read()
    .map(|config| config.get_or("server.port", 8080))
    .unwrap_or(8080);
Ok::<(), config_lib::Error>(())
```

### Multi-File Configuration

```rust,no_run
use config_lib::Config;

// Later files override earlier ones, key by key.
let mut config = Config::from_file("default.conf")?;
config.merge(&Config::from_file("environment.conf")?)?;
config.merge(&Config::from_file("local.conf")?)?;

let database_url: String = config.get_or("database.url", String::new());
let debug_mode = config.get_or("debug", false);
Ok::<(), config_lib::Error>(())
```

## Documentation & Resources

### Documentation
- **[Documentation Index](docs/README.md)** - Complete documentation hub and navigation
- **[API Reference](docs/API.md)** - Comprehensive API documentation with examples
- **[Valid Formats](docs/FORMATS.md)** - Detailed format specifications and examples
- **[Code Guidelines](docs/GUIDELINES.md)** - Development standards and best practices

### External Resources
- **[API Documentation](https://docs.rs/config-lib)** - Complete API reference with examples
- **[Crate Registry](https://crates.io/crates/config-lib)** - Official crate distribution
- **[Examples Directory](examples/)** - 9 runnable examples
- **[Performance Benchmarks](benches/)** - Detailed performance analysis and comparisons
- **[NOML Language](https://github.com/noml-lang/noml)** - NOML language specification and usage.

### Getting Started Guides
- **[Quick Start Guide](examples/basic.rs)** - Basic configuration loading and access
- **[Multi-Format Demo](examples/multi_format.rs)** - Working with different configuration formats
- **[Enterprise Features](examples/enterprise_demo.rs)** - Advanced caching and performance
- **[Hot Reloading](examples/hot_reload_demo.rs)** - Dynamic configuration updates
- **[Validation System](examples/validation_demo.rs)** - Schema validation and type checking

### Common Use Cases
- **Web Applications**: Environment overrides, JSON/TOML configs
- **DevOps Tools**: [HCL integration](examples/hcl_demo.rs), audit logging, hot reloading
- **Enterprise Systems**: [XML support](examples/xml_demo.rs), validation, caching
- **Microservices**: Multi-format support, environment-based configuration

<hr>

### Version Compatibility
- **Rust**: `1.75+` for the default feature set; `1.79+` with the `xml` feature (`quick-xml` 0.41); `1.82+` with the `noml` or `toml` features (`noml = "=0.9.2"`); `1.85+` with the `jsonx` feature (`jsonx-lib = "=0.5.1"`)
- **Edition**: `2021`
- **MSRV Policy**: MINOR releases may bump MSRV within the last-12-stable-Rust-versions window; PATCH releases never bump MSRV
- **API Stability**: v1.x public API is frozen per [`docs/STABILITY-1.0.md`](docs/STABILITY-1.0.md). Growth-likely enums (`Error`, `ConfigChangeEvent`, etc.) are `#[non_exhaustive]`; new variants land in MINOR releases without breaking SemVer
- **Feature Flags**: All optional features maintain independent compatibility

<br>

### Development Setup
```bash
git clone https://github.com/jamesgober/config-lib.git
cd config-lib
cargo test --all-features  # Run comprehensive test suite
cargo bench               # Performance benchmarks
cargo clippy              # Lint checks (should show zero warnings)
```

<br>

## Contributing
We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details.

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

<!-- FOOT COPYRIGHT
################################################# -->
<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>JAMES GOBER.</strong></sup>
</div>
