//! Configuration validation system
//!
//! Provides validation rules for configuration values

use crate::value::Value;
use std::fmt;

/// Trait for implementing custom validation rules
pub trait ValidationRule: Send + Sync {
    /// Returns the name of this validation rule
    fn name(&self) -> &str;
    /// Validates a value at a given path and returns the result
    fn validate(&self, path: &str, value: &Value) -> ValidationResult;
    /// Returns the priority of this rule (lower numbers = higher priority)
    fn priority(&self) -> u8 {
        50
    }
}

/// Result of a validation check
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Value passes validation
    Valid,
    /// Value fails validation with error details
    Invalid(ValidationError),
}

/// Detailed information about a validation failure
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// Path to the configuration key that failed validation
    pub path: String,
    /// Name of the validation rule that was violated
    pub rule: String,
    /// Human-readable error message describing the failure
    pub message: String,
    /// Severity level of this validation error
    pub severity: ValidationSeverity,
}

/// Severity levels for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ValidationSeverity {
    /// Critical error that must be fixed (severity 4)
    Critical = 4,
    /// Error that needs attention (severity 3)
    #[default]
    Error = 3,
    /// Warning that should be addressed (severity 2)
    Warning = 2,
    /// Informational message (severity 1)
    Info = 1,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.rule, self.path, self.message)
    }
}

impl ValidationError {
    /// Creates a new validation error with Error severity
    pub fn new(
        path: impl Into<String>,
        rule: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            rule: rule.into(),
            message: message.into(),
            severity: ValidationSeverity::Error,
        }
    }

    /// Sets the severity level and returns self for chaining
    pub fn with_severity(mut self, severity: ValidationSeverity) -> Self {
        self.severity = severity;
        self
    }
}



/// Collection of validation rules
#[derive(Default)]
pub struct ValidationRuleSet {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ValidationRuleSet {
    /// Creates a new empty validation rule set
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a validation rule to this set
    pub fn add_rule<R: ValidationRule + 'static>(mut self, rule: R) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Validates a value at the given path using all rules in this set
    pub fn validate(&mut self, path: &str, value: &Value) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Sort rules by priority (lower number = higher priority)
        self.rules.sort_by_key(|rule| rule.priority());

        for rule in &self.rules {
            if let ValidationResult::Invalid(error) = rule.validate(path, value) {
                errors.push(error);
            }
        }

        errors
    }

    /// Validates all values in a table recursively
    pub fn validate_all(
        &mut self,
        table: &std::collections::BTreeMap<String, Value>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (key, value) in table {
            errors.extend(self.validate(key, value));

            // Recursively validate nested tables
            if let Ok(nested_table) = value.as_table() {
                errors.extend(self.validate_all(nested_table));
            }
        }

        errors
    }
}

/// Value types for validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    /// String value type
    String,
    /// Integer value type
    Integer,
    /// Float value type
    Float,
    /// Boolean value type
    Boolean,
    /// Array value type
    Array,
    /// Table value type
    Table,
}

/// Validates that a value matches the expected type
#[derive(Debug)]
pub struct TypeValidator {
    expected_type: ValueType,
}

impl TypeValidator {
    /// Creates a new type validator for the specified type
    pub fn new(expected_type: ValueType) -> Self {
        Self { expected_type }
    }
}

impl ValidationRule for TypeValidator {
    fn name(&self) -> &str {
        "type_validator"
    }

    fn validate(&self, path: &str, value: &Value) -> ValidationResult {
        let matches = match self.expected_type {
            ValueType::String => value.as_string().is_ok(),
            ValueType::Integer => value.as_integer().is_ok(),
            ValueType::Float => value.as_float().is_ok(),
            ValueType::Boolean => value.as_bool().is_ok(),
            ValueType::Array => value.as_array().is_ok(),
            ValueType::Table => value.as_table().is_ok(),
        };

        if matches {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(ValidationError::new(
                path,
                self.name(),
                format!(
                    "Expected type {:?}, found different type",
                    self.expected_type
                ),
            ))
        }
    }

    fn priority(&self) -> u8 {
        10 // High priority - type checking should happen first
    }
}

/// Validates that numeric values fall within specified ranges
#[derive(Debug)]
pub struct RangeValidator {
    min: Option<f64>,
    max: Option<f64>,
}

impl RangeValidator {
    /// Creates a range validator with optional min and max bounds
    pub fn new(min: Option<f64>, max: Option<f64>) -> Self {
        Self { min, max }
    }

