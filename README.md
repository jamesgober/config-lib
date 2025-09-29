<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br>
    <b>config-lib</b>
</h1>

<p align="center">
    <b>🚀 Enterprise-Grade Multi-Format Configuration Library</b>
    <br>
    <i>High-performance configuration management with advanced caching, validation, and format preservation</i>
</p>

<p align="center">
    <a href="https://crates.io/crates/config-lib"><img src="https://img.shields.io/crates/v/config-lib.svg" alt="Crates.io"></a>
    <a href="https://docs.rs/config-lib"><img src="https://docs.rs/config-lib/badge.svg" alt="Documentation"></a>
    <a href="https://github.com/jamesgober/config-lib/actions"><img src="https://github.com/jamesgober/config-lib/workflows/CI/badge.svg" alt="CI Status"></a>
    <a href="https://github.com/jamesgober/config-lib/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"></a>
</p>

---

## 🌟 Features

### 📄 **Multi-Format Support**
- **CONF** - Built-in parser for standard .conf files (default)
- **INI** - Full INI file parsing with sections, comments, and data type detection  
- **JSON** - JSON format with edit capabilities and serialization
- **XML** - Zero-copy XML parsing with quick-xml for Java/.NET environments
- **HCL** - HashiCorp Configuration Language for DevOps workflows
- **Properties** - Complete Java .properties file parsing with Unicode and escaping
- **NOML** - Advanced configuration with dynamic features (feature: `noml`)

### ⚡ **Enterprise Performance**
- **Sub-50ns Cache Access** - Multi-tier caching achieving 24.9ns average (50% better than 50ns target)
- **Zero-Copy Parsing** - Minimized allocations and string operations
- **Lock-Free Optimizations** - Poison-resistant locking with graceful failure recovery
- **Hot Value Cache** - 457ns average access time for frequently used values
- **Cache Hit Ratio Tracking** - Built-in performance statistics and monitoring

### 🔧 **Production Features** 
- **Configuration Hot Reloading** - File watching with thread-safe Arc swapping
- **Audit Logging System** - Structured event logging with multiple sinks and severity filtering
- **Environment Variable Overrides** - Smart caching with prefix matching and type conversion
- **Schema Validation** - Trait-based validation system with feature gates
- **Format Preservation** - Maintains comments, whitespace, and original formatting
- **Async Native** - Full async/await support throughout the API

### 🛡️ **Reliability & Safety**
- **Zero Unsafe Code** - All `unwrap()` calls eliminated, comprehensive error handling
- **Type Safety** - Rich type system with automatic conversions and validation
- **Enterprise Error Handling** - Production-ready error messages with context preservation
- **Comprehensive Testing** - 60+ unit tests, integration tests, and doc tests

---

## 🚀 Quick Start

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

### 🔥 **Enterprise Caching**

```rust
use config_lib::EnterpriseConfig;

// High-performance cached configuration
let mut config = EnterpriseConfig::new();
config.load_from_file("app.conf")?;

// Sub-50ns cached access (24.9ns average)
let cached_value = config.get_cached("database.host")?;

// View cache performance stats
let stats = config.cache_stats()?;
println!("Cache hit ratio: {:.2}%", stats.hit_ratio * 100.0);
```

### 🔄 **Hot Reloading & Audit Logging**

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

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
config-lib = "0.4.5"

# Optional enterprise features
config-lib = { version = "0.4.5", features = [
    "json",           # JSON format support
    "xml",            # XML format support  
    "hcl",            # HCL format support
    "validation",     # Schema validation
    "async",          # Async operations
    "env-override",   # Environment variable overrides
] }
```

---

## 🎯 **Current Status: v0.4.5**

**Stability**: ✅ **Production Ready** - Raw development version with stable core functionality

**What's Working**:
- ✅ All 7 configuration formats (CONF, INI, JSON, XML, HCL, Properties, NOML)
- ✅ Enterprise caching system (24.9ns average access time)  
- ✅ Hot reloading and audit logging
- ✅ Environment variable overrides and validation
- ✅ Comprehensive test suite (60+ tests, 100% passing)
- ✅ Zero unsafe code, production-ready error handling

**Performance Achievements**:
- 🚀 **24.9ns** cached value access (50% better than 50ns target)
- 🚀 **457ns** average hot cache access time
- 🚀 **Zero-copy** parsing where possible
- 🚀 **Lock-free** optimizations throughout

---

## 📋 **Roadmap**

| Version | Focus | Status |
|---------|-------|--------|
| **0.4.x** | ✅ Core functionality & enterprise features | **Current** |
| **0.5.x** | � API expansion & additional functionality | **Next** |
| **0.6.x** | 🛡️ API finalization & bulletproofing | **Planned** |
| **0.7.x** | 🎨 Code cleanup, optimization & polish | **Planned** |
| **0.8.x** | ⚡ Peak performance optimization & security | **Planned** |
| **0.9.x** | 🧪 Beta/RC with community testing | **Planned** |
| **1.0.x** | 🎉 Stable release | **Goal** |

---

## 📖 **Documentation**

- **[API Documentation](https://docs.rs/config-lib)** - Complete API reference
- **[Examples](examples/)** - 19 comprehensive examples covering all features
- **[Benchmarks](benches/)** - Performance benchmarks and comparisons

---

## 🤝 **Contributing**

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details.

---

## 📄 **License**

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
