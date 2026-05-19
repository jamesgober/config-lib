//! Fuzz target: the top-level `parse` entry point with format auto-
//! detection (`config_lib::parse(content, None)`).
//!
//! This target exercises the format-detection heuristics in
//! `config_lib::parsers::detect_format` plus the dispatch into
//! whichever parser the heuristic chose. It complements the
//! per-parser targets above by catching:
//!
//! - misclassification (input that *looks* like format A but is
//!   actually format B and trips the wrong parser into a panic)
//! - detector bugs (the detector itself panicking on adversarial
//!   leading bytes)
//! - boundary cases between detection and parsing
//!
//! No `format` hint is passed — auto-detect is what we want to
//! stress.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parse(s, None);
    }
});
