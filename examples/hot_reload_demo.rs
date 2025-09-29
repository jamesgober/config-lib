use config_lib::hot_reload::HotReloadConfig;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

fn main() -> config_lib::Result<()> {
    println!("=== Hot Reload Configuration Demo ===");

    // Create a test config file
    let config_path = "demo_config.conf";
    let mut file = File::create(config_path)?;
    writeln!(file, "server_port=8080")?;
    writeln!(file, "debug_mode=false")?;
    writeln!(file, "max_connections=100")?;
    file.flush()?;
    drop(file);

    println!("1. Created initial config file: {}", config_path);

    // Create hot-reloadable configuration
    let (hot_config, event_receiver) = HotReloadConfig::from_file(config_path)?
        .with_poll_interval(Duration::from_millis(500)) // Check every 500ms
        .with_change_notifications();

    let config_ref = hot_config.config();

    // Start automatic watching
    let _handle = hot_config.start_watching();

    println!("2. Started hot reload monitoring (500ms polling)");

    // Display initial values
    {
        let config = config_ref.read().unwrap();
        println!("\n📊 Initial Configuration:");
        println!(
            "   Server Port: {}",
            config.get("server_port").unwrap().as_integer().unwrap()
        );
        println!(
            "   Debug Mode: {}",
            config.get("debug_mode").unwrap().as_bool().unwrap()
        );
        println!(
            "   Max Connections: {}",
            config.get("max_connections").unwrap().as_integer().unwrap()
        );
    }

    // Simulate application running and config changes
    for i in 1..=3 {
        println!("\n3.{} Waiting 2 seconds...", i);
        thread::sleep(Duration::from_secs(2));

        // Update configuration
        let mut file = File::create(config_path)?;
        writeln!(file, "server_port={}", 8080 + i * 10)?;
        writeln!(file, "debug_mode={}", i % 2 == 0)?;
        writeln!(file, "max_connections={}", 100 + i * 50)?;
        writeln!(file, "# Updated at iteration {}", i)?;
        file.flush()?;
        drop(file);

        println!("   📝 Updated config file (iteration {})", i);

        // Wait for reload and display new values
        thread::sleep(Duration::from_millis(600)); // Wait for polling + reload

        {
            let config = config_ref.read().unwrap();
            println!("   📊 New Configuration:");
            println!(
                "      Server Port: {}",
                config.get("server_port").unwrap().as_integer().unwrap()
            );
            println!(
                "      Debug Mode: {}",
                config.get("debug_mode").unwrap().as_bool().unwrap()
            );
            println!(
                "      Max Connections: {}",
                config.get("max_connections").unwrap().as_integer().unwrap()
            );
        }

        // Check for events
        while let Ok(event) = event_receiver.try_recv() {
            match event {
                config_lib::hot_reload::ConfigChangeEvent::FileModified { path, .. } => {
                    println!("   🔔 Event: File modified - {}", path.display());
                }
                config_lib::hot_reload::ConfigChangeEvent::Reloaded { path, .. } => {
                    println!(
                        "   ✅ Event: Configuration reloaded successfully - {}",
                        path.display()
                    );
                }
                config_lib::hot_reload::ConfigChangeEvent::ReloadFailed { path, error, .. } => {
                    println!(
                        "   ❌ Event: Reload failed for {} - {}",
                        path.display(),
                        error
                    );
                }
                config_lib::hot_reload::ConfigChangeEvent::FileDeleted { path, .. } => {
                    println!("   🗑️  Event: File deleted - {}", path.display());
                }
            }
        }
    }

    println!("\n4. Testing invalid configuration...");

    // Create invalid config to test error handling
    let mut file = File::create(config_path)?;
    writeln!(file, "invalid syntax here")?;
    writeln!(file, "===")?;
    file.flush()?;
    drop(file);

    println!("   📝 Created invalid config file");
    thread::sleep(Duration::from_millis(600));

    // Check for error events
    while let Ok(event) = event_receiver.try_recv() {
        if let config_lib::hot_reload::ConfigChangeEvent::ReloadFailed { path, error, .. } = event {
            println!(
                "   ✅ Properly caught error: {} - {}",
                path.display(),
                error
            );
        }
    }

    // Previous config should still be accessible
    {
        let config = config_ref.read().unwrap();
        println!("   📊 Previous valid config still available:");
        println!(
            "      Server Port: {}",
            config.get("server_port").unwrap().as_integer().unwrap()
        );
    }

    // Cleanup
    let _ = std::fs::remove_file(config_path);

    println!("\n✅ Hot reload demo completed successfully!");
    println!("   - Zero-downtime configuration updates ✓");
    println!("   - Event notifications ✓");
    println!("   - Error handling with fallback ✓");
    println!("   - Thread-safe concurrent access ✓");

    Ok(())
}
