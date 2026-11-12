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
//!   readers never block while the reloader parses the new file. The
//!   write lock is held only for the swap and the checks on it, never
//!   while handlers run.
//! - **Reload checks** (v1.4.0+): a file that changes while it is read, or
//!   that holds no values, does not replace the current configuration;
//!   a caller's validator and the configuration's validation rules must
//!   pass; read-only mode, defaults and rules carry over.
//! - **Lock-free in-process notification dispatch** (v1.0.0+). Register
//!   handlers via `HotReloadConfig::on_change` / `HotReloadHandle::on_change`;
//!   the reloader thread invokes every registered handler inline with
//!   no channel allocation and no cross-thread wakeup, after the new
//!   configuration is in place and with no lock held, so a handler can
//!   read the configuration. Backed by
//!   `ArcSwap<Vec<Handler>>` — snapshot reads cost a single atomic
//!   pointer load (~5 ns) regardless of how many handlers are
//!   registered. See `docs/PERFORMANCE.md` for measured numbers.
//! - **Panic isolation.** Each handler invocation is wrapped in
//!   `catch_unwind` so one bad handler can't take down the watcher
//!   or other handlers.
//! - **Polling fallback** (always available, used as the default when
//!   the `hot-reload` feature is disabled, or available as an opt-in
//!   on top of event-driven watching for environments where the kernel
//!   APIs are known-broken — network filesystems, some container
//!   layers).

use crate::config::Config;
use crate::error::{Error, Result};
use crate::load::LoadOptions;
use arc_swap::ArcSwap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, RwLock, Weak};
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

// =========================================================================
// HandlerList — lock-free in-process notification dispatch (v1.0.0+)
// =========================================================================

/// Boxed handler closure. `Arc` so that an iteration snapshot can
/// outlive registration / unregistration of the same handler without
/// invalidating the in-flight call.
type Handler = Arc<dyn Fn(&ConfigChangeEvent) + Send + Sync + 'static>;

/// Lock-free handler list backing [`HotReloadConfig::on_change`].
///
/// Reads (the dispatch path on the reloader thread) take a snapshot
/// of the current handler vector via one `ArcSwap::load` — a single
/// relaxed atomic pointer load. Writes (register / unregister)
/// allocate a new vector, copy the old contents minus/plus one
/// handler, and atomic-swap the pointer (`rcu`-style update). Writes
/// never block reads.
///
/// The handler list is shared between [`HotReloadConfig`] and its
/// spawned worker thread via `Arc<HandlerList>`, so handlers can be
/// registered before *or* after [`HotReloadConfig::start_watching`]
/// — see [`HotReloadHandle::on_change`].
pub(crate) struct HandlerList {
    handlers: ArcSwap<Vec<(u64, Handler)>>,
    next_id: AtomicU64,
}

impl HandlerList {
    fn new() -> Self {
        Self {
            handlers: ArcSwap::from_pointee(Vec::new()),
            next_id: AtomicU64::new(0),
        }
    }

    /// Dispatch an event to every registered handler.
    ///
    /// Each handler is invoked inline on the calling thread. Panic
    /// isolation: a handler that panics is caught via `catch_unwind`
    /// so subsequent handlers still run and the reloader thread is
    /// not torn down.
    fn dispatch(&self, event: &ConfigChangeEvent) {
        // Single atomic pointer load — the dispatch hot path.
        let snapshot = self.handlers.load();
        for (_id, handler) in snapshot.iter() {
            // Clone the Arc<dyn Fn> so the handler stays alive even
            // if it's concurrently unregistered during this loop
            // iteration. Refcount bump, no allocation.
            let h = Arc::clone(handler);
            // REPS-AUDIT: handlers are user-supplied; one panicking
            // handler must not break the watcher or other handlers.
            // `catch_unwind` swallows the panic and we discard the
            // payload (handlers are best-effort observers).
            let _ = catch_unwind(AssertUnwindSafe(move || {
                h(event);
            }));
        }
    }

    /// Register a new handler, returning its assigned id.
    fn register(&self, handler: Handler) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.handlers.rcu(|current| {
            let mut next = Vec::with_capacity(current.len() + 1);
            next.extend(current.iter().cloned());
            next.push((id, Arc::clone(&handler)));
            next
        });
        id
    }

    /// Remove the handler with the given id, if present. Idempotent.
    fn unregister(&self, id: u64) {
        self.handlers.rcu(|current| {
            current
                .iter()
                .filter(|(other_id, _)| *other_id != id)
                .cloned()
                .collect::<Vec<_>>()
        });
    }
}

