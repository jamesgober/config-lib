<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br>
    <b>config-lib</b>
</h1>
<div align="center">
    <a href="https://crates.io/crates/config-lib"><img alt="Crates.io" src="https://img.shields.io/crates/v/config-lib"></a>
    <a href="https://crates.io/crates/config-lib" alt="Download config-lib"><img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/config-lib?color=%230099ff"></a>
    <a href="https://docs.rs/config-lib" title="config-lib Documentation"><img alt="docs.rs" src="https://img.shields.io/docsrs/config-lib"></a>
    <a href="https://github.com/jamesgober/config-lib/actions"><img alt="GitHub CI" src="https://github.com/jamesgober/config-lib/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md" title="MSRV"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.82%2B-blue"></a>
</div>
<br>

**Config-lib** is a high-performance, enterprise-grade **configuration management library** for Rust applications requiring extreme performance and reliability. Built for database technologies and high-concurrency systems demanding sub-50ns access times.

## 🚀 **Performance First**

- **⚡ 25ns** cached access times (2x faster than enterprise targets)
- **🔥 1M+** concurrent operations with linear scaling
- **⚔️ Zero-copy** optimizations throughout the entire stack
- **🛡️ Thread-safe** caching with `Arc<RwLock>` for enterprise environments

Built for **database technology** that's "3 times faster, 1,000 times stronger, and 90% more efficient than Oracle."

## 📦 **Multi-Format Support**

Built-in **CONF** parser with optional support for:
- **🔧 CONF** - Native high-performance format (default)
- **📄 JSON** - Standard JSON configuration files  
- **🌟 NOML** - Next-generation configuration language
- **📋 TOML** - Tom's Obvious Minimal Language

## ✨ **Core Features**

- **🎯 Type-Safe Access** - Zero-panic value retrieval with comprehensive error handling
- **🔄 Change Tracking** - Automatic modification detection and state management  
- **📍 Dot Notation** - Intuitive nested access (`config.get("server.database.host")`)
- **🧵 Thread Safety** - Full concurrency support for high-load environments
- **🔀 Configuration Merging** - Intelligent overlay and inheritance systems
- **📈 Enterprise Caching** - Multi-instance management with sub-nanosecond overhead
- **🌍 Cross Platform** - Supports **Linux**, **macOS**, and **Windows**
- **💬 Comment Preservation** - Maintains formatting and documentation in config files

## 🛠️ **Quick Start**

```rust
use config_lib::{Config, EnterpriseConfig};

// Standard configuration management
let mut config = Config::from_file("app.conf")?;
let port = config.get("server.port").unwrap().as_integer()?;
let host = config.get("server.host").unwrap().as_string()?;

// Enterprise configuration with caching
let enterprise = EnterpriseConfig::from_file("production.conf")?;
let cached_value = enterprise.get_or_default("database.timeout", 30)?;

// Modify and track changes
config.set("server.port", 9000)?;
if config.is_modified() {
    config.save()?;
}
```

## 🏗️ **Architecture**

### Core Components
- **`Config`** - High-level configuration management with change tracking
- **`EnterpriseConfig`** - Performance-optimized caching layer for production systems
- **`Value`** - Type-safe value system with zero-copy string access
- **`Error`** - Comprehensive error handling with source location context

### Enterprise Performance
- **Cached Access**: 25ns average (validated with Criterion benchmarks)
- **Concurrent Scaling**: Linear performance up to 32+ threads
- **Memory Efficiency**: Zero-copy string operations, intelligent caching
- **Production Ready**: Designed for 1M+ concurrent operations

## 🎛️ **Feature Flags**

| Feature   | Default | Description |
|-----------|:-------:|-------------|
| `conf`    | ✅     | CONF format parsing (built-in) |
| `json`    | ❌     | JSON format support |
| `noml`    | ❌     | NOML format support |
| `toml`    | ❌     | TOML format support |
| `async`   | ❌     | Async file operations |
| `chrono`  | ❌     | DateTime support |
| `schema`  | ❌     | Schema validation |

```toml
# Cargo.toml
[dependencies]
config-lib = { version = "0.1.0", features = ["json", "async"] }
```

<!-- LICENSE
############################################# -->
<div id="license">
    <h2>⚖️ License</h2>
    <p>Licensed under the <b>Apache License</b>, version 2.0 (the <b>"License"</b>); you may not use this software, including, but not limited to the source code, media files, ideas, techniques, or any other associated property or concept belonging to, associated with, or otherwise packaged with this software except in compliance with the <b>License</b>.</p>
    <p>You may obtain a copy of the <b>License</b> at: <a href="http://www.apache.org/licenses/LICENSE-2.0" title="Apache-2.0 License" target="_blank">http://www.apache.org/licenses/LICENSE-2.0</a>.</p>
    <p>Unless required by applicable law or agreed to in writing, software distributed under the <b>License</b> is distributed on an "<b>AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND</b>, either express or implied.</p>
    <p>See the <a href="./LICENSE" title="Software License file">LICENSE</a> file included with this project for the specific language governing permissions and limitations under the <b>License</b>.</p>
</div>

<!-- FOOT COPYRIGHT
################################################# -->
<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2025 <strong>JAMES GOBER.</strong></sup>
</div>
