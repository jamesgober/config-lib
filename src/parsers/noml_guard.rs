//! # Checks on a NOML document before it is resolved
//!
//! The `noml` crate parses TOML and NOML into the same syntax tree, and its
//! resolver acts on everything NOML adds to TOML:
//!
//! - `include "path"` reads a file (relative to the working directory),
//! - `env("NAME")` copies an environment variable into a value,
//! - `@type(...)` builds a native value (`@size`, `@duration`, ...),
//! - a bare `${path}` copies another value.
//!
//! A file read with TOML semantics must do none of these, and the strict
//! NOML entry points must not read anything outside the text they are
//! given. [`check`] walks the tree before the resolver sees it and refuses
//! the first node a [`Policy`] does not allow, with its line and column.
//! The error never quotes the input: an include path or a variable name
//! can itself be sensitive.

use crate::error::{Error, Result};
use noml::parser::ast::{AstNode, AstValue};

/// What a document may contain
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Policy {
    /// TOML: plain values only. No `include`, no function calls, no
    /// `@type(...)` values and no bare `${...}`.
    Toml,
    /// Strict NOML: no `include` and no function calls (`env()` is the
    /// only one noml 0.9.4 knows). Native types and `${...}` stay, because
    /// they only read the document itself.
    // Used only by the NOML parser; `toml` alone compiles this module too
    #[cfg_attr(not(feature = "noml"), allow(dead_code))]
    StrictNoml,
}

impl Policy {
    fn format(self) -> &'static str {
        match self {
            Policy::Toml => "TOML",
            Policy::StrictNoml => "strict NOML",
        }
    }
}

/// Refuse the first node in document order that `policy` does not allow.
///
/// Iterative, so the walk uses no stack space per nesting level (the noml
/// parser also caps nesting at 64 levels).
pub(crate) fn check(document: &noml::Document, policy: Policy) -> Result<()> {
    let mut pending: Vec<&AstNode> = vec![&document.root];
    while let Some(node) = pending.pop() {
        if let Some(what) = refused(&node.value, policy) {
            return Err(Error::parse(
                format!("{what} is not allowed in {}", policy.format()),
                node.span.start_line,
                node.span.start_column,
            ));
        }
        // Children are pushed in reverse so they are checked in order
        match &node.value {
            AstValue::Array { elements, .. } => pending.extend(elements.iter().rev()),
            AstValue::Table { entries, .. } => {
                pending.extend(entries.iter().rev().map(|entry| &entry.value));
            }
            AstValue::FunctionCall { args, .. } | AstValue::Native { args, .. } => {
                pending.extend(args.iter().rev());
            }
            _ => {}
        }
    }
    Ok(())
}

/// The refusal message for `value` under `policy`, or `None` if allowed.
///
/// The text is fixed for each kind of node and never includes a name,
/// path or argument from the document.
fn refused(value: &AstValue, policy: Policy) -> Option<&'static str> {
    match value {
        AstValue::Include { .. } => Some("include"),
        AstValue::FunctionCall { name, .. } if name == "env" => Some("env()"),
        AstValue::FunctionCall { .. } => Some("a function call"),
        AstValue::Native { .. } if policy == Policy::Toml => Some("a native type (@type)"),
        AstValue::Interpolation { .. } if policy == Policy::Toml => {
            Some("an interpolation (${...})")
        }
        _ => None,
    }
}

