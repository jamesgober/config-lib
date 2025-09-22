use config_lib::{audit::*, Config};
use std::fs::File;
use std::io::Write;

fn main() -> config_lib::Result<()> {
    println!("=== Comprehensive Audit Logging Demo ===");

    // Initialize audit logging with multiple sinks
    let audit_logger = AuditLogger::new()
        .add_sink(Box::new(ConsoleSink::new(AuditSeverity::Info)))
        .add_sink(Box::new(FileSink::new(
            "config_audit.log",
            AuditSeverity::Warning,
        )));

    init_audit_logger(audit_logger);

    println!("1. Initialized audit logging (console + file)");

    // Create test configuration
    let config_path = "audit_demo.conf";
    let mut file = File::create(config_path)?;
    writeln!(file, "database_host=localhost")?;
    writeln!(file, "database_port=5432")?;
    writeln!(file, "debug_enabled=false")?;
    file.flush()?;
    drop(file);

    // Load configuration (this should be audited)
    println!("\n2. Loading configuration...");
    let mut config = Config::from_file(config_path)?;

    // Log the initial load
    if let Some(logger) = get_audit_logger() {
        logger.log_event(
            AuditEvent::new(AuditEventType::Load, AuditSeverity::Info)
                .with_source(config_path)
                .with_user_context("demo_user")
                .with_metadata("format", "conf")
                .with_metadata("keys_loaded", "3"),
        );
    }

    // Simulate configuration access (audit each access)
    println!("\n3. Accessing configuration values...");

    // Access database host
    if let Some(logger) = get_audit_logger() {
        logger.log_access("database_host", Some("demo_user"));
    }
    let host = config.get("database_host").unwrap().as_string().unwrap();
    println!("   Database Host: {}", host);

    // Access database port
    if let Some(logger) = get_audit_logger() {
        logger.log_access("database_port", Some("demo_user"));
    }
    let port = config.get("database_port").unwrap().as_integer().unwrap();
    println!("   Database Port: {}", port);

    // Access debug setting
    if let Some(logger) = get_audit_logger() {
        logger.log_access("debug_enabled", Some("demo_user"));
    }
    let debug = config.get("debug_enabled").unwrap().as_bool().unwrap();
    println!("   Debug Enabled: {}", debug);

    // Simulate configuration modifications
    println!("\n4. Modifying configuration values...");

    // Modify database port
    let old_port = config.get("database_port").cloned();
    config.set("database_port", 5433)?;

    if let Some(logger) = get_audit_logger() {
        logger.log_modification(
            "database_port",
            old_port.as_ref(),
            &config_lib::Value::Integer(5433),
            Some("demo_user"),
        );
    }
    println!("   Changed database port: 5432 -> 5433");

    // Modify debug setting
    let old_debug = config.get("debug_enabled").cloned();
    config.set("debug_enabled", true)?;

    if let Some(logger) = get_audit_logger() {
        logger.log_modification(
            "debug_enabled",
            old_debug.as_ref(),
            &config_lib::Value::Bool(true),
            Some("demo_user"),
        );
    }
    println!("   Changed debug mode: false -> true");

    // Add new configuration key
    config.set("max_connections", 100)?;

    if let Some(logger) = get_audit_logger() {
        logger.log_modification(
            "max_connections",
            None,
            &config_lib::Value::Integer(100),
            Some("demo_user"),
        );
    }
    println!("   Added new key: max_connections = 100");

    // Simulate validation failures
    println!("\n5. Simulating validation failures...");

    // Invalid port range
    if let Some(logger) = get_audit_logger() {
        logger.log_validation_failure(
            "database_port",
            "Port must be between 1024 and 65535",
            &config_lib::Value::Integer(99999),
            Some("demo_user"),
        );
    }
    println!("   Validation failed: database_port=99999 (out of range)");

    // Invalid host format
    if let Some(logger) = get_audit_logger() {
        logger.log_validation_failure(
            "database_host",
            "Invalid hostname format",
            &config_lib::Value::String("invalid..host".to_string()),
            Some("demo_user"),
        );
    }
    println!("   Validation failed: database_host='invalid..host' (invalid format)");

    // Simulate reload events
    println!("\n6. Simulating configuration reloads...");

    // Successful reload
    if let Some(logger) = get_audit_logger() {
        logger.log_reload(config_path, true, None);
    }
    println!("   Successful reload from {}", config_path);

    // Failed reload
    if let Some(logger) = get_audit_logger() {
        logger.log_reload("invalid_config.conf", false, Some("File not found"));
    }
    println!("   Failed reload from invalid_config.conf");

    // Log custom audit events
    println!("\n7. Logging custom audit events...");

    if let Some(logger) = get_audit_logger() {
        // Security event
        logger.log_event(
            AuditEvent::new(AuditEventType::Access, AuditSeverity::Critical)
                .with_key("admin_password")
                .with_user_context("unauthorized_user")
                .with_metadata("ip_address", "192.168.1.100")
                .with_metadata("attempt_count", "3")
                .with_error("Unauthorized access attempt"),
        );

        // Performance event
        logger.log_event(
            AuditEvent::new(AuditEventType::Access, AuditSeverity::Warning)
                .with_key("large_dataset")
                .with_user_context("batch_processor")
                .with_metadata("processing_time_ms", "5000")
                .with_metadata("memory_usage_mb", "512"),
        );
    }

    println!("   Logged security and performance events");

    // Flush audit logs
    println!("\n8. Flushing audit logs...");
    if let Some(logger) = get_audit_logger() {
        logger.flush();
    }

    // Show file audit log contents
    println!("\n9. File audit log contents:");
    if let Ok(contents) = std::fs::read_to_string("config_audit.log") {
        for line in contents.lines() {
            println!("   {}", line);
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_file("config_audit.log");

    println!("\n✅ Audit logging demo completed successfully!");
    println!("   - Structured event logging ✓");
    println!("   - Multiple severity levels ✓");
    println!("   - User context tracking ✓");
    println!("   - Before/after value tracking ✓");
    println!("   - Multiple output sinks ✓");
    println!("   - Performance metadata ✓");
    println!("   - Security event detection ✓");

    Ok(())
}
