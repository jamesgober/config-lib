# config-lib — Production Roadmap to 1.0

> **The engineering contract that takes `config-lib` from `0.9.1` to `1.0.0`.**
> Every phase has explicit, measurable exit criteria. Every claim must be backed by a benchmark or test before it ships in the README or rustdoc.
>
> **Reads:** `REPS.md` (supreme authority), `_strategy/UNIVERSAL_PROMPT.md` (peak performance + max efficiency + max concurrency + nuclear-proof security + cross-platform), `.dev/AUDIT-0.9.1.md` (current state assessment).
>
> **Target ship date:** 4-6 focused weeks from audit (2026-05-18). **Met.**
> **Status:** **`v1.0.0` shipped (2026-05-19).** The original "registry-io integration for v1.0.0" plan was scrapped after analysis showed it would have added a dependency boundary and 3-4 days of refactor work for zero perf gain over an in-tree `ArcSwap`-backed implementation. v1.0.0 instead delivers an **in-house lock-free notification subsystem** (`HotReloadConfig::on_change` + `Subscription` + `HandlerList`) that takes only the load-bearing primitive (`arc-swap = "1.7"`) without the wrapper crate. Hardware-deferred verification (committed benchmark numbers + 1-CPU-hour fuzz runs) lands as a v1.0.1 patch on canonical hardware.

---

## The 1.0 contract

When `config-lib 1.0.0` ships, it commits to:

### Functional contract

- **Single unified `Config` API** — consolidates current `Config` + `EnterpriseConfig`. `EnterpriseConfig` retained as `#[deprecated]` alias.
- **8 format parsers** — CONF, INI, Properties, JSON, XML, HCL **in default features**. NOML, TOML **opt-in only** (depend on pre-1.0 `noml` crate).
- **Event-driven hot reload** — `notify` crate (inotify/FSEvents/RDCW). Polling fallback opt-in only.
- **Audit logging** — compliance-grade structured logs with multiple sinks.
- **Environment variable overrides** — prefix-based, case-insensitive, typed.
- **Schema validation** — trait-based rule engine.
- **Multi-instance** — `ConfigManager` for named config instances within one process.
- **Write support** — `config.set()` + `config.save()` round-trip (format preservation for NOML/TOML via upstream crate; other formats document the limitation honestly).
- **Cross-platform** — Linux, macOS, Windows. Verified identical behavior on all three.

### Performance contract (every number verified by committed benchmark)

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Single-key cached `get` (warm, 1 thread) | **<50ns** | `criterion`, tight loop, current dev hardware |
| Single-key cached `get` (warm, 16 threads contended) | **<50ns** | `criterion`, parametric contention |
| Single-key cached `get` (cold, miss → populate) | **<5µs** | `criterion`, first-access path |
| Nested-key cached `get` (3 levels deep) | **<100ns** | `criterion`, dotted-key resolution |
| Typed accessor (`as_string`, `as_integer`, etc.) | **<10ns** | `criterion`, zero-allocation |
| `config.set()` cached write | **<500ns** | `criterion`, cache invalidation included |
| Hot reload detection latency | **<100ms** | Integration test, file modification → event |
| Cold parse — 1 KiB CONF file | **<10µs** | `criterion`, end-to-end |
| Cold parse — 100 KiB JSON file | **<500µs** | `criterion`, end-to-end |
| Memory overhead — empty `Config` instance | **<1 KiB** | `dhat` or manual sizeof analysis |
| Memory overhead — `Config` with 1000 cached keys | **<128 KiB** | `dhat` |

**Rule:** if a number above is not verified by a committed benchmark, the version that claims it does NOT ship.

### Stability contract

- **Public API frozen.** Every `pub` item in the crate root and in `pub` modules is part of the SemVer contract.
- **`#[non_exhaustive]`** on every enum that may grow (Error, ConfigChangeEvent, ValidationRule, ValidationSeverity).
- **MSRV 1.75** held for v1.x. Bumps within the last-12-stable-Rust-versions window in MINOR releases. PATCH releases never bump MSRV.
- **Edition 2024.**
- **Apache-2.0 OR MIT** dual licensed.
- **Deprecation policy:** items marked `#[deprecated]` keep working for at least one full MINOR cycle (target: 6 months minimum) before removal in the next MAJOR.
- **Yank policy:** critical correctness bugs trigger yank + same-day patch. Performance regressions do not.

### Security contract (nuclear-proof requirement)

- **Zero unsafe code** in the public API. Internal `unsafe {}` blocks (if any) carry `// SAFETY:` comments + are exercised by Miri.
- **Every parser fuzzed** for at least 1 CPU-hour without finding a panic, infinite loop, or OOM.
- **No untrusted input reaches `unwrap()` / `expect()`.** Enforced by lint.
- **`cargo audit` clean** at release. No known vulnerabilities in the dependency tree.
- **`cargo deny check` clean** at release. No license/policy violations.
- **No secrets logged.** Audit logging redaction policy documented and enforced.

### Quality contract

- Full REPS lint discipline in `src/lib.rs` (every lint deny listed in the directives).
- `cargo fmt --all -- --check` clean.
- `cargo clippy --all-targets --all-features -- -D warnings` clean.
- `cargo test --all-features` passing on Linux, macOS, Windows on stable + MSRV.
- `cargo doc --no-deps --all-features` produces zero warnings with `RUSTDOCFLAGS="-D warnings"`.
- Every public item: rustdoc + at least one runnable example.
- Every public function returning `Result`: a `# Errors` section.
- Every error variant: documented + tested.

---

## Phase 0.9.2 — Structure normalization

**Goal:** Bring the repository structure to portfolio standard. **No code logic changes.** Mechanical work only.

**Effort:** 1-2 days.

**Status:** Complete (2026-05-19). Released as [`v0.9.2`](../.dev/release/v0.9.2.md).

### Tasks

