use config_lib::{parsers, Config};
use std::path::Path;

fn main() -> config_lib::Result<()> {
    let path = Path::new("test.ini");
    let content = std::fs::read_to_string(path).unwrap();

    println!("=== Tracing Config::from_file Flow ===");

    // Step 1: Format detection
    let path_format = parsers::detect_format_from_path(path);
    let content_format = parsers::detect_format(&content);

    println!("1. Path format: {:?}", path_format);
    println!("2. Content format: {}", content_format);

    // Step 2: Final format selection (mimicking Config::from_file)
    let final_format = path_format.unwrap_or(content_format);
    println!("3. Final format: {}", final_format);

    // Step 3: Parse with explicit format
    println!("4. Parsing with format: {}", final_format);
    let parsed_result = parsers::parse_string(&content, Some(final_format))?;
    println!(
        "5. Parsed keys count: {}",
        match &parsed_result {
            config_lib::Value::Table(map) => map.len(),
            _ => 0,
        }
    );

    // Step 4: Create config the same way Config::from_file does
    let config = Config::from_string(&content, Some(final_format))?;

    // Step 5: Test key access
    println!("6. Testing key access...");
    if let Some(value) = config.get("app_name") {
        println!("   ✅ app_name: {:?}", value);
    } else {
        println!("   ❌ app_name not found");
    }

    if let Some(value) = config.get("database.host") {
        println!("   ✅ database.host: {:?}", value);
    } else {
        println!("   ❌ database.host not found");
    }

    Ok(())
}
