//! Integration test: delete a watched file, expect a `FileDeleted` event.
//!
//! The watcher must observe the deletion (because it watches the
//! *parent directory*, not the file inode — see `docs/PLATFORM-NOTES.md`)
//! and emit a `FileDeleted` event without crashing. The
//! last-known-good `Config` remains in place in the shared
//! `Arc<RwLock<Config>>`; readers see the old config.

#![cfg(feature = "hot-reload")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn deleted_file_emits_file_deleted_event() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("app.conf");
    {
        let mut f = File::create(&path).unwrap();
        f.write_all(b"key=value1\n").unwrap();
        f.sync_all().unwrap();
    }

    let (hot, rx) = HotReloadConfig::from_file(&path)
        .unwrap()
        .with_debounce(Duration::from_millis(25))
        .with_change_notifications();
    let config_ref = hot.config();
    let handle = hot.start_watching();

    std::thread::sleep(Duration::from_millis(150));
    std::fs::remove_file(&path).unwrap();

    // Wait up to 2 seconds for a `FileDeleted` event.
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let mut saw_deleted = false;
    while std::time::Instant::now() < deadline {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            if matches!(event, ConfigChangeEvent::FileDeleted { .. }) {
                saw_deleted = true;
                break;
            }
        }
    }
    assert!(saw_deleted, "no FileDeleted event arrived within 2 seconds");

    // Last-known-good `Config` is still in place — the watcher does
    // not zero out the shared handle on deletion.
    {
        let cfg = config_ref.read().unwrap();
        assert_eq!(cfg.get("key").unwrap().as_string().unwrap(), "value1");
    }

    handle.stop().unwrap();
}
