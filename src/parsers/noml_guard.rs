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
    /// only one noml 0.9.2 knows). Native types and `${...}` stay, because
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
