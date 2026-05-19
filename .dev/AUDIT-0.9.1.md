# config-lib — Pre-1.0 Audit

> **Audit completed: 2026-05-18**
> Conducted against: `0.9.1` (HEAD at audit time)
> Auditor context: New ownership of standards baseline (REPS + UNIVERSAL_PROMPT + VERSIONING + RELEASE_WORKFLOW + PROJECT_STRUCTURE).
> Purpose: Determine the work required to ship `1.0.0` as the canonical config library for the Hive DB stack.

---

## Executive summary

`config-lib` is **structurally mature** and **functionally substantial** — 6,606 lines of production code across 19 source files, supporting 8 configuration formats, with hot reloading, audit logging, schema validation, environment variable overrides, and a multi-tier caching layer. Existing claims target sub-50ns cached access. The crate is real, not a sketch.

However, the crate is **not yet 1.0-grade by the new portfolio standard**. The biggest gaps are:

1. **REPS compliance is partial.** Lint discipline in `src/lib.rs` is significantly weaker than the portfolio standard (only `missing_docs` and `clippy::all`).
2. **Project structure does not match `PROJECT_STRUCTURE.md`.** Missing: `REPS.md`, dual licensing, `.dev/` planning docs, `rustfmt.toml`, `clippy.toml`, root config-fixtures cluttering the project root, three duplicate typos config files.
3. **MSRV + edition out of date.** Rust 1.82 + edition 2021. Portfolio standard is MSRV 1.75 + edition 2024.
4. **Performance claims need verification.** "Sub-50ns" claims exist in the README and rustdoc but no committed benchmark baselines back them.
5. **Hot reload is poll-based, not event-driven.** Polling every N seconds works but is not enterprise-grade compared to inotify/kqueue/ReadDirectoryChangesW.
6. **Architecture asymmetry.** There are TWO parallel API surfaces — `Config` (in `config.rs`) and `EnterpriseConfig` (in `enterprise.rs`). Both serve overlapping use cases. This is confusing and increases maintenance burden.
7. **NOML dependency is structural risk.** `config-lib` depends on `noml`. NOML is in active development and on `0.9.x`. Using a pre-1.0 crate as a non-optional default feature is a stability risk for our own 1.0.
8. **Examples directory is bloated.** 20 examples, many of which are debug/test scratch files (`debug.rs`, `detection_debug.rs`, `ini_debug.rs`, `ini_direct_test.rs`, `path_detection_test.rs`). Examples are user-facing documentation; debug scratch belongs elsewhere or deleted.

The crate is **closer to 1.0 than the version number suggests**, but the polish gap is real. Most issues are mechanical (configuration, structure, docs) rather than architectural (the core engineering is sound).

---

## Inventory

### Codebase

| Component | Lines | Purpose |
|-----------|-------|---------|
| `src/lib.rs` | 188 | Public API surface, re-exports, module declarations |
| `src/config.rs` | 633 | Standard `Config` API |
| `src/enterprise.rs` | 635 | `EnterpriseConfig` API with multi-tier caching |
| `src/value.rs` | 592 | `Value` type system |
| `src/audit.rs` | 456 | Audit logging system |
| `src/hot_reload.rs` | 372 | File-watching hot-reload |
| `src/error.rs` | 251 | Error types |
| `src/env_override.rs` | 341 | Environment variable overrides |
| `src/schema.rs` | 372 | Schema validation system |
| `src/validation.rs` | 333 | Rule-based validation |
| `src/parsers/mod.rs` | 278 | Parser registry / format detection |
| `src/parsers/conf.rs` | 435 | CONF format parser |
| `src/parsers/ini_parser.rs` | 449 | INI format parser |
| `src/parsers/properties_parser.rs` | 316 | Java .properties parser |
| `src/parsers/xml_parser.rs` | 267 | XML parser (quick-xml backed) |
| `src/parsers/hcl_parser.rs` | 241 | HCL parser |
| `src/parsers/json_parser.rs` | 215 | JSON parser |
| `src/parsers/noml_parser.rs` | 124 | NOML parser (thin wrapper over `noml` crate) |
| `src/parsers/toml_parser.rs` | 108 | TOML parser (also via `noml` crate) |
| **Total source** | **6,606** | |
| `tests/integration_tests.rs` | unknown | Integration tests |
| `tests/validation_tests.rs` | unknown | Validation tests |
| `benches/enterprise_benchmarks.rs` | unknown | Performance benchmarks |
| Examples | 20 files | Mix of legitimate examples and debug scratch |

