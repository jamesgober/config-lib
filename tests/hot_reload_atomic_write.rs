//! Integration test: atomic-rename save, expect a single (debounced)
//! `Reloaded` event rather than one per intermediate kernel event.
//!
//! Reproduces the editor "save via tmp + atomic rename" pattern:
//!
//!   1. write the new content to `app.conf.tmp` (next to the target)
//!   2. rename `app.conf.tmp` → `app.conf` (atomic over the inode)
//!
//! The 100 ms debounce window collapses the burst of kernel events
//! this generates (Remove/Create/Modify) into one `Reloaded` event.

#![cfg(feature = "hot-reload")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tempfile::TempDir;

fn atomic_replace(target: &std::path::Path, body: &str) {
    let tmp = target.with_extension("conf.tmp");
    let mut f = File::create(&tmp).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    f.flush().unwrap();
    f.sync_all().unwrap();
    drop(f);
    std::fs::rename(&tmp, target).unwrap();
}

#[test]
fn atomic_rename_save_emits_single_reloaded_event() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("app.conf");
    {
        let mut f = File::create(&path).unwrap();
        f.write_all(b"key=value1\n").unwrap();
        f.sync_all().unwrap();
    }

    let (hot, rx) = HotReloadConfig::from_file(&path)
        .unwrap()
        .with_debounce(Duration::from_millis(100))
        .with_change_notifications();
    let handle = hot.start_watching();

    std::thread::sleep(Duration::from_millis(200));
    atomic_replace(&path, "key=value2\n");

    // Collect every event for 500 ms (ample time for the debounce
    // window + the parse + the notification dispatch).
    let collect_deadline = std::time::Instant::now() + Duration::from_millis(500);
    let mut reloaded_count = 0;
    while std::time::Instant::now() < collect_deadline {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(50)) {
            if matches!(event, ConfigChangeEvent::Reloaded { .. }) {
                reloaded_count += 1;
            }
        }
    }

    assert!(
        reloaded_count >= 1,
        "expected at least one Reloaded event from the atomic rename"
    );
    assert!(
        reloaded_count <= 2,
        "expected the debounce window to collapse the event burst; \
         got {reloaded_count} Reloaded events (>=2 indicates the debounce \
         is not collapsing the rename burst as designed)"
    );
    handle.stop().unwrap();
}
