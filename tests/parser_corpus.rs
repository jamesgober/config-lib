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
