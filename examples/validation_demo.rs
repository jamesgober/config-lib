#[allow(unused_imports)]
use config_lib::Config;

#[cfg(feature = "validation")]
use config_lib::validation::{RangeValidator, TypeValidator, ValidationRuleSet, ValueType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Enterprise Configuration Validation Demo ===\n");

    #[cfg(not(feature = "validation"))]
    {
        println!("⚠️  Validation feature not enabled.");
        println!(
            "To use this demo, run with: cargo run --features validation --example validation_demo"
        );
        Ok(())
    }

    #[cfg(feature = "validation")]
    {
        // Create a configuration with validation rules
        let mut config = Config::from_string(
            r#"
            server_port = 8080
            database_host = "localhost"
            max_connections = 100
            timeout = 30.5
            debug_mode = true
        "#,
            Some("conf"),
        )?;

        // Set up validation rules
        let validation_rules = ValidationRuleSet::new()
            .add_rule(TypeValidator::new(ValueType::Integer)) // This will apply to all integers
            .add_rule(RangeValidator::new(Some(1.0), Some(65535.0))); // Port range for integers

        config.set_validation_rules(validation_rules);

        // Validate the configuration
        println!("Validating configuration...");
        match config.validate() {
            Ok(errors) => {
                if errors.is_empty() {
                    println!("✅ Configuration is valid!");
                } else {
                    println!("❌ Found validation errors:");
                    for error in errors {
                        println!("  {error}");
                    }
                }
            }
            Err(e) => {
                println!("❌ Validation failed: {e}");
            }
        }

        // Test with invalid configuration
        println!("\n=== Testing Invalid Configuration ===");
        let mut invalid_config = Config::from_string(
            r#"
        server_port = 70000
        database_host = "localhost"
        max_connections = -5
    "#,
            Some("conf"),
        )?;

        let validation_rules =
            ValidationRuleSet::new().add_rule(RangeValidator::new(Some(1.0), Some(65535.0)));

        invalid_config.set_validation_rules(validation_rules);

        match invalid_config.validate() {
            Ok(errors) => {
                if errors.is_empty() {
                    println!("✅ Configuration is valid!");
                } else {
                    println!("❌ Found validation errors:");
                    for error in errors {
                        println!("  {error}");
                    }
                }
            }
            Err(e) => {
                println!("❌ Validation failed: {e}");
            }
        }

        // Test critical validation only
        println!("\n=== Testing Critical Validation ===");
        match invalid_config.validate_critical_only() {
            Ok(critical_errors) => {
                println!("Critical errors found: {}", critical_errors.len());
            }
            Err(e) => {
                println!("❌ Critical validation failed: {e}");
            }
        }

        // Test is_valid convenience method
        println!("\n=== Testing is_valid() Method ===");
        match invalid_config.is_valid() {
            Ok(is_valid) => {
                println!("Configuration is valid: {is_valid}");
            }
            Err(e) => {
                println!("❌ Validation check failed: {e}");
            }
        }

        println!("\n=== Enterprise Validation System Demo Complete ===");

        Ok(())
    } // End of #[cfg(feature = "validation")]
}
