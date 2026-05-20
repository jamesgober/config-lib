# config-lib — Architecture

This document is the internal-design reference for `config-lib`. It
covers module layout, the data-flow from file bytes to user-visible
`Config`, the caching architecture introduced in v0.9.9, the
hot-reload architecture introduced in v0.9.6, the thread-safety
guarantees, and the decision log for the more contested choices.

If you are reading this to make changes to the crate, you are in the
right place. If you are reading it as a user of the crate, prefer
`README.md` for getting started and `docs/STABILITY-1.0.md` for the
public contract — this document discusses internals that are
deliberately not part of the 1.0 SemVer surface.

---

## 1. Module layout

```
src/
├── lib.rs            — crate root + REPS lint discipline + re-exports
├── error.rs          — `Error` enum + constructor helpers (#[non_exhaustive])
├── value.rs          — `Value` enum (Null/Bool/Integer/Float/String/Array/Table)
├── config.rs         — `Config`, `ConfigOptions`, `ConfigBuilder`,
│                       `ConfigValue`, `CacheStats` (v0.9.5+)
├── enterprise.rs     — `ConfigManager` (multi-instance primitive),
│                       deprecated `EnterpriseConfig` + `direct::*` shims
├── hot_reload.rs     — `HotReloadConfig`, `HotReloadHandle`,
│                       `ConfigChangeEvent`; `notify`-backed watcher
│                       behind the `hot-reload` Cargo feature
├── audit.rs          — `AuditEvent`, `AuditLogger`, sink trait
├── env_override.rs   — `apply_env_overrides` (feature: env-override)
├── schema.rs         — `Schema`, `SchemaBuilder` (feature: schema)
├── validation.rs     — rule-based validation (feature: validation)
└── parsers/          — one file per format
    ├── mod.rs        — `detect_format`, `parse_string`, `parse_file`
    ├── conf.rs       — built-in CONF parser
    ├── ini_parser.rs — built-in INI parser
    ├── properties_parser.rs   — Java .properties parser
    ├── json_parser.rs         — serde_json wrapper (feature: json)
    ├── xml_parser.rs          — quick-xml wrapper (feature: xml)
    ├── hcl_parser.rs          — built-in HCL parser
    ├── noml_parser.rs         — noml crate wrapper (feature: noml)
    └── toml_parser.rs         — TOML via noml (feature: toml)
```

Re-exports at the crate root (`lib.rs`):

- `Config`, `ConfigBuilder`, `ConfigOptions`, `ConfigValue`, `CacheStats`
- `ConfigManager` (multi-instance)
- `EnterpriseConfig` (deprecated since v0.9.4 — see §6)
- `Error`, `Result`, `Value`
- `Schema`, `SchemaBuilder` (when `schema` feature is on)
- Validation types (when `validation` feature is on)
- Free functions: `parse`, `parse_file`, `validate`, `parse_file_async`

---

## 2. Data flow

```
                   ┌─────────────────────────────┐
        bytes ────▶│  parsers::detect_format     │
                   └────────────────┬────────────┘
                                    │ format hint
                                    ▼
                   ┌─────────────────────────────┐
                   │  parsers::<format>::parse   │
                   └────────────────┬────────────┘
                                    │ Value
                                    ▼
                   ┌─────────────────────────────┐
                   │  Config { values: Value }   │
                   └────────────────┬────────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              ▼                     ▼                     ▼
   ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
   │ Config::get      │  │ Config::get_arc  │  │ HotReloadConfig  │
   │ → Option<&Value> │  │ → Option<        │  │  background      │
   │ (single-thread,  │  │     Arc<Value>>  │  │  watcher swaps   │
   │  zero-copy)      │  │ (thread-safe,    │  │  values atomically │
   │                  │  │  cached)         │  │                  │
   └──────────────────┘  └──────────────────┘  └──────────────────┘
```

### Parse step

`parsers::detect_format` inspects the first few hundred bytes for
format-identifying patterns: JSON's `{` or `[`, XML's `<?xml`, TOML's
`[section]` style, etc. If detection fails, falls back to `conf`.
Per-format `parse` functions then take a `&str` and emit a `Value`
tree. Errors surface as `Error::Parse { line, column, message }`.

### Construct step

`Config::from_string(s, fmt)` and `Config::from_file(path)` wrap the
parse step, attach format-preservation data for NOML/TOML (when
those features are on), and initialise the cache + defaults table
(both empty).

### Access step

Two accessors with different semantics:

- **`Config::get(&self, path) -> Option<&Value>`**: walks the value
  tree directly, returns a borrowed reference whose lifetime is tied
  to `&self`. Zero allocation, single-threaded scope. The right
  choice for "peek and drop the borrow" use cases.
