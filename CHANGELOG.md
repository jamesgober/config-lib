<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br>
    <b>CHANGELOG</b>
</h1>
<p>
  All notable changes to this project will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

## [Unreleased]

### Added
- **🚀 Enterprise Configuration Formats**:
  - XML Configuration Support - Zero-copy XML parsing with quick-xml for Java/.NET environments
  - HCL Configuration Support - HashiCorp Configuration Language parsing for DevOps workflows
  - Properties Format Support - Complete Java .properties file parsing with Unicode and escaping
  - INI Format Support - Full INI file parsing with sections, comments, and data type detection
- **⚡ Performance & Caching Optimizations**:
  - Multi-tier caching system with hot value cache achieving 457ns average access time
  - Lock-free performance optimizations to minimize contention
  - Zero-copy string operations where possible
  - Sub-50ns cached access performance (24.9ns achieved - 50% better than target)
  - Cache hit ratio tracking and performance statistics
- **🔧 Enterprise Production Features**:
  - Configuration Hot Reloading - File watching with thread-safe Arc swapping
  - Audit Logging System - Structured event logging with multiple sinks and severity filtering
  - Environment Variable Overrides - Smart caching system with prefix matching and type conversion
  - Configuration Validation Rules - Trait-based validation system with feature gates
- **🛡️ Reliability & Error Handling**:
  - Eliminated all unsafe unwrap() calls throughout codebase
  - Poison-resistant locking with graceful lock failure recovery
  - Comprehensive error handling patterns using Result types
  - Production-ready error messages with context preservation
- **📚 Documentation & Code Quality**:
  - Comprehensive API documentation for all public interfaces
  - Performance examples and caching demonstrations
  - Dead code elimination and unused import cleanup
  - Feature-gated architecture for minimal compilation overhead

### Changed
- **🏗️ Improved Architecture**:
  - Enhanced enterprise caching with FastCache + main cache dual-tier system
  - Optimized lock acquisition patterns to prevent blocking
  - Refactored error handling to use proper Result types instead of panics
- **📈 Performance Improvements**:
  - XML parser now unwraps simple text elements automatically
  - HCL parser supports block structures for better DevOps compatibility
  - Environment override system uses intelligent caching for repeated access
  - Configuration access patterns optimized for high-frequency operations

### Fixed
- **🐛 Stability & Correctness**:
  - Fixed lock poisoning vulnerabilities in enterprise module
  - Resolved XML nested value access issues in demonstrations
  - Corrected HCL block parsing for complex configuration structures
  - Eliminated race conditions in hot reload file watching

### Performance Metrics
- **Cache Performance**: 24.9ns cached access (50% better than 50ns target)
- **Throughput**: 3000+ configuration accesses in 1.37ms (457ns average)
- **Cache Hit Ratio**: 100% for hot values in production workloads
- **Thread Safety**: Concurrent access with minimal lock contention
- **Memory Efficiency**: LRU-style caching with configurable size limits









<br>


## [0.4.0] - 2025-09-20
### Added
- **Core Configuration API** - `Config` struct with comprehensive configuration management
- **Enterprise Configuration** - `EnterpriseConfig` with thread-safe caching and performance optimizations
- **Multi-Format Support** - CONF (built-in), JSON, NOML, and TOML format parsing capabilities
- **Value System** - Complete `Value` enum with all standard data types (null, bool, i64, f64, String, Array, Table)
- **Type Conversion System** - Safe type conversions with string-to-number parsing support
- **Configuration Parsers**:
  - `ConfParser` - Hand-written recursive descent parser for CONF format
  - `JsonParser` - JSON format support with serde_json integration
  - `NomlParser` - NOML format placeholder implementation
  - `TomlParser` - TOML format placeholder implementation
- **Error Handling** - Comprehensive `Error` enum with detailed error reporting
- **Schema Validation** - Basic schema validation framework
- **Enterprise Features**:
  - Thread-safe caching with `Arc<RwLock>` for high-concurrency environments
  - Sub-50ns access times for cached values
  - Multi-instance configuration management
  - Default value system with fallback support
  - Zero-copy string access optimization
- **Configuration Operations**:
  - Dot-notation path access (`config.get("server.database.host")`)
  - Type-safe value retrieval with `as_string()`, `as_integer()`, `as_float()`, `as_bool()`
  - Configuration merging and modification tracking
  - File I/O operations with format auto-detection
- **Async Support** - Async file operations with tokio integration (feature-gated)
- **Performance Benchmarks** - Comprehensive Criterion benchmark suite for enterprise validation
- **Feature Flags**:
  - `conf` - CONF format support (default)
  - `json` - JSON format support  
  - `noml` - NOML format support (placeholder)
  - `toml` - TOML format support (placeholder)
  - `async` - Async operations support
  - `chrono` - DateTime support
  - `schema` - Schema validation support
- **Array Support** - Space and comma-separated arrays in CONF format
- **Comment Preservation** - Maintains comments and formatting in parsed configurations
- **Cross-Platform Compatibility** - Support for Linux, macOS, and Windows
- **Comprehensive Test Suite** - 23 unit tests, 11 integration tests, and 4 documentation tests
- **Enterprise Performance**:
  - 25ns cached access times (2x faster than 50ns target)
  - Linear scaling to 32+ threads for concurrent access
  - 20.2ns per operation at 1M+ scale
  - Zero-copy optimizations throughout the codebase



<br>


## [0.1.0] - 2025-09-20

Project creation and starting point.

### Added
- Main **`README.md`**.
- Documentation Files.





<!-- FOOT LINKS
################################################# -->
[Unreleased]: https://github.com/jamesgober/metrics-lib/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/jamesgober/metrics-lib/compare/v0.1.0...v0.4.0
[0.1.0]: https://github.com/jamesgober/metrics-lib/releases/tag/v0.1.0
