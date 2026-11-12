//! # Configuration Format Parsers
//!
//! Multi-format configuration parsing with automatic format detection
//! and high-performance implementations.

pub mod conf;

/// Java Properties format parser
pub mod properties_parser;

/// INI format parser
pub mod ini_parser;

#[cfg(any(feature = "json", feature = "jsonx"))]
pub mod json_parser;

/// XML format parser (enterprise feature)
#[cfg(feature = "xml")]
pub mod xml_parser;

/// HCL format parser (HashiCorp Configuration Language)
#[cfg(feature = "hcl")]
pub mod hcl_parser;

// TOML and NOML parsers now enabled with published crate
#[cfg(feature = "toml")]
pub mod toml_parser;

#[cfg(feature = "noml")]
pub mod noml_parser;

// Checks shared by the TOML parser and strict NOML
#[cfg(any(feature = "noml", feature = "toml"))]
mod noml_guard;

// NOML resolution within the limits of `LoadOptions`
#[cfg(feature = "noml")]
mod noml_resolve;

use crate::error::Result;
use crate::load::LoadOptions;
use crate::value::Value;
use std::path::Path;

/// Parse configuration from a string with optional format hint
pub fn parse_string(source: &str, format: Option<&str>) -> Result<Value> {
    parse_string_with(source, format, None, &LoadOptions::default())
}

/// Parse configuration from a string without reading anything outside it.
///
/// The same as [`parse_string`] for every format except NOML, which is
/// read with `noml_parser::parse_strict`: `include` and function calls
/// (`env()` included) are refused. TOML never resolves them in either
/// entry point. Without a format hint the text is never read as NOML.
pub fn parse_string_strict(source: &str, format: Option<&str>) -> Result<Value> {
    parse_string_with(source, format, None, &LoadOptions::default().strict(true))
}

/// Parse text read from `file` (if any) with `options`
pub(crate) fn parse_string_with(
    source: &str,
    format: Option<&str>,
    file: Option<&Path>,
    options: &LoadOptions,
) -> Result<Value> {
    #[cfg(not(feature = "noml"))]
    let _ = (file, options);
    let detected_format = format.unwrap_or_else(|| detect_format(source));

    match detected_format {
        "conf" => conf::parse(source),
        "properties" => properties_parser::parse(source),
        "ini" => ini_parser::parse(source),
        #[cfg(any(feature = "json", feature = "jsonx"))]
        "json" => json_parser::parse(source),
        #[cfg(feature = "xml")]
        "xml" => xml_parser::parse(source),
        #[cfg(feature = "hcl")]
        "hcl" => hcl_parser::parse(source),
        #[cfg(feature = "noml")]
        "noml" => noml_parser::parse_with(source, file, options),
        #[cfg(feature = "toml")]
        "toml" => toml_parser::parse(source),
        _ => {
            #[cfg(not(any(feature = "json", feature = "jsonx")))]
            if detected_format == "json" {
                return Err(crate::error::Error::feature_not_enabled("json"));
            }

            #[cfg(not(feature = "xml"))]
            if detected_format == "xml" {
                return Err(crate::error::Error::feature_not_enabled("xml"));
            }

            #[cfg(not(feature = "hcl"))]
            if detected_format == "hcl" {
                return Err(crate::error::Error::feature_not_enabled("hcl"));
            }

            #[cfg(not(feature = "noml"))]
            if detected_format == "noml" {
                return Err(crate::error::Error::feature_not_enabled("noml"));
            }

            #[cfg(not(feature = "toml"))]
            if detected_format == "toml" {
                return Err(crate::error::Error::feature_not_enabled("toml"));
            }

            // Any other name is read as CONF, as it always was
            conf::parse(source)
        }
    }
}

/// The format of a file: from its extension, or from its text when it has
/// none
pub(crate) fn file_format(path: &Path, content: &str) -> &'static str {
    detect_format_from_path(path).unwrap_or_else(|| detect_format(content))
}

