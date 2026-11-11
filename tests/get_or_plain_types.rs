//! Regression tests: `Config::get_or` with plain Rust types.
//!
//! Up to 1.3.1 config-lib implemented `TryFrom<Value>` for no plain type, so
//! `config.get_or("port", 8080)` did not compile (E0277) even though the
//! README taught it. These tests fail to build without the impls.

use config_lib::{Config, Value};

const SOURCE: &str = r#"
[server]
port = 8080
host = "0.0.0.0"
big = 70000
ratio = 0.75
whole = 3.0
half = 2.5
tls = true
quoted_port = "443"
"#;

fn config() -> Config {
    Config::from_string(SOURCE, Some("conf")).expect("test config parses")
}

#[test]
fn get_or_reads_present_values() {
    let c = config();
    let port: u16 = c.get_or("server.port", 1);
    let port_i64: i64 = c.get_or("server.port", 1);
    let host: String = c.get_or("server.host", String::new());
    let ratio: f64 = c.get_or("server.ratio", 0.0);
    let tls = c.get_or("server.tls", false);
    assert_eq!(port, 8080);
    assert_eq!(port_i64, 8080);
    assert_eq!(host, "0.0.0.0");
    assert!((ratio - 0.75).abs() < f64::EPSILON);
    assert!(tls);
}

#[test]
fn get_or_infers_from_the_default_literal() {
    let c = config();
    // `8080` alone infers i32; the documented one-liner must compile.
    assert_eq!(c.get_or("server.port", 8080), 8080);
    assert_eq!(c.get_or("server.missing", 8080), 8080);
}

#[test]
fn get_or_falls_back_when_missing() {
    let c = config();
    let workers: usize = c.get_or("server.workers", 4);
    let name: String = c.get_or("app.name", "DefaultApp".to_string());
    assert_eq!(workers, 4);
    assert_eq!(name, "DefaultApp");
}

#[test]
fn get_or_falls_back_when_out_of_range_or_wrong_type() {
    let c = config();
    let big: u16 = c.get_or("server.big", 1);
    let half: u32 = c.get_or("server.half", 1);
    let host_as_int: i64 = c.get_or("server.host", -1);
    let port_as_string: String = c.get_or("server.port", "none".to_string());
    assert_eq!(big, 1, "70000 does not fit u16");
    assert_eq!(half, 1, "2.5 is not a whole number");
    assert_eq!(host_as_int, -1);
    assert_eq!(port_as_string, "none");
}

#[test]
fn get_or_accepts_whole_floats_and_numeric_strings() {
    let c = config();
    let whole: u8 = c.get_or("server.whole", 0);
    let quoted: u16 = c.get_or("server.quoted_port", 0);
    assert_eq!(whole, 3);
    assert_eq!(quoted, 443);
}

#[test]
fn get_or_still_works_for_value() {
    let c = config();
    assert_eq!(c.get_or("server.port", Value::Null), Value::Integer(8080));
}

#[test]
fn try_from_value_is_public() {
    assert_eq!(u16::try_from(Value::Integer(8080)).ok(), Some(8080));
    assert_eq!(u16::try_from(&Value::Integer(8080)).ok(), Some(8080));
    assert!(u16::try_from(Value::Integer(-1)).is_err());
    assert_eq!(bool::try_from(Value::from("yes")).ok(), Some(true));
    assert_eq!(
        String::try_from(Value::from("x")).ok().as_deref(),
        Some("x")
    );
}
