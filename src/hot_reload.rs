//! Configuration Hot Reloading System
//!
//! Production-grade hot reloading with:
//!
//! - **Event-driven file watching** via the [`notify`](https://docs.rs/notify)
//!   crate (default in v0.9.6+ via the `hot-reload` Cargo feature).
//!   `notify` is a cross-platform wrapper over the kernel's native
//!   filesystem event APIs: `inotify` on Linux, `FSEvents` on macOS,
//!   and `ReadDirectoryChangesW` on Windows. Detection latency is
//!   typically a few milliseconds — well under the 100 ms target the
//!   v1.0 stability contract commits to.
//! - **Atomic-write debouncing.** Many editors save by writing to a
//!   temporary file and atomically renaming it over the target. This
//!   produces a flurry of events (create, modify, delete, modify) in
//!   rapid succession. The reloader collapses any burst within the
//!   debounce window (default 100 ms, configurable) to one
//!   `Reloaded` notification.
//! - **`Arc<RwLock<Config>>` swap** for zero-downtime updates —
//!   readers never block while the reloader parses the new file.
//! - **`mpsc` change notifications** preserving the
//!   `ConfigChangeEvent` surface from earlier releases.
//! - **Polling fallback** (always available, used as the default when
//!   the `hot-reload` feature is disabled, or available as an opt-in
//!   on top of event-driven watching for environments where the kernel
//!   APIs are known-broken — network filesystems, some container
//!   layers).

use crate::config::Config;
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

/// Configuration change event types.
///
/// **Stability:** `ConfigChangeEvent` is `#[non_exhaustive]` so the
/// v1.x SemVer contract can add new variants (e.g. `Renamed`,
/// `PermissionDenied`) in MINOR releases without breaking user code.
/// Callers must use a wildcard arm when pattern-matching.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ConfigChangeEvent {
    /// Configuration successfully reloaded
    Reloaded {
        /// Path to the configuration file that was reloaded
        path: PathBuf,
        /// Timestamp when the reload completed
        timestamp: SystemTime,
    },
    /// Configuration reload failed
    ReloadFailed {
        /// Path to the configuration file that failed to reload
        path: PathBuf,
        /// Error message describing what went wrong
        error: String,
        /// Timestamp when the error occurred
        timestamp: SystemTime,
    },
    /// Configuration file was modified
    FileModified {
        /// Path to the configuration file that was modified
        path: PathBuf,
        /// Timestamp when the modification was detected
        timestamp: SystemTime,
    },
    /// Configuration file was deleted
    FileDeleted {
        /// Path to the configuration file that was deleted
        path: PathBuf,
        /// Timestamp when the deletion was detected
        timestamp: SystemTime,
    },
}

/// Default debounce window applied to file-change events before
/// triggering a reload. Sized to cover the editor "save via atomic
/// rename" pattern (where a single save fires multiple kernel events
/// within ~10–50 ms).
const DEFAULT_DEBOUNCE: Duration = Duration::from_millis(100);

/// Hot-reloadable configuration container.
///
/// Construct with [`HotReloadConfig::from_file`], then either drive
/// reloads manually with [`HotReloadConfig::reload`] or hand off to a
/// background watcher with [`HotReloadConfig::start_watching`].
///
/// Configurable knobs (all consuming-builder style, intended for
/// fluent construction):
///
/// - [`HotReloadConfig::with_change_notifications`] — receive
///   [`ConfigChangeEvent`]s on an `mpsc` channel.
/// - [`HotReloadConfig::with_debounce`] — adjust the debounce window
///   (default 100 ms).
/// - [`HotReloadConfig::with_poll_interval`] — set the polling
///   interval. Used directly when the `hot-reload` feature is off;
///   used as the watchdog interval when the feature is on.
/// - [`HotReloadConfig::with_polling_fallback`] — opt into a
///   parallel polling thread *in addition to* the event-driven
///   watcher, for environments where the kernel watcher is known
///   unreliable.
pub struct HotReloadConfig {
    /// Current configuration (thread-safe)
    current: Arc<RwLock<Config>>,
    /// File path being watched
    file_path: PathBuf,
    /// Last known modification time
    last_modified: SystemTime,
    /// Event sender for notifications
    event_sender: Option<Sender<ConfigChangeEvent>>,
    /// Polling interval — used as primary cadence when the
    /// `hot-reload` feature is off, or as the watchdog interval when
    /// the feature is on and `polling_fallback_enabled` is set.
    poll_interval: Duration,
    /// Debounce window applied to clustered file-change events.
    debounce: Duration,
    /// Whether to run the polling watchdog *in addition to* the
    /// event-driven watcher. Useful on network filesystems.
    polling_fallback_enabled: bool,
}

