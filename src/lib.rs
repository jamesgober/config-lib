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
//! let port: i64 = config
//!     .get("port")
//!     .ok_or_else(|| config_lib::Error::key_not_found("port"))?
//!     .as_integer()?;
//! let name: String = config
//!     .get("name")
//!     .ok_or_else(|| config_lib::Error::key_not_found("name"))?
//!     .as_string()?
//!     .to_owned();
//!
//! // Modify and save (preserves format and comments)
//! config.set("port", 9000)?;
//! # let _ = (port, name);
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
//! - **High Performance** - Zero-copy parsing where possible
//! - **Format Preservation** - Maintains comments, whitespace, and formatting
//! - **Async Native** - Full async/await support (feature: `async`)
//! - **Schema Validation** - Type safety and validation (feature: `schema`)
//! - **Cross Platform** - Linux, macOS, and Windows support
//! - **Type Safety** - Rich type system with automatic conversions

// REPS — Rust Efficiency & Performance Standards (project-wide lint discipline).
// Shipping code MUST satisfy these denies; test code carries narrower
// allows with `REPS-AUDIT:` justifications where ergonomics outweigh the strict rule.
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::missing_safety_doc)]
#![warn(clippy::pedantic)]
// REPS-AUDIT: `clippy::pedantic` is a curated lint group, not a hard rule.
// The following lints are turned back to `allow` because they fire on
// idioms this crate uses deliberately, or because they trade real
// readability against pure stylistic preference. The denies above
// (REPS safety / correctness rules) remain in force.
//
// Style / API-shape noise:
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::similar_names)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::no_effect_underscore_binding)]
// Numeric casts at bounded parser/stats boundaries:
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
// Documentation style preference:
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// Refactor suggestions deferred to a dedicated cleanup phase
// (see `.dev/ROADMAP.md` post-1.0 backlog):
#![allow(clippy::unused_self)]
#![allow(clippy::self_only_used_in_recursion)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::single_match_else)]
#![allow(clippy::inline_always)]
#![allow(clippy::format_push_string)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::unnecessary_literal_bound)]
#![allow(clippy::if_not_else)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::needless_for_each)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::ignored_unit_patterns)]
// REPS-AUDIT: test modules use `.unwrap()` / `.expect()` / `panic!` for terse
// failure assertions, may emit `println!` / `eprintln!` for debug context,
// and use raw-string-with-hashes / direct float `==` / unseparated literals
// where the test input is more readable left as written. Allowed under
// `cfg(test)` only; never reachable in shipped binaries.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::print_stdout,
        clippy::print_stderr,
        clippy::needless_raw_string_hashes,
        clippy::float_cmp,
        clippy::unreadable_literal,
        clippy::manual_assert,
        clippy::ignore_without_reason,
    )
)]

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
/// let port = config
///     .get("port")
///     .ok_or_else(|| config_lib::Error::key_not_found("port"))?
///     .as_integer()?;
/// # let _ = port;
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
/// let port = config
///     .get("server.port")
///     .ok_or_else(|| config_lib::Error::key_not_found("server.port"))?
///     .as_integer()?;
/// # let _ = port;
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
