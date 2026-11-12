//! Fuzz target: NOML, default and strict (`config_lib::parse` and
//! `config_lib::parse_strict` with `Some("noml")`).
//!
//! NOML expands `${...}` references and reads `include`d files. The
//! expansion limits (64 MiB, 10,000 references, chains 32 deep) must hold
//! for any input, so the fuzzer wins with an out-of-memory error or a
//! timeout as well as with a panic. Includes resolve against the current
//! directory and may only read files inside it.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parse_strict(s, Some("noml"));
        let _ = config_lib::parse(s, Some("noml"));
    }
});
