//! Integration test: modify a watched file, expect a `Reloaded` event.
//!
//! Smoke test for the event-driven watcher landed in v0.9.6 and the
//! cache-invalidation behavior added in v0.9.9. Lives at
//! `tests/` (not `src/`) so it runs as a separate binary and
//! exercises the public API surface end-to-end.
//!
//! Skipped on the `--no-default-features` configuration because the
//! event-driven watcher requires the `hot-reload` Cargo feature.

#![cfg(feature = "hot-reload")]
// REPS-AUDIT: deliberately uses the v1.0.0-deprecated
// `with_change_notifications` API. These integration tests serve as
// regression coverage that the deprecated mpsc bridge still routes
// through the new lock-free dispatch path correctly. Tests of the
// new `on_change` API live in the `hot_reload::tests` module.
#![allow(deprecated)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tempfile::TempDir;

/// Helper: write `body` to `path` and fsync so the kernel surfaces
/// the modification event before the test thread proceeds.
fn write_conf(path: &std::path::Path, body: &str) {
    let mut f = File::create(path).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    f.flush().unwrap();
    f.sync_all().unwrap();
}

#[test]
fn modified_file_emits_reloaded_event() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("app.conf");
    write_conf(&path, "key=value1\n");

    let (hot, rx) = HotReloadConfig::from_file(&path)
        .unwrap()
        .with_debounce(Duration::from_millis(25))
        .with_change_notifications();
    let handle = hot.start_watching();

    // Give the watcher a moment to register.
    std::thread::sleep(Duration::from_millis(150));
    write_conf(&path, "key=value2\n");

    // Wait up to 2 seconds for a `Reloaded` event.
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let mut saw_reloaded = false;
    while std::time::Instant::now() < deadline {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            if matches!(event, ConfigChangeEvent::Reloaded { .. }) {
                saw_reloaded = true;
                break;
            }
        }
    }
    assert!(saw_reloaded, "no Reloaded event arrived within 2 seconds");
    handle.stop().unwrap();
}