### Features

| Feature | Status |
|---------|--------|
| Multi-format parsing (8 formats) | ✅ Implemented |
| Multi-tier caching (FastCache + main cache) | ✅ Implemented |
| Hot reloading (polling-based) | ✅ Implemented |
| Audit logging | ✅ Implemented |
| Environment variable overrides | ✅ Implemented |
| Schema validation | ✅ Implemented |
| Format preservation | ⚠️ Partial — claimed but NOML/TOML rely on the `noml` crate's preservation |
| Multi-instance support (`ConfigManager`) | ✅ Implemented |
| Read-only configuration access | ⚠️ Partial — `read_only: bool` field exists but enforcement varies |
| Write/modify configuration | ✅ Implemented (set/save) |
| Hot-reload event-driven (inotify/kqueue/RDCW) | ❌ Not implemented (polling only) |
| Async I/O | ✅ Implemented (feature-gated) |
| Concurrent benchmark | ⚠️ Single benchmark file, scope unknown |

---

## Standards compliance audit

### Against `REPS.md`

| Requirement | Status | Notes |
|-------------|--------|-------|
| `#![deny(missing_docs)]` | ⚠️ Uses `warn`, not `deny` | Easier to ship, weaker safety |
| `#![deny(unsafe_op_in_unsafe_fn)]` | ❌ Missing | |
| `#![deny(unused_must_use)]` | ❌ Missing | |
| `#![deny(unused_results)]` | ❌ Missing | |
| `#![deny(clippy::unwrap_used)]` | ❌ Missing | Recent CHANGELOG claims unwrap was eliminated, but no lint enforces this |
| `#![deny(clippy::expect_used)]` | ❌ Missing | |
| `#![deny(clippy::todo)]` | ❌ Missing | |
| `#![deny(clippy::unimplemented)]` | ❌ Missing | |
| `#![deny(clippy::print_stdout)]` | ❌ Missing | |
| `#![deny(clippy::print_stderr)]` | ❌ Missing | |
| `#![deny(clippy::dbg_macro)]` | ❌ Missing | |
| `#![deny(clippy::undocumented_unsafe_blocks)]` | ❌ Missing | |
| `#![deny(clippy::missing_safety_doc)]` | ❌ Missing | |
| Custom error types (no `Box<dyn Error>` in public API) | ✅ Has `Error` enum | Need to verify no `Box<dyn>` leaks |
| `thiserror` for libraries | ✅ Uses `thiserror = "1.0"` | |
| Zero-allocation hot path on cache hit | ⚠️ Uses `Arc<RwLock<BTreeMap<String, Value>>>` with clones on every get | This is the #1 perf opportunity |
| Lock-free where contention matters | ❌ `Arc<RwLock>` for cache; lock contention under high concurrency | |
| No `unwrap()` / `expect()` in shipping code | ✅ Claimed in 0.9.0 CHANGELOG | Not enforced by lint |
| MSRV documented | ✅ 1.82 declared | But target is 1.75 |
| Cross-platform CI | ⚠️ Has CI, need to verify platform matrix | |
| Apache-2.0 OR MIT dual license | ❌ Apache-2.0 only | |
| Edition 2024 | ❌ Edition 2021 | |

### Against `PROJECT_STRUCTURE.md`

