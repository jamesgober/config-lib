//! `TryFrom<Value>` and `TryFrom<&Value>` for plain Rust types.
//!
//! These impls are what make [`Config::get_or`](crate::Config::get_or)
//! work with ordinary types:
//!
//! ```rust
//! use config_lib::Config;
//!
//! # fn main() -> Result<(), config_lib::Error> {
//! let config = Config::from_string("port = 8080\ndebug = true", None)?;
//!
//! let port: u16 = config.get_or("port", 3000);
//! let debug = config.get_or("debug", false);
//! let workers: usize = config.get_or("workers", 4); // missing, so the default
//!
//! assert_eq!(port, 8080);
//! assert!(debug);
//! assert_eq!(workers, 4);
//! # Ok(())
//! # }
//! ```
//!
//! Conversion rules:
//!
//! - Integer types (`i8` to `i64`, `isize`, `u8` to `u64`, `usize`)
//!   accept an integer value, a float with no fractional part, or a string
//!   that parses as the target type. The result is range-checked: `70000`
//!   does not convert to `u16`, and `8080.5` does not convert to any
//!   integer type.
//! - `f64` follows [`Value::as_float`]: a float, an integer, or a string
//!   that parses as a float.
//! - `bool` follows [`Value::as_bool`]: a boolean, or one of the strings
//!   `true`/`false`, `yes`/`no`, `on`/`off`, `1`/`0` (any case).
//! - `String` accepts a string value only.
//!
//! A failed conversion returns [`Error::Type`](crate::Error::Type).

use crate::error::{Error, Result};
use crate::value::Value;

/// The `i64` range as floats (`-2^63` inclusive, `2^63` exclusive), checked
/// before a float is cast to `i64`. Both bounds are exact in `f64`.
const I64_UPPER_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0; // 2^63
const I64_LOWER_INCLUSIVE: f64 = -9_223_372_036_854_775_808.0; // -2^63

fn integer_from(value: &Value, type_name: &'static str) -> Result<i64> {
    match value {
        Value::Integer(i) => Ok(*i),
        Value::Float(f)
            if f.is_finite()
                && f.fract() == 0.0
                && *f >= I64_LOWER_INCLUSIVE
                && *f < I64_UPPER_EXCLUSIVE =>
        {
            Ok(*f as i64)
        }
        Value::Float(_) => Err(Error::type_error(
            "float with a fractional part or out of range",
            type_name,
            value.type_name(),
        )),
        _ => Err(Error::type_error(
            format!("Cannot convert to {type_name}"),
            type_name,
            value.type_name(),
        )),
    }
}

macro_rules! impl_try_from_integer {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TryFrom<&Value> for $ty {
                type Error = Error;

                fn try_from(value: &Value) -> Result<Self> {
                    let name = stringify!($ty);
                    if let Value::String(s) = value {
                        return s.parse::<$ty>().map_err(|_| {
                            Error::type_error(
                                format!("string is not a valid {name}"),
                                name,
                                value.type_name(),
                            )
                        });
                    }
                    let wide = integer_from(value, name)?;
                    <$ty>::try_from(wide).map_err(|_| {
                        Error::type_error(
                            format!("{wide} is out of range for {name}"),
                            name,
                            value.type_name(),
                        )
                    })
                }
            }

            impl TryFrom<Value> for $ty {
                type Error = Error;

                fn try_from(value: Value) -> Result<Self> {
                    <$ty>::try_from(&value)
                }
            }
        )*
    };
}

impl_try_from_integer!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl TryFrom<&Value> for bool {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self> {
        value.as_bool()
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        value.as_bool()
    }
}

impl TryFrom<&Value> for f64 {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self> {
        value.as_float()
    }
}

impl TryFrom<Value> for f64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        value.as_float()
    }
}

impl TryFrom<&Value> for String {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self> {
        value.as_string().map(str::to_owned)
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(s),
            other => other.as_string().map(str::to_owned),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_are_range_checked() {
        assert_eq!(u16::try_from(Value::Integer(8080)).unwrap(), 8080);
        assert!(u16::try_from(Value::Integer(70_000)).is_err());
        assert!(u8::try_from(Value::Integer(-1)).is_err());
        assert_eq!(i8::try_from(Value::Integer(-128)).unwrap(), -128);
        assert!(i8::try_from(Value::Integer(-129)).is_err());
        assert_eq!(usize::try_from(Value::Integer(4)).unwrap(), 4);
    }

    #[test]
    fn integers_from_floats_must_be_whole() {
        assert_eq!(i64::try_from(Value::Float(8080.0)).unwrap(), 8080);
        assert!(i64::try_from(Value::Float(8080.5)).is_err());
        assert!(i64::try_from(Value::Float(f64::NAN)).is_err());
        assert!(i64::try_from(Value::Float(f64::INFINITY)).is_err());
        assert!(i64::try_from(Value::Float(1e19)).is_err());
        assert_eq!(
            i64::try_from(Value::Float(-9_223_372_036_854_775_808.0)).unwrap(),
            i64::MIN
        );
    }

    #[test]
    fn integers_from_strings_parse_into_the_target_type() {
        assert_eq!(u16::try_from(Value::from("443")).unwrap(), 443);
        assert!(u16::try_from(Value::from("65536")).is_err());
        assert!(u16::try_from(Value::from("port")).is_err());
        assert_eq!(
            u64::try_from(Value::from("18446744073709551615")).unwrap(),
            u64::MAX
        );
    }

    #[test]
    fn integers_refuse_other_types() {
        assert!(i64::try_from(Value::Bool(true)).is_err());
        assert!(i64::try_from(Value::Null).is_err());
        assert!(i64::try_from(Value::Array(vec![])).is_err());
    }

    #[test]
    fn bool_float_and_string() {
        assert!(bool::try_from(Value::Bool(true)).unwrap());
        assert!(!bool::try_from(Value::from("off")).unwrap());
        assert!(bool::try_from(Value::Integer(1)).is_err());

        assert_eq!(f64::try_from(Value::Float(1.5)).unwrap(), 1.5);
        assert_eq!(f64::try_from(Value::Integer(2)).unwrap(), 2.0);
        assert!(f64::try_from(Value::Bool(false)).is_err());

        assert_eq!(String::try_from(Value::from("app")).unwrap(), "app");
        assert_eq!(String::try_from(&Value::from("app")).unwrap(), "app");
        assert!(String::try_from(Value::Integer(1)).is_err());
    }

    #[test]
    fn errors_are_type_errors() {
        let err = u16::try_from(Value::Integer(70_000)).unwrap_err();
        assert!(matches!(err, Error::Type { .. }), "{err:?}");
    }
}
