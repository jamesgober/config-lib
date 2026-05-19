//! Cache-warm read latency benchmarks.
//!
//! Targets the v1.0 stability contract's "<50 ns single-key cached
//! get, 1 thread, warm" claim. Pre-populates a `Config` with a
//! representative tree and then hits `get_arc` on a known-warm path
//! in a tight loop.
//!
//! Run on the maintainer's reference hardware and commit the numbers
//! to `benches/baselines.json`; the v1.0 release notes cite those
//! committed numbers, not numbers from a developer laptop.

use config_lib::Config;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::hint::black_box as hint_black_box;

fn build_warm_config() -> Config {
    let mut cfg = Config::new();
    cfg.set("server.port", 8080i64).unwrap();
    cfg.set("server.host", "localhost").unwrap();
    cfg.set("database.host", "127.0.0.1").unwrap();
    cfg.set("database.port", 5432i64).unwrap();
    cfg.set("database.max_connections", 100i64).unwrap();
    cfg.set("logging.level", "info").unwrap();
    cfg.set("logging.format", "json").unwrap();
    cfg.set("cache.ttl_seconds", 300i64).unwrap();
    cfg.set("cache.size_mb", 256i64).unwrap();
    cfg.set("debug", false).unwrap();
    // Prime the cache so we measure the warm-path cost.
    for key in [
        "server.port",
        "server.host",
        "database.host",
        "database.port",
        "database.max_connections",
        "logging.level",
        "logging.format",
        "cache.ttl_seconds",
        "cache.size_mb",
        "debug",
    ] {
        let _ = cfg.get_arc(key);
    }
    cfg
}

fn bench_get_arc_warm_simple(c: &mut Criterion) {
    let cfg = build_warm_config();
    c.bench_function("get_arc_warm_simple", |b| {
        b.iter(|| {
            let v = cfg.get_arc(black_box("debug"));
            hint_black_box(v);
        });
    });
}

fn bench_get_arc_warm_nested(c: &mut Criterion) {
    let cfg = build_warm_config();
    c.bench_function("get_arc_warm_nested", |b| {
        b.iter(|| {
            let v = cfg.get_arc(black_box("database.max_connections"));
            hint_black_box(v);
        });
    });
}

fn bench_get_borrowed_warm_simple(c: &mut Criterion) {
    let cfg = build_warm_config();
    c.bench_function("get_borrowed_warm_simple", |b| {
        b.iter(|| {
            let v = cfg.get(black_box("debug"));
            hint_black_box(v);
        });
    });
}

fn bench_set_cached(c: &mut Criterion) {
    c.bench_function("set_cached", |b| {
        let mut cfg = build_warm_config();
        b.iter(|| {
            cfg.set(black_box("debug"), black_box(true)).unwrap();
        });
    });
}

criterion_group!(
    cache_warm,
    bench_get_arc_warm_simple,
    bench_get_arc_warm_nested,
    bench_get_borrowed_warm_simple,
    bench_set_cached,
);
criterion_main!(cache_warm);
