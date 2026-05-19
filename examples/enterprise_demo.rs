//! # Enterprise Configuration Demo (migrated to unified `Config` API)
//!
//! This example was originally written against the now-deprecated
//! `EnterpriseConfig` and demonstrated multi-tier caching, defaults,
//! and the multi-instance `ConfigManager` primitive.
//!
//! As of v0.9.4 the canonical API is the unified [`config_lib::Config`].
//! It exposes every operation `EnterpriseConfig` had — with `&Value`
//! borrowed returns instead of owned clones — and v0.9.5 will give it
//! the same multi-tier caching that `EnterpriseConfig` provides today.
//!
//! Items in this example that are NOT YET on `Config` and land in v0.9.5
//! are noted inline as `TODO(v0.9.5)`. See `.dev/ROADMAP.md`.

use config_lib::{Config, ConfigOptions, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Configuration Library Demo (v0.9.4 unified Config API)");
    println!("======================================================");

    // -----------------------------------------------------------------
    // Demo 1: Basic Config with mutation
    // -----------------------------------------------------------------
    println!("\nDemo 1: Config with set/get");
    let mut config = Config::new();

    config.set("server.host", Value::string("localhost"))?;
    config.set("server.port", Value::integer(8080))?;
    config.set("database.max_connections", Value::integer(100))?;
    config.set("debug", Value::bool(true))?;

    println!(
        "  host:     {}",
        config
            .get("server.host")
            .ok_or("missing server.host")?
            .as_string()?
    );
    println!(
        "  port:     {}",
        config
            .get("server.port")
            .ok_or("missing server.port")?
            .as_integer()?
    );
    println!(
        "  max conn: {}",
        config
            .get("database.max_connections")
            .ok_or("missing database.max_connections")?
            .as_integer()?
    );
    println!(
        "  debug:    {}",
        config.get("debug").ok_or("missing debug")?.as_bool()?
    );

    // Nested key existence
    println!(
        "  exists server.host:    {}",
        config.contains_key("server.host")
    );
    println!(
        "  exists server.timeout: {}",
        config.contains_key("server.timeout")
    );

    // -----------------------------------------------------------------
    // Demo 2: Default-value lookup via explicit `get(..).and_then(..).unwrap_or(..)`.
    //
    // `EnterpriseConfig` had a separate `set_default(...)` + `get_or_default(...)`
    // pair backed by a parallel defaults table. The unified `Config` collapses
    // this to the same `Option`-chain pattern users already know from `HashMap`.
    // The richer separate-defaults-table feature returns in v0.9.5 once the
    // caching layer lands and `ConfigOptions` gains a `defaults` field.
    // -----------------------------------------------------------------
    println!("\nDemo 2: Default-value lookup");
    let timeout = config
        .get("server.timeout")
        .and_then(|v| v.as_integer().ok())
        .unwrap_or(30);
    let max_requests = config
        .get("server.max_requests")
        .and_then(|v| v.as_integer().ok())
        .unwrap_or(1000);
    println!("  timeout       (default): {timeout}");
    println!("  max_requests  (default): {max_requests}");

    // -----------------------------------------------------------------
    // Demo 3: Read-only mode (new in v0.9.4).
    //
    // `EnterpriseConfig::make_read_only()` becomes the [`ConfigOptions::read_only`]
    // knob on the unified API. Construction is explicit and intent is checked
    // at every mutating call.
    // -----------------------------------------------------------------
    println!("\nDemo 3: Read-only configuration via ConfigOptions");
    let mut locked = Config::with_options(ConfigOptions::new().read_only(true));
    let attempt = locked.set("foo", "bar");
    println!("  set() on read-only config returned: {}", attempt.is_err());

    // -----------------------------------------------------------------
    // Demo 4: Loading from a file (CONF format here)
    // -----------------------------------------------------------------
    println!("\nDemo 4: File-loaded configuration");
    let demo_path = std::env::temp_dir().join("config_lib_enterprise_demo.conf");
    std::fs::write(
        &demo_path,
        r#"
app_name = "Enterprise DB System"
version = "1.0.0"

[database]
driver = "postgresql"
host = "localhost"
port = 5432
max_connections = 50

[cache]
enabled = true
ttl = 300
size = "100MB"

[logging]
level = "info"
format = "json"
"#,
    )?;
    let loaded = Config::from_file(&demo_path)?;
    println!(
        "  loaded app: {}",
        loaded
            .get("app_name")
            .ok_or("missing app_name")?
            .as_string()?
    );
    println!(
        "  loaded db.host: {}",
        loaded
            .get("database.host")
            .ok_or("missing database.host")?
            .as_string()?
    );
    println!(
        "  loaded db.port: {}",
        loaded
            .get("database.port")
            .ok_or("missing database.port")?
            .as_integer()?
    );

    // -----------------------------------------------------------------
    // Demo 5: One-shot string parsing — replaces `enterprise::direct::parse_string`.
    //
    // The deprecated `direct::parse_string(content, fmt)` is just a thin
    // wrapper around `config_lib::parse(content, fmt)`; both call into the
    // same parser tree. Use the top-level free function instead.
    // -----------------------------------------------------------------
    println!("\nDemo 5: One-shot parsing via config_lib::parse");
    let snippet = r#"
name = "High Performance System"
concurrency = 1000000
file_access_time = 50  # nanoseconds
efficiency = 0.90
"#;
    let parsed = config_lib::parse(snippet, Some("conf"))?;
    if let Value::Table(table) = parsed {
        println!(
            "  system: {}",
            table.get("name").ok_or("missing name")?.as_string()?
        );
        println!(
            "  concurrency: {}",
            table
                .get("concurrency")
                .ok_or("missing concurrency")?
                .as_integer()?
        );
        println!(
            "  file access time: {}ns",
            table
                .get("file_access_time")
                .ok_or("missing file_access_time")?
                .as_integer()?
        );
        println!(
            "  efficiency: {:.0}%",
            table
                .get("efficiency")
                .ok_or("missing efficiency")?
                .as_float()?
                * 100.0
        );
    }

    // -----------------------------------------------------------------
    // Cleanup
    // -----------------------------------------------------------------
    let _ = std::fs::remove_file(&demo_path);

    // -----------------------------------------------------------------
    // What changed between this example and its EnterpriseConfig predecessor
    // -----------------------------------------------------------------
    //
    //   EnterpriseConfig                       Config (this file)
    //   ────────────────────────────────────   ──────────────────────────────────────
    //   EnterpriseConfig::new()                Config::new()
    //   EnterpriseConfig::from_string(s, f)    Config::from_string(s, f)
    //   EnterpriseConfig::from_file(p)         Config::from_file(p)
    //   cfg.get("k")        // Option<Value>   cfg.get("k")       // Option<&Value>
    //   cfg.set("k", v)                        cfg.set("k", v)
    //   cfg.exists("k")                        cfg.contains_key("k")
    //   cfg.keys()          // Vec<String>     cfg.keys()         // Result<Vec<&str>>
    //   cfg.save() / save_to(p)                cfg.save() / save_to_file(p)
    //   cfg.merge(other)                       cfg.merge(other)
    //   cfg.set_default(k, v)                  cfg.get("k").and_then(..).unwrap_or(d)
    //                                          (TODO(v0.9.5): typed defaults via
    //                                           `ConfigOptions::defaults`)
    //   cfg.cache_stats()                      TODO(v0.9.5)
    //   cfg.make_read_only()                   Config::with_options(
    //                                              ConfigOptions::new().read_only(true))
    //   ConfigManager (multi-instance)         Retained; internals migrate in v0.9.5
    //   direct::parse_string(s, f)             config_lib::parse(s, f)
    //   direct::parse_file(p)                  config_lib::parse_file(p)
    //
    Ok(())
}
