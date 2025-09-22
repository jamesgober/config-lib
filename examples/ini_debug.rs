use config_lib::Config;

fn main() -> config_lib::Result<()> {
    // Test INI file loading
    let config = Config::from_file("test.ini")?;

    println!("=== INI Configuration Debug ===");

    // Test various key combinations
    let test_keys = vec![
        "app_name",
        "version",
        "debug",
        "database.host",
        "database.port",
        "database.username",
        "logging.level",
        "features.feature1",
    ];

    for key in test_keys {
        if let Some(value) = config.get(key) {
            println!("✅ {}: {:?}", key, value);
        } else {
            println!("❌ {} not found", key);
        }
    }

    // Test global keys that should work
    if let Some(app_name) = config.get("app_name") {
        println!("App Name: {}", app_name.as_string().unwrap());
    } else {
        println!("app_name key not found");
    }

    // Test database.host
    if let Some(host) = config.get("database.host") {
        println!("Database Host: {}", host.as_string().unwrap());
    } else {
        println!("database.host key not found");
    }

    Ok(())
}