- [x] Audit document committed (`.dev/AUDIT-0.9.1.md`)
- [x] Roadmap committed (this file)
- [x] `REPS.md` at repo root (canonical, 47 KB, copied from `_strategy/REPS.md`)
- [x] `.dev/PROMPT.md` — project context, skill areas, scope
- [x] `.dev/DIRECTIVES.md` — project-specific directives
- [x] Dual licensing in place: `LICENSE-APACHE` + `LICENSE-MIT`
- [x] `rustfmt.toml` — portfolio standard
- [x] `clippy.toml` — portfolio standard
- [x] CI workflow renamed `main.yml` → `ci.yml` (matches badge + portfolio convention)
- [x] README updated for dual licensing + accurate pre-1.0 status
- [x] Move root config fixtures (`debug_test.conf`, `test.ini`, `test.properties`) into `tests/fixtures/`
- [x] Consolidate the three typos config files (`.typos.toml`, `_typos.toml`, `typos.toml`) into one — keep `typos.toml`, delete the others
- [x] Clean up examples directory:
  - Keep (8): `basic.rs`, `multi_format.rs`, `enterprise_demo.rs`, `hot_reload_demo.rs`, `validation_demo.rs`, `audit_demo.rs`, `xml_demo.rs`, `hcl_demo.rs`
  - Deleted (12): `debug.rs`, `detection_debug.rs`, `ini_debug.rs`, `ini_direct_test.rs`, `ini_test.rs`, `format_test.rs`, `path_detection_test.rs`, `test_properties.rs`, `config_trace.rs`, `caching_demo.rs`, `new_api_demo.rs`, `ini_demo.rs` (last one not on the original delete list but uses `.unwrap()` and references soon-to-be-moved root fixtures — its INI coverage is already in `multi_format.rs` and `tests/`)
- [x] ~~Create `docs/release-notes/` directory~~ — superseded: release notes live in `.dev/release/` per project directive, mirroring `metrics-lib/.dev/release/`. Roadmap reference paths updated accordingly.
- [x] Write `.dev/release/v0.9.2.md` (release notes for this phase, modeled on `metrics-lib/.dev/release/v0.9.2.md`)
- [x] Update `Cargo.toml`:
  - [x] `license = "Apache-2.0 OR MIT"`
  - [x] `homepage`, `repository`, `documentation` URLs verified correct
  - [x] `keywords` tightened: dropped duplicative `configuration` and weak `settings`; added high-signal `multi-format` and `hot-reload`
  - [x] `categories` corrected: dropped incorrect `template-engine`; kept `config`, `parsing`, `data-structures`, `development-tools`
  - [x] `version` bumped to `0.9.2`
- [x] `CHANGELOG.md` `[0.9.2]` section added; footer compare-URLs corrected (were pointing at `metrics-lib` from a copy-paste leftover)

### Exit criteria

- [x] Project structure matches `_strategy/PROJECT_STRUCTURE.md` 0.1.0 minimum + portfolio conventions
- [x] No clutter at repo root (only the standard files)
- [x] Examples directory is curated — every file is a real, runnable example
- [x] All standards documents present at root and in `.dev/`

---

## Phase 0.9.3 — Toolchain alignment + REPS lint discipline

**Goal:** Bring toolchain and lint configuration to portfolio standard. Fix any new lint violations.

**Effort:** 2-3 days.

**Status:** Complete (2026-05-19). Released as [`v0.9.3`](../.dev/release/v0.9.3.md). MSRV / edition bumps deferred to Phase 0.9.7 — see "Deferrals" below.

### Tasks

- [x] **Update `Cargo.toml`:**
  - [x] ~~`edition = "2024"` (from `2021`)~~ — **deferred.** Edition 2024 stabilized in Rust 1.85; the stability contract's MSRV 1.75 commitment makes the two mutually exclusive. Edition stays at `2021` for the lifetime of 1.x unless the MSRV policy changes.
  - [x] ~~`rust-version = "1.75"` (from `1.82`)~~ — **deferred to 0.9.7.** `noml 0.9.0` (currently a default-feature dependency) itself declares `rust-version = "1.82"`. Phase 0.9.7 makes NOML/TOML opt-in, at which point the default feature set becomes 1.75-compatible cleanly. Cargo.toml keeps `1.82` for 0.9.3.
  - [x] Verified every portfolio crate metadata field present
- [x] **Update `src/lib.rs` to full REPS lint configuration:**
  ```rust
  #![deny(missing_docs)]
  #![deny(unsafe_op_in_unsafe_fn)]
  #![deny(unused_must_use)]
  #![deny(clippy::unwrap_used)]
  #![deny(clippy::expect_used)]
  #![deny(clippy::todo)]
  #![deny(clippy::unimplemented)]
  #![deny(clippy::print_stdout)]
  #![deny(clippy::print_stderr)]
  #![deny(clippy::dbg_macro)]
  #![deny(clippy::undocumented_unsafe_blocks)]
  #![deny(clippy::missing_safety_doc)]
  #![warn(clippy::pedantic)]
  ```
  Test-only allowances scoped under `#![cfg_attr(test, allow(...))]` with REPS-AUDIT rationale. `unused_results` not included — it generates noise on legitimate `_ = sender.send(...)` patterns where the return value is best-effort and uninteresting.
- [x] **Fixed every lint violation introduced by the tighter rules:**
  - `audit.rs` — three `print_*` sites now carry site-level `#[allow]` + REPS-AUDIT comments (ConsoleSink writes to stdout by contract; sink-failure fallback writes to stderr by necessity)
  - `enterprise.rs` — nested `set_recursive` lifted to module scope; default `Duration` rewritten in clearer units
  - `parsers/hcl_parser.rs` + `parsers/xml_parser.rs` — test-only diagnostic `println!` calls removed (no assertion value)
  - `src/lib.rs` doctests — rewritten to use `ok_or_else(|| Error::key_not_found(...))?` instead of `.unwrap()`
- [x] **Verified CI-equivalent gate clean on stable:**
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo clippy --all-targets --all-features -- -D warnings` (zero warnings, zero errors)
  - [x] `cargo test --all-features` — 94 tests pass (63 unit + 14 integration + 11 validation + 6 doc)
  - [x] `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"`
  - [x] `cargo audit` clean (one allowed `rustls-pemfile` unmaintained warning, scoped to 0.9.7 NOML opt-in work)
  - [x] `cargo +1.82 check --all-features`
- [ ] **Update CI workflow** if needed to match the portfolio CI format (`ci.yml`, Node 24, matrix Linux/macOS/Windows × stable + 1.82.0 for now → 1.75.0 once 0.9.7 lands) — left to next CI touch