    /// Creates a range validator with only a minimum bound
    pub fn min(min: f64) -> Self {
        Self::new(Some(min), None)
    }

    /// Creates a range validator with only a maximum bound
    pub fn max(max: f64) -> Self {
        Self::new(None, Some(max))
    }
}

impl ValidationRule for RangeValidator {
    fn name(&self) -> &str {
        "range_validator"
    }

    fn validate(&self, path: &str, value: &Value) -> ValidationResult {
        let numeric_value = if let Ok(int_val) = value.as_integer() {
            int_val as f64
        } else if let Ok(float_val) = value.as_float() {
            float_val
        } else {
            // Not a numeric value, skip validation
            return ValidationResult::Valid;
        };

        if let Some(min) = self.min {
            if numeric_value < min {
                return ValidationResult::Invalid(ValidationError::new(
                    path,
                    self.name(),
                    format!("Value {} is below minimum {}", numeric_value, min),
                ));
            }
        }

        if let Some(max) = self.max {
            if numeric_value > max {
                return ValidationResult::Invalid(ValidationError::new(
                    path,
                    self.name(),
                    format!("Value {} exceeds maximum {}", numeric_value, max),
                ));
            }
        }

        ValidationResult::Valid
    }

    fn priority(&self) -> u8 {
        20 // After type validation
    }
}

/// Validates that required keys are present in table configurations
#[derive(Debug)]
pub struct RequiredKeyValidator {
    required_keys: Vec<String>,
}

impl RequiredKeyValidator {
    /// Creates a validator that checks for the presence of required keys
    pub fn new(required_keys: Vec<String>) -> Self {
        Self { required_keys }
    }

    /// Validates that all required keys are present in the configuration table
    pub fn validate_config(
        &self,
        config: &std::collections::BTreeMap<String, Value>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for key in &self.required_keys {
            if !config.contains_key(key) {
                errors.push(
                    ValidationError::new(
                        key,
                        "required_key_validator",
                        format!("Required key '{}' is missing", key),
                    )
                    .with_severity(ValidationSeverity::Critical),
                );
            }
        }

        errors
    }
}

impl ValidationRule for RequiredKeyValidator {
    fn name(&self) -> &str {
        "required_key_validator"
    }

    fn validate(&self, _path: &str, _value: &Value) -> ValidationResult {
        // This validator works at the table level, not individual values
        ValidationResult::Valid
    }

    fn priority(&self) -> u8 {
        5 // Very high priority - should check required keys first
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test_type_validator() {
        let validator = TypeValidator::new(ValueType::Integer);

        let int_value = Value::integer(42);
        assert_eq!(
            validator.validate("test", &int_value),
            ValidationResult::Valid
        );

        let string_value = Value::string("hello");
        matches!(
            validator.validate("test", &string_value),
            ValidationResult::Invalid(_)
        );
    }

    #[test]
    fn test_range_validator() {
        let validator = RangeValidator::new(Some(0.0), Some(100.0));

        let valid_value = Value::integer(50);
        assert_eq!(
            validator.validate("test", &valid_value),
            ValidationResult::Valid
        );

        let invalid_value = Value::integer(150);
        matches!(
            validator.validate("test", &invalid_value),
            ValidationResult::Invalid(_)
        );
    }

    #[test]
    fn test_required_key_validator() {
        let validator = RequiredKeyValidator::new(vec!["name".to_string(), "age".to_string()]);

        let mut config = std::collections::BTreeMap::new();
        config.insert("name".to_string(), Value::string("test"));
        config.insert("age".to_string(), Value::integer(25));

        let errors = validator.validate_config(&config);
        assert!(errors.is_empty());

        let mut incomplete_config = std::collections::BTreeMap::new();
        incomplete_config.insert("name".to_string(), Value::string("test"));

        let errors = validator.validate_config(&incomplete_config);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].path, "age");
    }

    #[test]
    fn test_validation_rule_set() {
        let mut rule_set = ValidationRuleSet::new()
            .add_rule(TypeValidator::new(ValueType::Integer))
            .add_rule(RangeValidator::new(Some(0.0), Some(100.0)));

        let valid_value = Value::integer(50);
        let errors = rule_set.validate("test", &valid_value);
        assert!(errors.is_empty());

        let invalid_value = Value::integer(150);
        let errors = rule_set.validate("test", &invalid_value);
        assert_eq!(errors.len(), 1);
    }
}
