//! # JSON Format Parser
//!
//! Reads and writes the `json` format. Two backends sit behind the same
//! functions, and this module is the only place that knows which one is in
//! use:
//!
//! - feature `jsonx`: [`jsonx-lib`](https://docs.rs/jsonx-lib), strict
//!   RFC 8259. Duplicate keys are refused, and the default budgets of
//!   `jsonx-lib` apply (nesting depth 128, input 64 MiB, one string 16 MiB,
//!   one number 1 KiB, 1 Mi values, 64 MiB of tree). It wins when both
//!   features are on.
//! - feature `json` (alone): `serde_json`. A repeated key keeps its last
//!   value.
//!
//! Both give the same [`Value`] and write the same text for the same
//! document (tested with both features on), except that a document with a
//! repeated key is refused by `jsonx`, and the `jsonx` writer escapes U+2028
//! and U+2029 inside strings (`\u2028`, `\u2029`), which `serde_json` writes
//! raw. Both forms read back as the same string. Floats are written as the
//! shortest text that reads back as the same `f64`, with a signed positive
//! exponent (`1e+16`), as serde_json 1.0.149 and later write them.
//!
//! Numbers: an integer that fits `i64` becomes [`Value::Integer`]; any other
//! number (a fraction, an exponent, an integer beyond `i64`, and `-0`)
//! becomes [`Value::Float`]. A number too large for `f64` is a parse error.
//!
//! `from_json_value` and `to_json_value` convert to and from
//! `serde_json::Value` and exist only with the `json` feature.

use crate::error::Result;
use crate::value::Value;

/// Parse JSON format configuration
///
/// Any JSON value is accepted at the top level. Errors are
/// [`Error::Parse`](crate::Error::Parse) with the line and column (both
/// from 1) of the mistake; the message never quotes the input.
pub fn parse(source: &str) -> Result<Value> {
    #[cfg(feature = "jsonx")]
    {
        jsonx_backend::parse(source)
    }
    #[cfg(not(feature = "jsonx"))]
    {
        serde_backend::parse(source)
    }
}

/// Serialize config-lib Value back to JSON
///
/// Pretty form: two spaces per level, `": "` after keys, members in the
/// table's key order, no trailing newline. A non-finite float is refused
/// with [`Error::Type`](crate::Error::Type); a `DateTime` is written as its
/// RFC 3339 string.
pub fn serialize(value: &Value) -> Result<String> {
    #[cfg(feature = "jsonx")]
    {
        jsonx_backend::serialize(value, non_finite_type_error)
    }
    #[cfg(not(feature = "jsonx"))]
    {
        serde_backend::serialize(value)
    }
}

/// The JSON load used by the deprecated `EnterpriseConfig`. Its errors keep
/// the shape that type has always returned on the `serde_json` backend.
pub(crate) fn parse_enterprise(source: &str) -> Result<Value> {
    #[cfg(feature = "jsonx")]
    {
        jsonx_backend::parse(source)
    }
    #[cfg(not(feature = "jsonx"))]
    {
        serde_backend::parse_enterprise(source)
    }
}

/// The JSON save used by the deprecated `EnterpriseConfig`. The text is the
/// same as [`serialize`]; a non-finite float gives the error that type has
/// always returned.
pub(crate) fn serialize_enterprise(value: &Value) -> Result<String> {
    #[cfg(feature = "jsonx")]
    {
        jsonx_backend::serialize(value, non_finite_serialize_error)
    }
    #[cfg(not(feature = "jsonx"))]
    {
        serde_backend::serialize_enterprise(value)
    }
}

/// The error [`serialize`] gives for NaN and the infinities.
#[cfg(feature = "jsonx")]
fn non_finite_type_error(f: f64) -> crate::Error {
    crate::Error::type_error(f.to_string(), "valid JSON number", "float")
}

/// The error `EnterpriseConfig` gives for NaN and the infinities.
#[cfg(feature = "jsonx")]
fn non_finite_serialize_error(_: f64) -> crate::Error {
    crate::Error::serialize("Invalid float value".to_string())
}

/// Convert serde_json::Value to config-lib Value (alias for enterprise usage)
#[cfg(feature = "json")]
pub fn from_json_value(json_value: serde_json::Value) -> Result<Value> {
    serde_backend::convert_json_value(json_value)
}