/// RAII handle for a registered change-notification handler.
///
/// Returned by [`HotReloadConfig::on_change`] /
/// [`HotReloadHandle::on_change`]. Drop the `Subscription` to
/// unregister the handler. The watcher itself outlives any individual
/// subscription — multiple subscriptions can come and go without
/// touching the underlying [`HotReloadConfig`].
///
/// # Example
///
/// ```rust,no_run
/// use config_lib::hot_reload::{HotReloadConfig, ConfigChangeEvent};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let hot = HotReloadConfig::from_file("app.conf")?;
/// let _subscription = hot.on_change(|event: &ConfigChangeEvent| {
///     println!("config changed: {event:?}");
/// });
/// // Drop `_subscription` (e.g. at end of scope) to unregister.
/// # Ok(())
/// # }
/// ```
#[must_use = "dropping the Subscription immediately unregisters the handler; bind to a name (`let _sub = ...`) or call `.forget()` to keep the handler alive"]
pub struct Subscription {
    list: Weak<HandlerList>,
    id: u64,
}

impl Subscription {
    /// Detach the subscription from its drop-based unregistration
    /// hook. The handler stays registered for the lifetime of the
    /// underlying watcher (until the [`HotReloadConfig`] or
    /// [`HotReloadHandle`] that produced it is dropped).
    ///
    /// Useful for global / process-lifetime handlers where the
    /// caller has no convenient owning scope to hold the
    /// `Subscription`.
    pub fn forget(mut self) {
        // Clear the weak reference so Drop becomes a no-op.
        self.list = Weak::new();
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(list) = self.list.upgrade() {
            list.unregister(self.id);
        }
    }
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("id", &self.id)
            .field("alive", &(self.list.strong_count() > 0))
            .finish()
    }
}

// =========================================================================
// Reloading: load, check, swap, then notify
// =========================================================================

/// A caller's check, run on every reloaded configuration before it goes
/// live. See [`HotReloadConfig::with_validator`].
type Validator = Arc<dyn Fn(&Config) -> Result<()> + Send + Sync + 'static>;

/// Shortest polling interval. `recv_timeout(0)` returns at once, so a zero
/// interval used to spin a core.
const MIN_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Most size checks made while waiting for a write to finish
const MAX_SETTLE_CHECKS: u32 = 50;

/// Size and modification time of the watched file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: Option<SystemTime>,
}

fn stamp(path: &Path) -> std::io::Result<Stamp> {
    let meta = std::fs::metadata(path)?;
    Ok(Stamp {
        len: meta.len(),
        modified: meta.modified().ok(),
    })
}

/// Why a reload did not happen
enum ReloadError {
    /// The file changed while it was read: a write is still going on
    Unsettled,
    /// The file could not be loaded, or the result was refused
    Failed(Error),
}

/// Everything a reload needs. Shared by [`HotReloadConfig::reload`] and the
/// background workers, so both apply the same checks.
#[derive(Clone)]
struct Reloader {
    current: Arc<RwLock<Config>>,
    file_path: PathBuf,
    handlers: Arc<HandlerList>,
    load: LoadOptions,
    allow_empty: bool,
    validator: Option<Validator>,
}

impl Reloader {
    fn io_error(&self, e: std::io::Error) -> ReloadError {
        ReloadError::Failed(Error::io(self.file_path.display().to_string(), e))
    }

    /// Load the file, check the result, and swap it in.
    ///
    /// The new configuration is refused, and the current one kept, when the
    /// file changed while it was read, when it holds no values but the
    /// current configuration does (unless empty reloads are allowed), when
    /// the caller's validator refuses it, or when the validation rules set
    /// on the current configuration find a critical error. Read-only mode,
    /// the defaults table and the validation rules carry over.
    ///
    /// The write lock is held only for the checks that need the current
    /// configuration and the swap; nothing is dispatched under it.
    fn reload(&self) -> std::result::Result<Stamp, ReloadError> {
        let before = stamp(&self.file_path).map_err(|e| self.io_error(e))?;
        let mut next =
            Config::from_file_with(&self.file_path, &self.load).map_err(ReloadError::Failed)?;
        let after = stamp(&self.file_path).map_err(|e| self.io_error(e))?;
        if before != after {
            return Err(ReloadError::Unsettled);
        }
        if let Some(validator) = &self.validator {
            validator(&next).map_err(ReloadError::Failed)?;
        }

        let mut current = self
            .current
            .write()
            .map_err(|_| ReloadError::Failed(Error::concurrency("configuration lock poisoned")))?;
        if !self.allow_empty && next.is_empty() && !current.is_empty() {
            return Err(ReloadError::Failed(Error::validation(
                "the file holds no values, so the current configuration is kept \
                 (a file read while it was being written looks like this; see \
                 HotReloadConfig::allow_empty_reload)",
            )));
        }
        next.inherit_runtime_state(&mut current);
        #[cfg(feature = "validation")]
        {
            match next.validate_critical_only() {
                Ok(errors) if errors.is_empty() => {}
                Ok(errors) => {
                    next.return_validation_rules(&mut current);
                    return Err(ReloadError::Failed(Error::validation(format!(
                        "the new configuration fails {} critical validation check(s), \
                         so the current configuration is kept",
                        errors.len()
                    ))));
                }
                Err(e) => {
                    next.return_validation_rules(&mut current);
                    return Err(ReloadError::Failed(e));
                }
            }
        }
        *current = next;
        drop(current);
        Ok(after)
    }

