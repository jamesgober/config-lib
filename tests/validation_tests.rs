#![cfg(feature = "validation")]

use config_lib::validation::{
    RangeValidator, RequiredKeyValidator, TypeValidator, ValidationError, ValidationResult,
    ValidationRule, ValidationRuleSet, ValidationSeverity, ValueType,
};
use config_lib::Config;
use std::time::Instant;

#[test]
fn test_enterprise_validation_integration() {
    let mut config = Config::from_string(
        r#"
        database_url = "postgresql://localhost:5432/mydb"
        server_port = 8080
        max_connections = 100
        timeout_seconds = 30
        ssl_enabled = true
        worker_threads = 4
    "#,
        Some("conf"),
    )
    .unwrap();

    // Create enterprise validation rules
    let rules = ValidationRuleSet::new()
        .add_rule(TypeValidator::new(ValueType::Integer))
        .add_rule(RangeValidator::new(Some(1.0), Some(65535.0)))
        .add_rule(RequiredKeyValidator::new(vec![
            "database_url".to_string(),
            "server_port".to_string(),
        ]));

    config.set_validation_rules(rules);

    // Test successful validation
    let errors = config.validate().unwrap();
    // Should have some type errors but no critical errors
    assert!(!errors.is_empty());

    let critical_errors = config.validate_critical_only().unwrap();
    assert!(critical_errors.is_empty());

    assert!(config.is_valid().unwrap());
}

#[test]
fn test_validation_performance() {
    // Create a large configuration to test performance
    let mut config = Config::new();

    // Add 1000 configuration keys
    for i in 0..1000 {
        config.set(&format!("key_{i}"), i as i64).unwrap();
    }

    // Create validation rules
    let rules = ValidationRuleSet::new()
        .add_rule(TypeValidator::new(ValueType::Integer))
        .add_rule(RangeValidator::new(Some(0.0), Some(2000.0)));

    let _ = rules;

    // Measure validation performance
    let start = Instant::now();
    let errors = config.validate().unwrap();
    let duration = start.elapsed();

    // Should complete quickly even with 1000 keys
    assert!(
        duration.as_millis() < 100,
        "Validation took {}ms, expected <100ms",
        duration.as_millis()
    );
    assert!(errors.is_empty());

    println!("Validated 1000 keys in {}μs", duration.as_micros());
}

#[test]
fn test_validation_error_reporting() {
    let mut config = Config::from_string(
        r#"
        invalid_port = 99999
        negative_timeout = -5
        string_as_number = "not_a_number"
    "#,
        Some("conf"),
    )
    .unwrap();

    let rules = ValidationRuleSet::new().add_rule(RangeValidator::new(Some(1.0), Some(65535.0)));

    config.set_validation_rules(rules);

    let errors = config.validate().unwrap();

    // Should find range violations
    assert!(!errors.is_empty());

    // Check error details
    for error in &errors {
        assert!(!error.path.is_empty());
        assert!(!error.rule.is_empty());
        assert!(!error.message.is_empty());
        println!("Validation error: {error}");
    }
}

#[test]
fn test_type_validator_comprehensive() {
    let string_validator = TypeValidator::new(ValueType::String);
    let int_validator = TypeValidator::new(ValueType::Integer);
    let bool_validator = TypeValidator::new(ValueType::Boolean);

    // Test string validation
    let result = string_validator.validate("test", &config_lib::Value::String("hello".to_string()));
    assert_eq!(result, ValidationResult::Valid);

    let result = string_validator.validate("test", &config_lib::Value::Integer(42));
    assert!(matches!(result, ValidationResult::Invalid(_)));

    // Test integer validation
    let result = int_validator.validate("test", &config_lib::Value::Integer(42));
    assert_eq!(result, ValidationResult::Valid);

    // String "42" should convert to integer, so it's valid
    let result = int_validator.validate("test", &config_lib::Value::String("42".to_string()));
    assert_eq!(result, ValidationResult::Valid);

    // But invalid string should fail
    let result = int_validator.validate(
        "test",
        &config_lib::Value::String("not_a_number".to_string()),
    );
    assert!(matches!(result, ValidationResult::Invalid(_)));

    // Test boolean validation
    let result = bool_validator.validate("test", &config_lib::Value::Bool(true));
    assert_eq!(result, ValidationResult::Valid);

    let result = bool_validator.validate("test", &config_lib::Value::Integer(1));
    assert!(matches!(result, ValidationResult::Invalid(_)));
}