/// Turn a noml error into a crate error that does not quote the input.
///
/// The noml crate's messages can hold text from the document (an unquoted
/// value, a key, a variable name, a path), so only the kind of problem and
/// its line and column are kept. `format` names the format in the message.
pub(crate) fn noml_error(error: noml::NomlError, format: &str) -> Error {
    use noml::NomlError as E;
    match error {
        E::Parse {
            message,
            line,
            column,
            ..
        } => {
            // The resolver's own limits and include checks (noml 0.9.4).
            // config-lib checks the same limits before the resolver runs, so
            // these get the text config-lib's own checks use.
            if let Some(what) = resolver_refusal(&message) {
                return Error::parse(what, line, column);
            }
            let what = if message.starts_with("Environment variable") {
                "an env() variable is not set and has no default"
            } else if message.starts_with("Duplicate key") {
                "a key is defined twice"
            } else if message.starts_with("Value is nested deeper") {
                "values are nested too deeply"
            } else if message.starts_with("Unknown function") {
                "unknown function"
            } else if message.starts_with("Unknown native type") {
                "unknown native type"
            } else if message.starts_with("Maximum include depth") {
                "includes are nested too deeply"
            } else if message.starts_with("include can only") {
                "include is only allowed as the value of a key or an array element"
            } else {
                "syntax error"
            };
            Error::parse(format!("{format} {what}"), line, column)
        }
        E::Interpolation { .. } => Error::general(format!(
            "{format}: a ${{...}} reference could not be resolved"
        )),
        E::CircularReference { .. } => {
            Error::general(format!("{format}: a reference or include refers to itself"))
        }
        E::EnvVar { .. } => Error::general(format!(
            "{format}: an env() variable is not set and has no default"
        )),
        E::Import { .. } | E::Io { .. } => {
            Error::general(format!("{format}: an included file could not be read"))
        }
        _ => Error::general(format!("{format}: the document is not valid")),
    }
}

/// The config-lib text for a noml resolver limit or include refusal, from
/// the start of the noml message. The rest of the message is never used: it
/// can name a path.
fn resolver_refusal(message: &str) -> Option<&'static str> {
    const REFUSALS: [(&str, &str); 6] = [
        (
            "Interpolation output exceeds",
            "the document expands past the size limit through ${...} references",
        ),
        (
            "Interpolation uses more than",
            "the document has too many ${...} references",
        ),
        (
            "Interpolation references are chained",
            "a chain of ${...} references is too long",
        ),
        ("Maximum include count", "too many included files"),
        (
            "Include refused",
            "the included file is outside the allowed directories",
        ),
        (
            "Failed to read include",
            "the included file could not be read",
        ),
    ];
    REFUSALS
        .iter()
        .find(|(prefix, _)| message.starts_with(prefix))
        .map(|&(_, what)| what)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A noml 0.9.4 resolver message, with text from the document in it
    fn parse_error(message: &str) -> noml::NomlError {
        noml::NomlError::parse(format!("{message} SECRET-TEXT"), 7, 3)
    }

    fn mapped(message: &str) -> (String, usize, usize) {
        match noml_error(parse_error(message), "NOML") {
            Error::Parse {
                message,
                line,
                column,
                ..
            } => {
                assert!(!message.contains("SECRET-TEXT"), "{message}");
                (message, line, column)
            }
            other => panic!("expected a parse error, got {other:?}"),
        }
    }

    #[test]
    fn output_limit_is_the_size_limit() {
        let (message, line, column) =
            mapped("Interpolation output exceeds the limit of 67108864 bytes");
        assert_eq!(
            message,
            "the document expands past the size limit through ${...} references"
        );
        assert_eq!((line, column), (7, 3));
    }

    #[test]
    fn reference_limit_is_too_many_references() {
        let (message, ..) = mapped("Interpolation uses more than the limit of 10000 references");
        assert_eq!(message, "the document has too many ${...} references");
    }

    #[test]
    fn chain_limit_is_a_chain_too_long() {
        let (message, ..) =
            mapped("Interpolation references are chained deeper than the limit of 32 levels");
        assert_eq!(message, "a chain of ${...} references is too long");
    }

    #[test]
    fn include_count_is_too_many_included_files() {
        let (message, ..) = mapped("Maximum include count (64) exceeded");
        assert_eq!(message, "too many included files");
    }

    #[test]
    fn include_refused_is_outside_the_allowed_directories() {
        for text in [
            "Include refused: the file is outside the include root",
            "Include refused: the document has no base path or include root",
        ] {
            let (message, ..) = mapped(text);
            assert_eq!(
                message,
                "the included file is outside the allowed directories"
            );
        }
    }

    #[test]
    fn unreadable_include_does_not_quote_the_path() {
        let (message, ..) = mapped("Failed to read include 'secret/path.noml': not found");
        assert_eq!(message, "the included file could not be read");
    }

    #[test]
    fn other_parse_errors_keep_their_mapping() {
        let (message, ..) = mapped("Unexpected token");
        assert_eq!(message, "NOML syntax error");
        let (message, ..) = mapped("Maximum include depth (10) exceeded");
        assert_eq!(message, "NOML includes are nested too deeply");
    }
}