    fn dispatch(&self, event: ConfigChangeEvent) {
        self.handlers.dispatch(&event);
    }

    fn failed(&self, error: &Error) {
        self.dispatch(ConfigChangeEvent::ReloadFailed {
            path: self.file_path.clone(),
            error: error.to_string(),
            timestamp: SystemTime::now(),
        });
    }

    /// Reload and notify the handlers, after the lock is released. Returns
    /// the stamp to compare later changes against: the loaded file's on
    /// success, the refused file's on failure (so it is not retried until
    /// it changes again), and `last` when a write was still going on.
    fn reload_and_notify(&self, last: Option<Stamp>) -> Option<Stamp> {
        match self.reload() {
            Ok(loaded) => {
                self.dispatch(ConfigChangeEvent::Reloaded {
                    path: self.file_path.clone(),
                    timestamp: SystemTime::now(),
                });
                Some(loaded)
            }
            Err(ReloadError::Unsettled) => last,
            Err(ReloadError::Failed(e)) => {
                self.failed(&e);
                stamp(&self.file_path).ok().or(last)
            }
        }
    }

    /// Wait until the file's size and modification time stop changing, so
    /// a write in progress is not read half-way. Checks every `settle`, at
    /// most [`MAX_SETTLE_CHECKS`] times.
    fn wait_until_settled(&self, settle: Duration, stop: &AtomicBool) {
        let mut last = stamp(&self.file_path).ok();
        for _ in 0..MAX_SETTLE_CHECKS {
            thread::sleep(settle);
            if stop.load(Ordering::Relaxed) {
                return;
            }
            let now = stamp(&self.file_path).ok();
            if now == last {
                return;
            }
            last = now;
        }
    }

    /// Poll the file every `interval` until `stop` is set
    fn poll_loop(
        &self,
        stop: &AtomicBool,
        interval: Duration,
        settle: Duration,
        mut last: Option<Stamp>,
    ) {
        let mut deleted = false;
        while !stop.load(Ordering::Relaxed) {
            match stamp(&self.file_path) {
                Ok(now) => {
                    deleted = false;
                    if Some(now) != last {
                        self.dispatch(ConfigChangeEvent::FileModified {
                            path: self.file_path.clone(),
                            timestamp: SystemTime::now(),
                        });
                        self.wait_until_settled(settle, stop);
                        last = self.reload_and_notify(last);
                    }
                }
                Err(_) => {
                    if !deleted {
                        deleted = true;
                        self.dispatch(ConfigChangeEvent::FileDeleted {
                            path: self.file_path.clone(),
                            timestamp: SystemTime::now(),
                        });
                    }
                }
            }
            thread::sleep(interval);
        }
    }
}

// =========================================================================
// HotReloadConfig
// =========================================================================

