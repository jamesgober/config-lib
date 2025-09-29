use config_lib::Config;

fn main() -> config_lib::Result<()> {
    // Test INI file loading
    let config = Config::from_file("test.ini")?;

    println!("=== INI Configuration Test ===");

    // Test global keys
    println!(
        "App Name: {}",
        config.get("app_name").unwrap().as_string().unwrap()
    );
    println!(
        "Version: {}",
        config.get("version").unwrap().as_string().unwrap()
    );
    println!("Debug: {}", config.get("debug").unwrap().as_bool().unwrap());

    // Test section keys
    println!("\n=== Database Section ===");
    println!(
        "Host: {}",
        config
            .get("database.host")
            .and_then(|v| v.as_string().ok())
            .unwrap_or("<not found>")
    );
    println!(
        "Port: {}",
        config
            .get("database.port")
            .map(|v| v.as_integer().unwrap_or(-1))
            .unwrap_or(-1)
    );
    println!(
        "Username: {}",
        config
            .get("database.username")
            .unwrap()
            .as_string()
            .unwrap()
    );
    println!(
        "Password: {}",
        config
            .get("database.password")
            .unwrap()
            .as_string()
            .unwrap()
    );
    println!(
        "Pool Size: {}",
        config
            .get("database.pool_size")
            .unwrap()
            .as_integer()
            .unwrap()
    );

    // Test logging section
    println!("\n=== Logging Section ===");
    println!(
        "Level: {}",
        config.get("logging.level").unwrap().as_string().unwrap()
    );
    println!(
        "File: {}",
        config.get("logging.file").unwrap().as_string().unwrap()
    );
    println!(
        "Max Size: {}",
        config.get("logging.max_size").unwrap().as_string().unwrap()
    );

    // Test features with colon separator
    println!("\n=== Features Section ===");
    println!(
        "Feature1: {}",
        config
            .get("features.feature1")
            .unwrap()
            .as_string()
            .unwrap()
    );
    println!(
        "Feature2: {}",
        config
            .get("features.feature2")
            .unwrap()
            .as_string()
            .unwrap()
    );
    println!(
        "Experimental: {}",
        config
            .get("features.experimental")
            .unwrap()
            .as_bool()
            .unwrap()
    );

    println!("\n✅ INI format support is working correctly!");

    Ok(())
}