- **`Config::get_arc(&self, path) -> Option<Arc<Value>>`**: consults
  the resolved-path cache; on hit returns a cheap `Arc::clone` of
  the cached `Arc<Value>`. On miss, walks the tree, allocates an
  `Arc<Value>` containing a clone of the resolved node, populates
  the cache, and returns it. The right choice for multi-threaded
  reads and hot loops. v0.9.9+.

---

## 3. Caching architecture

The cache layer (v0.9.9+) sits behind `Config::get_arc`.

### Backend: `DashMap<Box<str>, Arc<Value>>`

`DashMap` is a sharded concurrent `HashMap`. Reads acquire a
short shard-local lock; writes acquire a single shard's write lock.
Under the read-mostly access pattern that defines a configuration
library's hot path, contention is effectively zero. The key type is
`Box<str>` rather than `String` to keep the hot-path key small (24
bytes vs 24 bytes — same size, but `Box<str>` makes the immutability
intent explicit in the type).

### Decision log: why DashMap over alternatives

| Backend                       | Pro                                            | Con                                                       |
|-------------------------------|------------------------------------------------|-----------------------------------------------------------|
| `Arc<RwLock<BTreeMap>>` (v0.9.x EnterpriseConfig) | simple, in-stdlib                  | reads serialize on lock; doesn't scale past ~4 threads    |
| `parking_lot::RwLock<HashMap>` | faster RwLock; no shard logic                 | still serializes; same scaling ceiling                    |
| **`DashMap`**                 | sharded; near-linear scaling to ~16 threads    | adds a dep; per-shard locks have small constant cost      |
| `ArcSwap<Arc<HashMap>>`       | truly lock-free reads                          | every write clones the whole map (bad for cache invalidation pattern) |
| `evmap` (left/right paired)   | lock-free reads, optimised for read-heavy      | complex; eventually-consistent semantics surprise users   |

DashMap was chosen because it matches the workload shape
(read-mostly, occasional invalidation) and offers the simplest API
without sacrificing the sub-50 ns warm-read target. The decision
was made for v0.9.9 alongside `Config::get_arc` and is documented
here as the canonical justification.

### Invalidation

The cache is invalidated **wholesale** on every `Config::set` /
`Config::remove` / `Config::merge`. A per-prefix invalidation
strategy is post-1.0 backlog — writes are rare in the design
envelope, and the design goal of "no stale reads after a write" is
easier to reason about with a `clear()` per write than with prefix
matching.

The `Config::clear_cache()` public method is the explicit
invalidation hook for callers that have changed the underlying
data via a path the cache doesn't observe (e.g., the hot-reload
watcher swapping the `Arc<RwLock<Config>>` contents).

### Counter ordering

`Config::cache_hits` and `Config::cache_misses` are `AtomicU64`
loaded and incremented with `Ordering::Relaxed`. The hit/miss
classification is best-effort — under concurrent reads, two threads
may both increment the miss counter for the same key before either
populates the cache. This is acceptable because the counters are
diagnostic, not behavioural.

---

## 3a. Notification dispatch (v1.0.0+)

`HotReloadConfig::on_change` and `HotReloadHandle::on_change` are the
public surface for receiving `ConfigChangeEvent`s. The dispatch path
is **lock-free, allocation-free, and channel-free** — handlers run
inline on the reloader thread.

### Backend: `ArcSwap<Vec<(u64, Arc<dyn Fn>)>>`

The handler list lives behind an `arc_swap::ArcSwap` pointer:

```text
HandlerList {
    handlers: ArcSwap<Vec<(u64, Handler)>>,  // <-- snapshot pointer
    next_id:  AtomicU64,                      // <-- monotonic ids
}
```

- **Dispatch** (`HandlerList::dispatch`) issues one
  `ArcSwap::load` — a relaxed atomic pointer load — then iterates
  the resulting `&[(u64, Handler)]`. Each handler is invoked
  through `Arc<dyn Fn>` (refcount bump only), inside a
  `catch_unwind` so a panicking handler can't take down the
  watcher or block subsequent handlers.

- **Register** (`HandlerList::register`) issues an `ArcSwap::rcu`
  copy-on-write: allocate a new Vec with capacity `n + 1`, copy
  the old contents, push the new handler, atomic-swap the pointer.
  An in-flight `dispatch` continues iterating the old snapshot
  safely; the next dispatch sees the new handler.

- **Unregister** (`HandlerList::unregister`) does the same RCU
  pattern with one entry filtered out. Idempotent.

### Subscription lifecycle

`HotReloadConfig::on_change` returns a [`Subscription`] — an RAII
guard whose `Drop` impl calls `HandlerList::unregister(id)`. The
common pattern is `let _sub = hot.on_change(...);` — the handler
runs for the surrounding scope and unregisters at scope exit. For
process-lifetime handlers, `Subscription::forget()` detaches the
drop hook.

