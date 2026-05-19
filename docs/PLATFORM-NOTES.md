# Platform Notes

This document captures platform-specific behaviour that affects how
`config-lib` operates on Linux, macOS, and Windows. The library is
designed to behave identically across all three — these notes call
out the exceptions and the reasons for them.

## Hot reload (event-driven via `notify`, v0.9.6+)

`HotReloadConfig::start_watching()` registers a
`notify::RecommendedWatcher` on the **parent directory** of the
watched file (not on the file inode directly). Watching the parent
directory is essential because most editors save by writing to a
temporary file and atomically renaming it over the target — the
original inode disappears, and watchers attached to the inode would
silently stop seeing events after the first save.

The cross-platform abstraction sits on top of three different kernel
APIs:

| Platform | Backend                          | Notes                                                                                   |
|----------|----------------------------------|-----------------------------------------------------------------------------------------|
| Linux    | `inotify`                        | Per-watch-descriptor limit (`/proc/sys/fs/inotify/max_user_watches`) — usually 8192+.   |
| macOS    | `FSEvents`                       | Coalesces rapid events at the kernel level; debounce window may overlap with kernel coalescing. |
| Windows  | `ReadDirectoryChangesW`          | Reports paths in 8.3 / NTFS-canonical form; the library canonicalises before comparison. |

### Debounce window

`HotReloadConfig` applies a debounce window (default 100 ms,
configurable via [`HotReloadConfig::with_debounce`]) before triggering
a reload. The debounce collapses bursts of events from a single save
(typical for atomic-rename editors: `create temp`, `write temp`,
`rename temp over target`, `remove temp`) into one `Reloaded`
notification.

If your editor save semantics are unusual — for example, a tool that
writes a single 50 MB config in chunks across multiple seconds — you
may want to raise the debounce. Conversely, a CI test that wants the
fastest possible reaction can drop the debounce to ~10 ms:

```rust
let cfg = HotReloadConfig::from_file("app.conf")?
    .with_debounce(std::time::Duration::from_millis(10));
```

### Latency

Typical detection latency, from `fsync()` returning on the writer to
the `Reloaded` notification being delivered, on a quiescent system:

- **Linux (inotify):** 2–10 ms before debounce, 100–110 ms after.
- **macOS (FSEvents):** 5–30 ms before debounce. FSEvents adds its
  own ~10 ms coalescing layer. After our debounce, 100–130 ms.
- **Windows (RDCW):** 10–40 ms before debounce, 100–140 ms after.

All comfortably under the v1.0 stability contract's <100 ms target
*before* debounce. The debounce window itself is the dominant
post-debounce latency contributor — tune it to taste.

### Network filesystems and unusual filesystem layers

`notify::RecommendedWatcher` relies on the kernel surfacing events
synchronously. Several environments do not:

- **NFS** — silent. `inotify` does not observe events from remote
  writers. Use [`HotReloadConfig::with_polling_fallback`] when
  watching files on NFS mounts. (The opt-in parallel polling
  watchdog is reserved for a follow-up release; v0.9.6 surfaces a
  `ReloadFailed` event if the kernel watcher cannot be constructed,
  and you should fall back to manual `HotReloadConfig::reload()`
  calls in that case.)
- **SMB / CIFS** — same caveat as NFS.
- **OverlayFS (some Docker layered FS)** — partial support;
  events on the *upper* layer are reliable, but writes into the
  underlying lower layer are not observed by watchers on the upper
  layer. Behaviour depends on the container runtime version.
- **macOS FSEvents inside a Docker volume mount** — Docker for Mac
  has had a history of FSEvents propagation issues across the
  host→container boundary. If you're hot-reloading config across
  the boundary, opt into the polling fallback.

### File deletion / re-creation

If the watched file is deleted and re-created (a less common but
legitimate save pattern), v0.9.6 emits `FileDeleted` and then a
`Reloaded` event when the new file appears. The
last-known-good `Config` is preserved in `Arc<RwLock<Config>>`
between the deletion and the re-creation — readers see the old
config until the new one is parsed and atomically swapped in.

### Permissions changes

If the watched file becomes unreadable (e.g. a `chmod` that strips
your read bit), the next reload attempt emits `ReloadFailed` with
the underlying `Error::io` as context. The last-known-good `Config`
remains in place; the watcher does not give up.

## Line endings (all platforms)

All parsers accept `\n`, `\r\n`, and `\r` line endings.
Configurations written on Windows with CRLF terminators are loaded
identically to the same configuration written on Unix with LF
terminators. The serializer writes the host platform's default
line ending; we do not attempt to preserve the original terminator
mid-document.

## Paths

`config-lib` uses `std::path::Path` and `PathBuf` throughout; raw
string paths are never assumed. Forward slashes and backslashes
are both accepted on Windows. Trailing slashes and `.`/`..`
components are normalised by the OS at `read_to_string` time;
no further normalisation is applied.

## Async file I/O (`async` feature)

When the `async` feature is enabled, `Config::from_file_async` and
`Config::save_async` use `tokio::fs` rather than `std::fs`. On
Linux and macOS, `tokio::fs` blocks on a pool of worker threads
because the kernel APIs are inherently blocking; this means async
file I/O is **not faster** than sync file I/O on a single-threaded
benchmark — its benefit is that it does not block the current
async runtime's executor thread.

On Windows, `tokio::fs` similarly uses a worker-thread pool.

## Filesystem timestamps

On Linux and macOS, `std::fs::Metadata::modified()` returns
nanosecond-resolution timestamps. On most Windows filesystems
(NTFS), the resolution is 100 ns; on FAT32 it can be as coarse as
2 s. The `last_modified` comparison inside `HotReloadConfig`
treats "equal mtime" as "unchanged", so on FAT32 a save within the
same 2 s window as the previous save may be missed if it happens
to round to the same timestamp. Use the event-driven watcher
(`hot-reload` feature, default in v0.9.6+) rather than polling
when targeting low-resolution filesystems.

## NOML / TOML format preservation

Format preservation for NOML and TOML configurations (comments,
whitespace, key ordering on save) depends on the `noml` crate's
`Document` API. The behaviour is identical on all three platforms.

When the `noml` feature is **off** (planned default in Phase 0.9.7),
NOML and TOML format preservation is unavailable. JSON, XML, and
HCL never had format preservation — they round-trip through
`serde_json` / `quick-xml` / the internal HCL parser without
preserving comments or whitespace. This is unchanged across
platforms.
