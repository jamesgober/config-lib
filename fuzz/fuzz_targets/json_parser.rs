//! Fuzz target: the JSON parser wrapper
//! (`config_lib::parsers::json_parser::parse`).
//!
//! The wrapper delegates to `serde_json` for tokenisation, then
//! transforms the resulting `serde_json::Value` tree into a
//! `config_lib::Value`. `serde_json` is independently well-fuzzed;
//! this target is specifically looking for panics in the *wrapper
//! layer's* tree-walk and type conversions on adversarial input
//! (deeply nested structures, exotic number encodings, etc.).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parsers::json_parser::parse(s);
    }
});
