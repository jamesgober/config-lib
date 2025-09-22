# config-lib

## config-lib - Multi-Format Configuration Library

A high-performance configuration management library supporting multiple formats
including CONF, NOML, TOML, and JSON with advanced features like format preservation,
async operations, and schema validation.

### Quick Start

```rust
use config_lib::Config;

// Parse any supported format automatically
let mut config = Config::from_string("port = 8080\nname = \"MyApp\"", None)?;

// Access values with type safety
let port = config.get("port").unwrap().as_integer()?;
let name = config.get("name").unwrap().as_string()?;

// Modify and save (preserves format and comments)
config.set("port", 9000)?;
```

### Supported Formats

- **CONF** - Built-in parser for standard .conf files (default)
- **NOML** - Advanced configuration with dynamic features (feature: `noml`)
- **TOML** - Standard TOML format with format preservation (feature: `toml`)
- **JSON** - JSON format with edit capabilities (feature: `json`)

### Features

- **🚀 High Performance** - Zero-copy parsing where possible
- **💾 Format Preservation** - Maintains comments, whitespace, and formatting
- **⚡ Async Native** - Full async/await support (feature: `async`)
- **🔍 Schema Validation** - Type safety and validation (feature: `schema`)
- **🌐 Cross Platform** - Linux, macOS, and Windows support
- **🔧 Type Safety** - Rich type system with automatic conversions

License: Apache-2.0