/// Hot-reloadable configuration container.
///
/// Construct with [`HotReloadConfig::from_file`], then either drive
/// reloads manually with [`HotReloadConfig::reload`] or hand off to a
/// background watcher with [`HotReloadConfig::start_watching`].
///
/// Every reload applies the same checks before the new configuration
/// replaces the current one (since v1.4.0):
///
/// - the file must not change while it is read, and the watcher waits for
///   its size and modification time to settle first;
/// - a file that holds no values does not replace a configuration that
///   does, unless [`HotReloadConfig::allow_empty_reload`] is set;
/// - the [`HotReloadConfig::with_validator`] check, and the validation
///   rules set on the current configuration (feature `validation`), must
///   pass;
/// - read-only mode, the defaults table and the validation rules carry
///   over to the new configuration.
///
/// A refused reload keeps the current configuration and dispatches
/// [`ConfigChangeEvent::ReloadFailed`]. Handlers run after the new
/// configuration is in place and the lock is released, so a handler may
/// read the configuration.
///
/// Writers should still replace the file atomically (write a temporary
/// file, then rename it over the target, as [`Config::save_to_file`] does):
/// a writer that pauses for longer than the debounce window in the middle
/// of an in-place rewrite can leave a partial file that parses.
///
/// Configurable knobs (all consuming-builder style, intended for
/// fluent construction):
///
/// - [`HotReloadConfig::on_change`] — register a change handler
///   (v1.0.0+; the recommended notification API).
/// - [`HotReloadConfig::with_change_notifications`] — receive
///   [`ConfigChangeEvent`]s on an `mpsc` channel (deprecated since
///   v1.0.0; kept as a bridge for backwards compatibility).
/// - [`HotReloadConfig::with_debounce`] — adjust the debounce window
///   (default 100 ms).
/// - [`HotReloadConfig::with_poll_interval`] — set the polling
///   interval. Used directly when the `hot-reload` feature is off;
///   used as the watchdog interval when the feature is on.
/// - [`HotReloadConfig::with_polling_fallback`] — opt into a
///   parallel polling watchdog *in addition to* the event-driven
///   watcher, for environments where the kernel watcher is known
///   unreliable.
/// - [`HotReloadConfig::with_validator`] and
///   [`HotReloadConfig::allow_empty_reload`]: reload checks (v1.4.0+).
pub struct HotReloadConfig {
    /// Current configuration (thread-safe)
    current: Arc<RwLock<Config>>,
    /// File path being watched
    file_path: PathBuf,
    /// Last known modification time
    last_modified: SystemTime,
    /// Size of the file when it was last loaded
    last_len: Option<u64>,
    /// Lock-free handler list. Shared with the worker thread via
    /// `Arc<HandlerList>` once `start_watching` is called.
    handlers: Arc<HandlerList>,
    /// Bridge subscriptions kept alive by the deprecated
    /// `with_change_notifications` API. Each entry registers a
    /// closure that forwards events to the corresponding
    /// `mpsc::Sender`; dropping the subscription stops the
    /// forwarding. Stored here so the bridge lives at least as
    /// long as the `HotReloadConfig` itself, and is moved into the
    /// [`HotReloadHandle`] when `start_watching` consumes self.
    bridges: Vec<Subscription>,
    /// Polling interval — used as primary cadence when the
    /// `hot-reload` feature is off, or as the watchdog interval when
    /// the feature is on and `polling_fallback_enabled` is set.
    poll_interval: Duration,
    /// Debounce window applied to clustered file-change events.
    debounce: Duration,
    /// Whether to run the polling watchdog *in addition to* the
    /// event-driven watcher. Useful on network filesystems.
    polling_fallback_enabled: bool,
    /// Options every reload loads the file with
    load: LoadOptions,
    /// Whether an empty file may replace a non-empty configuration
    allow_empty: bool,
    /// Caller's check on every reloaded configuration
    validator: Option<Validator>,
}

