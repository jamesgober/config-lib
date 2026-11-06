//! The `json` format through the public API, on whichever backend is
//! enabled (`jsonx`, or `json` alone). The backends' own unit tests, and the
//! parity tests that compare them, are in `src/parsers/json_parser.rs`.

#![cfg(any(feature = "json", feature = "jsonx"))]

use config_lib::{Config, Error, Value};

const DOC: &str = r#"{
  "app": "svc",
  "port": 8080,
  "ratio": 0.25,
  "big": 18446744073709551615,
  "tags": ["a", "bé"],
  "db": {"host": "localhost", "pool": {"max": 16}},
  "none": null
}"#;

#[test]
fn file_round_trip_through_save_and_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("service.json");
    std::fs::write(&path, DOC).unwrap();

    let mut config = Config::from_file(&path).unwrap();
    assert_eq!(config.format(), "json");
    assert_eq!(config.get("db.pool.max"), Some(&Value::Integer(16)));
    assert_eq!(
        config.get("big"),
        Some(&Value::Float(18_446_744_073_709_551_615.0))
    );
    assert_eq!(
        config.get("tags").unwrap().as_array().unwrap()[1]
            .as_string()
            .unwrap(),
        "b\u{e9}"
    );

    config.set("port", 9090).unwrap();
    config.set("db.pool.min", 2).unwrap();
    config.save().unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("{\n  \"app\": \"svc\",\n"), "{text}");
    let reloaded = Config::from_file(&path).unwrap();
    assert_eq!(reloaded.as_value(), config.as_value());
    assert_eq!(reloaded.get("port"), Some(&Value::Integer(9090)));
    assert_eq!(reloaded.get("db.pool.min"), Some(&Value::Integer(2)));
    // Saving again writes the same text.
    assert_eq!(reloaded.serialize().unwrap(), text);
}

#[test]
fn format_detected_from_content() {
    let config = Config::from_string(r#"{"key": "value"}"#, None).unwrap();
    assert_eq!(config.format(), "json");
}

#[test]
fn bad_document_is_a_parse_error() {
    let err = Config::from_string(r#"{"a": [1, 2,]}"#, Some("json")).unwrap_err();
    assert!(matches!(err, Error::Parse { .. }), "{err:?}");
}

#[test]
fn non_finite_float_not_written() {
    let mut config = Config::from_string("{}", Some("json")).unwrap();
    config.set("x", f64::NAN).unwrap();
    assert!(matches!(config.serialize(), Err(Error::Type { .. })));
}

#[cfg(feature = "jsonx")]
#[test]
fn jsonx_error_gives_position_and_never_the_secret() {
    let text = "{\n  \"token\": \"s3cr3t-value\",\n  \"retries\": 3,\n}";
    let err = Config::from_string(text, Some("json")).unwrap_err();
    match &err {
        Error::Parse { line, column, .. } => assert_eq!((*line, *column), (4, 1)),
        other => panic!("expected a parse error, got {other:?}"),
    }
    let shown = format!("{err} {err:?}");
    assert!(!shown.contains("s3cr3t"), "{shown}");
    assert!(!shown.contains("token"), "{shown}");
}

#[cfg(feature = "jsonx")]
#[test]
fn jsonx_refuses_duplicate_keys() {
    let err = Config::from_string("{\"user\": \"guest\",\n \"user\": \"admin\"}", Some("json"))
        .unwrap_err();
    match err {
        Error::Parse { line, column, .. } => assert_eq!((line, column), (2, 2)),
        other => panic!("expected a parse error, got {other:?}"),
    }
}

#[cfg(feature = "jsonx")]
#[test]
fn jsonx_refuses_deep_nesting() {
    let deep = format!("{{\"a\": {}{}}}", "[".repeat(200), "]".repeat(200));
    let err = Config::from_string(&deep, Some("json")).unwrap_err();
    assert!(matches!(err, Error::Parse { .. }), "{err:?}");
}

#[cfg(all(feature = "json", not(feature = "jsonx")))]
#[test]
fn serde_json_keeps_last_duplicate() {
    let config =
        Config::from_string(r#"{"user": "guest", "user": "admin"}"#, Some("json")).unwrap();
    assert_eq!(config.get("user"), Some(&Value::String("admin".into())));
}

#[test]
#[allow(deprecated)]
fn enterprise_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("enterprise.json");
    std::fs::write(&path, DOC).unwrap();

    let config = config_lib::EnterpriseConfig::from_file(&path).unwrap();
    assert_eq!(
        config.get("db.host"),
        Some(Value::String("localhost".into()))
    );
    let copy = dir.path().join("copy.json");
    config.save_to(&copy).unwrap();

    let a = Config::from_file(&path).unwrap();
    let b = Config::from_file(&copy).unwrap();
    assert_eq!(a.as_value(), b.as_value());
}

#[cfg(feature = "env-override")]
#[test]
fn env_override_reads_json_values() {
    use config_lib::env_override::EnvOverrideSystem;

    // A prefix no other test uses, so the process-wide environment is ours.
    std::env::set_var("CLJSONX_DB_POOL", r#"{"max": 32}"#);
    std::env::set_var("CLJSONX_TAGS", r#"["x"]"#);
    std::env::set_var("CLJSONX_APP", "{not json");

    let base = Config::from_string(DOC, Some("json")).unwrap();
    let system = EnvOverrideSystem::with_prefix("CLJSONX_");
    let value = system.apply_overrides(base.as_value().clone()).unwrap();

    let pool = value.get("db.pool").unwrap();
    assert_eq!(pool.get("max"), Some(&Value::Integer(32)));
    assert_eq!(
        value.get("tags"),
        Some(&Value::Array(vec![Value::String("x".into())]))
    );
    assert_eq!(value.get("app"), Some(&Value::String("{not json".into())));
}