/// Parse configuration from a file with automatic format detection
///
/// Applies the default [`LoadOptions`]: regular files of at most 16 MiB,
/// NOML includes only from the file's directory.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path = path.as_ref();
    let options = LoadOptions::default();
    let content = crate::load::read_file(path, options.max_file_size)?;
    parse_string_with(
        &content,
        Some(file_format(path, &content)),
        Some(path),
        &options,
    )
}

/// Async version of parse_file
#[cfg(feature = "async")]
pub async fn parse_file_async<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path = path.as_ref();
    let options = LoadOptions::default();
    let content = crate::load::read_file_async(path, options.max_file_size).await?;
    parse_string_with(
        &content,
        Some(file_format(path, &content)),
        Some(path),
        &options,
    )
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
///
/// Only text with an unambiguous signature is routed away from CONF:
///
/// - XML: starts with `<?xml`, or with `<` and has a closing or
///   self-closing tag;
/// - JSON: starts with `{`, or with `[` when the first line is not a
///   `[section]` header;
/// - INI: a line starts with `;` (CONF has no `;` comments).
///
/// Everything else is CONF. TOML, NOML, HCL and Properties are never
/// detected from text: pass the format, or load a file with the matching
/// extension. Since v1.4.0; before that, substring guesses could send
/// ordinary CONF text to another parser (a quoted `\n` meant Properties,
/// a `T` and a `Z` on one line meant TOML, a comment mentioning `include`
/// meant NOML).
pub fn detect_format(content: &str) -> &'static str {
    let trimmed = content.trim_start_matches('\u{feff}').trim();

    if trimmed.starts_with("<?xml")
        || (trimmed.starts_with('<') && (trimmed.contains("</") || trimmed.contains("/>")))
    {
        return "xml";
    }

    if trimmed.starts_with('{') {
        return "json";
    }
    if trimmed.starts_with('[') {
        let first_line = trimmed.lines().next().unwrap_or("").trim();
        if !is_section_header(first_line) {
            return "json";
        }
    }

    if content
        .lines()
        .any(|line| line.trim_start().starts_with(';'))
    {
        return "ini";
    }

    "conf"
}

/// True for a `[name]` line whose name is a plain section name
fn is_section_header(line: &str) -> bool {
    let inner = match line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
        Some(inner) => inner.trim(),
        None => return false,
    };
    !inner.is_empty()
        && inner
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | ' '))
}

/// Parse `text` as a float written in decimal syntax: digits with an
/// optional sign, a fractional part, and an exponent (`1.5`, `-2e3`).
///
/// `str::parse::<f64>` also takes `nan`, `inf` and `infinity`, which turned
/// a value spelled like that into a float. A whole number with no fraction
/// or exponent is not a float here either: one too large for `i64` stays
/// text rather than becoming a rounded float.
pub(crate) fn decimal_float(text: &str) -> Option<f64> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    let (mantissa, exponent) = match digits.find(['e', 'E']) {
        Some(i) => (&digits[..i], Some(&digits[i + 1..])),
        None => (digits, None),
    };
    let (whole, fraction) = match mantissa.find('.') {
        Some(i) => (&mantissa[..i], Some(&mantissa[i + 1..])),
        None => (mantissa, None),
    };
    let all_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    let mantissa_ok = all_digits(whole)
        && fraction.map_or(true, all_digits)
        && !(whole.is_empty() && fraction.map_or(true, str::is_empty));
    let exponent_ok = exponent.map_or(true, |e| {
        let e = e.strip_prefix(['+', '-']).unwrap_or(e);
        !e.is_empty() && all_digits(e)
    });
    if !mantissa_ok || !exponent_ok || (fraction.is_none() && exponent.is_none()) {
        return None;
    }
    text.parse::<f64>().ok().filter(|f| f.is_finite())
}
