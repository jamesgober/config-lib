use config_lib::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Properties format");

    // Test with explicit format hint
    let config = Config::from_string(
        r#"
# Test properties format
app_name=MyApp
server_port=8080
database_enabled=true
"#,
        Some("properties"),
    )?;

    println!("Config parsed successfully!");
    println!("app_name = {:?}", config.get("app_name"));
    println!("server_port = {:?}", config.get("server_port"));
    println!("database_enabled = {:?}", config.get("database_enabled"));

    // Test the original format detection
    let config2 = Config::from_string(
        r#"
# Test properties format
app.name=MyApp
server.port=8080
database.enabled=true
"#,
        None,
    )?;

    println!("\nFormat detected config:");
    println!("Keys available: {:?}", config2.keys());

    Ok(())
}
