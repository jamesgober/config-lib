//! Concurrent-read scaling benchmarks.
//!
//! Targets the v1.0 stability contract's "<50 ns single-key cached
//! get sustained across 1-16 threads" claim. Spins N worker threads
//! that hammer `Config::get_arc` on the same warm `Arc<Config>` and
//! measures total throughput; criterion's parametric API records
//! per-thread-count numbers.
//!
//! The choice of DashMap as the cache backend (v0.9.9) is justified
//! by these numbers: sharded reads should scale near-linearly to ~16
//! threads on typical x86_64 hardware before false-sharing on shard
//! locks starts to matter. If the numbers say otherwise on
//! maintainer hardware, the backend choice gets revisited before
//! v1.0.

use config_lib::Config;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use std::thread;

fn build_warm() -> Arc<Config> {
    let mut cfg = Config::new();
    cfg.set("server.port", 8080i64).unwrap();
    cfg.set("server.host", "localhost").unwrap();
    cfg.set("database.host", "127.0.0.1").unwrap();
    cfg.set("database.max_connections", 100i64).unwrap();
    let _ = cfg.get_arc("server.port");
    let _ = cfg.get_arc("database.max_connections");
    Arc::new(cfg)
}

fn bench_concurrent_get_arc(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_get_arc");
    for thread_count in [1usize, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &threads| {
                let cfg = build_warm();
                b.iter(|| {
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let cfg = Arc::clone(&cfg);
                            thread::spawn(move || {
                                // 100 reads per worker per iteration
                                // — enough to amortize the spawn cost
                                // without leaving the criterion
                                // iteration so small the variance
                                // dominates.
                                for _ in 0..100 {
                                    let v = cfg.get_arc(black_box("server.port"));
                                    std::hint::black_box(v);
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(cache_concurrent, bench_concurrent_get_arc);
criterion_main!(cache_concurrent);
