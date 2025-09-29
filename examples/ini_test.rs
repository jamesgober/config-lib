use config_lib::parsers::ini_parser::parse_ini;

fn main() -> config_lib::Result<()> {
    let content = r#"
global=value1

[section1]
key1=value2
    "#;

    let result = parse_ini(content)?;
    println!("Parsed INI result: {result:#?}");

    Ok(())
}
