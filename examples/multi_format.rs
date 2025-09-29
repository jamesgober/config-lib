//! # Multi-Format Configuration Example
//!
//! Demonstrates parsing different configuration formats
//! and format-specific features.

use config_lib::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 Multi-Format Configuration Example\n");

    // 1. CONF Format (built-in)
    println!("📄 CONF Format Example:");
    let conf_content = r#"
        # CONF format configuration
        app_name = "conf-app"
        port = 8080
        debug = true
        
        [server]
        host = "localhost"
        workers = 4
        
        # Arrays
        plugins = auth cache logging
    "#;

    let conf_config = Config::from_string(conf_content, Some("conf"))?;
    println!("  ✓ Parsed CONF format");
    println!(
        "  📋 App: {}",
        conf_config.get("app_name").unwrap().as_string()?
    );
    println!(
        "  🔧 Debug: {}",
        conf_config.get("debug").unwrap().as_bool()?
    );

    // 2. JSON Format (if feature enabled)
    #[cfg(feature = "json")]
    {
        println!("\n📄 JSON Format Example:");
        let json_content = r#"
        {
            "app_name": "json-app",
            "port": 3000,
            "debug": false,
            "server": {
                "host": "0.0.0.0",
                "workers": 8
            },
            "plugins": ["auth", "cache", "logging"]
        }
        "#;

        let json_config = Config::from_string(json_content, Some("json"))?;
        println!("  ✓ Parsed JSON format");
        println!(
            "  📋 App: {}",
            json_config.get("app_name").unwrap().as_string()?
        );
        println!(
            "  🏠 Host: {}",
            json_config.get("server.host").unwrap().as_string()?
        );

        // Demonstrate JSON serialization
        let serialized = json_config.serialize()?;
        println!("  💾 Serialized back to JSON:");
        println!(
            "  {}",
            serialized.lines().take(3).collect::<Vec<_>>().join("\n  ")
        );
    }

    // 3. TOML Format (if feature enabled)
    #[cfg(feature = "toml")]
    {
        println!("\n📄 TOML Format Example:");
        let toml_content = r#"
        app_name = "toml-app"
        port = 5000
        debug = true
        
        [server]
        host = "127.0.0.1"
        workers = 2
        
        [database]
        url = "postgresql://localhost/mydb"
        max_connections = 20
        
        plugins = ["auth", "cache"]
        "#;

        let toml_config = Config::from_string(toml_content, Some("toml"))?;
        println!("  ✓ Parsed TOML format");
        println!(
            "  📋 App: {}",
            toml_config.get("app_name").unwrap().as_string()?
        );
        println!(
            "  🗄️  DB URL: {}",
            toml_config.get("database.url").unwrap().as_string()?
        );
    }

    // 4. NOML Format (if feature enabled) - Advanced features
    #[cfg(feature = "noml")]
    {
        use std::env;

        println!("\n📄 NOML Format Example (Advanced Features):");

        // Set environment variable for demo
        env::set_var("APP_ENV", "production");
        env::set_var("DB_HOST", "prod-db.example.com");

        let noml_content = r#"
        app_name = "noml-app"
        environment = env("APP_ENV", "development")
        port = 6000
        
        # Native types
        max_file_size = @size("100MB")
        timeout = @duration("30s")
        api_url = @url("https://api.example.com")
        
        [database]
        host = env("DB_HOST", "localhost")
        port = 5432
        
        # Arrays with different syntax
        features = ["auth", "cache", "monitoring"]
        "#;

        let noml_config = Config::from_string(noml_content, Some("noml"))?;
        println!("  ✓ Parsed NOML format with advanced features");
        println!(
            "  📋 App: {}",
            noml_config.get("app_name").unwrap().as_string()?
        );
        println!(
            "  🌍 Environment: {}",
            noml_config.get("environment").unwrap().as_string()?
        );
        println!(
            "  🗄️  DB Host: {}",
            noml_config.get("database.host").unwrap().as_string()?
        );

        // Native types are converted to basic types
        println!(
            "  📏 Max file size (bytes): {}",
            noml_config.get("max_file_size").unwrap().as_integer()?
        );
        println!(
            "  ⏱️  Timeout (seconds): {}",
            noml_config.get("timeout").unwrap().as_float()?
        );
    }

    // 5. Format Detection
    println!("\n🔍 Automatic Format Detection:");

    let unknown_content = r#"{"test": "json"}"#;
    let detected_config = Config::from_string(unknown_content, None)?;
    println!("  ✓ Auto-detected format: {}", detected_config.format());

    let unknown_content2 = r#"test = "conf""#;
    let detected_config2 = Config::from_string(unknown_content2, None)?;
    println!("  ✓ Auto-detected format: {}", detected_config2.format());

    // 6. Merging configurations
    println!("\n🔄 Configuration Merging:");
    let mut base_config = Config::from_string(
        "app = base\nport = 8080\n[logging]\nlevel = info",
        Some("conf"),
    )?;

    let override_config =
        Config::from_string("port = 9000\n[logging]\nfile = app.log", Some("conf"))?;

    println!(
        "  Before merge - Port: {}",
        base_config.get("port").unwrap().as_integer()?
    );
    base_config.merge(&override_config)?;
    println!(
        "  After merge - Port: {}",
        base_config.get("port").unwrap().as_integer()?
    );
    println!(
        "  Added logging file: {}",
        base_config.get("logging.file").unwrap().as_string()?
    );
    println!(
        "  Preserved logging level: {}",
        base_config.get("logging.level").unwrap().as_string()?
    );

    println!("\n🎉 Multi-format example completed!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        // JSON detection
        let json = r#"{"key": "value"}"#;
        let config = Config::from_string(json, None).unwrap();
        assert_eq!(config.format(), "json");

        // CONF detection (default)
        let conf = "key = value";
        let config = Config::from_string(conf, None).unwrap();
        assert_eq!(config.format(), "conf");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_json_format() {
        let json = r#"{"name": "test", "port": 8080}"#;
        let config = Config::from_string(json, Some("json")).unwrap();

        assert_eq!(config.get("name").unwrap().as_string().unwrap(), "test");
        assert_eq!(config.get("port").unwrap().as_integer().unwrap(), 8080);
    }

    #[test]
    fn test_merge_configs() {
        let mut config1 = Config::from_string("a = 1\nb = 2", Some("conf")).unwrap();
        let config2 = Config::from_string("b = 3\nc = 4", Some("conf")).unwrap();

        config1.merge(&config2).unwrap();

        assert_eq!(config1.get("a").unwrap().as_integer().unwrap(), 1); // Preserved
        assert_eq!(config1.get("b").unwrap().as_integer().unwrap(), 3); // Overridden
        assert_eq!(config1.get("c").unwrap().as_integer().unwrap(), 4); // Added
    }
}
