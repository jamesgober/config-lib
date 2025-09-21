use config_lib::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;

// ENTERPRISE PERFORMANCE BENCHMARKS
// For database technology requiring 50ns file access and 1M+ concurrency

fn bench_config_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_parsing");
    
    // Small config for latency-critical operations
    let small_config = r#"
app_name = "test"
port = 8080
debug = true
host = "localhost"
"#;

    // Medium config for typical enterprise use
    let medium_config = r#"
app_name = "enterprise-db"
port = 8080
debug = false

[database]
host = "localhost"
port = 5432
user = "admin"
password = "secret"
max_connections = 100
timeout = 30

[logging]
level = "info"
file = "/var/log/app.log"
max_size = 1024
rotate = true

[cache]
size = 256
ttl = 3600
enabled = true

[performance]
threads = 8
queue_size = 10000
batch_size = 100
"#;

    // Large config for stress testing
    let large_config = generate_large_config(1000); // 1000 keys
    
    group.bench_function("small_config_parse", |b| {
        b.iter(|| {
            let config = Config::from_string(black_box(small_config), Some("conf")).unwrap();
            black_box(config);
        });
    });

    group.bench_function("medium_config_parse", |b| {
        b.iter(|| {
            let config = Config::from_string(black_box(medium_config), Some("conf")).unwrap();
            black_box(config);
        });
    });

    group.bench_function("large_config_parse", |b| {
        b.iter(|| {
            let config = Config::from_string(black_box(&large_config), Some("conf")).unwrap();
            black_box(config);
        });
    });

    group.finish();
}

fn bench_config_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_access");
    
    let config = Config::from_string(r#"
app_name = "test"
port = 8080
debug = true

[database]
host = "localhost"
port = 5432
user = "admin"
max_connections = 100

[cache]
size = 256
ttl = 3600
"#, Some("conf")).unwrap();

    group.bench_function("simple_key_access", |b| {
        b.iter(|| {
            let value = config.get(black_box("app_name")).unwrap();
            black_box(value);
        });
    });

    group.bench_function("nested_key_access", |b| {
        b.iter(|| {
            let value = config.get(black_box("database.host")).unwrap();
            black_box(value);
        });
    });

    group.bench_function("deep_nested_access", |b| {
        b.iter(|| {
            let value = config.get(black_box("database.max_connections")).unwrap();
            black_box(value);
        });
    });

    group.bench_function("type_conversion", |b| {
        b.iter(|| {
            let port = config.get("port").unwrap().as_integer().unwrap();
            black_box(port);
        });
    });

    group.finish();
}

fn bench_enterprise_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("enterprise_cache");
    
    let config = EnterpriseConfig::from_string(r#"
app_name = "enterprise-db"
port = 8080
cache_size = 1000
"#, Some("conf")).unwrap();

    group.bench_function("cached_access", |b| {
        b.iter(|| {
            let value = config.get(black_box("app_name"));
            black_box(value);
        });
    });

    group.bench_function("cache_miss_then_hit", |b| {
        b.iter(|| {
            // First access (cache miss)
            let value1 = config.get(black_box("port"));
            // Second access (cache hit)
            let value2 = config.get(black_box("port"));
            black_box((value1, value2));
        });
    });

    group.finish();
}

fn bench_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");
    
    let config = Arc::new(EnterpriseConfig::from_string(r#"
app_name = "enterprise-db"
port = 8080
threads = 16

[database]
host = "localhost"
port = 5432
max_connections = 1000
"#, Some("conf")).unwrap());

    // Simulate high concurrency typical of enterprise DB systems
    for thread_count in [1, 2, 4, 8, 16, 32].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_reads", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let mut handles = vec![];
                    
                    for _ in 0..thread_count {
                        let config_clone = Arc::clone(&config);
                        let handle = thread::spawn(move || {
                            // Simulate database-like access patterns
                            for _ in 0..100 {
                                let _app_name = config_clone.get("app_name");
                                let _port = config_clone.get("port");
                                let _db_host = config_clone.get("database.host");
                                let _max_conn = config_clone.get("database.max_connections");
                            }
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    group.bench_function("value_creation", |b| {
        b.iter(|| {
            let mut values = Vec::new();
            for i in 0..1000 {
                values.push(Value::integer(black_box(i)));
                values.push(Value::string(format!("value_{}", i)));
                values.push(Value::bool(i % 2 == 0));
            }
            black_box(values);
        });
    });

    group.bench_function("table_creation", |b| {
        b.iter(|| {
            let mut table = BTreeMap::new();
            for i in 0..100 {
                table.insert(format!("key_{}", i), Value::integer(i));
            }
            let value = Value::table(table);
            black_box(value);
        });
    });

    group.finish();
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let config = Config::from_string(generate_large_config(100).as_str(), Some("conf")).unwrap();

    group.bench_function("serialize_to_string", |b| {
        b.iter(|| {
            let serialized = config.serialize().unwrap();
            black_box(serialized);
        });
    });

    group.finish();
}

// ENTERPRISE: Stress test for 1M+ operations
fn bench_stress_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_test");
    group.sample_size(10); // Fewer samples for stress tests
    
    let config = Arc::new(EnterpriseConfig::from_string(r#"
app_name = "high-performance-db"
port = 8080
max_connections = 1000000
"#, Some("conf")).unwrap());

    group.bench_function("million_operations", |b| {
        b.iter(|| {
            let config_clone = Arc::clone(&config);
            
            // Simulate 1M operations in batches
            for batch in 0..1000 {
                for _ in 0..1000 {
                    let _value = config_clone.get("app_name");
                }
                if batch % 100 == 0 {
                    // Simulate some writes
                    let _ = config_clone.exists("port");
                }
            }
        });
    });

    group.finish();
}

fn generate_large_config(num_keys: usize) -> String {
    let mut config = String::new();
    
    for i in 0..num_keys {
        config.push_str(&format!("key_{} = \"value_{}\"\n", i, i));
        if i % 10 == 0 {
            config.push_str(&format!("[section_{}]\n", i / 10));
            config.push_str(&format!("nested_key = {}\n", i));
        }
    }
    
    config
}

criterion_group!(
    benches,
    bench_config_parsing,
    bench_config_access,
    bench_enterprise_cache,
    bench_concurrent_access,
    bench_memory_allocation,
    bench_serialization,
    bench_stress_test
);

criterion_main!(benches);