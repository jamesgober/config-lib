//! # Configuration Format Parsers
//!
//! Multi-format configuration parsing with automatic format detection
//! and high-performance implementations.

pub mod conf;

#[cfg(feature = "json")]
pub mod json_parser;

// Disabled for now - require external NOML crate
// pub mod toml_parser;
// pub mod noml_parser;

use crate::error::{Error, Result};
use crate::value::Value;
use std::path::Path;

/// Parse configuration from a string with optional format hint
/// Uses zero-copy AST parser for enterprise performance
pub fn parse_string(source: &str, format: Option<&str>) -> Result<Value> {
    let detected_format = format.unwrap_or_else(|| detect_format(source));

    match detected_format {
        "conf" => conf::parse(source),
        #[cfg(feature = "json")]
        "json" => json_parser::parse(source),
        _ => {
            #[cfg(not(feature = "json"))]
            if detected_format == "json" {
                return Err(Error::feature_not_enabled("json"));
            }
            
            // For now, treat everything else as conf format
            conf::parse(source)
        }
    }
}

/// Parse configuration from a file with automatic format detection
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::io(path.display().to_string(), e))?;

    let format = detect_format_from_path(path)
        .or_else(|| Some(detect_format(&content)));

    parse_string(&content, format)
}

/// Async version of parse_file
#[cfg(feature = "async")]
pub async fn parse_file_async<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path = path.as_ref();
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| Error::io(path.display().to_string(), e))?;

    let format = detect_format_from_path(path)
        .or_else(|| Some(detect_format(&content)));

    parse_string(&content, format)
}

/// Detect configuration format from file path
pub fn detect_format_from_path(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "conf" | "config" | "cfg" => "conf",
            "toml" => "toml",
            "json" => "json",
            "noml" => "noml",
            _ => "conf", // Default to conf for unknown extensions
        })
}

/// Detect configuration format from content
pub fn detect_format(content: &str) -> &'static str {
    let trimmed = content.trim();
    
    // JSON detection - starts with { or [
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return "json";
    }
    
    // NOML detection - look for NOML-specific features
    if contains_noml_features(content) {
        return "noml";
    }
    
    // TOML detection - look for TOML-specific syntax
    if contains_toml_features(content) {
        return "toml";
    }
    
    // Default to conf format
    "conf"
}

/// Check if content contains NOML-specific features
fn contains_noml_features(content: &str) -> bool {
    // Look for NOML-specific syntax
    content.contains("env(") ||
    content.contains("include ") ||
    content.contains("${") ||
    content.contains("@size(") ||
    content.contains("@duration(") ||
    content.contains("@url(") ||
    content.contains("@ip(")
}

/// Check if content contains TOML-specific features
fn contains_toml_features(content: &str) -> bool {
    // Look for TOML-specific syntax patterns
    content.lines().any(|line| {
        let trimmed = line.trim();
        // TOML section headers
        if trimmed.starts_with('[') && trimmed.ends_with(']') && !trimmed.contains('=') {
            return true;
        }
        // TOML datetime format
        if trimmed.contains("T") && trimmed.contains("Z") {
            return true;
        }
        false
    })
}