### Exit criteria

- [x] All REPS lint denies in place — no violations on the shipping crate (`src/`)
- [x] All local gates green on stable + MSRV 1.82
- [x] No `unwrap` / `expect` / `todo` / `unimplemented` / `print_*` / `dbg!` in shipping code that isn't behind an explicit site-level `#[allow]` + REPS-AUDIT rationale
- [x] No `Box<dyn Error>` in the public API
- [x] Every public item has rustdoc

### Deferrals to Phase 0.9.7

The roadmap's original Phase 0.9.3 task list bundled the lint discipline with two toolchain bumps (`edition = "2024"`, `rust-version = "1.75"`) that turned out to be in tension with each other and with the wider dependency graph:

1. **Edition 2024 requires Rust 1.85** (stabilization release). It cannot coexist with MSRV 1.75 in the same `Cargo.toml`.
2. **MSRV 1.75 is blocked by `noml 0.9.0`,** which declares its own `rust-version = "1.82"`. The dependency resolver refuses the default build under any older toolchain.

The honest fix is to make NOML/TOML opt-in first (Phase 0.9.7 in the existing roadmap) — at which point the default build no longer pulls in the 1.82-bound noml crate and MSRV 1.75 becomes deliverable without pinning a chain of older transitive crates that would each carry their own security trade-offs.

Edition 2024 stays unscheduled. Post-1.0 the MINOR-release MSRV-bump policy (already in the roadmap) is the right venue if there's a real reason to revisit.

---

## Phase 0.9.4 — Architectural consolidation (deprecation phase)

**Goal:** Begin the unification of `Config` + `EnterpriseConfig` by marking the dual surface deprecated, landing the forward-compatible `ConfigOptions` knob struct, and migrating the user-facing example/README to lead with `Config`.

**Effort:** 1 day (was originally scoped as 1 week, but the **data-model merger** is paired with the v0.9.5 caching work — see "Deferrals" below).

**Status:** Complete (2026-05-19). Released as [`v0.9.4`](../.dev/release/v0.9.4.md).

### Background

The two surfaces are not source-compatible:

- `Config::get(&str) -> Option<&Value>` (borrowed)
- `EnterpriseConfig::get(&str) -> Option<Value>` (owned clone, from behind `Arc<RwLock<BTreeMap>>`)

Folding them requires picking one borrow convention. Returning `&Value` from a cached, thread-safe `Config` is the right contract for a database-tier library — but it requires the architectural work that Phase 0.9.5 is built around (lock-free backend, `ArcSwap` for whole-tree swap). Shipping the surface merger ahead of the caching architecture would either freeze the wrong return type or force a second migration.

v0.9.4 therefore delivers the **deprecation announcement** and the forward-compatible knob structure; v0.9.5 delivers the **data-model merger** alongside the caching it depends on.

### Tasks

- [x] **`ConfigOptions` designed and implemented.**
  - `#[non_exhaustive]` struct of public fields (`read_only`, `cache_enabled`, `cache_capacity`)
  - Consuming builder methods (`new`, `read_only`, `cache_enabled`, `cache_capacity`)
  - `Default` impl produces the canonical Hive-DB-tuned configuration
  - `Config::with_options(ConfigOptions)`, `Config::options()`, `Config::is_read_only()` constructors and accessors
  - `ensure_writable()` helper wired into `Config::set` / `remove` / `merge`
- [x] **`#[deprecated]` markers** on every public item in `enterprise.rs`:
  - `EnterpriseConfig` struct
  - `ConfigManager` struct (will change return-type shape in v0.9.5)
  - `enterprise::direct::parse_string` and `parse_file` (duplicate of `crate::parse` / `parse_file`)
  - Module-level `#![allow(deprecated)]` keeps internal references compiling
- [x] **`ConfigManager` audited.** Retained as multi-instance primitive; only the **internal storage type** changes in v0.9.5 (BTreeMap of `EnterpriseConfig` → BTreeMap of `Config`).
- [x] **`examples/enterprise_demo.rs` rewritten** end-to-end to model the migration. Five demos covering the original feature surface; migration table reproduced inline.
- [x] **`benches/enterprise_benchmarks.rs`** carries `#![allow(deprecated)]` with REPS-AUDIT rationale — keeps comparison baselines measurable across the v0.9.4 → v0.9.5 transition.
- [x] **README updated** to lead with `Config` everywhere — Enterprise Caching section now Read-only / ConfigOptions section; Method 2 defaults section rewritten; troubleshooting tip no longer recommends `EnterpriseConfig`.
- [x] **CHANGELOG** carries the full migration guide as a table.

### Deferrals to Phase 0.9.5

The original Phase 0.9.4 task list bundled the deprecation surface with the actual **data-model merger** (folding the cached `BTreeMap` storage of `EnterpriseConfig` into `Config` while keeping `Config::get -> Option<&Value>`). The merger requires the lock-free backend that Phase 0.9.5 is designed around — `ArcSwap<Arc<Value>>` for whole-tree swap, `DashMap` or equivalent for resolved-key caching. Shipping the merger ahead of that architecture would either lock in the wrong return semantics or force a second user migration. The merger is therefore paired with the v0.9.5 caching work; Phase 0.9.5's exit criteria are extended below to absorb the data-model merger as one of its deliverables.

### Exit criteria

- [x] Public docs lead with one `Config` API (README + rustdoc + example)
- [x] `EnterpriseConfig` works unchanged for existing call-sites — every method `#[deprecated]` with a migration target named
- [x] Example demonstrates the migration; tests pass against the unified surface
- [x] CHANGELOG migration table present
- [x] No public API breakage — no symbol removed, no signature changed

---

## Phase 0.9.5 — Lock-free caching + Config/EnterpriseConfig data-model merger

**Goal:** Replace `Arc<RwLock<BTreeMap>>` caching with a lock-free implementation **and** fold the cached storage into the unified `Config` so `EnterpriseConfig` becomes redundant in fact, not just in deprecation marker. **Verify sub-50ns claim by committed benchmark.**

**Effort:** 1-1.5 weeks (absorbs the data-model-merger work originally in Phase 0.9.4 — see notes there). Split across two releases:

- **0.9.5 — Foundation** (Complete; 2026-05-19, released as [`v0.9.5`](../.dev/release/v0.9.5.md))
- **0.9.9 — Implementation** (Complete; 2026-05-19, absorbed into the final pre-1.0 release [`v0.9.9`](../.dev/release/v0.9.9.md))

### Foundation half (v0.9.5 — Complete)

What ships as v0.9.5:

- [x] **`CacheStats`** struct (`#[non_exhaustive]`, fields `hits`/`misses`/`hit_ratio`) — public, re-exported at crate root
- [x] **`Config::cache_stats() -> CacheStats`** accessor; `Ordering::Relaxed` loads on the underlying `AtomicU64` counters
- [x] **Internal `cache_hits` / `cache_misses` atomic fields** on `Config`, wired through every constructor — placeholder for the v0.9.5 Implementation wire-up
- [x] **`#[non_exhaustive]` hardening** on `Error`, `ConfigChangeEvent`, `ValidationSeverity`, `AuditEventType`, `AuditSeverity`, `FieldType`, `CacheStats` — the 1.0 stability contract requirement
- [x] **`examples/hot_reload_demo.rs`** updated for the new non-exhaustive `ConfigChangeEvent` with a stability-contract comment
- [x] All exit-criteria gates green (fmt, clippy `-D warnings`, 96 tests, doc with `-D warnings`, audit)

### Implementation half (v0.9.9 — Complete)

- [x] **Cache backend selected: `DashMap<Box<str>, Arc<Value>>`.** Decision logged in `docs/ARCHITECTURE.md` §3. Alternatives (`Arc<RwLock<BTreeMap>>`, `parking_lot::RwLock<HashMap>`, `ArcSwap<Arc<HashMap>>`, `evmap`) were considered systematically; DashMap matched the read-mostly access pattern with the simplest API and no surprising semantics.
- [x] **`Config::get_arc(path) -> Option<Arc<Value>>`** — cache-backed thread-safe accessor with sub-50ns warm target.
- [x] **`Config::clear_cache()`** — explicit invalidation hook.
- [x] **`cache_hits` / `cache_misses` atomic counters wired** to the cache lookup path. `Config::cache_stats()` now returns meaningful non-zero numbers when `get_arc` is in use.
- [x] **`ConfigOptions::cache_enabled` honored at runtime** — `false` makes every `get_arc` walk the tree and allocate a fresh `Arc<Value>` (useful for write-heavy workloads).
- [x] **`ConfigOptions::defaults` backed by `Arc<RwLock<BTreeMap>>`.** Accessible via `Config::set_default` / `Config::get_or_default`. Independent of the `read_only` flag.
- [x] **`Config::make_read_only()` ergonomic helper** — one-way post-construction switch.
- [x] **`ConfigManager` internals migrated** from `EnterpriseConfig` storage to `Arc<RwLock<Config>>` storage. `ConfigManager::get` return type changed to `Arc<RwLock<Config>>`; `ConfigManager` un-deprecated.
- [x] **`Config: Debug` derived** (required for `ConfigManager: Debug`).
- [x] **Four criterion benchmark harnesses** (`benches/cache_warm.rs`, `benches/cache_concurrent.rs`, `benches/value_accessors.rs`, `benches/parse_throughput.rs`) targeting every operation in the Performance Contract table.
- [x] **`docs/PERFORMANCE.md`** — methodology, targets, per-bench-file explanation, tuning guidance, baselines schema.
- [ ] **Commit `benches/baselines.json`** with reference-hardware numbers — lands alongside the v1.0.0 cut on the maintainer's reference hardware (the v0.9.9 code is in place; only the canonical measurements remain).

### Absorbed from Phase 0.9.4 (lands in the Implementation half)

- `Config::get` return semantics under concurrent reads — finalized once the backend is chosen
- `ConfigOptions::defaults` (rich separate-defaults-table feature)
- `Config::make_read_only()` ergonomic helper (paired with the existing `ConfigOptions::read_only` knob)
- `ConfigManager` internals migrate from `EnterpriseConfig` storage to `Config` storage
- The deprecated `enterprise.rs` module remains compilable through this phase; v1.0 is the earliest sensible removal point.

### Background

Per the audit, the current caching layer is a performance ceiling:

```rust
fast_cache: Arc<RwLock<FastCache>>,   // Write lock on EVERY read (hits counter!)
cache: Arc<RwLock<BTreeMap<String, Value>>>,  // Serializes ALL reads
defaults: Arc<RwLock<BTreeMap<String, Value>>>,
```

This cannot hit sub-50ns under 16+ thread contention. Max concurrency requirement (per UNIVERSAL_PROMPT) requires:

- Lock-free reads
- Sharded writes
- Atomic counters for statistics
- Zero-allocation hot path

### Tasks

- [ ] **Prototype caching backends** in `benches/cache_backend.rs`:
  - [ ] `DashMap` — sharded concurrent map
  - [ ] `ArcSwap<HashMap>` — fully lock-free reads, atomic pointer swap
  - [ ] `evmap` — left-right paired (read-optimized)
- [ ] **Benchmark each backend** across these scenarios:
  - [ ] 1 thread, single-key get, 10M iterations
  - [ ] 4 threads, single-key contended, 10M iterations each
  - [ ] 16 threads, single-key contended, 1M iterations each
  - [ ] 64 threads, single-key contended, 100K iterations each
  - [ ] 16 threads mixed read/write (90/10), 1M iterations each
  - [ ] Memory footprint at 1000 keys, 10000 keys, 100000 keys
- [ ] **Pick the winner** based on:
  - Read latency at 1-16 threads (PRIMARY criterion)
  - Memory overhead (SECONDARY criterion)
  - Code complexity (TERTIARY criterion)
- [ ] **Replace cache layer** in unified `Config`:
  - [ ] Main cache → chosen lock-free backend
  - [ ] Fast cache → either eliminate (if main is fast enough) or redesign as thread-local
  - [ ] Defaults → either fold into main cache or `ArcSwap` (read-mostly)
- [ ] **Statistics via atomic counters:**
  ```rust
  hits: AtomicU64,
  misses: AtomicU64,
  // No more write-lock-on-read pattern
  ```
