use config_lib::parsers::detect_format_from_path;
use std::path::Path;

fn main() {
    let path = Path::new("test.ini");
    let format = detect_format_from_path(path);
    println!("Path-based format detection: {format:?}");

    if let Some(format_str) = format {
        println!("Format string: '{format_str}'");
    }
}
