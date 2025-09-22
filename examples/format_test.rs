use config_lib::parsers::{detect_format, detect_format_from_path};
use std::path::Path;

fn main() -> config_lib::Result<()> {
    let path = Path::new("test.ini");
    let content = std::fs::read_to_string("test.ini").unwrap();

    println!("=== Format Detection Test ===");
    println!("Path-based detection: {:?}", detect_format_from_path(path));
    println!("Content-based detection: {}", detect_format(&content));

    Ok(())
}
