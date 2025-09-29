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
pub mod enterprise; // Enterprise API with caching and performance
pub mod error;
pub mod parsers;
pub mod value;

#[cfg(feature = "schema")]
pub mod schema;

#[cfg(feature = "validation")]
pub mod validation;

/// Hot reloading system for zero-downtime configuration updates
pub mod hot_reload;

/// Comprehensive audit logging system for configuration operations
pub mod audit;

/// Environment variable override system for smart configuration overrides
#[cfg(feature = "env-override")]
pub mod env_override;

// Re-export main types for convenience
pub use config::{Config, ConfigBuilder, ConfigValue};
pub use enterprise::{ConfigManager, EnterpriseConfig};
pub use error::{Error, Result};
pub use value::Value;

#[cfg(feature = "schema")]
pub use schema::{Schema, SchemaBuilder};

#[cfg(feature = "validation")]
pub use validation::{
    ValidationError, ValidationResult, ValidationRule, ValidationRuleSet, ValidationSeverity,
};

use std::path::Path;

/// Parse configuration from a string with optional format hint
///
/// This is the primary entry point for parsing configuration data from strings.
/// Automatically detects format if not specified.
///
/// # Arguments
///
/// * `source` - The configuration data as a string
/// * `format` - Optional format hint ("conf", "toml", "json", "noml")
///
/// # Returns
///
/// Returns a [`Value`] containing the parsed configuration data.
///
/// # Errors
///
/// Returns an error if:
/// - The input format is unknown or unsupported
/// - The input contains syntax errors
/// - Required features are not enabled for the detected format
///
/// # Examples
///
/// ```rust
/// use config_lib::parse;
///
/// let config = parse("port = 8080\nname = \"MyApp\"", Some("conf"))?;
/// let port = config.get("port").unwrap().as_integer()?;
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
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read (I/O error)
/// - The file format cannot be detected
/// - The file contains syntax errors
/// - Required features are not enabled for the detected format
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
