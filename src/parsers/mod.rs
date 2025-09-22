//! # Configuration Format Parsers
//!
//! Multi-format configuration parsing with automatic format detection
//! and high-performance implementations.

pub mod conf;

/// Java Properties format parser
pub mod properties_parser;

/// INI format parser
pub mod ini_parser;

#[cfg(feature = "json")]
pub mod json_parser;

/// XML format parser (enterprise feature)
#[cfg(feature = "xml")]
pub mod xml_parser;

/// HCL format parser (HashiCorp Configuration Language)
#[cfg(feature = "hcl")]
pub mod hcl_parser;

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
        "properties" => {
            let mut parser = properties_parser::PropertiesParser::new(source.to_string());
            parser.parse()
        }
        "ini" => ini_parser::parse_ini(source),
        #[cfg(feature = "json")]
        "json" => json_parser::parse(source),
        #[cfg(feature = "xml")]
        "xml" => xml_parser::parse_xml(source),
        #[cfg(feature = "hcl")]
        "hcl" => hcl_parser::parse_hcl(source),
        _ => {
            #[cfg(not(feature = "json"))]
            if detected_format == "json" {
                return Err(Error::feature_not_enabled("json"));
            }

            #[cfg(not(feature = "xml"))]
            if detected_format == "xml" {
                return Err(Error::feature_not_enabled("xml"));
            }

            #[cfg(not(feature = "hcl"))]
            if detected_format == "hcl" {
                return Err(Error::feature_not_enabled("hcl"));
            }

            // For now, treat everything else as conf format
            conf::parse(source)
        }
    }
}

/// Parse configuration from a file with automatic format detection
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path = path.as_ref();
    let content =
        std::fs::read_to_string(path).map_err(|e| Error::io(path.display().to_string(), e))?;

    let format = detect_format_from_path(path).or_else(|| Some(detect_format(&content)));

    parse_string(&content, format)
}

/// Async version of parse_file
#[cfg(feature = "async")]
pub async fn parse_file_async<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path = path.as_ref();
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| Error::io(path.display().to_string(), e))?;

    let format = detect_format_from_path(path).or_else(|| Some(detect_format(&content)));

    parse_string(&content, format)
}

/// Detect configuration format from file path
pub fn detect_format_from_path(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "conf" | "config" | "cfg" => "conf",
            "properties" => "properties",
            "ini" => "ini",
            "toml" => "toml",
            "json" => "json",
            "noml" => "noml",
            "xml" => "xml",
            "hcl" | "tf" => "hcl", // .tf files are Terraform HCL
            _ => "conf",           // Default to conf for unknown extensions
        })
}

/// Detect configuration format from content
pub fn detect_format(content: &str) -> &'static str {
    let trimmed = content.trim();

    // XML detection - starts with < and contains XML tags
    if trimmed.starts_with('<') && contains_xml_features(content) {
        return "xml";
    }

    // JSON detection - starts with { or [
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return "json";
    }

    // HCL detection - look for HCL-specific features
    if contains_hcl_features(content) {
        return "hcl";
    }

    // NOML detection - look for NOML-specific features
    if contains_noml_features(content) {
        return "noml";
    }

    // INI detection - look for section headers (before properties since INI can use colons)
    if contains_ini_features(content) {
        return "ini";
    }

    // Properties detection - look for properties-specific features
    if contains_properties_features(content) {
        return "properties";
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
    content.contains("env(")
        || content.contains("include ")
        || content.contains("${")
        || content.contains("@size(")
        || content.contains("@duration(")
        || content.contains("@url(")
        || content.contains("@ip(")
}

/// Check if content contains Properties-specific features
fn contains_properties_features(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        // Properties comments with ! (specific to Java Properties)
        if trimmed.starts_with('!') {
            return true;
        }
        // Properties escape sequences (more specific than CONF)
        if trimmed.contains("\\n") || trimmed.contains("\\t") || trimmed.contains("\\u") {
            return true;
        }
        // Properties use : separator more commonly than CONF
        if trimmed.contains(':') && !trimmed.contains('=') && !trimmed.starts_with('#') {
            return true;
        }
        false
    })
}

/// Check if content contains INI-specific features
fn contains_ini_features(content: &str) -> bool {
    let mut has_section = false;
    let mut has_ini_comment = false;
    let mut has_key_value_in_section = false;
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // INI section headers [section] but not TOML arrays
        if trimmed.starts_with('[') && trimmed.ends_with(']') && !trimmed.contains('=') {
            // Make sure it's not a TOML array of tables
            let section_content = &trimmed[1..trimmed.len() - 1];
            if !section_content.contains('[') && !section_content.contains(']') {
                has_section = true;
                in_section = true;
                continue;
            }
        }

        // INI comments with ; (TOML uses #)
        if trimmed.starts_with(';') {
            has_ini_comment = true;
        }

        // Key-value pairs in sections
        if in_section
            && (trimmed.contains('=') || trimmed.contains(':'))
            && !trimmed.starts_with('#')
            && !trimmed.starts_with(';')
        {
            has_key_value_in_section = true;
        }
    }

    // INI is likely if we have sections with key-value pairs OR semicolon comments
    has_section && has_key_value_in_section || has_ini_comment
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

/// Check if content contains XML-specific features
fn contains_xml_features(content: &str) -> bool {
    let trimmed = content.trim();

    // Look for XML declaration
    if trimmed.starts_with("<?xml") {
        return true;
    }

    // Look for closing XML tags
    if trimmed.contains("</") {
        return true;
    }

    // Look for XML namespaces
    if trimmed.contains("xmlns") {
        return true;
    }

    // Look for self-closing tags
    if trimmed.contains("/>") {
        return true;
    }

    // Check for balanced XML structure
    let open_tags = trimmed.matches('<').count();
    let close_tags = trimmed.matches('>').count();

    // Basic XML structure validation
    open_tags > 0 && close_tags > 0 && open_tags <= close_tags
}

/// Check if content contains HCL-specific features
fn contains_hcl_features(content: &str) -> bool {
    // Look for HCL-specific syntax patterns
    for line in content.lines() {
        let trimmed = line.trim();

        // HCL block syntax: resource "type" "name" {
        if trimmed.contains(" \"") && trimmed.contains("\" {") {
            return true;
        }

        // HCL variable/output blocks
        if trimmed.starts_with("variable ") || trimmed.starts_with("output ") {
            return true;
        }

        // HCL resource/data blocks
        if trimmed.starts_with("resource ") || trimmed.starts_with("data ") {
            return true;
        }

        // HCL provider blocks
        if trimmed.starts_with("provider ") {
            return true;
        }

        // HCL terraform blocks
        if trimmed.starts_with("terraform ") {
            return true;
        }

        // HCL module blocks
        if trimmed.starts_with("module ") {
            return true;
        }

        // HCL functions and interpolation
        if trimmed.contains("${") && trimmed.contains("}") {
            return true;
        }
    }

    false
}