impl HotReloadConfig {
    /// Create a new hot-reloadable configuration from a file.
    ///
    /// Loads with the default [`LoadOptions`], as [`Config::from_file`]
    /// does.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, parsed, or stat'd
    /// for its modification time.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file_with(path, LoadOptions::default())
    }

    /// Create a hot-reloadable configuration that reads NOML strictly
    /// (no `include`, no `env()`), on the first load and on every reload.
    ///
    /// Since v1.4.0.
    ///
    /// # Errors
    ///
    /// As [`HotReloadConfig::from_file`].
    pub fn from_file_strict<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file_with(path, LoadOptions::strict_defaults())
    }

    /// Create a hot-reloadable configuration that loads the file with
    /// `options`, on the first load and on every reload.
    ///
    /// Since v1.4.0.
    ///
    /// # Errors
    ///
    /// As [`HotReloadConfig::from_file`].
    pub fn from_file_with<P: AsRef<Path>>(path: P, options: LoadOptions) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let config = Config::from_file_with(&path, &options)?;

        let metadata =
            std::fs::metadata(&path).map_err(|e| Error::io(path.display().to_string(), e))?;
        let last_modified = metadata
            .modified()
            .map_err(|e| Error::io(path.display().to_string(), e))?;

        Ok(Self {
            current: Arc::new(RwLock::new(config)),
            file_path: path,
            last_modified,
            last_len: Some(metadata.len()),
            handlers: Arc::new(HandlerList::new()),
            bridges: Vec::new(),
            poll_interval: Duration::from_secs(1),
            debounce: DEFAULT_DEBOUNCE,
            polling_fallback_enabled: false,
            load: options,
            allow_empty: false,
            validator: None,
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
    ///
    /// Intervals below 10 ms are raised to 10 ms (since v1.4.0; a zero
    /// interval used to spin a core).
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval.max(MIN_POLL_INTERVAL);
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

    /// Let a reload whose file holds no values replace a configuration
    /// that has values.
    ///
    /// Off by default (since v1.4.0): a file that is truncated and then
    /// rewritten in place is empty for a moment, and reading it then would
    /// swap in an empty configuration, so every read would fall back to
    /// its default.
    pub fn allow_empty_reload(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    /// Check every reloaded configuration before it goes live.
    ///
    /// The check runs on the newly loaded configuration. When it returns
    /// an error, the current configuration is kept and
    /// [`ConfigChangeEvent::ReloadFailed`] carries the error's text. It
    /// does not run on the first load.
    ///
    /// Since v1.4.0.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use config_lib::hot_reload::HotReloadConfig;
    /// use config_lib::Error;
    ///
    /// # fn main() -> Result<(), Error> {
    /// let hot = HotReloadConfig::from_file("app.conf")?.with_validator(|config| {
    ///     if config.contains_key("server.port") {
    ///         Ok(())
    ///     } else {
    ///         Err(Error::validation("server.port is required"))
    ///     }
    /// });
    /// let _handle = hot.start_watching();
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&Config) -> Result<()> + Send + Sync + 'static,
    {
        self.validator = Some(Arc::new(validator));
        self
    }

    /// Register a change handler. **Recommended notification API**
    /// (v1.0.0+).
    ///
    /// The handler is invoked inline on the reloader thread every
    /// time a [`ConfigChangeEvent`] is produced — typically a few
    /// milliseconds after the underlying filesystem event, plus the
    /// debounce window. Dispatch overhead is a single atomic pointer
    /// load (~5 ns) plus the handler's own cost; multiple handlers
    /// are called in registration order with no per-handler channel
    /// allocation.
    ///
    /// Handlers run with no lock held: on
    /// [`ConfigChangeEvent::Reloaded`] the new configuration is already in
    /// place, and the handler may read it through
    /// [`HotReloadConfig::config`]. (Before v1.4.0 the background watcher
    /// dispatched while it still held the write lock, so a handler that
    /// read the configuration deadlocked every reader.)
    ///
    /// Returns a [`Subscription`] whose `Drop` unregisters the
    /// handler. Bind to a `let _sub = ...` if you want the handler
    /// to live for the surrounding scope, or call
    /// [`Subscription::forget`] to detach the drop hook.
    ///
    /// # Panics
    ///
    /// If the handler itself panics, the panic is caught via
    /// `catch_unwind` and discarded — other handlers continue to
    /// receive the event and the reloader thread is not torn down.
    /// Handler authors should avoid panicking, but a buggy handler
    /// won't take down the whole library.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use config_lib::hot_reload::{HotReloadConfig, ConfigChangeEvent};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let hot = HotReloadConfig::from_file("app.conf")?;
    /// let shared = hot.config();
    /// let _sub = hot.on_change(move |event: &ConfigChangeEvent| {
    ///     if let ConfigChangeEvent::Reloaded { .. } = event {
    ///         let port: u16 = shared.read().map(|c| c.get_or("port", 8080)).unwrap_or(8080);
    ///         println!("port is now {port}");
    ///     }
    /// });
    /// let _handle = hot.start_watching();
    /// // ... `_sub` and `_handle` are alive for the rest of the scope
    /// # Ok(())
    /// # }
    /// ```
    pub fn on_change<F>(&self, handler: F) -> Subscription
    where
        F: Fn(&ConfigChangeEvent) + Send + Sync + 'static,
    {
        let id = self.handlers.register(Arc::new(handler));
        Subscription {
            list: Arc::downgrade(&self.handlers),
            id,
        }
    }

    /// Enable channel-based change notifications. **Deprecated since
    /// v1.0.0** — prefer [`HotReloadConfig::on_change`].
    ///
    /// Internally bridges to the same lock-free handler list as
    /// `on_change` by registering a closure that forwards events
    /// to an `mpsc::Sender`. The bridge subscription is held by
    /// `self` so the channel keeps receiving events for the
    /// lifetime of the watcher.
    ///
    /// The bridge pays one extra `mpsc::Sender::send` per event
    /// (~100–200 ns) on top of the lock-free dispatch cost — the
    /// inverse of why `on_change` exists. Existing code using
    /// `Receiver<ConfigChangeEvent>` continues to work unchanged.
    #[deprecated(
        since = "1.0.0",
        note = "use `on_change` for lock-free dispatch; this method continues to work but pays an mpsc allocation per event"
    )]
    pub fn with_change_notifications(mut self) -> (Self, Receiver<ConfigChangeEvent>) {
        let (tx, rx) = mpsc::channel();
        let bridge = self.on_change(move |event| {
            let _ = tx.send(event.clone());
        });
        self.bridges.push(bridge);
        (self, rx)
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
        Config::from_file_with(&self.file_path, &self.load)
    }

    fn reloader(&self) -> Reloader {
        Reloader {
            current: Arc::clone(&self.current),
            file_path: self.file_path.clone(),
            handlers: Arc::clone(&self.handlers),
            load: self.load.clone(),
            allow_empty: self.allow_empty,
            validator: self.validator.clone(),
        }
    }

    /// Wait between size checks for a write to settle: half the debounce
    /// window, between 10 ms and 100 ms
    fn settle_time(&self) -> Duration {
        (self.debounce / 2).clamp(MIN_POLL_INTERVAL, DEFAULT_DEBOUNCE)
    }

    /// Manually trigger a reload check.
    ///
    /// Re-stats the file, and re-parses it if its modification time is
    /// newer or its size differs from the last load. Dispatches a
    /// [`ConfigChangeEvent::Reloaded`] or
    /// [`ConfigChangeEvent::ReloadFailed`] notification through the
    /// handler list, after the new configuration is in place.
    ///
    /// The new configuration goes through the checks listed on
    /// [`HotReloadConfig`]; when one refuses it, the current configuration
    /// is kept and the error is returned.
    ///
    /// Returns `Ok(true)` if a reload was performed, `Ok(false)` if
    /// the file was unchanged since the last check.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be stat'd, read, or parsed, if
    /// it changed while it was read, or if a reload check refused it.
    pub fn reload(&mut self) -> Result<bool> {
        let now = stamp(&self.file_path)
            .map_err(|e| Error::io(self.file_path.display().to_string(), e))?;
        let newer = now.modified.map_or(true, |m| m > self.last_modified);
        if !newer && Some(now.len) == self.last_len {
            return Ok(false);
        }

        let reloader = self.reloader();
        match reloader.reload() {
            Ok(loaded) => {
                if let Some(m) = loaded.modified {
                    self.last_modified = m;
                }
                self.last_len = Some(loaded.len);
                reloader.dispatch(ConfigChangeEvent::Reloaded {
                    path: self.file_path.clone(),
                    timestamp: SystemTime::now(),
                });
                Ok(true)
            }
            Err(ReloadError::Unsettled) => {
                let e = Error::general("the file changed while it was read; try again");
                reloader.failed(&e);
                Err(e)
            }
            Err(ReloadError::Failed(e)) => {
                reloader.failed(&e);
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
    ///
    /// If the event watcher cannot be started, or stops, the worker
    /// dispatches [`ConfigChangeEvent::ReloadFailed`] and polls the file
    /// instead (since v1.4.0; before, it stopped reloading).
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

    fn initial_stamp(&self) -> Option<Stamp> {
        Some(Stamp {
            len: self.last_len?,
            modified: Some(self.last_modified),
        })
    }

    // -----------------------------------------------------------------
    // Polling watcher — used as the primary watcher when the
    // `hot-reload` Cargo feature is disabled, and when the event-driven
    // watcher cannot start or stops.
    // -----------------------------------------------------------------

    #[cfg(not(feature = "hot-reload"))]
    fn start_watching_polling(self) -> HotReloadHandle {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);
        let reloader = self.reloader();
        let poll_interval = self.poll_interval;
        let settle = self.settle_time();
        let initial = self.initial_stamp();

        let handle = thread::spawn(move || {
            reloader.poll_loop(&stop_clone, poll_interval, settle, initial);
        });

        HotReloadHandle {
            handle: Some(handle),
            stop,
            handlers: self.handlers,
            _bridges: self.bridges,
        }
    }

    // -----------------------------------------------------------------
    // Event-driven watcher — gated on the `hot-reload` feature.
    // -----------------------------------------------------------------

    #[cfg(feature = "hot-reload")]
    fn start_watching_event_driven(self) -> HotReloadHandle {
        use notify::{Event, RecursiveMode, Watcher};

        let stop = Arc::new(AtomicBool::new(false));
        let reloader = self.reloader();
        let file_path = self.file_path.clone();
        let debounce = self.debounce;
        let settle = self.settle_time();
        let poll_interval = self.poll_interval;
        let polling_fallback = self.polling_fallback_enabled;
        let initial = self.initial_stamp();

        // Channel from the notify callback to the reload worker.
        // This `mpsc` is purely internal — between the notify
        // callback (which runs on `notify`'s own thread) and our
        // worker thread. It is NOT the user-facing notification
        // channel; that path is via `HandlerList::dispatch`.
        let (tx, rx) = mpsc::channel::<Event>();

        // Build the watcher. We watch the *parent* directory (not the
        // file itself) so that atomic-rename saves — where the file's
        // inode is replaced — still surface as events on our target.
        let watcher_dir = match file_path.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
            _ => PathBuf::from("."),
        };

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
                // The platform event API is unavailable (watch limit
                // reached, permissions, an unsupported filesystem). The
                // worker polls instead.
                reloader.failed(&Error::general(format!(
                    "the file watcher could not start ({e}); polling every {} ms instead",
                    poll_interval.as_millis()
                )));
                None
            }
        };
        let watching = watcher.is_some();

        // Reload worker — consumes events from the notify callback,
        // debounces, and re-parses on change.
        let target_file = file_path;
        let stop_for_worker = Arc::clone(&stop);

        let handle = thread::spawn(move || {
            let mut last = initial;
            if !watching {
                reloader.poll_loop(&stop_for_worker, poll_interval, settle, last);
                return;
            }
            let mut deleted = false;
            while !stop_for_worker.load(Ordering::Relaxed) {
                // Block up to `poll_interval` for the next event so
                // the stop flag is observed promptly even when the
                // file is quiet.
                let first = match rx.recv_timeout(poll_interval) {
                    Ok(ev) => Some(ev),
                    Err(mpsc::RecvTimeoutError::Timeout) => None,
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        reloader.failed(&Error::general(format!(
                            "the file watcher stopped; polling every {} ms instead",
                            poll_interval.as_millis()
                        )));
                        reloader.poll_loop(&stop_for_worker, poll_interval, settle, last);
                        return;
                    }
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
                match stamp(&target_file) {
                    Ok(now) => {
                        deleted = false;
                        if !relevant && Some(now) == last {
                            continue;
                        }

                        reloader.dispatch(ConfigChangeEvent::FileModified {
                            path: target_file.clone(),
                            timestamp: SystemTime::now(),
                        });
                        reloader.wait_until_settled(settle, &stop_for_worker);
                        last = reloader.reload_and_notify(last);
                    }
                    Err(_) => {
                        // File missing — likely deleted between
                        // events. Emit FileDeleted once but keep the
                        // last-known-good config in place.
                        if !deleted {
                            deleted = true;
                            reloader.dispatch(ConfigChangeEvent::FileDeleted {
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
            handlers: self.handlers,
            _bridges: self.bridges,
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

/// Handle for controlling the hot-reload background thread.
///
/// Returned by [`HotReloadConfig::start_watching`]. Dropping the
/// handle (or calling [`HotReloadHandle::stop`]) signals the worker
/// to exit and tears down the kernel-level watch registration.
/// Holding the handle keeps the watcher alive.
///
/// New handlers can be registered after `start_watching` via
/// [`HotReloadHandle::on_change`] — the handler list is shared
/// between the original `HotReloadConfig` and the spawned worker
/// thread.
pub struct HotReloadHandle {
    handle: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    /// Lock-free handler list shared with the worker thread.
    /// Carried here so that `on_change` continues to work after
    /// the original `HotReloadConfig` has been consumed by
    /// `start_watching`.
    handlers: Arc<HandlerList>,
    /// Bridge subscriptions kept alive for the lifetime of the
    /// watcher. The deprecated `with_change_notifications` path
    /// installed these; dropping the handle drops them, which
    /// unregisters the bridge handlers.
    _bridges: Vec<Subscription>,
    /// Watcher kept alive for the duration of the watch. Dropping
    /// the watcher tears down the kernel registration. Only carried
    /// when the `hot-reload` feature is on.
    #[cfg(feature = "hot-reload")]
    _watcher: Option<notify::RecommendedWatcher>,
}

impl HotReloadHandle {
    /// Register a change handler after `start_watching` has been
    /// called.
    ///
    /// Semantics match [`HotReloadConfig::on_change`]. Useful when
    /// the consumer of the handle is a different component from
    /// whoever called `start_watching` — pass the handle around and
    /// let each component install its own handler.
    pub fn on_change<F>(&self, handler: F) -> Subscription
    where
        F: Fn(&ConfigChangeEvent) + Send + Sync + 'static,
    {
        let id = self.handlers.register(Arc::new(handler));
        Subscription {
            list: Arc::downgrade(&self.handlers),
            id,
        }
    }

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
    use std::sync::atomic::AtomicUsize;
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
    #[allow(deprecated)]
    fn test_hot_reload_notifications_deprecated_bridge() {
        // Tests that the deprecated `with_change_notifications` bridge
        // still produces events through the new dispatch path.
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
    fn test_on_change_single_handler() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let mut hot = HotReloadConfig::from_file(&config_path).unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let _sub = hot.on_change(move |event| {
            if matches!(event, ConfigChangeEvent::Reloaded { .. }) {
                c.fetch_add(1, Ordering::Relaxed);
            }
        });

        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value2\n");
        hot.reload().unwrap();

        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_on_change_multiple_handlers_fire_in_order() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let mut hot = HotReloadConfig::from_file(&config_path).unwrap();
        let order: Arc<std::sync::Mutex<Vec<u8>>> = Arc::new(std::sync::Mutex::new(Vec::new()));

        let o1 = Arc::clone(&order);
        let _s1 = hot.on_change(move |_e| o1.lock().unwrap().push(1));
        let o2 = Arc::clone(&order);
        let _s2 = hot.on_change(move |_e| o2.lock().unwrap().push(2));
        let o3 = Arc::clone(&order);
        let _s3 = hot.on_change(move |_e| o3.lock().unwrap().push(3));

        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value2\n");
        hot.reload().unwrap();

        // Three handlers see the Reloaded event in registration order.
        // (Each `reload` produces exactly one Reloaded event.)
        let final_order = order.lock().unwrap().clone();
        assert_eq!(final_order, vec![1u8, 2, 3]);
    }

    #[test]
    fn test_on_change_drop_unregisters() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let mut hot = HotReloadConfig::from_file(&config_path).unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let sub = hot.on_change(move |_e| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        // First reload: handler fires.
        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value2\n");
        hot.reload().unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        // Drop the subscription — handler is unregistered.
        drop(sub);

        // Second reload: handler does NOT fire.
        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value3\n");
        hot.reload().unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_on_change_panic_isolation() {
        // A handler that panics must NOT prevent subsequent handlers
        // from running and must NOT poison the watcher thread.
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let mut hot = HotReloadConfig::from_file(&config_path).unwrap();
        let after_panic = Arc::new(AtomicUsize::new(0));

        let _s_panic = hot.on_change(|_e| {
            panic!("handler-side panic; should be swallowed");
        });
        let after = Arc::clone(&after_panic);
        let _s_after = hot.on_change(move |_e| {
            after.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value2\n");
        hot.reload().unwrap();

        // The second handler ran despite the first panicking.
        assert_eq!(after_panic.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_on_change_forget_keeps_handler_alive() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let mut hot = HotReloadConfig::from_file(&config_path).unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);

        // forget() — handler is now process-lifetime (or, more
        // precisely, lifetime of the HotReloadConfig's HandlerList).
        hot.on_change(move |_e| {
            c.fetch_add(1, Ordering::Relaxed);
        })
        .forget();

        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value2\n");
        hot.reload().unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        thread::sleep(Duration::from_millis(10));
        write_conf(&config_path, "key=value3\n");
        hot.reload().unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_handle_on_change_after_start_watching() {
        // Verify that `HotReloadHandle::on_change` works post-
        // start_watching — handlers registered via the handle see
        // events the same as handlers registered before.
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let hot = HotReloadConfig::from_file(&config_path)
            .unwrap()
            .with_poll_interval(Duration::from_millis(50))
            .with_debounce(Duration::from_millis(25));

        let handle = hot.start_watching();

        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let _sub = handle.on_change(move |_e| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(Duration::from_millis(150));
        write_conf(&config_path, "key=value2\n");
        thread::sleep(Duration::from_millis(500));

        assert!(
            counter.load(Ordering::Relaxed) >= 1,
            "handle.on_change handler never fired"
        );

        handle.stop().unwrap();
    }

    #[test]
    fn test_automatic_watching() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.conf");
        write_conf(&config_path, "key=value1\n");

        let counter = Arc::new(AtomicUsize::new(0));
        let hot = HotReloadConfig::from_file(&config_path)
            .unwrap()
            .with_poll_interval(Duration::from_millis(50))
            .with_debounce(Duration::from_millis(25));

        let c = Arc::clone(&counter);
        let _sub = hot.on_change(move |event| {
            if matches!(event, ConfigChangeEvent::Reloaded { .. }) {
                c.fetch_add(1, Ordering::Relaxed);
            }
        });

        let config_ref = hot.config();
        let handle = hot.start_watching();

        thread::sleep(Duration::from_millis(100));
        write_conf(&config_path, "key=value2\n");
        thread::sleep(Duration::from_millis(500));

        {
            let config_read = config_ref.read().unwrap();
            assert_eq!(
                config_read.get("key").unwrap().as_string().unwrap(),
                "value2"
            );
        }
        assert!(
            counter.load(Ordering::Relaxed) >= 1,
            "expected at least one Reloaded event"
        );

        handle.stop().unwrap();
    }
}
