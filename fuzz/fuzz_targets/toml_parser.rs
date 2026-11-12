//! Fuzz target: TOML (`config_lib::parse` with `Some("toml")`).
//!
//! TOML is read through the noml crate with NOML syntax refused before
//! resolving, so no input may read a file, the environment, or expand a
//! reference.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parse(s, Some("toml"));
    }
});
