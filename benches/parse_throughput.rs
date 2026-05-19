//! Cold-parse throughput benchmarks.
//!
//! Targets the v1.0 stability contract's "<10 µs cold parse — 1 KiB
//! CONF" and "<500 µs cold parse — 100 KiB JSON" claims. Each
//! benchmark parses a fixed-size representative input from scratch
//! (no cache, no warm-up) and reports the time-to-`Config`.

use config_lib::Config;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SMALL_CONF: &str = include_str!("../tests/fixtures/test.ini"); // ~200 B, mixed sections

fn bench_parse_small_conf(c: &mut Criterion) {
    c.bench_function("parse_small_conf", |b| {
        b.iter(|| {
            let cfg = Config::from_string(black_box(SMALL_CONF), Some("conf"));
            std::hint::black_box(cfg)
        });
    });
}

fn bench_parse_1kib_conf(c: &mut Criterion) {
    // Synthesize a ~1 KiB CONF body: 20 sections × ~50 B each.
    let body = (0..20)
        .map(|i| format!("[section_{i}]\nkey_a=value_a\nkey_b={i}\nkey_c=true\n"))
        .collect::<Vec<_>>()
        .join("\n");
    c.bench_function("parse_1kib_conf", |b| {
        b.iter(|| {
            let cfg = Config::from_string(black_box(&body), Some("conf"));
            std::hint::black_box(cfg)
        });
    });
}

#[cfg(feature = "json")]
fn bench_parse_100kib_json(c: &mut Criterion) {
    // Synthesize a ~100 KiB JSON body: 1000 entries × ~100 B each.
    let mut body = String::from("{\n");
    for i in 0..1000 {
        body.push_str(&format!(
            "  \"entry_{i}\": {{ \"a\": {i}, \"b\": \"value_{i}\", \"c\": true, \"d\": [1, 2, 3] }}"
        ));
        if i < 999 {
            body.push(',');
        }
        body.push('\n');
    }
    body.push('}');
    c.bench_function("parse_100kib_json", |b| {
        b.iter(|| {
            let cfg = Config::from_string(black_box(&body), Some("json"));
            std::hint::black_box(cfg)
        });
    });
}

#[cfg(feature = "json")]
criterion_group!(
    parse_throughput,
    bench_parse_small_conf,
    bench_parse_1kib_conf,
    bench_parse_100kib_json,
);

#[cfg(not(feature = "json"))]
criterion_group!(
    parse_throughput,
    bench_parse_small_conf,
    bench_parse_1kib_conf
);

criterion_main!(parse_throughput);