| Standard file | Status |
|---------------|--------|
| `REPS.md` (canonical) | ❌ Missing |
| `LICENSE-APACHE` + `LICENSE-MIT` | ❌ Single `LICENSE` (Apache only) |
| `rustfmt.toml` | ❌ Missing |
| `clippy.toml` | ❌ Missing |
| `.editorconfig` | ✅ Present |
| `.gitignore` | ✅ Present |
| `.gitattributes` | ✅ Present |
| `.dev/PROMPT.md` | ❌ Missing |
| `.dev/DIRECTIVES.md` | ❌ Missing |
| `.dev/ROADMAP.md` | ❌ Missing |
| `.dev/release/` | ❌ Missing (created now) |
| `.dev/AUDIT-*.md` | ❌ Missing (this is the first one) |
| `docs/release-notes/` | ❌ Missing |
| `docs/ARCHITECTURE.md` | ❌ Missing |
| `docs/PERFORMANCE.md` | ❌ Missing |
| `docs/PLATFORM-NOTES.md` | ❌ Missing |
| `docs/STABILITY-1.0.md` | ❌ Missing (required for 1.0) |
| Root has stray config-fixture files | ❌ `debug_test.conf`, `test.ini`, `test.properties` in root |
| Three duplicate typos configs | ❌ `.typos.toml`, `typos.toml`, `_typos.toml` |
| Examples are clean (no debug scratch) | ❌ ~10 of 20 examples are debug scratch |

### Against `UNIVERSAL_PROMPT.md`

| Principle | Status |
|-----------|--------|
| Peak performance | ⚠️ Performance work exists; claims unverified; lock-based caching limits ceiling |
| Maximum efficiency | ⚠️ `String` allocation on every cache key lookup; `Value::clone()` on every get |
| Maximum concurrency | ⚠️ `Arc<RwLock>` not lock-free; FastCache uses `write` lock to read (hits + misses counters mutated) |
| Maximum security | ✅ No unsafe code; thiserror; recent hardening pass |
| Cross-platform first-class | ✅ Pure Rust, no platform-specific code |
| Production-ready from line one | ⚠️ Lint discipline weak |
| Comprehensive testing | ⚠️ Two test files in `tests/`; need to verify coverage |
| Error hardening | ⚠️ `Result<T, Error>` everywhere, but no test for every error path verified |
| Documentation as deliverable | ✅ rustdoc is solid |

---

## Architectural concerns

### 1. Dual API surfaces

The crate exposes TWO APIs:

```rust
pub use config::{Config, ConfigBuilder, ConfigValue};
pub use enterprise::{ConfigManager, EnterpriseConfig};
```

`Config` and `EnterpriseConfig` overlap significantly. This forces users to choose, fragments documentation, and doubles maintenance. **For 1.0, we should commit to ONE API.**

Two paths forward:
- **(A)** Unify: replace `Config` with `EnterpriseConfig` (rename to `Config`) and drop the standard one. Standard becomes a deprecation alias.
- **(B)** Tier: keep both, but make `Config` literally `EnterpriseConfig` with caching disabled (no separate code paths).

Path (A) is cleaner. Path (B) preserves the current public API for users on `0.9.x`.

### 2. Lock-based caching limits performance ceiling

```rust
pub struct EnterpriseConfig {
    fast_cache: Arc<RwLock<FastCache>>,
    cache: Arc<RwLock<BTreeMap<String, Value>>>,
    defaults: Arc<RwLock<BTreeMap<String, Value>>>,
    ...
}
```

`RwLock` is OK for reader-heavy loads, but:
- The "fast cache" hit path takes a **write** lock just to bump the hit counter
- `BTreeMap` operations under a global lock get serialized

For real sub-50ns cached access under contention, this needs:
- **`DashMap`** for sharded concurrent access, OR
- **`arc-swap::ArcSwap<HashMap>`** for fully lock-free reads with rare swaps, OR
- **`evmap`** for lock-free read-side, occasional writer