/// Convert config-lib Value to serde_json::Value for enterprise usage
#[cfg(feature = "json")]
pub fn to_json_value(value: &Value) -> Result<serde_json::Value> {
    use crate::error::Error;
    match value {
        Value::Null => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(*i))),
        Value::Float(f) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(*f)
                .ok_or_else(|| Error::serialize("Invalid float value".to_string()))?,
        )),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Array(arr) => {
            let converted: Result<Vec<serde_json::Value>> = arr.iter().map(to_json_value).collect();
            Ok(serde_json::Value::Array(converted?))
        }
        Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (key, value) in table {
                map.insert(key.clone(), to_json_value(value)?);
            }
            Ok(serde_json::Value::Object(map))
        }
        #[cfg(feature = "chrono")]
        Value::DateTime(dt) => Ok(serde_json::Value::String(dt.to_rfc3339())),
    }
}

/// The `serde_json` backend: in use when `json` is on and `jsonx` is not.
/// With both on it still backs [`from_json_value`], and the parity tests
/// compare it with the `jsonx` backend.
#[cfg(feature = "json")]
#[cfg_attr(feature = "jsonx", allow(dead_code))]
mod serde_backend {
    use crate::error::{Error, Result};
    use crate::value::Value;
    use std::collections::BTreeMap;

    pub(super) fn parse(source: &str) -> Result<Value> {
        let json_value: serde_json::Value = serde_json::from_str(source)
            .map_err(|e| Error::parse(format!("JSON parse error: {e}"), e.line(), e.column()))?;

        convert_json_value(json_value)
    }

    pub(super) fn parse_enterprise(source: &str) -> Result<Value> {
        let parsed: serde_json::Value = serde_json::from_str(source)
            .map_err(|e| Error::general(format!("JSON parse error: {e}")))?;
        convert_json_value(parsed)
    }

    /// Convert serde_json::Value to config-lib Value
    pub(super) fn convert_json_value(json_value: serde_json::Value) -> Result<Value> {
        match json_value {
            serde_json::Value::Null => Ok(Value::Null),
            serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Float(f))
                } else {
                    Err(Error::parse(
                        format!("Invalid number: {n}"),
                        1,
                        1, // JSON parser doesn't give us position info
                    ))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s)),
            serde_json::Value::Array(arr) => {
                let converted: Result<Vec<Value>> =
                    arr.into_iter().map(convert_json_value).collect();
                Ok(Value::Array(converted?))
            }
            serde_json::Value::Object(obj) => {
                let mut converted = BTreeMap::new();
                for (key, value) in obj {
                    converted.insert(key, convert_json_value(value)?);
                }
                Ok(Value::Table(converted))
            }
        }
    }

    pub(super) fn serialize(value: &Value) -> Result<String> {
        let json_value = convert_to_json_value(value)?;
        serde_json::to_string_pretty(&json_value)
            .map_err(|e| Error::internal(format!("JSON serialization error: {e}")))
    }

    pub(super) fn serialize_enterprise(value: &Value) -> Result<String> {
        let json_value = super::to_json_value(value)?;
        serde_json::to_string_pretty(&json_value)
            .map_err(|e| Error::general(format!("JSON serialize error: {e}")))
    }

    /// Convert config-lib Value to serde_json::Value
    fn convert_to_json_value(value: &Value) -> Result<serde_json::Value> {
        match value {
            Value::Null => Ok(serde_json::Value::Null),
            Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
            Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
            Value::Float(f) => {
                if let Some(n) = serde_json::Number::from_f64(*f) {
                    Ok(serde_json::Value::Number(n))
                } else {
                    Err(Error::type_error(
                        f.to_string(),
                        "valid JSON number",
                        "float",
                    ))
                }
            }
            Value::String(s) => Ok(serde_json::Value::String(s.clone())),
            Value::Array(arr) => {
                let converted: Result<Vec<serde_json::Value>> =
                    arr.iter().map(convert_to_json_value).collect();
                Ok(serde_json::Value::Array(converted?))
            }
            Value::Table(table) => {
                let mut converted = serde_json::Map::new();
                for (key, value) in table {
                    converted.insert(key.clone(), convert_to_json_value(value)?);
                }
                Ok(serde_json::Value::Object(converted))
            }
            #[cfg(feature = "chrono")]
            Value::DateTime(dt) => Ok(serde_json::Value::String(dt.to_rfc3339())),
        }
    }
}

