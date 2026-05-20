//! Lock-free change-notification demo (v1.0.0+).
//!
//! Shows the `on_change` API for registering handlers on a
//! hot-reloaded configuration. Multiple components can register
//! their own handlers; the reloader dispatches inline with sub-
//! microsecond overhead and no channel allocation.
//!
//! Run:
//!
//!     cargo run --example on_change_demo --features hot-reload
//!
//! The demo creates a temp config file, registers three handlers,
//! mutates the file in two beats, and prints what each handler
//! observed. Then it drops one subscription mid-demo to show that
//! `Drop`-based unregistration takes effect immediately.

use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn write_conf(path: &std::path::Path, body: &str) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(body.as_bytes())?;
    f.flush()?;
    f.sync_all()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("config-lib on_change demo (v1.0.0 lock-free notifications)");
    println!("===========================================================");

    // ---------------------------------------------------------------
    // Set up a hot-reloadable config in a temp dir.
    // ---------------------------------------------------------------
    let dir = std::env::temp_dir().join("config_lib_on_change_demo");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("app.conf");
    write_conf(&path, "log_level=info\nworkers=4\n")?;

    let hot = HotReloadConfig::from_file(&path)?.with_debounce(Duration::from_millis(25));

    // ---------------------------------------------------------------
    // Three components each register their own handler.
    // ---------------------------------------------------------------
    let metrics = Arc::new(AtomicUsize::new(0));
    let logging_changes = Arc::new(AtomicUsize::new(0));

    // Component A: a metrics counter, just counts every Reloaded event.
    let metrics_c = Arc::clone(&metrics);
    let _sub_metrics = hot.on_change(move |event| {
        if matches!(event, ConfigChangeEvent::Reloaded { .. }) {
            metrics_c.fetch_add(1, Ordering::Relaxed);
        }
    });

    // Component B: a logging-specific listener.
    let logging_c = Arc::clone(&logging_changes);
    let _sub_logging = hot.on_change(move |event| {
        if matches!(event, ConfigChangeEvent::Reloaded { .. }) {
            logging_c.fetch_add(1, Ordering::Relaxed);
        }
    });

    // Component C: a temporary observer we'll drop mid-demo.
    let temp_observed = Arc::new(AtomicUsize::new(0));
    let temp_c = Arc::clone(&temp_observed);
    let sub_temp = hot.on_change(move |event| {
        if matches!(event, ConfigChangeEvent::Reloaded { .. }) {
            temp_c.fetch_add(1, Ordering::Relaxed);
        }
    });

    // ---------------------------------------------------------------
    // Start the background watcher.
    // ---------------------------------------------------------------
    let handle = hot.start_watching();

    // Give the watcher a moment to register.
    std::thread::sleep(Duration::from_millis(150));

    // ---------------------------------------------------------------
    // First change: all three handlers see it.
    // ---------------------------------------------------------------
    println!("\n1. Writing first config change...");
    write_conf(&path, "log_level=debug\nworkers=8\n")?;
    std::thread::sleep(Duration::from_millis(500));

    println!(
        "   metrics counter:         {}",
        metrics.load(Ordering::Relaxed)
    );
    println!(
        "   logging-change counter:  {}",
        logging_changes.load(Ordering::Relaxed)
    );
    println!(
        "   temp observer counter:   {}",
        temp_observed.load(Ordering::Relaxed)
    );

    // ---------------------------------------------------------------
    // Drop the temporary subscription — handler unregisters immediately.
    // ---------------------------------------------------------------
    println!("\n2. Dropping temp subscription...");
    drop(sub_temp);

    // ---------------------------------------------------------------
    // Second change: only the two surviving handlers see it.
    // ---------------------------------------------------------------
    println!("\n3. Writing second config change...");
    write_conf(&path, "log_level=trace\nworkers=16\n")?;
    std::thread::sleep(Duration::from_millis(500));

    println!(
        "   metrics counter:         {}",
        metrics.load(Ordering::Relaxed)
    );
    println!(
        "   logging-change counter:  {}",
        logging_changes.load(Ordering::Relaxed)
    );
    println!(
        "   temp observer counter:   {}  <- unchanged (subscription dropped)",
        temp_observed.load(Ordering::Relaxed)
    );

    // ---------------------------------------------------------------
    // Cleanup
    // ---------------------------------------------------------------
    handle.stop()?;
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);

    println!("\nDone.");
    println!("\nKey points:");
    println!("- Each handler runs INLINE on the reloader thread (no channel hops)");
    println!("- Dispatch is one atomic load + N closure calls");
    println!("- Dropping a Subscription unregisters its handler atomically");
    println!("- A panicking handler would not affect the others (catch_unwind isolation)");

    Ok(())
}