Current design will not hit sub-50ns under 8+ thread contention.

### 3. NOML dependency creates 1.0 risk

`config-lib`'s NOML and TOML parsers both route through the `noml` crate (which is itself pre-1.0). When we ship `config-lib 1.0`, our 1.0 stability contract is tied to `noml 0.9.x` — a pre-1.0 crate.

**Resolution paths:**
- **(A)** Pin to exact `noml = "=0.9.x"` version. Promise to bump in MINOR releases. Document the dependency caveat in `STABILITY-1.0.md`.
- **(B)** Make NOML/TOML optional non-default features. Users opt-in if they need them.
- **(C)** Vendor or replace with an internal minimal NOML/TOML parser (heavy, probably not worth it).

Path (B) is the cleanest for our 1.0 stability promise. Hive DB can opt-in to NOML/TOML features explicitly.

### 4. Hot reload is polling-based

`hot_reload.rs` uses thread-based polling with `Duration`-based intervals. This works but:
- Wastes CPU when nothing changes
- Has detection latency equal to the poll interval
- Doesn't integrate with the OS file notification mechanisms

For 1.0, **`notify` crate** is the standard. It uses inotify (Linux), FSEvents (macOS), and ReadDirectoryChangesW (Windows). Latency drops from poll-interval-seconds to ~milliseconds.

### 5. Format preservation claim is partial

The crate claims "format preservation" but:
- NOML/TOML preservation is delegated to the `noml` crate
- CONF parser does not preserve comments and whitespace on round-trip
- INI parser does not preserve preservation either
- JSON parser uses `serde_json` which doesn't preserve formatting

For a true "format-preserving" claim, we'd need a NOML-style CST (concrete syntax tree) for at least the in-house formats. **For 1.0, scope down the claim** to "NOML/TOML format preservation" only, OR build CST round-tripping for CONF/INI.

NOML's approach (per James's request to borrow) is the right model — CST-based parsing where every byte is accounted for. Implementing this for CONF and INI is ~2 weeks of focused work.

### 6. Read-only / dynamic mixed configuration

The current `read_only: bool` field is too binary. A real "read-only sections + dynamic sections" architecture needs:
- Per-key read-only flags, OR
- Compile-time marker types (`ReadOnly` vs `Mutable`)
- Clear API distinction between get-immutable and get-mutable

This is a 1.0 feature to design carefully because it affects the entire public API.

---

## Performance opportunities (concrete)

1. **Eliminate `String` allocation on cache key lookups.** Use `&str` keys via `Cow<'_, str>` or interned strings.
2. **Eliminate `Value::clone()` on every `get()`.** Return `&Value` or `Cow<Value>` or arena-allocated values.
3. **Replace `Arc<RwLock<BTreeMap>>` with `DashMap` or `ArcSwap`.** Lock-free reads.
4. **Use `FxHashMap` (rustc-hash) over std `HashMap`.** ~30% faster for short string keys.
5. **Inline hot accessor methods explicitly.** `#[inline(always)]` on `Value::as_string`, `Value::as_integer`, etc.
6. **SIMD where applicable.** Format detection scans (find first non-whitespace byte) can use `memchr`.
7. **Memory-map large config files** instead of `read_to_string`. For files > 64 KiB.
8. **Profile-guided optimization (PGO).** Configure in `Cargo.toml` for users who want it.

---

## Testing gaps

| Category | Current | Target for 1.0 |
|----------|---------|----------------|
| Unit tests | 44 (claimed in CHANGELOG) | Same or grown — verify coverage |
| Integration tests | 2 files | At least one integration file per major feature: parsers, caching, hot-reload, audit, env-override, multi-instance |
| Doc tests | 5 (claimed) | Every public function with a runnable doctest |
| Property tests (proptest) | 0 | At least: parser round-trips, value type conversions, cache invariants |
| Concurrency tests (loom) | 0 | Cache + hot-reload core under loom |
| Benchmarks with committed baselines | 1 file, baselines uncertain | Full hot-path coverage with committed `baselines.json` |
| Fuzz targets | 0 | Each parser fuzzed (parsers ingest untrusted input from disk) |
| Cross-platform CI | Status unknown | Verify all three platforms in matrix |

