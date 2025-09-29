use config_lib::parsers::ini_parser::parse_ini;

fn main() -> config_lib::Result<()> {
    let content = std::fs::read_to_string("test.ini").unwrap();

    println!("=== Testing INI Parser Directly ===");
    let result = parse_ini(&content)?;
    println!("Parsed result: {result:#?}");

    Ok(())
}
