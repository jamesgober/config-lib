//! Regression tests for fuzz-discovered parser inputs.
//!
//! This file is the bridge between `cargo fuzz`'s ephemeral working
//! corpus (which lives at `fuzz/corpus/<target>/` and is *not*
//! committed) and the permanent regression test suite (which runs
//! under `cargo test` and is committed). The workflow:
//!
//! 1. `cargo +nightly fuzz run <target> -- -max_total_time=3600`
//! 2. If the fuzzer produces an interesting input (a panic / hang /
//!    OOM in pre-fix builds, or a particularly tricky edge case
//!    worth permanent coverage), copy the relevant
//!    `fuzz/corpus/<target>/<hash>` file into
//!    `tests/corpus_seeds/<target>/<descriptive-name>` (committed).
//! 3. Add a `#[test]` block below that loads the file via
//!    `include_bytes!`, transcodes to `&str`, and calls the
//!    matching parser. The regression contract is "no panic, no
//!    hang, no OOM" — `cargo test`'s normal runner enforces all
//!    three.
//!
//! The current corpus is empty: v0.9.9 ships the harness and
//! v0.9.9.x / v1.0.x populates `tests/corpus_seeds/` as the
//! maintainer's clean fuzz runs surface inputs worth keeping
//! around as permanent regressions.

#![allow(clippy::unwrap_used, clippy::expect_used)]

// =========================================================================
// Per-seed regression tests live below.
//
// Template (uncomment + adjust when a seed lands):
//
//     #[test]
//     fn hcl_parser_01_nested_block_terminator_in_string() {
//         let bytes: &[u8] = include_bytes!(
//             "corpus_seeds/hcl_parser/01_nested_block_terminator_in_string"
//         );
//         if let Ok(s) = std::str::from_utf8(bytes) {
//             let _ = config_lib::parsers::hcl_parser::parse(s);
//         }
//     }
// =========================================================================

/// Sanity test so the integration-test binary isn't empty. Proves
/// the parser module re-exports are wired correctly and the file
/// compiles. Replaced or supplemented by real corpus entries as
/// they accumulate.
#[test]
fn corpus_harness_compiles() {
    let _ = config_lib::parsers::conf::parse("sanity=1\n");
}

// =========================================================================
// v1.3.0: inputs found in review rather than by the fuzzer. Kept inline
// because each is a few bytes.
// =========================================================================

/// CONF sliced at a character count instead of a byte offset and
/// panicked when non-ASCII text came before a section header.
#[test]
fn conf_parser_non_ascii_before_section() {
    let value = config_lib::parsers::conf::parse("#\u{e9}\u{e9}\u{e9}\n[a]\nk = 1\n").unwrap();
    assert_eq!(value.get("a.k").unwrap().as_integer().unwrap(), 1);
}

/// CONF parsed nested arrays recursively with no limit, so a run of `[`
/// overflowed the stack.
#[test]
fn conf_parser_deep_array_nesting() {
    let input = format!("k = {}", "[".repeat(100_000));
    assert!(config_lib::parsers::conf::parse(&input).is_err());
}

/// INI looped forever on a non-ASCII character and on a line starting
/// with a separator.
#[test]
fn ini_parser_non_ascii_and_leading_separator() {
    assert!(config_lib::parsers::ini_parser::parse("; \u{e9}\nk=1\n").is_ok());
    assert!(config_lib::parsers::ini_parser::parse("=x\n").is_err());
}

/// Properties refused a valid file with non-ASCII text in it.
#[test]
fn properties_parser_non_ascii() {
    assert!(config_lib::parsers::properties_parser::parse("# \u{e9}\u{e9}\u{e9}\nk=v\n").is_ok());
}