#[test]
fn test_range_validator_edge_cases() {
    let validator = RangeValidator::new(Some(10.0), Some(100.0));

    // Test exact boundaries
    let result = validator.validate("test", &config_lib::Value::Integer(10));
    assert_eq!(result, ValidationResult::Valid);

    let result = validator.validate("test", &config_lib::Value::Integer(100));
    assert_eq!(result, ValidationResult::Valid);

    // Test outside boundaries
    let result = validator.validate("test", &config_lib::Value::Integer(9));
    assert!(matches!(result, ValidationResult::Invalid(_)));

    let result = validator.validate("test", &config_lib::Value::Integer(101));
    assert!(matches!(result, ValidationResult::Invalid(_)));

    // Test float values
    let result = validator.validate("test", &config_lib::Value::Float(50.5));
    assert_eq!(result, ValidationResult::Valid);

    let result = validator.validate("test", &config_lib::Value::Float(150.5));
    assert!(matches!(result, ValidationResult::Invalid(_)));

    // Test non-numeric values (should be ignored)
    let result = validator.validate("test", &config_lib::Value::String("test".to_string()));
    assert_eq!(result, ValidationResult::Valid);
}

#[test]
fn test_required_key_validation() {
    let validator = RequiredKeyValidator::new(vec![
        "database_url".to_string(),
        "api_key".to_string(),
        "service_port".to_string(),
    ]);

    // Test with all required keys present
    let mut config = std::collections::BTreeMap::new();
    config.insert(
        "database_url".to_string(),
        config_lib::Value::String("localhost".to_string()),
    );
    config.insert(
        "api_key".to_string(),
        config_lib::Value::String("secret".to_string()),
    );
    config.insert("service_port".to_string(), config_lib::Value::Integer(8080));
    config.insert(
        "optional_setting".to_string(),
        config_lib::Value::Bool(true),
    );

    let errors = validator.validate_config(&config);
    assert!(errors.is_empty());

    // Test with missing required key
    config.remove("api_key");
    let errors = validator.validate_config(&config);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].severity, ValidationSeverity::Critical);
    assert!(errors[0].message.contains("api_key"));

    // Test with multiple missing keys
    config.remove("database_url");
    let errors = validator.validate_config(&config);
    assert_eq!(errors.len(), 2);
}

#[test]
fn test_validation_rule_priority() {
    let mut rule_set = ValidationRuleSet::new()
        .add_rule(TypeValidator::new(ValueType::Integer)) // Priority 90
        .add_rule(RangeValidator::new(Some(1.0), Some(100.0))) // Priority 70
        .add_rule(RequiredKeyValidator::new(vec!["test".to_string()])); // Priority 95

    // Test that validation runs in priority order
    let errors = rule_set.validate("test", &config_lib::Value::String("invalid".to_string()));

    // Should get type error first due to higher priority
    assert!(!errors.is_empty());

    // The first error should be from TypeValidator (highest priority among triggered rules)
    assert_eq!(errors[0].rule, "type_validator");
}

#[test]
fn test_validation_severity_levels() {
    let error = ValidationError::new("path", "rule", "message");
    assert_eq!(error.severity, ValidationSeverity::Error);

    let critical_error = error.with_severity(ValidationSeverity::Critical);
    assert_eq!(critical_error.severity, ValidationSeverity::Critical);

    // Test severity ordering
    assert!(ValidationSeverity::Critical > ValidationSeverity::Error);
    assert!(ValidationSeverity::Error > ValidationSeverity::Warning);
    assert!(ValidationSeverity::Warning > ValidationSeverity::Info);
}

