//! # config-lib - Multi-Format Configuration Library
//!
//! A high-performance configuration management library supporting multiple formats
//! including CONF, NOML, TOML, and JSON with advanced features like format preservation,
//! async operations, and schema validation.
//!
//! ## Quick Start
//!
//! ```rust
//! use config_lib::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse any supported format automatically
//! let mut config = Config::from_string("port = 8080\nname = \"MyApp\"", None)?;
//! 
//! // Access values with type safety
//! let port = config.get("port").unwrap().as_integer()?;
//! let name = config.get("name").unwrap().as_string()?;
//!
//! // Modify and save (preserves format and comments)
//! config.set("port", 9000)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Supported Formats
//!
//! - **CONF** - Built-in parser for standard .conf files (default)
//! - **NOML** - Advanced configuration with dynamic features (feature: `noml`)
//! - **TOML** - Standard TOML format with format preservation (feature: `toml`)
//! - **JSON** - JSON format with edit capabilities (feature: `json`)
//!
//! ## Features
//!
//! - **🚀 High Performance** - Zero-copy parsing where possible
//! - **💾 Format Preservation** - Maintains comments, whitespace, and formatting
//! - **⚡ Async Native** - Full async/await support (feature: `async`)
//! - **🔍 Schema Validation** - Type safety and validation (feature: `schema`)
//! - **🌐 Cross Platform** - Linux, macOS, and Windows support
//! - **🔧 Type Safety** - Rich type system with automatic conversions

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
/// Enterprise-grade configuration management with advanced caching, performance optimizations,
/// and multi-instance support. Provides thread-safe caching with `Arc<RwLock>` for high-concurrency
/// environments and sub-50ns access times for cached values.
pub mod enterprise;  // Enterprise API with caching and performance
pub mod error;
pub mod parsers;
pub mod value;

#[cfg(feature = "schema")]
pub mod schema;

// Re-export main types for convenience
pub use config::Config;
pub use enterprise::{EnterpriseConfig, ConfigManager};
pub use error::{Error, Result};
pub use value::Value;

#[cfg(feature = "schema")]
pub use schema::{Schema, SchemaBuilder};

use std::path::Path;

/// Parse configuration from a string, auto-detecting format
///
/// This is the main entry point for parsing configuration data. The format
/// is automatically detected based on the content structure and syntax.
///
/// # Arguments
///
/// * `source` - Configuration text to parse
/// * `format` - Optional format hint (auto-detected if None)
///
/// # Returns
///
/// Returns a [`Value`] containing the parsed configuration data.
///
/// # Examples
///
/// ```rust
/// use config_lib::parse;
///
/// let config = parse(r#"
///     app_name = "my-service"
///     port = 8080
///     debug = true
///     
///     [database]
///     host = "localhost"
///     max_connections = 100
/// "#, None)?;
///
/// assert_eq!(config.get("app_name").unwrap().as_string().unwrap(), "my-service");
/// assert_eq!(config.get("database.host").unwrap().as_string().unwrap(), "localhost");
///
/// # Ok::<(), config_lib::Error>(())
/// ```
pub fn parse(source: &str, format: Option<&str>) -> Result<Value> {
    parsers::parse_string(source, format)
}

/// Parse configuration from a file, auto-detecting format from extension
///
/// Reads a configuration file from disk and automatically detects the format
/// based on the file extension (.conf, .toml, .json, .noml).
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// Returns a [`Value`] containing the parsed configuration data.
///
/// # Examples
///
/// ```rust,no_run
/// use config_lib::parse_file;
///
/// let config = parse_file("app.conf")?;
/// let port = config.get("server.port").unwrap().as_integer()?;
///
/// # Ok::<(), config_lib::Error>(())
/// ```
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Value> {
    parsers::parse_file(path)
}

/// Validate configuration against a schema
///
/// Performs comprehensive validation of configuration data against a provided
/// schema definition.
///
/// # Arguments
///
/// * `config` - Configuration value to validate
/// * `schema` - Schema definition for validation
///
/// # Returns
///
/// Returns `Ok(())` if validation passes, or an error describing the issue.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "schema")]
/// # {
/// use config_lib::{parse, SchemaBuilder};
///
/// let config = parse(r#"
///     name = "my-app"
///     port = 8080
/// "#, None)?;
///
/// let schema = SchemaBuilder::new()
///     .require_string("name")
///     .require_integer("port")
///     .build();
///
/// config_lib::validate(&config, &schema)?;
/// # }
///
/// # Ok::<(), config_lib::Error>(())
/// ```
#[cfg(feature = "schema")]
pub fn validate(config: &Value, schema: &Schema) -> Result<()> {
    schema.validate(config)
}

/// Async version of parse_file
///
/// Available when the `async` feature is enabled.
#[cfg(feature = "async")]
pub async fn parse_file_async<P: AsRef<Path>>(path: P) -> Result<Value> {
    parsers::parse_file_async(path).await
}