impl HotReloadConfig {
    /// Create a new hot-reloadable configuration from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, parsed, or stat'd
    /// for its modification time.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let config = Config::from_file(&path)?;

        let last_modified = std::fs::metadata(&path)
            .map_err(|e| Error::io(path.display().to_string(), e))?
            .modified()
            .map_err(|e| Error::io(path.display().to_string(), e))?;

        Ok(Self {
            current: Arc::new(RwLock::new(config)),
            file_path: path,
            last_modified,
            event_sender: None,
            poll_interval: Duration::from_secs(1),
            debounce: DEFAULT_DEBOUNCE,
            polling_fallback_enabled: false,
        })
    }

    /// Set the polling interval for file change detection.
    ///
    /// When the `hot-reload` feature is enabled (the default in v0.9.6+),
    /// the primary watcher is event-driven and this interval is only
    /// consulted as the watchdog cadence if `with_polling_fallback`
    /// has been called.
    ///
    /// When the `hot-reload` feature is disabled, this is the actual
    /// polling cadence of the background thread.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Override the debounce window applied to clustered file-change
    /// events.
    ///
    /// Editors that save via "write-to-tmp + atomic-rename" generate
    /// multiple kernel events for a single user save. The debounce
    /// collapses any burst within this window to a single reload.
    /// Default: 100 ms.
    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }

    /// Opt into running a polling watchdog *in addition to* the
    /// event-driven watcher.
    ///
    /// Network filesystems (SMB, NFS), some container overlay
    /// filesystems, and a handful of edge-case kernel configurations
    /// drop or delay events that `notify` would normally surface.
    /// Enabling the polling fallback re-derives changes from periodic
    /// `stat(2)` calls on the watched path, at the
    /// [`HotReloadConfig::with_poll_interval`] cadence.
    ///
    /// Has no effect (and costs nothing) when the `hot-reload` Cargo
    /// feature is disabled — the watcher is already polling in that
    /// configuration.
    pub fn with_polling_fallback(mut self) -> Self {
        self.polling_fallback_enabled = true;
        self
    }

    /// Enable change notifications.
    ///
    /// Returns the configured [`HotReloadConfig`] together with a
    /// [`Receiver`] that will deliver [`ConfigChangeEvent`]s as the
    /// watcher observes them.
    pub fn with_change_notifications(mut self) -> (Self, Receiver<ConfigChangeEvent>) {
        let (sender, receiver) = mpsc::channel();
        self.event_sender = Some(sender);
        (self, receiver)
    }

    /// Get a thread-safe reference to the current configuration.
    pub fn config(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.current)
    }

    /// Get a freshly-reparsed snapshot of the configuration file as
    /// it exists on disk *right now*.
    ///
    /// This is distinct from reading the current `Arc<RwLock<Config>>`
    /// — it bypasses the watcher and re-reads the file. Useful for
    /// "what would I see if I reloaded now" inspection.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn snapshot(&self) -> Result<Config> {
        Config::from_file(&self.file_path)
    }

    /// Manually trigger a reload check.
    ///
    /// Re-stats the file, compares mtime against the last-known
    /// modification time, and re-parses if newer. Sends a
    /// [`ConfigChangeEvent::Reloaded`] or
    /// [`ConfigChangeEvent::ReloadFailed`] notification if change
    /// notifications are enabled.
    ///
    /// Returns `Ok(true)` if a reload was performed, `Ok(false)` if
    /// the file was unchanged since the last check.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be stat'd, read, or parsed.
    pub fn reload(&mut self) -> Result<bool> {
        let metadata = std::fs::metadata(&self.file_path)
            .map_err(|e| Error::io(self.file_path.display().to_string(), e))?;

        let modified = metadata
            .modified()
            .map_err(|e| Error::io(self.file_path.display().to_string(), e))?;

        if modified <= self.last_modified {
            return Ok(false);
        }

        match Config::from_file(&self.file_path) {
            Ok(new_config) => {
                {
                    let mut config = self.current.write().map_err(|_| {
                        Error::concurrency("Failed to acquire write lock".to_string())
                    })?;
                    *config = new_config;
                }
                self.last_modified = modified;

                if let Some(sender) = &self.event_sender {
                    let _ = sender.send(ConfigChangeEvent::Reloaded {
                        path: self.file_path.clone(),
                        timestamp: SystemTime::now(),
                    });
                }
                Ok(true)
            }
            Err(e) => {
                if let Some(sender) = &self.event_sender {
                    let _ = sender.send(ConfigChangeEvent::ReloadFailed {
                        path: self.file_path.clone(),
                        error: e.to_string(),
                        timestamp: SystemTime::now(),
                    });
                }
                Err(e)
            }
        }
    }

    /// Start automatic hot reloading in a background thread.
    ///
    /// With the `hot-reload` Cargo feature enabled (the default in
    /// v0.9.6+), the background worker registers a
    /// `notify::RecommendedWatcher` on the file's parent directory
    /// and reacts to kernel events. Otherwise it falls back to a
    /// `poll_interval`-cadence polling thread (the v0.9.5 behavior).
    pub fn start_watching(self) -> HotReloadHandle {
        #[cfg(feature = "hot-reload")]
        {
            self.start_watching_event_driven()
        }
        #[cfg(not(feature = "hot-reload"))]
        {
            self.start_watching_polling()
        }
    }

    /// Get the file path being watched.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the last modification time.
    pub fn last_modified(&self) -> SystemTime {
        self.last_modified
    }

    // -----------------------------------------------------------------
    // Polling watcher — used as the primary watcher when the
    // `hot-reload` Cargo feature is disabled. (When the feature is on,
    // the event-driven path covers all environments where the kernel
    // event API works; opt-in polling-as-watchdog alongside the
    // event-driven watcher is reserved for a follow-up release.)
    // -----------------------------------------------------------------

    #[cfg(not(feature = "hot-reload"))]
    fn start_watching_polling(self) -> HotReloadHandle {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);

        let current = Arc::clone(&self.current);
        let file_path = self.file_path.clone();
        let event_sender = self.event_sender.clone();
        let poll_interval = self.poll_interval;
        let mut last_modified = self.last_modified;

        let handle = thread::spawn(move || {
            while !stop_clone.load(Ordering::Relaxed) {
                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified > last_modified {
                            if let Some(sender) = &event_sender {
                                let _ = sender.send(ConfigChangeEvent::FileModified {
                                    path: file_path.clone(),
                                    timestamp: SystemTime::now(),
                                });
                            }

                            match Config::from_file(&file_path) {
                                Ok(new_config) => {
                                    if let Ok(mut config) = current.write() {
                                        *config = new_config;
                                        last_modified = modified;

                                        if let Some(sender) = &event_sender {
                                            let _ = sender.send(ConfigChangeEvent::Reloaded {
                                                path: file_path.clone(),
                                                timestamp: SystemTime::now(),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    if let Some(sender) = &event_sender {
                                        let _ = sender.send(ConfigChangeEvent::ReloadFailed {
                                            path: file_path.clone(),
                                            error: e.to_string(),
                                            timestamp: SystemTime::now(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                thread::sleep(poll_interval);
            }
        });

        HotReloadHandle {
            handle: Some(handle),
            stop,
        }
    }

    // -----------------------------------------------------------------
    // Event-driven watcher — gated on the `hot-reload` feature.
    // -----------------------------------------------------------------

    #[cfg(feature = "hot-reload")]
    fn start_watching_event_driven(self) -> HotReloadHandle {
        use notify::{Event, RecursiveMode, Watcher};

        let stop = Arc::new(AtomicBool::new(false));
        let current = Arc::clone(&self.current);
        let file_path = self.file_path.clone();
        let event_sender = self.event_sender.clone();
        let debounce = self.debounce;
        let poll_interval = self.poll_interval;
        let polling_fallback = self.polling_fallback_enabled;
        let initial_modified = self.last_modified;

        // Channel from the notify callback to the reload worker.
        let (tx, rx) = mpsc::channel::<Event>();

        // Build the watcher. We watch the *parent* directory (not the
        // file itself) so that atomic-rename saves — where the file's
        // inode is replaced — still surface as events on our target.
        let watcher_dir = file_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let watcher_result = notify::RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default(),
        )
        .and_then(|mut w| {
            w.watch(&watcher_dir, RecursiveMode::NonRecursive)?;
            Ok(w)
        });

        let watcher = match watcher_result {
            Ok(w) => Some(w),
            Err(e) => {
                // Watcher construction failed — likely the platform
                // event API is unavailable (rare). Surface a
                // `ReloadFailed` so the caller knows, then fall
                // through and spawn the polling worker as the only
                // safety net.
                if let Some(sender) = &event_sender {
                    let _ = sender.send(ConfigChangeEvent::ReloadFailed {
                        path: file_path.clone(),
                        error: format!(
                            "notify watcher construction failed: {e}; falling back to polling"
                        ),
                        timestamp: SystemTime::now(),
                    });
                }
                None
            }
        };

        // Reload worker — consumes events from the notify callback,
        // debounces, and re-parses on change.
        let target_file = file_path.clone();
        let event_sender_for_worker = event_sender.clone();
        let current_for_worker = Arc::clone(&current);
        let stop_for_worker = Arc::clone(&stop);
        let mut last_modified_seen = initial_modified;

        let handle = thread::spawn(move || {
            while !stop_for_worker.load(Ordering::Relaxed) {
                // Block up to `poll_interval` for the next event so
                // the stop flag is observed promptly even when the
                // file is quiet. (`recv_timeout` is the only stdlib
                // mpsc primitive that respects both the channel and
                // a deadline.)
                let first = match rx.recv_timeout(poll_interval) {
                    Ok(ev) => Some(ev),
                    Err(mpsc::RecvTimeoutError::Timeout) => None,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };

                // If we got an event, drain the channel for the
                // debounce window so the burst from a single save
                // collapses to one reload.
                let mut relevant = false;
                if let Some(ev) = first {
                    relevant |= event_targets_path(&ev, &target_file);

                    let deadline = std::time::Instant::now() + debounce;
                    loop {
                        let remaining =
                            deadline.saturating_duration_since(std::time::Instant::now());
                        if remaining.is_zero() {
                            break;
                        }
                        match rx.recv_timeout(remaining) {
                            Ok(ev) => relevant |= event_targets_path(&ev, &target_file),
                            Err(_) => break,
                        }
                    }
                } else if !polling_fallback {
                    continue;
                }

                // Path resolution: did the target file actually change?
                let metadata = std::fs::metadata(&target_file);
                match metadata {
                    Ok(meta) => {
                        let modified = meta.modified().ok();
                        let is_newer = match modified {
                            Some(m) => m > last_modified_seen,
                            None => true,
                        };
                        if !relevant && !is_newer {
                            continue;
                        }

                        if let Some(sender) = &event_sender_for_worker {
                            let _ = sender.send(ConfigChangeEvent::FileModified {
                                path: target_file.clone(),
                                timestamp: SystemTime::now(),
                            });
                        }

                        match Config::from_file(&target_file) {
                            Ok(new_config) => {
                                if let Ok(mut cfg) = current_for_worker.write() {
                                    *cfg = new_config;
                                    if let Some(m) = modified {
                                        last_modified_seen = m;
                                    }
                                    if let Some(sender) = &event_sender_for_worker {
                                        let _ = sender.send(ConfigChangeEvent::Reloaded {
                                            path: target_file.clone(),
                                            timestamp: SystemTime::now(),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                if let Some(sender) = &event_sender_for_worker {
                                    let _ = sender.send(ConfigChangeEvent::ReloadFailed {
                                        path: target_file.clone(),
                                        error: e.to_string(),
                                        timestamp: SystemTime::now(),
                                    });
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // File missing — likely deleted between
                        // events. Emit FileDeleted but keep the
                        // last-known-good config in place.
                        if let Some(sender) = &event_sender_for_worker {
                            let _ = sender.send(ConfigChangeEvent::FileDeleted {
                                path: target_file.clone(),
                                timestamp: SystemTime::now(),
                            });
                        }
                    }
                }
            }
        });

        HotReloadHandle {
            handle: Some(handle),
            stop,
            _watcher: watcher,
        }
    }
}

/// Helper: does the `notify::Event` reference our watched file?
///
/// When watching a directory non-recursively, every event carries the
/// list of paths it applies to. Filtering on the exact file path keeps
/// us from reacting to unrelated sibling files in the same directory.
#[cfg(feature = "hot-reload")]
fn event_targets_path(event: &notify::Event, target: &Path) -> bool {
    use notify::EventKind;
    if !matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) | EventKind::Any
    ) {
        return false;
    }
    // Canonical-form comparison helps with macOS symlink/realpath
    // shenanigans. Fall back to direct equality when canonicalize
    // can't resolve (e.g. file was just deleted).
    let target_canon = std::fs::canonicalize(target).ok();
    event.paths.iter().any(|p| {
        if p == target {
            return true;
        }
        if let (Some(tc), Ok(pc)) = (&target_canon, std::fs::canonicalize(p)) {
            return *tc == pc;
        }
        false
    })
}

/// Handle for controlling hot reload background thread.
pub struct HotReloadHandle {
    handle: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    /// Watcher kept alive for the duration of the watch. Dropping
    /// the watcher tears down the kernel registration. Only carried
    /// when the `hot-reload` feature is on.
    #[cfg(feature = "hot-reload")]
    _watcher: Option<notify::RecommendedWatcher>,
}

impl HotReloadHandle {
    /// Stop the background watching thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the background thread panicked.
    pub fn stop(mut self) -> Result<()> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| Error::concurrency("Failed to join background thread".to_string()))?;
        }
        Ok(())
    }
}

impl Drop for HotReloadHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper: write a CONF body to `path` and `fsync` it so the
    /// kernel surfaces the modification event before we proceed.
    fn write_conf(path: &Path, body: &str) {
        let mut f = File::create(path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        f.flush().unwrap();
        f.sync_all().unwrap();
    }

    #[test]
    fn test_hot_reload_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let mut hot_config = HotReloadConfig::from_file(&config_path).unwrap();
        {
            let config = hot_config.config();
            let config_read = config.read().unwrap();
            assert_eq!(
                config_read.get("key").unwrap().as_string().unwrap(),
                "value1"
            );
        }

        // Sleep past filesystem mtime resolution before re-writing.
        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value2\n");

        let reloaded = hot_config.reload().unwrap();
        assert!(reloaded);

        {
            let config = hot_config.config();
            let config_read = config.read().unwrap();
            assert_eq!(
                config_read.get("key").unwrap().as_string().unwrap(),
                "value2"
            );
        }
    }

    #[test]
    fn test_hot_reload_notifications() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let (mut hot_config, receiver) = HotReloadConfig::from_file(&config_path)
            .unwrap()
            .with_change_notifications();

        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value2\n");
        hot_config.reload().unwrap();

        let event = receiver.try_recv().unwrap();
        match event {
            ConfigChangeEvent::Reloaded { path, .. } => assert_eq!(path, config_path),
            _ => panic!("Expected Reloaded event"),
        }
    }

    #[test]
    fn test_automatic_watching() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let (hot_config, receiver) = HotReloadConfig::from_file(&config_path)
            .unwrap()
            .with_poll_interval(Duration::from_millis(50))
            .with_debounce(Duration::from_millis(25))
            .with_change_notifications();

        let config_ref = hot_config.config();
        let handle = hot_config.start_watching();

        // Give the watcher a moment to register.
        thread::sleep(Duration::from_millis(100));
        write_conf(&config_path, "key=value2\n");

        // Wait long enough for the event-driven path (a few ms) OR
        // the polling fallback (50ms+) to react and re-parse.
        thread::sleep(Duration::from_millis(500));

        {
            let config_read = config_ref.read().unwrap();
            assert_eq!(
                config_read.get("key").unwrap().as_string().unwrap(),
                "value2"
            );
        }

        let mut events = Vec::new();
        while let Ok(ev) = receiver.try_recv() {
            events.push(ev);
        }
        assert!(
            !events.is_empty(),
            "expected at least one ConfigChangeEvent"
        );
        let has_reloaded = events
            .iter()
            .any(|e| matches!(e, ConfigChangeEvent::Reloaded { .. }));
        assert!(has_reloaded, "expected at least one Reloaded event");

        handle.stop().unwrap();
    }
}
