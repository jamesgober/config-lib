<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br>
    <b>config-lib</b>
    <br>
    <sub><sup>DOCUMENTATION INDEX</sup></sub>
</h1>

<p align="center">
    <b>Enterprise-Grade Multi-Format Configuration Library</b>
    <br>
    <i>Complete documentation for production-ready configuration management</i>
</p>

<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>DOCS</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="./API.md" title="API Reference"><b>API</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./GUIDELINES.md" title="Developer Guidelines"><b>GUIDELINES</b></a>
    </sup>
</div>

<br>

Welcome to the **config-lib** documentation hub. This comprehensive documentation covers everything you need to know about using config-lib in production environments, from basic usage to advanced enterprise features.

---

## 📚 **Core Documentation**

### **Getting Started**
- **[Project Overview](../README.md)** - Main project documentation with features and examples
- **[Installation Guide](../README.md#installation)** - Setup instructions and feature flags
- **[Quick Start Examples](../README.md#quick-start)** - Basic usage patterns and common scenarios

### **API Reference**
- **[Complete API Documentation](./API.md)** - Comprehensive API reference with examples
  - [Core Functions](./API.md#core-functions) - `parse()`, `parse_file()`, `validate()`
  - [Config API](./API.md#config-api) - High-level configuration management
  - [ConfigBuilder API](./API.md#configbuilder-api) - Fluent configuration building
  - [Value API](./API.md#value-api) - Type system and conversions
  - [EnterpriseConfig API](./API.md#enterpriseconfig-api) - Performance and caching
  - [Feature-Specific APIs](./API.md#advanced-features) - Hot reload, audit, validation

### **Development**
- **[Developer Guidelines](./GUIDELINES.md)** - Best practices, coding standards, and contribution guidelines
- **[Change History](../CHANGELOG.md)** - Version history and breaking changes

---

## 🗂️ **Format Support**

config-lib supports **8 configuration formats** with consistent API access:

### **Built-in Formats** *(always available)*
- **[CONF](#conf-format)** - Standard `.conf` files with key=value syntax
- **[INI](#ini-format)** - INI files with sections and comments
- **[Properties](#properties-format)** - Java `.properties` files with Unicode support

### **Optional Formats** *(feature flags required)*
- **[JSON](#json-format)** - JSON with edit capabilities (feature: `json`)
- **[XML](#xml-format)** - Zero-copy XML parsing (feature: `xml`)
- **[HCL](#hcl-format)** - HashiCorp Configuration Language (feature: `hcl`)
- **[TOML](#toml-format)** - TOML with format preservation (feature: `toml`)
- **[NOML](#noml-format)** - Advanced NOML with dynamic features (feature: `noml`)

---

## ⚡ **Enterprise Features**

### **Performance & Caching**
- **[Sub-50ns Access](./API.md#enterprise-performance)** - 24.9ns average cached access
- **[Multi-Tier Caching](./API.md#enterprise-performance)** - Fast cache + main cache system
- **[Performance Monitoring](./API.md#enterprise-monitoring)** - Cache statistics and metrics

### **Production Features**
- **[Hot Reloading](./API.md#hot-reload-api)** - Zero-downtime configuration updates
- **[Audit Logging](./API.md#audit-api)** - Comprehensive operation logging for compliance
- **[Environment Overrides](./API.md#env-override-api)** - Smart environment variable system
- **[Schema Validation](./API.md#schema-api)** - Type safety and validation rules
- **[Async Operations](./API.md#async-api)** - Non-blocking file operations

### **Safety & Reliability**
- **Zero Unsafe Code** - Comprehensive error handling without panics
- **Thread Safety** - Poison-resistant locking with graceful failure recovery
- **Format Preservation** - Maintains comments, whitespace, and original formatting
- **Type Safety** - Rich conversion system with automatic type detection

---

## 🚀 **Quick Navigation**

### **By Use Case**
- **Web Applications** → [Environment Overrides](./API.md#env-override-api) + [JSON Support](./API.md#feature-flags)
- **DevOps Tools** → [HCL Integration](./API.md#feature-flags) + [Hot Reloading](./API.md#hot-reload-api)
- **Enterprise Systems** → [XML Support](./API.md#feature-flags) + [Audit Logging](./API.md#audit-api)
- **Microservices** → [Multi-Format](./API.md#core-functions) + [Async Operations](./API.md#async-api)

### **By Experience Level**
- **New Users** → Start with [Project Overview](../README.md) and [Quick Start](../README.md#quick-start)
- **Integrating** → See [API Reference](./API.md) and [Examples](../examples/)
- **Advanced Users** → Check [Enterprise Features](./API.md#enterpriseconfig-api) and [Guidelines](./GUIDELINES.md)
- **Contributors** → Read [Developer Guidelines](./GUIDELINES.md) and [Examples](../examples/)

---

## 📖 **Learning Resources**

### **Examples & Tutorials**
- **[Examples Directory](../examples/)** - 20+ comprehensive examples covering all features
  - [Basic Usage](../examples/basic.rs) - Getting started
  - [Multi-Format](../examples/multi_format.rs) - Working with different formats
  - [Enterprise Demo](../examples/enterprise_demo.rs) - Performance features
  - [Hot Reloading](../examples/hot_reload_demo.rs) - Dynamic updates
  - [Validation](../examples/validation_demo.rs) - Schema validation

### **Performance & Benchmarks**
- **[Benchmark Suite](../benches/)** - Performance testing and comparisons
  - [Parser Benchmarks](../benches/parser_benchmarks.rs) - Format parsing performance
  - [Enterprise Benchmarks](../benches/enterprise_benchmarks.rs) - Caching performance

### **Testing & Quality**
- **[Integration Tests](../tests/)** - Comprehensive test coverage
- **[Continuous Integration](../.github/workflows/)** - Automated testing and quality checks

---

## 🔧 **Technical Specifications**

### **System Requirements**
- **Rust Version**: 1.82+ (2021 edition)
- **Platform Support**: Linux, macOS, Windows
- **Memory**: Optimized for minimal allocations
- **Dependencies**: See [`Cargo.toml`](../Cargo.toml) for complete list

### **Feature Flags**
| Feature | Default | Description |
|---------|:-------:|-------------|
| Basic formats | ✅ | CONF, INI, Properties (always available) |
| `json` | ❌ | JSON format support |
| `xml` | ❌ | XML format support |
| `hcl` | ❌ | HashiCorp Configuration Language |
| `toml` | ❌ | TOML format with preservation |
| `noml` | ❌ | NOML format with dynamic features |
| `async` | ❌ | Async file operations |
| `validation` | ❌ | Schema validation system |
| `env-override` | ❌ | Environment variable overrides |
| `audit` | ❌ | Audit logging for compliance |
| `hot-reload` | ❌ | Zero-downtime configuration updates |

### **Performance Benchmarks**
- **Cached Access**: 24.9ns average (50% better than 50ns target)
- **Hot Cache**: 457ns average for frequently accessed values
- **First Access**: ~3µs (populates cache)
- **Thread Safety**: Maintains performance under concurrent load

---

## 🤝 **Community & Support**

### **Getting Help**
- **[GitHub Issues](https://github.com/jamesgober/config-lib/issues)** - Bug reports and feature requests
- **[GitHub Discussions](https://github.com/jamesgober/config-lib/discussions)** - Questions and community support
- **[API Documentation](https://docs.rs/config-lib)** - Online API reference
- **[Examples](../examples/)** - Practical usage examples

### **Contributing**
- **[Developer Guidelines](./GUIDELINES.md)** - Coding standards and best practices
- **[Contributing Guide](../CONTRIBUTING.md)** - How to contribute (if present)
- **[Issue Templates](../.github/ISSUE_TEMPLATE/)** - Bug reports and feature requests
- **[Code of Conduct](../CODE_OF_CONDUCT.md)** - Community standards (if present)

---

## 📄 **Format Specifications**

<h3 id="conf-format">CONF Format</h3>

Standard configuration format with key=value pairs and section support.

```conf
# Comments start with #
app_name = "MyApplication"
server_port = 8080
debug_mode = true

[database]
host = "localhost"
port = 5432
```

<h3 id="ini-format">INI Format</h3>

Traditional INI files with sections, comments, and type detection.

```ini
; Comments with semicolon
[server]
host = localhost
port = 8080

[database]
url = postgres://localhost/mydb
pool_size = 10
```

<h3 id="properties-format">Properties Format</h3>

Java-style properties files with Unicode and escaping support.

```properties
# Java properties format
app.name=MyApplication
server.port=8080
database.url=jdbc:postgresql://localhost/mydb
```

<h3 id="json-format">JSON Format</h3>

Standard JSON with edit capabilities and serialization support.

```json
{
  "app": {
    "name": "MyApplication",
    "version": "1.0.0"
  },
  "server": {
    "port": 8080,
    "host": "localhost"
  }
}
```

<h3 id="xml-format">XML Format</h3>

XML configuration with zero-copy parsing via quick-xml.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<config>
    <app name="MyApplication" version="1.0.0"/>
    <server port="8080" host="localhost"/>
    <database url="postgres://localhost/mydb"/>
</config>
```

<h3 id="hcl-format">HCL Format</h3>

HashiCorp Configuration Language for DevOps workflows.

```hcl
app "myapp" {
  name    = "MyApplication"
  version = "1.0.0"
}

server {
  port = 8080
  host = "localhost"
}
```

<h3 id="toml-format">TOML Format</h3>

TOML format with complete format preservation.

```toml
[app]
name = "MyApplication"
version = "1.0.0"

[server]
port = 8080
host = "localhost"

[database]
url = "postgres://localhost/mydb"
pool_size = 10
```

<h3 id="noml-format">NOML Format</h3>

Advanced NOML with dynamic features and variable interpolation.

```noml
app = {
    name = "MyApplication"
    version = env("APP_VERSION", "1.0.0")
}

server = {
    port = 8080
    host = "localhost"
    url = "http://${server.host}:${server.port}"
}
```

---

<div align="center">
    <h2></h2>
    <b>config-lib Documentation v0.9.0</b><br>
    <sub>Enterprise-grade configuration management for Rust</sub>
    <br><br>
    <sup>COPYRIGHT © 2025 <strong>JAMES GOBER</strong> • APACHE 2.0 LICENSE</sup>
</div>