The handler list is shared between the `HotReloadConfig` and its
spawned worker thread via `Arc<HandlerList>`, so handlers can be
registered both **before** `start_watching` (on the config) and
**after** (on the returned `HotReloadHandle`).

### Why not registry-io?

An early v1.0.0 design proposal pulled in the
[`registry-io`](https://crates.io/crates/registry-io) crate for
the same `ArcSwap`-backed dispatch pattern. We rejected it: the
performance characteristics are identical (it's the same
underlying primitive), but it adds a dependency, a wrapper type
mismatch with our existing `Receiver`-style API, and a
1-for-1-replacement-shape problem. We took the load-bearing
primitive (`arc-swap`) and built the dispatch surface in-tree.
See the canonical decision log in `.dev/registry-io-integration.md`
and the v1.0.0 release notes for the full rationale.

### The deprecated `with_change_notifications` bridge

For v0.9.x source compatibility, `with_change_notifications()`
still returns `(Self, Receiver<ConfigChangeEvent>)`. Internally it
calls `on_change` with a closure that forwards each event into an
`mpsc::Sender`. The bridge `Subscription` is stored in
`HotReloadConfig::bridges` (and moves into `HotReloadHandle` on
`start_watching`) so the channel keeps receiving events for the
lifetime of the watcher. The bridge pays one `mpsc::send` per event
on top of the lock-free dispatch — exactly the cost the new API
exists to avoid. Marked `#[deprecated(since = "1.0.0")]`.

---

## 4. Hot reload architecture

The hot-reload layer (v0.9.6+) lives in `src/hot_reload.rs`. It is
behind the `hot-reload` Cargo feature, which is default-on.

### Watcher topology

When the `hot-reload` feature is on, `HotReloadConfig::start_watching`
registers a `notify::RecommendedWatcher` on the **parent directory**
of the watched file rather than on the file inode. This is essential
because most editors save by writing to a temporary file and
atomically renaming it over the target — the original inode
disappears, and an inode-attached watcher would silently stop
observing events after the first save. Watching the parent directory
and filtering events to the target path is what `cargo-watch`,
`mdbook-serve`, and the wider Rust filesystem-watching ecosystem do.

### Event pipeline

```
   kernel (inotify / FSEvents / RDCW)
            │
            ▼
   notify::RecommendedWatcher callback
            │  Event { kind, paths }
            ▼
   mpsc::Sender<Event>
            │
            ▼
   reload-worker thread
            │  ┌──── filter to target path (canonical-form compare)
            │  ├──── debounce: drain channel for `debounce` window
            │  ├──── re-stat target; check mtime monotonicity
            │  ├──── parse file (Config::from_file)
            │  └──── atomic swap into Arc<RwLock<Config>>
            ▼
   ConfigChangeEvent { Reloaded / ReloadFailed / FileDeleted /
                       FileModified } emitted on
                       mpsc::Sender<ConfigChangeEvent>
```

The debounce window (default 100 ms, tunable via
`HotReloadConfig::with_debounce`) is what collapses the burst of
events that an atomic-rename save produces — `Remove`, `Create`,
`Modify` in rapid succession — into a single `Reloaded`
notification.

When the `hot-reload` feature is **off**, the polling worker takes
over: same `mpsc::Sender<ConfigChangeEvent>` API, but the inner
worker is a `thread::sleep(poll_interval)` loop that re-stats the
file periodically. The user-visible API is identical.

### Shutdown

`HotReloadHandle` carries an `Arc<AtomicBool>` shutdown flag. On
`HotReloadHandle::stop()` or `Drop`, the flag is set; the worker
checks it on every channel timeout and exits. The watcher itself
is owned by the handle (`_watcher: Option<notify::RecommendedWatcher>`
when the feature is on); dropping the watcher tears down the
kernel-level watch registration.

---

## 5. Thread safety

| Type                       | Send | Sync | Notes                                                                       |
|----------------------------|------|------|-----------------------------------------------------------------------------|
| `Config`                   | yes  | yes  | All fields are `Send + Sync` (DashMap, Arc, atomic counters, primitives)    |
| `ConfigOptions`            | yes  | yes  | Plain data (`bool`, `usize`)                                                |
| `CacheStats`               | yes  | yes  | Plain data (`u64`, `f64`)                                                    |
| `ConfigManager`            | yes  | yes  | Internal `Arc<RwLock<HashMap<String, Arc<RwLock<Config>>>>>`                |
| `Value`                    | yes  | yes  | `BTreeMap` and `Vec` of `Send + Sync` variants                              |
| `Error`                    | yes  | yes  | Auto-derived; `io::Error` is `Send + Sync`                                  |
| `HotReloadConfig`          | yes  | yes  | Carries `Arc<RwLock<Config>>` internally                                    |
| `HotReloadHandle`          | yes  | yes  | Owns `thread::JoinHandle`, `Arc<AtomicBool>`, optional `notify::Watcher`    |

### Sharing patterns

- **Single thread**: hold a `Config` directly. Use `get` (borrowed)
  for peeks, `set` / `remove` / `merge` for mutations.
- **Multi-thread, read-only**: wrap `Config` in `Arc<Config>` and
  share. `get_arc` is the right accessor; the cache scales.
- **Multi-thread, mutable**: `Arc<RwLock<Config>>` — `ConfigManager`
  hands these out by default. Writers acquire the write lock,
  readers acquire the read lock. The internal DashMap is itself
  thread-safe; the RwLock guards the rest of the struct's mutable
  state (file_path, modified flag, etc.).

### Lock ordering

Within the crate's internal code, the lock hierarchy is:

1. `ConfigManager.configs` lock (outermost)
2. Individual `Config` RwLocks (held briefly to clone the `Arc`)
3. `Config.cache` DashMap shard locks (innermost, very brief)
4. `Config.defaults` RwLock (separate from the above)

No code holds (2) while acquiring (1), no code holds (3) while
acquiring (1) or (2), and no code holds (4) while acquiring (1) or
(2). This eliminates the possibility of deadlock between the lock
levels.

---

## 6. The `EnterpriseConfig` deprecation

`EnterpriseConfig` was the v0.4.x – v0.9.3 cached-and-thread-safe
configuration type. v0.9.9 has folded its capabilities into the
unified `Config`: the cache layer lives behind `Config::get_arc`,
the defaults table is `ConfigOptions::defaults` (accessed via
`Config::set_default` / `Config::get_or_default`), and `read_only`
is the `ConfigOptions::read_only` knob (or `Config::make_read_only()`
post-construction).

`EnterpriseConfig` remains in the crate marked `#[deprecated(since
= "0.9.4")]` for backwards compatibility through the v0.9.x line
and well into v1.x. It is scheduled for removal in `v2.0.0` per the
stability contract's deprecation policy (one MINOR cycle minimum,
six months minimum). Users have a clear migration path:

```text
EnterpriseConfig::get(k)               →  Config::get_arc(k)
EnterpriseConfig::get_or(k, default)   →  Config::get_or_default(k)
EnterpriseConfig::set(k, v)            →  Config::set(k, v)
EnterpriseConfig::set_default(k, v)    →  Config::set_default(k, v)
EnterpriseConfig::cache_stats()        →  Config::cache_stats()
EnterpriseConfig::make_read_only()     →  Config::make_read_only()
ConfigManager::get(name)               →  Same shape, now returns
                                          Arc<RwLock<Config>> instead
                                          of Arc<RwLock<EnterpriseConfig>>
```

The `ConfigManager` API surface itself is unchanged across the
migration — only the `Config` vs `EnterpriseConfig` payload type
moved. `ConfigManager` was un-deprecated in v0.9.9 alongside the
storage migration.

---

## 7. Parsers

Each parser is an independent module in `src/parsers/`. The shared
contract:

```rust
pub fn parse(source: &str) -> Result<Value>;
```

Every parser returns a `Value::Table` for non-empty input, or an
appropriate `Error::Parse { line, column, message }` for syntax
errors. There is no shared parser-state, no per-format trait —
the dispatch is a `match` on the detected format string inside
`parsers::parse_string`.

Three parsers are built-in: `conf`, `hcl`, `properties`. Three are
opt-in wrappers around upstream crates: `json` (serde_json), `xml`
(quick-xml), `noml` + `toml` (noml). Format-preservation on save is
available only for NOML/TOML, via the upstream noml crate's
`Document` type.

The fuzz harnesses in `fuzz/fuzz_targets/<parser>.rs` exercise each
`parse` function on arbitrary `&[u8]` input. See `docs/SECURITY.md`
for the fuzz methodology and `tests/parser_corpus.rs` for the
regression-test infrastructure populated from fuzz findings.

---

## 8. Where to look for what

| If you're trying to …                              | Read …                                              |
|----------------------------------------------------|-----------------------------------------------------|
| Add a new format parser                            | `src/parsers/mod.rs` (dispatch), then a new module  |
| Change the cache backend                           | `src/config.rs` (the `cache` field and `get_arc`)   |
| Tune hot-reload latency                            | `src/hot_reload.rs` + `docs/PLATFORM-NOTES.md`      |
| Audit the security posture                         | `docs/SECURITY.md`                                  |
| Verify a 1.0 stability promise                     | `docs/STABILITY-1.0.md`                             |
| Verify a performance number                        | `docs/PERFORMANCE.md` + `benches/`                  |
| Trace an error variant to its callers              | `src/error.rs` constructor methods + `grep`         |
| Understand the `ConfigManager` semantics           | This file, §5 ("Thread safety")                     |

---

<sub>Last reviewed: 2026-05-19 (v0.9.9).</sub>