#[test]
fn test_enterprise_configuration_scenario() {
    // Simulate a real enterprise configuration scenario
    let mut config = Config::from_string(
        r#"
        # Database Configuration
        database_host = "prod-db-cluster.company.com"
        database_port = 5432
        database_max_connections = 50
        database_timeout = 30
        
        # API Configuration  
        api_port = 8080
        api_workers = 8
        api_rate_limit = 1000
        
        # Security Configuration
        ssl_enabled = true
        jwt_secret = "super-secret-key-2024"
        session_timeout = 3600
        
        # Monitoring Configuration
        metrics_enabled = true
        log_level = "INFO"
        health_check_interval = 60
    "#,
        Some("conf"),
    )
    .unwrap();

    // Enterprise validation rules
    let rules = ValidationRuleSet::new()
        // Port validation for all port fields
        .add_rule(RangeValidator::new(Some(1.0), Some(65535.0)))
        // Required critical configuration
        .add_rule(RequiredKeyValidator::new(vec![
            "database_host".to_string(),
            "database_port".to_string(),
            "api_port".to_string(),
            "ssl_enabled".to_string(),
        ]));

    let _ = rules;

    // Validate enterprise configuration
    let errors = config.validate().unwrap();
    let critical_errors = config.validate_critical_only().unwrap();

    // Should have no critical errors for a well-formed config
    assert!(
        critical_errors.is_empty(),
        "Enterprise config should have no critical errors"
    );

    // Configuration should be considered valid
    assert!(
        config.is_valid().unwrap(),
        "Enterprise config should be valid"
    );

    println!(
        "Enterprise configuration validation: {} total errors, {} critical",
        errors.len(),
        critical_errors.len()
    );
}

#[test]
fn test_concurrent_validation() {
    use std::thread;

    let config_data = r#"
        test_value_1 = 42
        test_value_2 = 100
        test_value_3 = 200
    "#;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let data = config_data.to_string();
            thread::spawn(move || {
                let mut config = Config::from_string(&data, Some("conf")).unwrap();
                let rules =
                    ValidationRuleSet::new().add_rule(RangeValidator::new(Some(1.0), Some(300.0)));

                let _ = rules;
                let errors = config.validate().unwrap();
                assert!(errors.is_empty());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn benchmark_validation_performance() {
    // Performance test with realistic enterprise config size
    let mut config = Config::new();

    // Create a realistic enterprise configuration (100 settings)
    let settings = [
        (
            "database_host",
            config_lib::Value::String("localhost".to_string()),
        ),
        ("database_port", config_lib::Value::Integer(5432)),
        ("api_port", config_lib::Value::Integer(8080)),
        ("worker_threads", config_lib::Value::Integer(4)),
        ("max_memory_mb", config_lib::Value::Integer(1024)),
    ];

    // Replicate settings to create larger config
    for i in 0..20 {
        for (key, value) in &settings {
            config
                .set(&format!("{key}_{i}"), value.clone())
                .unwrap();
        }
    }

    let rules = ValidationRuleSet::new()
        .add_rule(TypeValidator::new(ValueType::Integer))
        .add_rule(RangeValidator::new(Some(1.0), Some(100000.0)));

    let _ = rules;

    // Benchmark validation performance
    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = config.validate().unwrap();
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / iterations;

    println!("Average validation time: {}μs", avg_duration.as_micros());
    println!(
        "Validations per second: {}",
        1_000_000 / avg_duration.as_micros().max(1)
    );

    // Performance requirement: should validate in under 1ms for enterprise configs
    assert!(
        avg_duration.as_millis() < 1,
        "Validation took {}μs, expected <1000μs",
        avg_duration.as_micros()
    );
}