- [ ] **Use `Arc<str>` over `String`** for cache keys:
  - Cheap clone on hit (refcount bump, no allocation)
  - Reduces memory pressure
- [ ] **Use `FxHashMap`** if HashMap backend chosen (rustc-hash crate, ~30% faster on short string keys)
- [ ] **Inline hot accessors:**
  - `Config::get` — `#[inline]`
  - `Value::as_string` / `as_integer` / `as_bool` / etc. — `#[inline]`
  - Avoid `#[inline(always)]` unless measurement proves it helps
- [ ] **Write criterion benchmarks** covering every operation in the Performance Contract table:
  - [ ] `benches/cache_warm.rs` — warm cache reads
  - [ ] `benches/cache_cold.rs` — cold misses
  - [ ] `benches/cache_concurrent.rs` — contention across thread counts
  - [ ] `benches/parse_throughput.rs` — cold parse for each format
  - [ ] `benches/value_accessors.rs` — typed accessor performance
- [ ] **Commit benchmark baselines** to `benches/baselines.json`
- [ ] **Verify Performance Contract** — every target met
- [ ] **Write `docs/PERFORMANCE.md`** documenting:
  - Methodology (hardware, isolation, warmup)
  - Results table
  - Tuning guidance for users

### Exit criteria

Foundation half (Complete):

- [x] `CacheStats` + `Config::cache_stats()` API shipping; `#[non_exhaustive]`
- [x] Atomic counter infrastructure in place; ready to be wired by the Implementation half
- [x] 1.0-stability-contract enums hardened with `#[non_exhaustive]`
- [x] All workspace gates green; no breaking signature changes

Implementation half (Pending):

