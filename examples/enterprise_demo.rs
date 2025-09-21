#!/usr/bin/env rust

//! Enterprise Configuration Library - Performance Demo
//! 
//! Demonstrates the enterprise-grade configuration management with
//! caching, defaults, and multi-instance support.

use config_lib::{EnterpriseConfig, ConfigManager, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enterprise Configuration Library Demo");
    println!("==========================================");
    
    // Demo 1: Basic Enterprise Config with Caching
    println!("\n📦 Demo 1: Enterprise Config with Caching");
    let mut config = EnterpriseConfig::new();
    
    // Set some values
    config.set("server.host", Value::string("localhost"))?;
    config.set("server.port", Value::integer(8080))?;
    config.set("database.max_connections", Value::integer(100))?;
    config.set("debug", Value::bool(true))?;
    
    // Get values with zero-copy access
    println!("✅ Host: {}", config.get("server.host").unwrap().as_string().unwrap());
    println!("✅ Port: {}", config.get("server.port").unwrap().as_integer().unwrap());
    println!("✅ Max Connections: {}", config.get("database.max_connections").unwrap().as_integer().unwrap());
    println!("✅ Debug: {}", config.get("debug").unwrap().as_bool().unwrap());
    
    // Test nested key existence
    println!("✅ Exists server.host: {}", config.exists("server.host"));
    println!("✅ Exists server.timeout: {}", config.exists("server.timeout"));
    
    // Demo 2: Default Values
    println!("\n📦 Demo 2: Default Values");
    config.set_default("server.timeout", Value::integer(30));
    config.set_default("server.max_requests", Value::integer(1000));
    
    println!("✅ Timeout (from default): {}", 
             config.get_or_default("server.timeout").unwrap().as_integer().unwrap());
    println!("✅ Max Requests (from default): {}", 
             config.get_or_default("server.max_requests").unwrap().as_integer().unwrap());
    
    // Demo 3: Configuration Manager (Multi-instance)
    println!("\n📦 Demo 3: Multi-Instance Configuration Manager");
    let manager = ConfigManager::new();
    
    // Create temp config file
    std::fs::write("demo.conf", r#"
# Demo configuration
app_name = "Enterprise DB System"
version = "1.0.0"

[database]
driver = "postgresql"
host = "localhost"
port = 5432
max_connections = 50

[cache]
enabled = true
ttl = 300
size = "100MB"

[logging]
level = "info"
format = "json"
"#)?;
    
    // Load config
    manager.load("main", "demo.conf")?;
    println!("✅ Loaded configuration: main");
    
    // List configurations
    println!("✅ Available configs: {:?}", manager.list());
    
    // Demo 4: Parse from String (Direct API)
    println!("\n📦 Demo 4: Direct Parsing (High Performance)");
    use config_lib::enterprise::direct;
    
    let config_content = r#"
name = "High Performance System"
concurrency = 1000000
file_access_time = 50  # nanoseconds
efficiency = 0.90
"#;
    
    let value = direct::parse_string(config_content, Some("conf"))?;
    if let Value::Table(table) = value {
        println!("✅ System: {}", table.get("name").unwrap().as_string().unwrap());
        println!("✅ Concurrency: {}", table.get("concurrency").unwrap().as_integer().unwrap());
        println!("✅ File Access: {}ns", table.get("file_access_time").unwrap().as_integer().unwrap());
        println!("✅ Efficiency: {}%", (table.get("efficiency").unwrap().as_float().unwrap() * 100.0));
    }
    
    // Demo 5: Enterprise Features Summary
    println!("\n📦 Demo 5: Enterprise Performance Features");
    println!("✅ ⚡ Zero-copy parsing where possible");
    println!("✅ 🔄 In-memory caching for repeated access");
    println!("✅ 🎯 Default value system");
    println!("✅ 🔑 Nested key access (database.host)");
    println!("✅ 🗂️  Multi-instance configuration management");
    println!("✅ 🛡️  Read-only configurations");
    println!("✅ 💾 Format preservation for save operations");
    println!("✅ 🔧 Type-safe value access");
    
    // Cleanup
    std::fs::remove_file("demo.conf").ok();
    
    println!("\n🎉 Demo completed successfully!");
    println!("🚀 Ready for enterprise workloads!");
    
    Ok(())
}