**Fuzz testing is required.** A config library ingests arbitrary user files. Without fuzzing, we ship security holes.

---

## Documentation gaps

| Doc | Status |
|-----|--------|
| `README.md` | ✅ Excellent, marketing-grade |
| `CHANGELOG.md` | ✅ Maintained, well-categorized |
| `docs/API.md` | ✅ Exists |
| `docs/FORMATS.md` | ✅ Exists |
| `docs/GUIDELINES.md` | ✅ Exists |
| `docs/README.md` | ✅ Exists |
| `docs/ARCHITECTURE.md` | ❌ Missing |
| `docs/PERFORMANCE.md` | ❌ Missing (benchmark methodology + tuning guide) |
| `docs/PLATFORM-NOTES.md` | ❌ Missing |
| `docs/STABILITY-1.0.md` | ❌ Missing (REQUIRED for 1.0) |
| `docs/release-notes/v*.md` | ❌ Missing entirely |
| `.dev/PROMPT.md`, `.dev/DIRECTIVES.md`, `.dev/ROADMAP.md` | ❌ Missing |

---

## Recommended scope for 1.0

The fastest path to a high-quality 1.0 is to:

1. **Bring structure to standard** (configs, licensing, .dev/ docs, REPS, clean root).
2. **Enforce REPS via lint discipline** in `src/lib.rs`.
3. **Replace the caching layer** with a lock-free implementation (`DashMap` or `ArcSwap` — choose one).
4. **Replace polling hot-reload** with `notify`-based event-driven reload.
5. **Add fuzz harnesses** for every parser.
6. **Add criterion baselines** committed to `benches/baselines.json`.
7. **Make NOML/TOML opt-in features** for the 1.0 stability contract.
8. **Write `docs/STABILITY-1.0.md`** defining the 1.0 contract.
9. **Consolidate `Config` and `EnterpriseConfig`** into a single API.
10. **Clean up examples** (delete debug scratch, keep curated examples).

**Out of scope for 1.0** (deferred to 1.1+):
- Full CST-based format preservation for CONF/INI (substantial work)
- Mixed read-only / dynamic config (needs design work)
- Async hot-reload integration with tokio runtime (post-1.0 add-on)
- Distributed configuration (etcd, consul integration)

---

## Risk register

| Risk | Severity | Mitigation |
|------|----------|------------|
| NOML dependency on a 0.9.x crate | High | Make NOML/TOML opt-in, pin exact version |
| Lock-based caching limits perf ceiling | High | Replace with `DashMap` or `ArcSwap` before 1.0 |
| Polling hot-reload wastes CPU | Medium | Replace with `notify`-based hot-reload |
| Format preservation overclaim | Medium | Scope down claim or implement CST round-tripping |
| Dual API surface complexity | Medium | Consolidate to a single `Config` |
| No fuzz testing | High | Add cargo-fuzz harnesses for all parsers |
| Polish quality below portfolio standard | Medium | This audit + roadmap fixes it |
| MSRV / edition out of sync with portfolio | Low | Bump in foundation phase |

---

## Sign-off

This audit reflects the state of `config-lib` at `0.9.1` on 2026-05-18. The crate is fundamentally sound — the work needed is polish, performance hardening, structure normalization, and one focused round of architectural cleanup (consolidate API, lock-free caching, event-driven hot reload).

Estimated path to 1.0: **roughly 4-6 focused weeks of work** if the AI editor + maintainer iterate at the cadence of the rest of the portfolio. Roadmap in `.dev/ROADMAP.md`.

---

<sub>config-lib audit — Copyright &copy; 2026 James Gober. Apache-2.0 OR MIT (post-1.0 dual licensing).</sub>
