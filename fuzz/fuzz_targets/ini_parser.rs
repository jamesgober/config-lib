//! Fuzz target: the built-in INI parser
//! (`config_lib::parsers::ini_parser::parse`).
//!
//! Looking for panics, infinite loops, OOMs on arbitrary byte input.
//! INI is particularly interesting to fuzz because of its section
//! / dotted-key resolution, comment handling (`;` and `#`), and
//! escape-sequence support in quoted values.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parsers::ini_parser::parse(s);
    }
});
