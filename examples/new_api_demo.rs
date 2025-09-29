use config_lib::ConfigBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Demo the new convenience methods and builder pattern
    
    // Using ConfigBuilder for fluent configuration creation
    let config = ConfigBuilder::new()
        .format("conf")
        .from_string(r#"
[server]
port = 8080
host = "localhost"
"#)?;
    
    // Using the new .key() method for ergonomic access
    let port = config.key("server.port").as_integer()?;
    let host = config.key("server.host").as_string()?;
    let name = config.key("app.name").as_string_or("MyApp");
    
    println!("Server: {}:{}", host, port);
    println!("App name: {}", name);
    
    // Check if values exist
    println!("Has server config: {}", config.has("server"));
    println!("Has database config: {}", config.has("database"));
    
    // Using traditional get with defaults for complex types
    let timeout = config.key("timeout").as_integer().unwrap_or(30);
    let debug = config.key("debug").as_bool().unwrap_or(false);
    
    println!("Timeout: {}", timeout);
    println!("Debug mode: {}", debug);
    
    Ok(())
}