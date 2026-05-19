//! Typed-accessor latency benchmarks.
//!
//! Targets the v1.0 stability contract's "<10 ns typed accessor"
//! claim. Measures the cost of `Value::as_string` / `as_integer` /
//! `as_bool` / `as_float` on already-resolved `&Value` references,
//! independent of the cache layer.

use config_lib::Value;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_as_integer(c: &mut Criterion) {
    let v = Value::integer(8080);
    c.bench_function("value_as_integer", |b| {
        b.iter(|| {
            let result = black_box(&v).as_integer();
            std::hint::black_box(result)
        });
    });
}

fn bench_as_string(c: &mut Criterion) {
    let v = Value::string("hello");
    c.bench_function("value_as_string", |b| {
        b.iter(|| {
            let result = black_box(&v).as_string();
            std::hint::black_box(result)
        });
    });
}

fn bench_as_bool(c: &mut Criterion) {
    let v = Value::bool(true);
    c.bench_function("value_as_bool", |b| {
        b.iter(|| {
            let result = black_box(&v).as_bool();
            std::hint::black_box(result)
        });
    });
}

fn bench_as_float(c: &mut Criterion) {
    // 4.2 (rather than 3.14) so clippy doesn't flag this as an
    // approximation of `f64::consts::PI`.
    let v = Value::float(4.2);
    c.bench_function("value_as_float", |b| {
        b.iter(|| {
            let result = black_box(&v).as_float();
            std::hint::black_box(result)
        });
    });
}

criterion_group!(
    value_accessors,
    bench_as_integer,
    bench_as_string,
    bench_as_bool,
    bench_as_float,
);
criterion_main!(value_accessors);
