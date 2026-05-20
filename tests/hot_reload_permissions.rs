//! Integration test: file becomes unreadable, expect graceful
//! `ReloadFailed` rather than a panic or hang.
//!
//! On Unix, `chmod 000` strips the read bit. On Windows the
//! equivalent permission manipulation is awkward and the integration
//! is filed as a follow-up — this test is `#[cfg(unix)]`-gated so
//! it runs on Linux and macOS CI but is silently skipped on Windows.
//! See `docs/PLATFORM-NOTES.md`.

#![cfg(all(feature = "hot-reload", unix))]
// REPS-AUDIT: deliberately uses the v1.0.0-deprecated
// `with_change_notifications` API to verify the deprecated mpsc
// bridge routes through the new lock-free dispatch correctly.
#![allow(deprecated)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};
use std::fs::{File, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use tempfile::TempDir;

fn write_conf(path: &std::path::Path, body: &str) {
    let mut f = File::create(path).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    f.flush().unwrap();
    f.sync_all().unwrap();
}

#[test]
fn unreadable_file_emits_reload_failed() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("app.conf");
    write_conf(&path, "key=value1\n");

    let (hot, rx) = HotReloadConfig::from_file(&path)
        .unwrap()
        .with_debounce(Duration::from_millis(25))
        .with_change_notifications();
    let handle = hot.start_watching();

    std::thread::sleep(Duration::from_millis(150));

    // Trigger a change AND strip read permission. The watcher
    // should attempt to reload (Modify event fires), fail because
    // the file is unreadable, and emit `ReloadFailed` rather than
    // panicking.
    std::fs::set_permissions(&path, Permissions::from_mode(0o000)).unwrap();
    // Touch the mtime so the watcher actually observes a change.
    // (On some filesystems chmod alone doesn't update mtime.)
    let _ = std::fs::write(dir.path().join(".trigger"), b"x");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let mut saw_failed = false;
    while std::time::Instant::now() < deadline {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            // Either a ReloadFailed for the unreadable target, OR a
            // FileDeleted-style report — both are graceful failure
            // modes. The thing we're verifying is "no panic, no hang".
            if matches!(
                event,
                ConfigChangeEvent::ReloadFailed { .. } | ConfigChangeEvent::FileDeleted { .. }
            ) {
                saw_failed = true;
                break;
            }
        }
    }

    // Restore permissions so the TempDir cleanup can delete the file.
    let _ = std::fs::set_permissions(&path, Permissions::from_mode(0o644));

    assert!(
        saw_failed,
        "no graceful ReloadFailed / FileDeleted event arrived; the watcher \
         may have panicked or hung on the unreadable file"
    );

    handle.stop().unwrap();
}
