//! # Basic Configuration Library Usage
//!
//! This example demonstrates the core functionality of config-lib
//! including parsing, accessing values, and format preservation.

use config_lib::{Config, Value};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Config-lib Basic Example\n");

    // 1. Create configuration from string
    let config_content = r#"
        # Application Configuration
        app_name = "demo-app"
        port = 8080
        debug = true
        version = 1.0
        
        # Array values  
        servers = ["alpha","beta","gamma"]
        allowed_ips = ["192.168.1.1","10.0.0.1","127.0.0.1"]
        
        # Database settings
        [database]
        host = "localhost"
        port = 5432
        username = "user"
        password = "secret"
        max_connections = 100
    "#;

    let mut config = Config::from_string(config_content, Some("conf"))?;

    println!("✅ Configuration loaded successfully!");
    println!("📄 Format: {}", config.format());
    println!("🔧 Modified: {}\n", config.is_modified());

    // 2. Access values with type safety
    println!("📋 Reading configuration values:");

    let app_name = config.get("app_name").unwrap().as_string()?;
    println!("  App Name: {}", app_name);

    let port = config.get("port").unwrap().as_integer()?;
    println!("  Port: {}", port);

    let debug = config.get("debug").unwrap().as_bool()?;
    println!("  Debug: {}", debug);

    let version = config.get("version").unwrap().as_float()?;
    println!("  Version: {}", version);

    // 3. Access nested values
    println!("\n🗄️  Database configuration:");
    let db_host = config.get("database.host").unwrap().as_string()?;
    let db_port = config.get("database.port").unwrap().as_integer()?;
    let max_conn = config
        .get("database.max_connections")
        .unwrap()
        .as_integer()?;

    println!("  Host: {}", db_host);
    println!("  Port: {}", db_port);
    println!("  Max Connections: {}", max_conn);

    // 4. Access arrays (now at root level)
    println!("\n📊 Array values:");

    let servers = config.get("servers");
    if let Some(servers_val) = servers {
        if let Ok(servers_array) = servers_val.as_array() {
            println!("  Servers: {:?}", servers_array);
        } else {
            println!("  Servers value is not an array: {:?}", servers_val);
        }
    } else {
        println!("  Servers not found");
    }

    let allowed_ips = config.get("allowed_ips");
    if let Some(ips_val) = allowed_ips {
        if let Ok(ips_array) = ips_val.as_array() {
            println!("  Allowed IPs: {:?}", ips_array);
        } else {
            println!("  Allowed IPs value is not an array: {:?}", ips_val);
        }
    } else {
        println!("  Allowed IPs not found");
    }

    // 5. Modify configuration
    println!("\n✏️  Modifying configuration:");
    config.set("port", 9000)?;
    config.set("database.timeout", "30s")?;
    config.set("new_feature", true)?;

    println!(
        "  ✓ Port changed to: {}",
        config.get("port").unwrap().as_integer()?
    );
    println!(
        "  ✓ Database timeout added: {}",
        config.get("database.timeout").unwrap().as_string()?
    );
    println!(
        "  ✓ New feature flag added: {}",
        config.get("new_feature").unwrap().as_bool()?
    );
    println!("  🔧 Modified: {}", config.is_modified());

    // 6. Work with Value directly
    println!("\n🎯 Direct Value manipulation:");
    let mut new_section = BTreeMap::new();
    new_section.insert("enabled".to_string(), Value::bool(true));
    new_section.insert("level".to_string(), Value::string("info"));
    new_section.insert("max_size".to_string(), Value::integer(1048576));

    config.set("logging", Value::table(new_section))?;

    println!("  ✓ Added logging section");
    println!(
        "    Enabled: {}",
        config.get("logging.enabled").unwrap().as_bool()?
    );
    println!(
        "    Level: {}",
        config.get("logging.level").unwrap().as_string()?
    );
    println!(
        "    Max Size: {}",
        config.get("logging.max_size").unwrap().as_integer()?
    );

    // 7. List all keys
    println!("\n🔑 Root level keys:");
    match config.keys() {
        Ok(keys) => {
            for key in keys {
                println!("  - {}", key);
            }
        }
        Err(e) => println!("Error getting keys: {}", e),
    }

    // 8. Serialize back to string
    println!("\n💾 Serialized configuration:");
    let serialized = config.serialize()?;
    println!("{}", serialized);

    println!("\n🎉 Example completed successfully!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let config_content = "key = value\nport = 8080";
        let config = Config::from_string(config_content, Some("conf")).unwrap();

        assert_eq!(config.get("key").unwrap().as_string().unwrap(), "value");
        assert_eq!(config.get("port").unwrap().as_integer().unwrap(), 8080);
    }

    #[test]
    fn test_nested_access() {
        let config_content = "[section]\nkey = value";
        let config = Config::from_string(config_content, Some("conf")).unwrap();

        assert_eq!(
            config.get("section.key").unwrap().as_string().unwrap(),
            "value"
        );
    }

    #[test]
    fn test_modification() {
        let config_content = "key = old_value";
        let mut config = Config::from_string(config_content, Some("conf")).unwrap();

        assert!(!config.is_modified());
        config.set("key", "new_value").unwrap();
        assert!(config.is_modified());
        assert_eq!(config.get("key").unwrap().as_string().unwrap(), "new_value");
    }
}
