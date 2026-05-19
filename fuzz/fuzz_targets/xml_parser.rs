//! Fuzz target: the XML parser wrapper
//! (`config_lib::parsers::xml_parser::parse`).
//!
//! The wrapper sits on top of `quick-xml`'s event reader. The
//! interesting failure modes are at the wrapper layer: attribute /
//! text resolution, nested element flattening, self-closing tag
//! handling, namespace stripping. `quick-xml` itself is
//! independently well-fuzzed.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = config_lib::parsers::xml_parser::parse(s);
    }
});
