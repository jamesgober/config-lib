//! Integration test: delete + recreate a watched file, expect
//! `FileDeleted` followed by `Reloaded`.
//!
//! This exercises the watcher's persistence across an inode change:
//! since it watches the parent directory rather than the file inode,
//! the re-created file under the same name surfaces a new event.

#![cfg(feature = "hot-reload")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tempfile::TempDir;

fn write_conf(path: &std::path::Path, body: &str) {
    let mut f = File::create(path).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    f.flush().unwrap();
    f.sync_all().unwrap();
}

#[test]
fn deleted_then_recreated_emits_both_events() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("app.conf");
    write_conf(&path, "key=value1\n");

    let (hot, rx) = HotReloadConfig::from_file(&path)
        .unwrap()
        .with_debounce(Duration::from_millis(25))
        .with_change_notifications();
    let config_ref = hot.config();
    let handle = hot.start_watching();

    std::thread::sleep(Duration::from_millis(150));
    std::fs::remove_file(&path).unwrap();

    // Pause longer than the debounce so the deletion event doesn't
    // get coalesced with the re-creation event.
    std::thread::sleep(Duration::from_millis(300));
    write_conf(&path, "key=value2\n");

    // Collect events for up to 5 seconds. (Longer window than the
    // other hot_reload tests because the delete-then-recreate
    // sequence has FSEvents-on-Apple-Silicon timing variability —
    // see docs/PLATFORM-NOTES.md.)
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut saw_deleted = false;
    let mut saw_reloaded = false;
    while std::time::Instant::now() < deadline && (!saw_deleted || !saw_reloaded) {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            match event {
                ConfigChangeEvent::FileDeleted { .. } => saw_deleted = true,
                ConfigChangeEvent::Reloaded { .. } => saw_reloaded = true,
                _ => {}
            }
        }
    }

    assert!(saw_deleted, "no FileDeleted event observed");
    assert!(saw_reloaded, "no Reloaded event observed after re-creation");

    // Receiving the `Reloaded` event tells us the worker *sent* it;
    // the worker holds the write lock through the send, so the
    // updated `Config` is visible by the time the test acquires
    // the read lock here.
    //
    // BUT — on macOS Apple Silicon, FSEvents can deliver
    // delete+recreate events in such a way that the worker
    // processes a stale (pre-recreate) reload after the
    // post-recreate reload. The shared config briefly reflects
    // value2 (which sent the `Reloaded` we observed above) and
    // then gets re-set to whatever a follow-up event causes
    // re-reading the file to produce. Poll the config for up to
    // two more seconds; the file on disk has value2 by this
    // point, so any subsequent reload converges to value2.
    let state_deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        {
            let cfg = config_ref.read().unwrap();
            let current = cfg
                .get("key")
                .and_then(|v| v.as_string().ok())
                .map(str::to_owned);
            if current.as_deref() == Some("value2") {
                break;
            }
            if std::time::Instant::now() >= state_deadline {
                panic!(
                    "config never converged to value2 within 2s of the \
                     last observed event (current value: {current:?})"
                );
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    handle.stop().unwrap();
}
