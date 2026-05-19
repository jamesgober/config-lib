//! Fuzz target: the built-in HCL parser
//! (`config_lib::parsers::hcl_parser::parse`).
//!
//! The HCL parser is the youngest and least battle-tested of the
//! built-in parsers, and HCL itself has rich block / nested-object
//! semantics that have historically attracted parsing bugs. This
//! target is the highest-yield of the seven for adversarial finds.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parsers::hcl_parser::parse(s);
    }
});