/// The `jsonx-lib` backend: in use whenever `jsonx` is on.
#[cfg(feature = "jsonx")]
mod jsonx_backend {
    use crate::error::{Error, Result};
    use crate::value::Value;
    use jsonx_lib::{Number, Writer};
    use std::borrow::Cow;
    use std::collections::{btree_map, BTreeMap};

    pub(super) fn parse(source: &str) -> Result<Value> {
        let doc = jsonx_lib::parse(source).map_err(|e| {
            let (line, column) = line_column(source, e.offset());
            // jsonx-lib's message is fixed text plus the byte offset; it
            // never quotes the input, which may hold secrets.
            Error::parse(format!("JSON parse error: {e}"), line, column)
        })?;
        convert(source, doc)
    }

    /// Line and column, both from 1, of byte `offset` in `source`. The
    /// column counts characters. A line ends at `\n`, `\r\n`, or `\r`, as
    /// jsonx-lib counts them.
    pub(super) fn line_column(source: &str, offset: usize) -> (usize, usize) {
        let bytes = source.as_bytes();
        let end = offset.min(bytes.len());
        let mut line = 1;
        let mut column = 1;
        let mut i = 0;
        while i < end {
            match bytes[i] {
                b'\n' => {
                    line += 1;
                    column = 1;
                }
                // `\r\n` is one line end, counted at the `\n`.
                b'\r' => {
                    if bytes.get(i + 1) != Some(&b'\n') {
                        line += 1;
                        column = 1;
                    }
                }
                // UTF-8 continuation bytes are part of the character before.
                b if b & 0xc0 == 0x80 => {}
                _ => column += 1,
            }
            i += 1;
        }
        (line, column)
    }

