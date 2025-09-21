//! # Error Handling
//!
//! Comprehensive error system for config-lib operations.
//! Designed for clarity, debuggability, and extensibility.

use std::io;
use thiserror::Error;

/// The main result type used throughout config-lib operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error types for all config-lib operations.
///
/// This error system provides maximum clarity about what went wrong,
/// where it happened, and how to fix it. Each variant includes enough context
/// for both developers and end users to understand and resolve issues.
#[derive(Error, Debug)]
pub enum Error {
    /// Parsing errors - when the input cannot be parsed due to syntax issues
    #[error("Parse error at line {line}, column {column}: {message}")]
    Parse {
        /// Human-readable error message
        message: String,
        /// Line number where error occurred (1-indexed)
        line: usize,
        /// Column number where error occurred (1-indexed)
        column: usize,
        /// File path where error occurred (if applicable)
        file: Option<String>,
    },

    /// Format detection errors
    #[error("Unknown format: {format}")]
    UnknownFormat {
        /// The format that couldn't be detected/parsed
        format: String,
    },

    /// Key access errors - when requesting non-existent keys
    #[error("Key '{key}' not found")]
    KeyNotFound {
        /// The key that was requested
        key: String,
        /// Available keys at that level (for suggestions)
        available: Vec<String>,
    },

    /// Type conversion errors - when values cannot be converted to requested type
    #[error("Type error: cannot convert '{value}' to {expected_type}")]
    Type {
        /// The value that couldn't be converted
        value: String,
        /// The expected type
        expected_type: String,
        /// The actual type found
        actual_type: String,
    },

    /// File I/O errors - wraps std::io::Error with additional context
    #[error("File error for '{path}': {source}")]
    Io {
        /// Path to the file that caused the error
        path: String,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Schema validation errors
    #[cfg(feature = "schema")]
    #[error("Schema error at '{path}': {message}")]
    Schema {
        /// Path where schema validation failed
        path: String,
        /// Description of the schema violation
        message: String,
        /// Expected schema type/format
        expected: Option<String>,
    },

    /// General validation errors
    #[error("Validation error: {message}")]
    Validation {
        /// Description of the validation error
        message: String,
    },

    /// Generic error for other cases
    #[error("{message}")]
    General {
        /// Generic error message
        message: String,
    },

    /// Feature not enabled errors
    #[error("Feature '{feature}' is not enabled. Enable with features = [\"{feature}\"]")]
    FeatureNotEnabled {
        /// The feature that needs to be enabled
        feature: String,
    },

    /// NOML library errors (when using NOML format)
    #[cfg(feature = "noml")]
    #[error("NOML error: {source}")]
    Noml {
        /// The underlying NOML error
        #[from]
        source: noml::NomlError,
    },

    /// Internal errors - these should never happen in normal operation
    #[error("Internal error: {message}")]
    Internal {
        /// Description of the internal error
        message: String,
        /// Optional context about where this occurred
        context: Option<String>,
    },
}

impl Error {
    /// Create a parse error with position information
    pub fn parse(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::Parse {
            message: message.into(),
            line,
            column,
            file: None,
        }
    }

    /// Create a parse error with file context
    pub fn parse_with_file(
        message: impl Into<String>,
        line: usize,
        column: usize,
        file: impl Into<String>,
    ) -> Self {
        Self::Parse {
            message: message.into(),
            line,
            column,
            file: Some(file.into()),
        }
    }

    /// Create a key not found error
    pub fn key_not_found(key: impl Into<String>) -> Self {
        Self::KeyNotFound {
            key: key.into(),
            available: Vec::new(),
        }
    }

    /// Create a key not found error with suggestions
    pub fn key_not_found_with_suggestions(
        key: impl Into<String>,
        available: Vec<String>,
    ) -> Self {
        Self::KeyNotFound {
            key: key.into(),
            available,
        }
    }

    /// Create a type conversion error
    pub fn type_error(
        value: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::Type {
            value: value.into(),
            expected_type: expected.into(),
            actual_type: actual.into(),
        }
    }

    /// Create an I/O error with file context
    pub fn io(path: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// Create an unknown format error
    pub fn unknown_format(format: impl Into<String>) -> Self {
        Self::UnknownFormat {
            format: format.into(),
        }
    }

    /// Create a feature not enabled error
    pub fn feature_not_enabled(feature: impl Into<String>) -> Self {
        Self::FeatureNotEnabled {
            feature: feature.into(),
        }
    }

    /// Create a serialization error
    pub fn serialize(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
        }
    }

    /// Create a schema validation error
    #[cfg(feature = "schema")]
    pub fn schema(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Schema {
            path: path.into(),
            message: message.into(),
            expected: None,
        }
    }

    /// Create a schema validation error with expected type
    #[cfg(feature = "schema")]
    pub fn schema_with_expected(
        path: impl Into<String>,
        message: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        Self::Schema {
            path: path.into(),
            message: message.into(),
            expected: Some(expected.into()),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a general error
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            context: None,
        }
    }

    /// Create an internal error with context
    pub fn internal_with_context(
        message: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            context: Some(context.into()),
        }
    }
}

/// Convert from std::io::Error
impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: "unknown".to_string(),
            source,
        }
    }
}