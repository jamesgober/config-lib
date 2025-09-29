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
            println!("✅ {key}: {value:?}");
        } else {
            println!("❌ {key} not found");
        }
    }

    // Test global keys that should work
    if let Some(app_name) = config.get("app_name") {
        println!("App Name: {}", app_name.as_string().unwrap());
    } else {
        println!("app_name key not found");
    }

    // Debug: print all available keys
    match config.keys() {
        Ok(keys) => {
            println!("\\n🔍 All available keys:");
            for key in keys {
                println!("  - {key}");
            }
        }
        Err(e) => println!("Error getting keys: {e}"),
    }

    Ok(())
}
