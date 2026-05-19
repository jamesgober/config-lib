//! Fuzz target: the Java `.properties` parser
//! (`config_lib::parsers::properties_parser::parse`).
//!
//! Looking for panics, infinite loops, OOMs on arbitrary byte input.
//! Java `.properties` files have rich escape-sequence semantics
//! (`\uXXXX` unicode escapes, line continuations with trailing `\`,
//! colon-or-equals key/value separators) — the kind of surface that
//! tends to attract parsing-edge-case bugs.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parsers::properties_parser::parse(s);
    }
});
