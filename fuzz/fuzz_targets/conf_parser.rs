//! Fuzz target: the built-in CONF parser (`config_lib::parsers::conf::parse`).
//!
//! Looking for panics, infinite loops, OOMs on arbitrary byte input.
//! Correctness of the parse result is not asserted — every `Err` is a
//! valid outcome. The fuzzer wins by producing input that crashes or
//! hangs the process.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The CONF parser takes `&str`. Skip non-UTF-8 inputs — the
    // public API requires the caller to have a valid `&str` already,
    // so a UTF-8 violation is a caller bug, not a parser bug.
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parsers::conf::parse(s);
    }
});