- [ ] **Sub-50ns single-key cached get sustained across 1-16 threads** (verified by `criterion`)
- [ ] **Sub-500ns cached write** (verified)
- [ ] **<10ns typed accessor** (verified)
- [ ] All other Performance Contract targets met
- [ ] `benches/baselines.json` committed
- [ ] `docs/PERFORMANCE.md` documents methodology + results
- [ ] No regression in cold-parse performance (it shouldn't change in this phase, but verify)
- [ ] Allocation profile clean — `dhat` shows zero allocations on cached read path
- [ ] `Config::cache_stats()` returns meaningful non-zero counters

---

## Phase 0.9.6 — Event-driven hot reload

**Goal:** Replace polling-based `hot_reload.rs` with `notify`-backed event-driven file watching.

**Effort:** 4-5 days. Split across two releases:

- **0.9.6 — Foundation** (Complete; 2026-05-19, released as [`v0.9.6`](../.dev/release/v0.9.6.md))
- **0.9.6.x — Cross-platform tests + latency benchmarks** (Pending canonical CI matrix wire-up)

### Foundation half (v0.9.6 — Complete)

- [x] `notify = "6"` added as feature-gated optional dependency
- [x] `hot-reload` Cargo feature added; included in `default` so the new behavior is what users get out of the box
- [x] `hot_reload.rs` rewritten end-to-end: event-driven worker reads from a `notify::RecommendedWatcher` registered on the parent directory; debounce window collapses bursts; canonical-form path comparison handles macOS realpath quirks
- [x] Polling worker preserved behind `#[cfg(not(feature = "hot-reload"))]` as the fallback when users opt out of the new dependency
- [x] Public API surface preserved: every `HotReloadConfig` method on v0.9.5 still compiles and behaves the same way; new `with_debounce(Duration)` and `with_polling_fallback()` knobs added additively
- [x] `HotReloadHandle` cleanup primitive switched to `Arc<AtomicBool>` (same observable semantics; lets both watcher paths share one shutdown primitive)
- [x] `docs/PLATFORM-NOTES.md` created — covers the three kernel backends, debounce tuning, latency expectations, network-filesystem caveats, deletion/re-creation handling, permissions changes, line-ending acceptance, path handling, async file I/O, filesystem-timestamp resolution, NOML/TOML format-preservation footprint
- [x] 3 in-module tests pass against the new implementation (basic reload, change-notification, automatic-watching with the event-driven path)
- [x] All workspace gates green (fmt, clippy `-D warnings`, 96 tests, doc with `-D warnings`, audit)

### Implementation half (v0.9.9 — Complete)

- [x] **Five cross-platform integration tests** in `tests/hot_reload_*.rs`:
  - [x] `tests/hot_reload_modified.rs` — modify file, expect `Reloaded` event
  - [x] `tests/hot_reload_atomic_write.rs` — atomic rename, expect single `Reloaded` (debounced)
  - [x] `tests/hot_reload_deleted.rs` — delete file, expect `FileDeleted` event; last-known-good `Config` preserved
  - [x] `tests/hot_reload_recreated.rs` — delete + recreate, expect `FileDeleted` then `Reloaded`
  - [x] `tests/hot_reload_permissions.rs` — `#[cfg(unix)]`-gated; file becomes unreadable, expect graceful `ReloadFailed` / `FileDeleted` (Windows equivalent is awkward, filed as follow-up)
- [ ] **Committed cross-platform latency baseline** — target <100 ms from `fsync` return to `Reloaded` event delivery on each platform. The *measurement* lands alongside the v1.0.0 cut on the CI matrix; the *integration tests that produce the measurement* are now in place.

### Exit criteria

Foundation half (Complete):

- [x] `notify` integrated; default feature includes it; v0.9.5 polling preserved as fallback
- [x] Public API preserved; new knobs additive only
- [x] In-module tests pass against the event-driven path
- [x] Platform documentation written
- [x] All gates green

v0.9.6.x patch:

- [ ] Hot reload detection latency <100ms on all three platforms (measured, committed)
- [ ] No CPU usage when no file changes occur (verified with `top` or equivalent)
- [ ] All five cross-platform integration tests passing on all three platforms
- [ ] `docs/PLATFORM-NOTES.md` updated with measured numbers

### Background

Pre-v0.9.6 `hot_reload.rs` used a thread-based polling loop with a `Duration`-based interval. Per UNIVERSAL_PROMPT (max efficiency requirement), that wastes CPU and has latency equal to the poll interval. Event-driven file watching via the OS kernel APIs is the standard; v0.9.6 lands that as the default behavior.

---

## Phase 0.9.7 — Dependency hygiene + NOML/TOML opt-in + MSRV 1.75

**Goal:** Lock down the 1.0 stability contract by isolating pre-1.0 dependencies behind opt-in features. **Also delivers the MSRV 1.75 commitment** deferred from Phase 0.9.3 (which was blocked by `noml 0.9.0`'s own `rust-version = "1.82"`).

**Effort:** 2-3 days.

**Status:** Complete (2026-05-19). Released as [`v0.9.7`](../.dev/release/v0.9.7.md). MSRV 1.75 delivered; pre-1.0 deps gone from default; `docs/STABILITY-1.0.md` canonical.

### Background

`config-lib`'s 1.0 stability contract depends on the stability of its public dependencies. Currently:
- `noml = "0.9"` is in the default features — meaning a 1.0 user pulls in a pre-1.0 crate
- This is a documented risk in the audit

### Tasks

- [x] **NOML/TOML out of default features.** `default = ["conf", "hot-reload"]`. Zero pre-1.0 deps in the default tree.
- [x] **`noml` pinned to `=0.9.0` exactly.** A `cargo update` can no longer silently promote to a `noml 0.9.1` this crate hasn't validated.
- [x] **`docs/STABILITY-1.0.md`** — canonical 1.0 stability contract. §3.2 covers the MSRV asymmetry; §4.3 covers the NOML pre-1.0 caveat; §9 documents the direct 0.9.9 → 1.0.0 release path (no `1.0.0-rc.1` cut).
- [x] **Dependency audit:** `thiserror`/`serde` (post-1.0, kept); `tokio`/`chrono`/`serde_json`/`regex`/`quick-xml`/`notify` (kept optional); **`base64`** removed entirely (was dead — referenced under `#[cfg(feature = "base64")]` but the feature was never defined).
- [x] **`cargo audit`** clean (one allowed `rustls-pemfile` unmaintained warning — now correctly only fires when the user opts into `noml`/`toml`, since that's the only path to `reqwest 0.11.27`).
- [x] **`cargo deny check`** clean across `advisories`, `bans`, `licenses`, `sources`. `deny.toml` allow-list extended with `CC0-1.0` (the license `notify 6.x` is published under).
- [x] **MSRV 1.75 verified** via `cargo +1.75 check` on the default feature set. Feature-flag asymmetry: `noml`/`toml` need 1.82 because upstream noml does. Documented in §3.2 of the stability contract.

### Exit criteria

- [x] Default feature set has zero pre-1.0 dependencies
- [x] NOML/TOML are clean opt-in features, documented in README and `docs/STABILITY-1.0.md`
- [x] `cargo audit` clean
- [x] `cargo deny check` clean
- [x] All default-feature dependencies MSRV-compatible with Rust 1.75; noml/toml asymmetry documented
- [x] `docs/STABILITY-1.0.md` is the canonical 1.0 contract

---

## Phase 0.9.8 — Fuzz testing (nuclear-proof security)

**Goal:** Add `cargo-fuzz` harnesses for every parser. Each must run for at least 1 CPU-hour clean.

**Effort:** 3-4 days. Split across two releases:

- **0.9.8 — Foundation** (Complete; 2026-05-19, released as [`v0.9.8`](../.dev/release/v0.9.8.md))
- **0.9.8.x — 1-CPU-hour clean runs + corpus + regression tests + CI fuzz pass** (Pending nightly-Rust + Linux + maintainer time)

### Background

Per UNIVERSAL_PROMPT security requirement (nuclear-proof, impenetrable):
- Every parser ingests untrusted user input
- A panic on malformed input is an availability bug
- An infinite loop on adversarial input is a DoS vector
- An OOM on crafted input is a DoS vector

These must be eliminated before 1.0.

### Foundation half (v0.9.8 — Complete)

- [x] **`fuzz/` workspace** set up with standalone `[workspace]` declaration so libFuzzer's nightly-only requirement does not contaminate stable builds of `config-lib`
- [x] **`fuzz/Cargo.toml`** with seven `[[bin]]` blocks pinning each target name + path
- [x] **`fuzz/.gitignore`** ignoring `target/`, `corpus/`, `artifacts/` (the harness build/run artifacts) while leaving room for a future `fuzz/corpus_seeds/` directory committed to git
- [x] **Seven fuzz targets** in `fuzz/fuzz_targets/`:
  - [x] `conf_parser.rs` — `parsers::conf::parse`
  - [x] `ini_parser.rs` — `parsers::ini_parser::parse`
  - [x] `properties_parser.rs` — `parsers::properties_parser::parse`
  - [x] `json_parser.rs` — `parsers::json_parser::parse` (gated on `config-lib/json`)
  - [x] `xml_parser.rs` — `parsers::xml_parser::parse` (gated on `config-lib/xml`)
  - [x] `hcl_parser.rs` — `parsers::hcl_parser::parse` (highest-yield target — youngest parser with rich nested-block semantics)
  - [x] `format_detection.rs` — `config_lib::parse(content, None)` (auto-dispatch)
- [x] **`docs/SECURITY.md`** — canonical security document. Threat model, lint-enforced defenses, fuzz methodology + per-target descriptions + triage workflow, dependency hygiene, the zero-unsafe-in-shipping-code property, security-disclosure email
- [x] **`[package].exclude`** updated to keep `fuzz/` out of the published crate
- [x] All workspace gates green (fmt, clippy `-D warnings`, 96 tests, doc with `-D warnings`, audit, deny, `cargo +1.75 check`)

### Implementation half (v0.9.9 — Complete, except canonical-hardware runs)

- [x] **`tests/parser_corpus.rs` regression harness** — documents the seed-promotion workflow; macro-free hand-written `#[test]` template for each future seed; sanity test proves the file compiles. Corpus is empty at v0.9.9; populated by the maintainer's runs.
- [ ] **Run each target for at least 1 CPU-hour** on the maintainer machine
  - Target: 0 panics, 0 OOMs, 0 infinite loops
- [ ] **Fix every finding:**
  - Panic → replace with `Result<_, Error>`
  - Infinite loop → add iteration cap with clear error
  - OOM → add input size limits with clear error
- [ ] **Collect interesting corpus** under `tests/corpus_seeds/<target>/` — committed to git as they appear
- [ ] **Add corpus inputs as regression tests** in `tests/parser_corpus.rs` (template in place)
- [ ] **CI fuzz pass** (~10 CPU-minutes per target) on every PR

The three remaining items above need nightly Rust + libFuzzer + extended CPU time + maintainer attention. The harness code is in place; the runs themselves land alongside the v1.0.0 cut as the verification data the v1.0 stability contract cites.

### Exit criteria

Foundation half (Complete):

- [x] Seven harness files in place, parser-by-parser + format-detection auto-dispatch
- [x] `fuzz/` standalone workspace doesn't contaminate stable builds of `config-lib`
- [x] `docs/SECURITY.md` documents methodology, threat model, and reproduction
- [x] All non-fuzz workspace gates green

v0.9.8.x patch (Pending):

- [ ] 6+ fuzz targets running clean for 1 CPU-hour each
- [ ] Corpus inputs committed
- [ ] Regression tests added for every corpus input
- [ ] CI optionally runs short fuzz pass on every PR (10-minute time budget)

---

## Phase 0.9.9 — Final pre-1.0 release (absorbed all remaining 0.9.x work)

**Goal:** Final pre-1.0 release. Closes out the entire 0.9.x roadmap by absorbing the implementation halves of Phase 0.9.5 (lock-free cache + data-model merger), Phase 0.9.6 (cross-platform integration tests), and Phase 0.9.8 (parser-corpus regression infrastructure), alongside the original Phase 0.9.9 polish work (`docs/ARCHITECTURE.md`, `docs/PERFORMANCE.md`). No `1.0.0-rc.1` cut — `v1.0.0` ships directly from `v0.9.9`.

**Effort:** 3-4 days for the original polish scope; the absorbed work brings it to ~1 focused week.

**Status:** Complete (2026-05-19). Released as [`v0.9.9`](../.dev/release/v0.9.9.md). The next release is `v1.0.0` with **new architectural scope** — see Phase 1.0.0 below.

### Tasks

- [x] **`docs/STABILITY-1.0.md`** — 1.0 stability contract. *(Landed in Phase 0.9.7.)*
- [x] **`docs/ARCHITECTURE.md`** — internal structure: module layout, data flow, caching architecture (DashMap decision log), hot reload architecture, thread safety + lock ordering, EnterpriseConfig deprecation map, parser contract, "where to look for what" index.
- [x] **`docs/PERFORMANCE.md`** — methodology, targets table, per-bench-file explanation, tuning guidance, baselines schema.
- [x] **`docs/PLATFORM-NOTES.md`** — already in place from 0.9.6 Foundation; no further changes required for the v0.9.9 cut.
- [x] **`docs/SECURITY.md`** — already in place from 0.9.8 Foundation; no further changes required for the v0.9.9 cut.
- [x] **`cargo doc --no-deps --all-features` clean** with `RUSTDOCFLAGS="-D warnings"`.
- [x] **Cut `v0.9.9`** per the usual release workflow (Cargo.toml bumped, CHANGELOG rolled, release note written, ROADMAP synced).
- [ ] **Soak period** — at least one week with `v0.9.9` published before cutting `v1.0.0`. Watch crates.io download stats and the GitHub issue tracker for surprises. Address any critical findings with `v0.9.9.x` patches. (Begins once the maintainer pushes the v0.9.9 commit.)

### Exit criteria

- [x] All required docs in place — `STABILITY-1.0.md`, `ARCHITECTURE.md`, `PERFORMANCE.md`, `PLATFORM-NOTES.md`, `SECURITY.md`, `FORMATS.md`, `API.md`, `GUIDELINES.md`
- [x] All gates green: fmt, clippy `-D warnings`, 100+ tests, doc `-D warnings`, audit, deny, MSRV 1.75 check
- [ ] `v0.9.9` published to crates.io as a normal release (not a pre-release) — maintainer action
- [ ] At least 1 week of post-publish soak with no critical issues — maintainer observation
- [ ] No outstanding issues blocking the `v1.0.0` cut — maintainer judgment

---

## Phase 1.0.0 — Stable release + in-house lock-free notification refactor

**Goal:** Ship `config-lib 1.0.0` — the stable, canonical configuration library — with a lock-free in-process notification subsystem replacing the v0.9.x `mpsc::channel`-based design.

**Effort:** ~1 focused session for the refactor + release prep.

**Status:** **Complete (2026-05-19).** Released as [`v1.0.0`](../.dev/release/v1.0.0.md).

### Design decision: in-house instead of `registry-io`

An earlier v1.0.0 plan (`.dev/PROMPT-v1.0.0.md`, `.dev/registry-io-integration.md`) called for taking a dependency on a separate `registry-io` crate that provides typed event-registry dispatch. **That plan was scrapped after analysis.** The reasoning, in full:

- **No performance gap to close.** `registry-io` internally uses `ArcSwap<Vec<Handler>>` — the same primitive we'd use ourselves. There's no perf delta vs. an in-tree implementation.
- **API-shape mismatch.** config-lib's existing change-notification surface returned `(Self, Receiver<ConfigChangeEvent>)`. `registry-io` is `SyncRegistry<T>::register(closure) → HandlerId`. The two don't compose 1-for-1; integrating would have meant a wrapper layer AND a deprecation cycle for the old `Receiver`-flavored API anyway — same migration cost, plus a new external dependency.
- **REPS principle: take the load-bearing primitive, not the wrapper.** We added `arc-swap = "1.7"` (~700 LOC, MSRV 1.46, stable, transitive-dep in many ecosystems) and built the dispatch surface inline. ~80 lines of real code in `src/hot_reload.rs` + ~30 lines of docs.

The decision is recorded in `docs/ARCHITECTURE.md` §3a ("Why not registry-io?") and the v1.0.0 release notes.

### What shipped

- **`HotReloadConfig::on_change(closure) → Subscription`** — new lock-free notification API.
- **`HotReloadHandle::on_change(closure) → Subscription`** — same API accessible post-`start_watching`.
- **`Subscription`** — RAII guard with `Drop`-based unregister; `Subscription::forget()` detaches the drop hook for process-lifetime handlers.
- **`HandlerList` (private)** — `ArcSwap<Vec<(u64, Arc<dyn Fn>)>>` backing; dispatch = one `load()` + iterate + `Arc::clone` + `catch_unwind`-wrapped call per handler.
- **`with_change_notifications` deprecated** with internal bridge: registers a closure that forwards events to an `mpsc::Sender`. Existing v0.9.x users keep compiling.
- **`arc-swap = "1.7"`** as a regular (non-optional) dependency.
- **`examples/on_change_demo.rs`** — runnable demo of the new API.
- **`examples/hot_reload_demo.rs`** migrated off the deprecated API.
- **`benches/notification.rs`** — criterion harness for the dispatch path.
- **`docs/STABILITY-1.0.md`** — opening line updated to "in force as of v1.0.0".
- **`docs/ARCHITECTURE.md`** — new §3a "Notification dispatch (v1.0.0+)" + decision log.
- **`docs/PERFORMANCE.md`** — five new Performance Contract rows for notification dispatch; `benches/notification.rs` documented.
- **README** — version-compat section rewritten for v1.x; deprecation banners simplified; install snippets bumped from `"0.9"` to `"1.0"`.

### Pre-flight verification (all met)

- [x] **No critical issues** from the v0.9.9 soak
- [x] **Final API freeze verification** — no breaking changes; `with_change_notifications` continues to work via bridge
- [x] **All local gates green** on stable + 1.82
- [x] **`cargo +1.75 check`** passes on default features
- [x] **`cargo audit`** clean
- [x] **`cargo deny check`** clean
- [x] **Documentation review** — `docs/STABILITY-1.0.md` accurate

Deferred to **v1.0.1** patch (pending canonical hardware):

- [ ] Committed `benches/baselines.json` from reference hardware
- [ ] Cross-platform hot-reload latency baselines from the CI matrix
- [ ] 1-CPU-hour clean fuzz runs per target

### Release sequence

- [ ] Update `Cargo.toml` version → `1.0.0`
- [ ] Move `[Unreleased]` CHANGELOG → `[1.0.0] - <date>`
- [ ] Finalize `.dev/release/v1.0.0.md`
- [ ] Commit: `Milestone Update v1.0.0`
- [ ] Push to `main`
- [ ] Verify CI green
- [ ] Tag: `git tag -a v1.0.0 -m "v1.0.0"`
- [ ] Push tag: `git push origin v1.0.0`
- [ ] Create GitHub release (NOT marked as pre-release):
  - Title: `v1.0.0 — First Stable Release`
  - Body: contents of `.dev/release/v1.0.0.md`
- [ ] `cargo publish --dry-run` → verify clean
- [ ] `cargo publish` → ship it
- [ ] Verify crates.io shows `1.0.0`
- [ ] Verify docs.rs builds `1.0.0` clean (allow ~5 min)
- [ ] **Hive DB integration ready** — `config-lib = "1.0"` consumable

### Post-release

- [ ] Announcement (project README, Hive DB README, optional blog post)
- [ ] Begin tracking 1.1+ backlog (deferred items, see below)

### Exit criteria

- [ ] `config-lib 1.0.0` live on crates.io
- [ ] docs.rs builds clean
- [ ] Hive DB pulls `config-lib = "1.0"` and consumes it for its configuration layer

---

## Post-1.0 backlog (deferred to 1.1+)

Explicitly NOT in 1.0 scope, tracked for future planning:

### Performance / efficiency

- [ ] **Profile-guided optimization (PGO)** build profile in `Cargo.toml` for users who want it
- [ ] **`mmap`-based loading** for large config files (>64 KiB threshold)
- [ ] **SIMD format detection** using `memchr` for first non-whitespace byte
- [ ] **Allocator integration** — let users plug in `jemalloc` / `mimalloc` (already possible via `#[global_allocator]`, but document patterns)

### Features

- [ ] **CST-based format preservation** for CONF and INI (NOML-inspired, ~2 weeks of focused work)
- [ ] **Typestate API** for read-only / mutable distinction (compile-time enforcement)
- [ ] **Async hot reload** integration with `tokio::sync::watch` (current sync version stays default)
- [ ] **`serde::Deserialize`** impl for `Value` type (post-1.0 convenience layer)
- [ ] **`Config` diffing API** — compute diff between two `Config` instances
- [ ] **Configuration merge strategies** — beyond override/additive (deep merge with conflict resolution)

### Integrations

- [ ] **Distributed configuration sources** — `etcd`, `Consul`, `Vault` adapters (separate crates)
- [ ] **Encryption-at-rest** for sensitive values (separate crate or feature)
- [ ] **Prometheus metrics** — counters/histograms for cache hits, parse errors, etc.

### Testing

- [ ] **`proptest` invariants** — parser round-trip properties, value type conversions
- [ ] **`loom` model checking** for cache + hot-reload interaction
- [ ] **Continuous fuzzing** in CI (currently: manual pre-release runs)

---

## Quick reference

```
==============================================================
config-lib roadmap to 1.0
==============================================================
0.9.2  Structure normalization              1-2 days
0.9.3  Toolchain + REPS lint discipline     2-3 days
0.9.4  Architectural consolidation          1 week
0.9.5  Lock-free caching (Max-Perf)         1 week
0.9.6  Event-driven hot reload              4-5 days
0.9.7  Dependency hygiene + NOML opt-in     2-3 days
0.9.8  Fuzz testing (nuclear-proof)         3-4 days
0.9.9  Docs + final pre-1.0 polish           3-4 days
1.0.0  Stable Release                       1 day
==============================================================
Total: ~4-6 focused weeks
==============================================================
```

---

## Roadmap discipline

- **Every task has a checkbox.** Track completion explicitly.
- **Every phase has exit criteria.** Don't move to the next phase until current phase exits cleanly.
- **No skipping phases** unless explicitly justified in writing in this document.
- **No performance claim without committed benchmark.**
- **No "production-grade" claim without REPS lint compliance.**
- **CHANGELOG updated under `[Unreleased]` in every commit that changes user-visible behavior.**
- **`Milestone Update vX.Y.Z` commit format** for every phase release (per RELEASE_WORKFLOW).

---

<sub>config-lib roadmap — Copyright &copy; 2026 James Gober. Apache-2.0 OR MIT.</sub>