    /// One container being converted: the members still to read, and what
    /// has been built from the ones before.
    enum Building<'a> {
        Array {
            items: std::vec::IntoIter<jsonx_lib::Value<'a>>,
            built: Vec<Value>,
        },
        Table {
            members: std::vec::IntoIter<(Cow<'a, str>, jsonx_lib::Value<'a>)>,
            key: String,
            built: BTreeMap<String, Value>,
        },
    }

    impl<'a> Building<'a> {
        /// Takes the value built for the member last handed out.
        fn accept(&mut self, value: Value) {
            match self {
                Self::Array { built, .. } => built.push(value),
                Self::Table { key, built, .. } => {
                    built.insert(std::mem::take(key), value);
                }
            }
        }

        /// The next member to convert, if any.
        fn next_member(&mut self) -> Option<jsonx_lib::Value<'a>> {
            match self {
                Self::Array { items, .. } => items.next(),
                Self::Table { members, key, .. } => members.next().map(|(k, v)| {
                    *key = k.into_owned();
                    v
                }),
            }
        }

        fn finish(self) -> Value {
            match self {
                Self::Array { built, .. } => Value::Array(built),
                Self::Table { built, .. } => Value::Table(built),
            }
        }
    }

    /// Turns a parsed tree into a config-lib value, with an explicit stack
    /// rather than recursion, so the conversion's stack use does not grow
    /// with the nesting depth.
    fn convert(source: &str, doc: jsonx_lib::Value<'_>) -> Result<Value> {
        let mut open: Vec<Building<'_>> = Vec::new();
        let mut next = doc;
        loop {
            let mut done = match next {
                jsonx_lib::Value::Null => Some(Value::Null),
                jsonx_lib::Value::Bool(b) => Some(Value::Bool(b)),
                jsonx_lib::Value::Number(n) => Some(number(source, &n)?),
                jsonx_lib::Value::String(s) => Some(Value::String(s.into_owned())),
                jsonx_lib::Value::Array(items) => {
                    let items = items.into_vec();
                    open.push(Building::Array {
                        built: Vec::with_capacity(items.len()),
                        items: items.into_iter(),
                    });
                    None
                }
                jsonx_lib::Value::Object(members) => {
                    open.push(Building::Table {
                        members: members.into_vec().into_iter(),
                        key: String::new(),
                        built: BTreeMap::new(),
                    });
                    None
                }
            };
            // Hand finished values up until a container has another member.
            loop {
                let member = match open.last_mut() {
                    None => {
                        return done
                            .ok_or_else(|| Error::internal("JSON conversion lost its value"))
                    }
                    Some(building) => {
                        if let Some(value) = done.take() {
                            building.accept(value);
                        }
                        building.next_member()
                    }
                };
                match member {
                    Some(member) => {
                        next = member;
                        break;
                    }
                    None => done = open.pop().map(Building::finish),
                }
            }
        }
    }

    /// Integer text in `i64`'s range is an integer; everything else is the
    /// nearest `f64`. `-0` is the float `-0.0`, as `serde_json` reads it.
    fn number(source: &str, n: &Number<'_>) -> Result<Value> {
        let text = n.as_str();
        if text != "-0" {
            if let Some(i) = n.as_i64() {
                return Ok(Value::Integer(i));
            }
        }
        match n.as_f64() {
            Some(f) => Ok(Value::Float(f)),
            None => {
                // The number's text is a slice of `source`, so its address
                // gives its offset.
                let offset = (text.as_ptr() as usize).saturating_sub(source.as_ptr() as usize);
                let (line, column) = line_column(source, offset);
                Err(Error::parse(
                    format!("JSON parse error: number out of range at byte {offset}"),
                    line,
                    column,
                ))
            }
        }
    }

    /// One open container while writing.
    enum Open<'v> {
        Array(std::slice::Iter<'v, Value>),
        Table(btree_map::Iter<'v, String, Value>),
    }

    /// Writes `value` in the pretty layout, without recursion.
    pub(super) fn serialize(value: &Value, non_finite: fn(f64) -> Error) -> Result<String> {
        let mut out = String::new();
        let mut writer = Writer::new(&mut out).pretty();
        let mut open: Vec<Open<'_>> = Vec::new();
        let mut next = Some(value);
        loop {
            if let Some(value) = next.take() {
                match value {
                    Value::Array(items) => {
                        writer.begin_array().map_err(write_error)?;
                        open.push(Open::Array(items.iter()));
                    }
                    Value::Table(table) => {
                        writer.begin_object().map_err(write_error)?;
                        open.push(Open::Table(table.iter()));
                    }
                    scalar => write_scalar(&mut writer, scalar, non_finite)?,
                }
            }
            match open.last_mut() {
                None => break,
                Some(Open::Array(items)) => match items.next() {
                    Some(item) => next = Some(item),
                    None => {
                        writer.end_array().map_err(write_error)?;
                        open.pop();
                    }
                },
                Some(Open::Table(members)) => match members.next() {
                    Some((key, member)) => {
                        writer.key(key).map_err(write_error)?;
                        next = Some(member);
                    }
                    None => {
                        writer.end_object().map_err(write_error)?;
                        open.pop();
                    }
                },
            }
        }
        writer.finish().map_err(write_error)?;
        Ok(out)
    }

    fn write_scalar(
        writer: &mut Writer<'_>,
        value: &Value,
        non_finite: fn(f64) -> Error,
    ) -> Result<()> {
        match value {
            Value::Null => writer.value(&()),
            Value::Bool(b) => writer.value(b),
            Value::Integer(i) => writer.value(i),
            Value::Float(f) => {
                let Some(shortest) = Number::from_f64(*f) else {
                    return Err(non_finite(*f));
                };
                let text = signed_exponent(shortest.as_str());
                match Number::from_text(&text) {
                    Some(n) => writer.value(&n),
                    None => writer.value(&shortest),
                }
            }
            Value::String(s) => writer.value(s.as_str()),
            #[cfg(feature = "chrono")]
            Value::DateTime(dt) => writer.value(dt.to_rfc3339().as_str()),
            // Containers are opened by `serialize`.
            Value::Array(_) | Value::Table(_) => {
                return Err(Error::internal("JSON writer reached a container"))
            }
        }
        .map_err(write_error)
    }

    /// jsonx-lib writes `1e16`; serde_json (1.0.149 and later) writes
    /// `1e+16`. The digits and the layout are otherwise the same, so adding
    /// the sign to a positive exponent gives serde_json's text.
    fn signed_exponent(text: &str) -> Cow<'_, str> {
        match text.split_once('e') {
            Some((mantissa, exponent)) if !exponent.starts_with('-') => {
                Cow::Owned(format!("{mantissa}e+{exponent}"))
            }
            _ => Cow::Borrowed(text),
        }
    }

    fn write_error(e: jsonx_lib::WriteError) -> Error {
        Error::internal(format!("JSON serialization error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use std::collections::BTreeMap;

    #[test]
    fn test_basic_json() -> crate::Result<()> {
        let config = parse(
            r#"
        {
            "name": "test",
            "port": 8080,
            "database": {
                "host": "localhost"
            }
        }
        "#,
        )?;

        assert_eq!(
            config
                .get("name")
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: "name".to_string(),
                    available: vec![
                        "name".to_string(),
                        "port".to_string(),
                        "database".to_string()
                    ],
                })?
                .as_string()?,
            "test"
        );
        assert_eq!(
            config
                .get("port")
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: "port".to_string(),
                    available: vec![
                        "name".to_string(),
                        "port".to_string(),
                        "database".to_string()
                    ],
                })?
                .as_integer()?,
            8080
        );
        assert_eq!(
            config
                .get("database.host")
                .ok_or_else(|| crate::Error::KeyNotFound {
                    key: "database.host".to_string(),
                    available: vec![
                        "name".to_string(),
                        "port".to_string(),
                        "database".to_string()
                    ],
                })?
                .as_string()?,
            "localhost"
        );
        Ok(())
    }

    #[test]
    fn test_json_arrays() -> crate::Result<()> {
        let config = parse(
            r#"
        {
            "servers": ["alpha", "beta", "gamma"],
            "ports": [8001, 8002, 8003]
        }
        "#,
        )?;

        let servers = config
            .get("servers")
            .ok_or_else(|| crate::Error::KeyNotFound {
                key: "servers".to_string(),
                available: vec!["servers".to_string(), "ports".to_string()],
            })?
            .as_array()?;
        assert_eq!(servers.len(), 3);
        assert_eq!(servers[0].as_string()?, "alpha");

        let ports = config
            .get("ports")
            .ok_or_else(|| crate::Error::KeyNotFound {
                key: "ports".to_string(),
                available: vec!["servers".to_string(), "ports".to_string()],
            })?
            .as_array()?;
        assert_eq!(ports[0].as_integer()?, 8001);
        Ok(())
    }

    #[test]
    fn test_json_serialization() -> crate::Result<()> {
        let mut table = BTreeMap::new();
        table.insert("name".to_string(), Value::String("test".to_string()));
        table.insert("port".to_string(), Value::Integer(8080));
        let config = Value::Table(table);

        let json = serialize(&config)?;
        assert!(json.contains("\"name\": \"test\""));
        assert!(json.contains("\"port\": 8080"));
        Ok(())
    }

    // Behaviour both backends share.

    fn table(pairs: &[(&str, Value)]) -> Value {
        Value::Table(
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_string(), v.clone()))
                .collect(),
        )
    }

    #[test]
    fn numbers_integer_or_float() {
        let v = parse(
            r#"{"small": -42, "min": -9223372036854775808, "max": 9223372036854775807,
                "above_i64": 9223372036854775808, "above_u64": 123456789012345678901234567890,
                "below_i64": -9223372036854775809, "frac": 2.5, "exp": 1e2, "neg_zero": -0,
                "tiny": 1e-400}"#,
        )
        .unwrap();
        assert_eq!(v.get("small"), Some(&Value::Integer(-42)));
        assert_eq!(v.get("min"), Some(&Value::Integer(i64::MIN)));
        assert_eq!(v.get("max"), Some(&Value::Integer(i64::MAX)));
        assert_eq!(
            v.get("above_i64"),
            Some(&Value::Float(9_223_372_036_854_775_808.0))
        );
        assert_eq!(
            v.get("above_u64"),
            Some(&Value::Float(1.234_567_890_123_456_8e29))
        );
        assert_eq!(
            v.get("below_i64"),
            Some(&Value::Float(-9_223_372_036_854_775_808.0))
        );
        assert_eq!(v.get("frac"), Some(&Value::Float(2.5)));
        assert_eq!(v.get("exp"), Some(&Value::Float(100.0)));
        match v.get("neg_zero") {
            Some(Value::Float(f)) => assert!(*f == 0.0 && f.is_sign_negative()),
            other => panic!("-0 read as {other:?}"),
        }
        assert_eq!(v.get("tiny"), Some(&Value::Float(0.0)));
    }

    #[test]
    fn number_beyond_f64_refused() {
        for text in ["{\"a\":\n  1e400}", "[-1e400]"] {
            assert!(matches!(parse(text), Err(Error::Parse { .. })), "{text}");
        }
    }

    #[test]
    fn unicode_escapes() {
        let v = parse(
            r#"{"e": "caf\u00e9", "pair": "\ud83d\ude00", "raw": "日本", "ctl": "a\tb\u0001"}"#,
        )
        .unwrap();
        assert_eq!(v.get("e"), Some(&Value::String("caf\u{e9}".into())));
        assert_eq!(v.get("pair"), Some(&Value::String("\u{1f600}".into())));
        assert_eq!(v.get("raw"), Some(&Value::String("日本".into())));
        assert_eq!(v.get("ctl"), Some(&Value::String("a\tb\u{1}".into())));
        // A lone surrogate is not text.
        assert!(matches!(parse(r#"["\ud800"]"#), Err(Error::Parse { .. })));
    }

    /// Bad documents, and where the `jsonx` backend reports them.
    const INVALID: &[(&str, (usize, usize))] = &[
        ("{\"a\": 1,}", (1, 9)),
        ("{\"a\" 1}", (1, 6)),
        ("[1, 2", (1, 6)),
        ("", (1, 1)),
        ("{\"a\": tru}", (1, 10)),
        ("{\"a\": 01}", (1, 8)),
        ("{} {}", (1, 4)),
        ("{\n\"a\": [1,\n  ]}", (3, 3)),
    ];

    #[test]
    fn invalid_documents_refused() {
        for (text, _) in INVALID {
            assert!(matches!(parse(text), Err(Error::Parse { .. })), "{text:?}");
        }
    }

    #[test]
    fn top_level_need_not_be_an_object() {
        assert_eq!(
            parse("[1, true]").unwrap(),
            Value::Array(vec![Value::Integer(1), Value::Bool(true)])
        );
        assert_eq!(parse("\"s\"").unwrap(), Value::String("s".into()));
        assert_eq!(parse("null").unwrap(), Value::Null);
    }

    #[test]
    fn serialize_layout() {
        let v = table(&[
            (
                "b",
                Value::Array(vec![Value::Integer(1), Value::Float(1.5)]),
            ),
            ("a", table(&[])),
            ("c", Value::Array(vec![])),
            ("d", Value::String("q\"\\\n".into())),
            ("e", Value::Float(1.0)),
            ("f", Value::Null),
        ]);
        assert_eq!(
            serialize(&v).unwrap(),
            "{\n  \"a\": {},\n  \"b\": [\n    1,\n    1.5\n  ],\n  \"c\": [],\n  \"d\": \"q\\\"\\\\\\n\",\n  \"e\": 1.0,\n  \"f\": null\n}"
        );
    }

    #[test]
    fn serialize_refuses_non_finite() {
        for f in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let v = table(&[("x", Value::Float(f))]);
            assert!(matches!(serialize(&v), Err(Error::Type { .. })));
            assert!(matches!(
                serialize_enterprise(&v),
                Err(Error::General { .. })
            ));
        }
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn serialize_datetime_as_rfc3339() {
        let dt = chrono::DateTime::parse_from_rfc3339("2026-11-04T10:00:00+00:00")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let v = table(&[("t", Value::DateTime(dt))]);
        assert_eq!(
            serialize(&v).unwrap(),
            "{\n  \"t\": \"2026-11-04T10:00:00+00:00\"\n}"
        );
    }

    // The jsonx backend's own rules.

    #[cfg(feature = "jsonx")]
    mod jsonx {
        use super::super::{jsonx_backend, parse, serialize};
        use crate::{Error, Value};

        #[test]
        fn errors_carry_line_and_column() {
            for (text, at) in super::INVALID {
                match parse(text) {
                    Err(Error::Parse { line, column, .. }) => {
                        assert_eq!((line, column), *at, "{text:?}");
                    }
                    other => panic!("{text:?} gave {other:?}"),
                }
            }
            match parse("{\"a\":\n  1e400}") {
                Err(Error::Parse {
                    line,
                    column,
                    message,
                    ..
                }) => {
                    assert_eq!((line, column), (2, 3));
                    assert!(message.contains("out of range at byte 8"), "{message}");
                }
                other => panic!("1e400 gave {other:?}"),
            }
        }

        #[test]
        fn duplicate_keys_refused() {
            for (text, at) in [
                (r#"{"a": 1, "a": 2}"#, (1, 10)),
                ("{\"x\": {\"k\": 1,\n \"k\": 1}}", (2, 2)),
                (r#"{"a\u0062": 1, "ab": 2}"#, (1, 16)),
            ] {
                match parse(text) {
                    Err(Error::Parse {
                        line,
                        column,
                        message,
                        ..
                    }) => {
                        assert_eq!((line, column), at, "{text:?}");
                        assert!(message.contains("duplicate"), "{message}");
                    }
                    other => panic!("{text:?} gave {other:?}"),
                }
            }
        }

        #[test]
        fn error_never_quotes_the_input() {
            let text = "{\n  \"db\": {\n    \"password\": \"hunter2-secret\"\n    \"user\": \"admin\"\n  }\n}";
            let err = parse(text).unwrap_err();
            match &err {
                Error::Parse { line, column, .. } => assert_eq!((*line, *column), (4, 5)),
                other => panic!("expected a parse error, got {other:?}"),
            }
            let shown = format!("{err} {err:?}");
            assert!(!shown.contains("hunter2"), "{shown}");
            assert!(!shown.contains("admin"), "{shown}");
            assert!(!shown.contains("password"), "{shown}");
        }

        #[test]
        fn deep_nesting_refused_without_overflow() {
            // Run on a small stack: a recursive reader would overflow it.
            let worker = std::thread::Builder::new()
                .stack_size(256 * 1024)
                .spawn(|| {
                    let deep = "[".repeat(1_000_000);
                    let err = parse(&deep).unwrap_err();
                    let within = format!("{}1{}", "[".repeat(128), "]".repeat(128));
                    let ok = parse(&within).map(|_| ());
                    let over = format!("{}1{}", "{\"a\":".repeat(129), "}".repeat(129));
                    (err, ok, parse(&over).unwrap_err())
                })
                .unwrap();
            let (err, ok, over) = worker.join().unwrap();
            match err {
                Error::Parse {
                    line,
                    column,
                    message,
                    ..
                } => {
                    assert_eq!((line, column), (1, 129));
                    assert!(message.contains("depth"), "{message}");
                }
                other => panic!("expected a parse error, got {other:?}"),
            }
            assert!(ok.is_ok());
            assert!(matches!(over, Error::Parse { .. }));
        }

        #[test]
        fn deep_value_serializes_without_recursion() {
            // Deep enough that a recursive writer would overflow a 128 KiB
            // stack. The pretty layout's text grows with the square of the
            // depth, which keeps the test from going much deeper.
            let mut v = Value::Integer(1);
            for _ in 0..3_000 {
                v = Value::Array(vec![v]);
            }
            let worker = std::thread::Builder::new()
                .stack_size(128 * 1024)
                .spawn(move || {
                    let n = serialize(&v).map(|s| s.len());
                    // Taking the value apart one level at a time keeps the
                    // drop from recursing on this small stack.
                    let mut v = v;
                    while let Value::Array(mut items) = v {
                        v = items.pop().unwrap_or(Value::Null);
                    }
                    n
                })
                .unwrap();
            assert!(worker.join().unwrap().unwrap() > 3_000 * 3_000);
        }

        #[test]
        fn line_column_counts_like_jsonx() {
            for text in [
                "{\"a\": x}",
                "{\r\n\"a\":\r\n x}",
                "{\r\"a\":\r x}",
                "[\"\u{1f600}\",\n tru]",
                "[\"caf\u{e9}\", ?]",
                "\n\n\n",
            ] {
                let err = jsonx_lib::parse(text).unwrap_err();
                assert_eq!(
                    jsonx_backend::line_column(text, err.offset()),
                    err.line_col(text),
                    "{text:?}"
                );
            }
        }

        #[test]
        fn line_separators_escaped() {
            let v = Value::Array(vec![Value::String("a\u{2028}b\u{2029}".into())]);
            let text = serialize(&v).unwrap();
            assert_eq!(text, "[\n  \"a\\u2028b\\u2029\"\n]");
            assert_eq!(parse(&text).unwrap(), v);
        }
    }

    // With both features on, the two backends must agree.

    #[cfg(all(feature = "json", feature = "jsonx"))]
    mod parity {
        use super::super::{jsonx_backend, serde_backend};
        use crate::Value;

        const DOCUMENTS: &[&str] = &[
            "{}",
            "[]",
            "null",
            "true",
            "\"text\"",
            "0",
            "-0",
            "-0.0",
            "[1, -1, 2.5, -2.5, 1e2, 1E-2, 1.5e300, 5e-324, 1e-400]",
            "[9223372036854775807, -9223372036854775808, 9223372036854775808, 18446744073709551615, 18446744073709551616, -9223372036854775809]",
            "[123456789012345678901234567890, 0.1, 0.30000000000000004, 100000000000000000000, 1e15, 1e16, 1e-5, 1e-6, 123.456e-7]",
            "[3.141592653589793, 2.718281828459045, 1.7976931348623157e308, 2.2250738585072014e-308]",
            r#"{"name": "app", "port": 8080, "debug": false, "ratio": 0.75, "none": null}"#,
            r#"{"z": 1, "a": 2, "m": {"y": [1, {"b": 2, "a": 1}], "x": []}}"#,
            r#"{"s": "q\"b\\s/\b\f\n\r\t", "u": "caf\u00e9 \ud83d\ude00 \u0000 \u001f \u007f", "raw": "日本語 é"}"#,
            r#"{"html": "<script>&</script>", "slash": "a\/b", "empty": "", "": "empty key"}"#,
            r#"{"nested": [[[[[[[[[[1]]]]]]]]]], "objs": [{}, {"a": {}}, [[], {}]]}"#,
            " \t\r\n{ \"spaced\" :\t[ 1 , 2 ] }\n ",
        ];

        #[test]
        fn same_value_and_same_text() {
            for text in DOCUMENTS {
                let ours = jsonx_backend::parse(text).unwrap();
                let theirs = serde_backend::parse(text).unwrap();
                assert_eq!(bits(&ours), bits(&theirs), "value of {text}");
                assert_eq!(
                    jsonx_backend::serialize(&ours, super::super::non_finite_type_error).unwrap(),
                    serde_backend::serialize(&theirs).unwrap(),
                    "text of {text}"
                );
            }
        }

        #[test]
        fn same_text_for_built_values() {
            let mut floats = vec![
                0.0,
                -0.0,
                1.0,
                -1.0,
                0.1,
                1.0 / 3.0,
                1e15,
                1e16,
                1e17,
                1.5e-5,
                1e-5,
                1e-6,
                123_456_789.123,
                f64::MAX,
                f64::MIN_POSITIVE,
                5e-324,
            ];
            let mut x = 1.234_567_890_123_e-30;
            while x < 1e300 {
                floats.push(x);
                floats.push(-x * 7.0);
                x *= 13.7;
            }
            let v = Value::Array(floats.into_iter().map(Value::Float).collect());
            let ints = Value::Array(
                [0, 1, -1, i64::MIN, i64::MAX, 1 << 53]
                    .into_iter()
                    .map(Value::Integer)
                    .collect(),
            );
            for v in [v, ints] {
                let ours =
                    jsonx_backend::serialize(&v, super::super::non_finite_type_error).unwrap();
                assert_eq!(ours, serde_backend::serialize(&v).unwrap());
                assert_eq!(bits(&jsonx_backend::parse(&ours).unwrap()), bits(&v));
            }
        }

        #[test]
        fn same_verdict_on_bad_documents() {
            for text in [
                "",
                "{",
                "[1,]",
                "{\"a\":1,}",
                "{'a': 1}",
                "[01]",
                "[1.]",
                "[.5]",
                "[+1]",
                "[NaN]",
                "[Infinity]",
                "\u{feff}{}",
                "{\"a\" 1}",
                "[\"\\x\"]",
                "[\"\\ud800\"]",
                "[\"a\u{1}\"]",
                "{} x",
                "[1e400]",
                "[-1e400]",
                "tru",
                "nul",
            ] {
                assert!(
                    jsonx_backend::parse(text).is_err(),
                    "jsonx accepted {text:?}"
                );
                assert!(
                    serde_backend::parse(text).is_err(),
                    "serde_json accepted {text:?}"
                );
            }
        }

        #[test]
        fn only_duplicate_keys_differ() {
            let text = r#"{"a": 1, "a": 2}"#;
            assert!(jsonx_backend::parse(text).is_err());
            assert_eq!(
                serde_backend::parse(text).unwrap().get("a"),
                Some(&Value::Integer(2))
            );
        }

        /// A comparable form of a value: floats by their bits, so `-0.0`
        /// and `0.0` differ and NaN equals itself.
        fn bits(v: &Value) -> String {
            match v {
                Value::Float(f) => format!("f{:x}", f.to_bits()),
                Value::Array(items) => {
                    let inner: Vec<String> = items.iter().map(bits).collect();
                    format!("[{}]", inner.join(","))
                }
                Value::Table(t) => {
                    let inner: Vec<String> = t
                        .iter()
                        .map(|(k, v)| format!("{k:?}:{}", bits(v)))
                        .collect();
                    format!("{{{}}}", inner.join(","))
                }
                other => format!("{other:?}"),
            }
        }
    }
}
