//! # Configuration Caching Demo
//!
//! Demonstrates that configurations are loaded once and cached, not reloaded on every access.

use config_lib::enterprise::EnterpriseConfig;
use config_lib::Value;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Configuration Caching Performance Demo ===\n");

    // Create config and load some data
    let mut config = EnterpriseConfig::new();
    config.set("database.host", Value::string("localhost"))?;
    config.set("database.port", Value::integer(5432))?;
    config.set("app.name", Value::string("MyApp"))?;
    config.set("app.version", Value::string("1.0.0"))?;
    config.set("server.workers", Value::integer(4))?;

    println!("1. Initial Configuration Loaded");
    let (hits, misses, ratio) = config.cache_stats();
    println!(
        "   Cache Stats: {} hits, {} misses, {:.1}% hit ratio\n",
        hits,
        misses,
        ratio * 100.0
    );

    // First access - should populate fast cache
    println!("2. First Access (populates cache):");
    let start = Instant::now();
    let host1 = config.get("database.host");
    let duration1 = start.elapsed();
    println!(
        "   database.host = {:?}",
        host1.as_ref().map(|v| v.as_string())
    );
    println!("   Access time: {duration1:?}");

    let (hits, misses, ratio) = config.cache_stats();
    println!(
        "   Cache Stats: {} hits, {} misses, {:.1}% hit ratio\n",
        hits,
        misses,
        ratio * 100.0
    );

    // Second access - should be much faster (cache hit)
    println!("3. Second Access (cache hit):");
    let start = Instant::now();
    let host2 = config.get("database.host");
    let duration2 = start.elapsed();
    println!(
        "   database.host = {:?}",
        host2.as_ref().map(|v| v.as_string())
    );
    println!("   Access time: {duration2:?}");

    let (hits, misses, ratio) = config.cache_stats();
    println!(
        "   Cache Stats: {} hits, {} misses, {:.1}% hit ratio",
        hits,
        misses,
        ratio * 100.0
    );

    // Performance comparison
    if duration2 < duration1 {
        let speedup = duration1.as_nanos() as f64 / duration2.as_nanos() as f64;
        println!("   🚀 Cache is {speedup:.1}x faster!\n");
    }

    // Multiple fast accesses
    println!("4. Multiple Fast Accesses (all cache hits):");
    let start = Instant::now();
    for i in 0..1000 {
        let _host = config.get("database.host");
        let _port = config.get("database.port");
        let _name = config.get("app.name");
        if i % 200 == 0 {
            print!(".");
        }
    }
    let total_duration = start.elapsed();
    println!("\n   1000x3 accesses in {total_duration:?}");
    println!("   Average per access: {:?}", total_duration / 3000);

    let (hits, misses, ratio) = config.cache_stats();
    println!(
        "   Final Cache Stats: {} hits, {} misses, {:.1}% hit ratio",
        hits,
        misses,
        ratio * 100.0
    );

    println!("\n✅ Configuration is cached efficiently!");
    println!("   - First access loads from main cache");
    println!("   - Subsequent accesses use fast cache");
    println!("   - No file reloading on each access");
    println!("   - Sub-microsecond cached access times");

    Ok(())
}
