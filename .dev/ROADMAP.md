# config-lib — Roadmap to 1.0

> Path from current `0.9.1` (audited 2026-05-18) to `1.0.0` stable.
> **Estimated work: 4-6 focused weeks.**
> See `.dev/AUDIT-0.9.1.md` for the full audit that informs this roadmap.

---

## The 1.0 vision

`config-lib 1.0.0` is the **canonical configuration library for the Hive DB stack** and a production-grade Rust config library for the wider ecosystem.

At 1.0, it provides:

- **Single unified `Config` API** — no more `Config` vs `EnterpriseConfig` split.
- **Lock-free cached reads** — sub-50ns access verified by benchmark, not just claimed.
- **Event-driven hot reload** — `notify` crate integration (inotify / kqueue / RDCW), ~ms detection.
- **Multi-format support** — CONF, INI, Properties, JSON, XML, HCL built-in. NOML, TOML opt-in.
- **Format preservation** — for NOML/TOML via the upstream `noml` crate. Other formats document this honestly.
- **Audit logging + env overrides + schema validation** — all retained, polished.
- **Multi-instance via `ConfigManager`** — multiple named configs spun from a single process.
- **Read-only + dynamic split** — basic markers in 1.0; richer typestate API deferred to 1.1+.
- **Write support** — `config.set()` + `config.save()` preserves format where supported.
- **Cross-platform** — Linux, macOS, Windows. Same behavior on all three.
- **Fuzz-tested** — every parser has a `cargo-fuzz` target.
- **Production-ready lint discipline** — full REPS enforcement.

---

## Phase 0.9.2 — Structure normalization (current sprint)

**Goal:** Bring the repository structure to portfolio standard. No code logic changes.

**Estimated work:** 1-2 days.

### Tasks

- [x] Audit document committed to `.dev/AUDIT-0.9.1.md`
- [x] This roadmap committed to `.dev/ROADMAP.md`
- [ ] Add `REPS.md` at repo root (canonical copy from `_strategy/REPS.md`)
- [ ] Add `.dev/PROMPT.md` — project context, skill areas, scope
- [ ] Add `.dev/DIRECTIVES.md` — project-specific directives
- [ ] Add dual licensing: `LICENSE-APACHE` + `LICENSE-MIT` (replace single `LICENSE`)
- [ ] Add `rustfmt.toml` (portfolio standard)
- [ ] Add `clippy.toml` (portfolio standard)
- [ ] Move root config fixtures (`debug_test.conf`, `test.ini`, `test.properties`) into `tests/fixtures/`
- [ ] Consolidate the three typos config files into one (`typos.toml` is the canonical one; remove `.typos.toml` and `_typos.toml`)
- [ ] Clean up examples — keep curated 5-7 examples, move debug scratch to `.dev/scratch/` or delete
- [ ] Update `Cargo.toml`: `license = "Apache-2.0 OR MIT"`
- [ ] Create `docs/release-notes/` directory
- [ ] Move existing `docs/README.md`, `docs/API.md`, `docs/FORMATS.md`, `docs/GUIDELINES.md` content into the documented portfolio layout
- [ ] Add `.github/workflows/ci.yml` if not already in portfolio CI format (Node 24, Linux/macOS/Windows × stable + MSRV)

### Exit criteria

- Repository structure passes `PROJECT_STRUCTURE.md` 0.1.0 minimum + matches what other portfolio crates have.
- All standards documents present at root and in `.dev/`.

---

## Phase 0.9.3 — REPS compliance + edition/MSRV alignment

**Goal:** Bring lint discipline and toolchain configuration to portfolio standard. No feature changes.

**Estimated work:** 2-3 days.

### Tasks

- [ ] Update `Cargo.toml`:
  - [ ] `edition = "2024"` (from 2021)
  - [ ] `rust-version = "1.75"` (from 1.82)
  - [ ] Verify portfolio crate metadata fields all present (homepage, repository, documentation, keywords, categories)
- [ ] Update `src/lib.rs` lint configuration to portfolio standard:
  - [ ] `#![deny(missing_docs)]` (upgrade from `warn`)
  - [ ] `#![deny(unsafe_op_in_unsafe_fn)]`
  - [ ] `#![deny(unused_must_use)]`
  - [ ] `#![deny(unused_results)]`
  - [ ] `#![deny(clippy::unwrap_used)]`
  - [ ] `#![deny(clippy::expect_used)]`
  - [ ] `#![deny(clippy::todo)]`
  - [ ] `#![deny(clippy::unimplemented)]`
  - [ ] `#![deny(clippy::print_stdout)]`
  - [ ] `#![deny(clippy::print_stderr)]`
  - [ ] `#![deny(clippy::dbg_macro)]`
  - [ ] `#![deny(clippy::undocumented_unsafe_blocks)]`
  - [ ] `#![deny(clippy::missing_safety_doc)]`
- [ ] Fix any violations introduced by the lint tightening (expected: small number of cleanups in `audit.rs`, `enterprise.rs`)
- [ ] Verify `cargo fmt --all -- --check` clean
- [ ] Verify `cargo clippy --all-targets --all-features -- -D warnings` clean
- [ ] Verify `cargo test --all-features` passes on all three platforms
- [ ] Verify `cargo doc --no-deps --all-features` produces zero warnings with `RUSTDOCFLAGS="-D warnings"`

### Exit criteria

- All REPS lint denies in place.
- All CI checks green on Linux, macOS, Windows on stable + MSRV.

---

## Phase 0.9.4 — Architectural consolidation

**Goal:** Unify the dual `Config` / `EnterpriseConfig` API into a single ergonomic surface. Minimize duplication.

**Estimated work:** 1 week.

### Tasks

- [ ] Design unified `Config` type combining the best of both:
  - [ ] Caching is on by default (no separate `EnterpriseConfig` type)
  - [ ] Caching can be disabled via `ConfigOptions::without_cache()` for testing
  - [ ] `Config::builder()` returns a `ConfigBuilder` with full options
- [ ] Decide on `ConfigManager` keep-or-merge — likely keep as separate type since multi-instance is a distinct concept
- [ ] Deprecate `EnterpriseConfig` as type alias for the new unified `Config` (with `#[deprecated]` note)
- [ ] Update all examples to use the new unified API
- [ ] Update all integration tests to use the new unified API
- [ ] Update rustdoc for every public item touched by the consolidation
- [ ] Document the migration path from `EnterpriseConfig` to `Config` in CHANGELOG

### Exit criteria

- Single `Config` API in public docs.
- `EnterpriseConfig` is a deprecated type alias, still works for `0.9.x` users.
- All examples and tests use the new API.

---

## Phase 0.9.5 — Lock-free caching

**Goal:** Replace `Arc<RwLock<BTreeMap>>` caching with a lock-free implementation. Validate sub-50ns claim with committed benchmarks.

**Estimated work:** 1 week.

### Tasks

- [ ] Evaluate caching backends:
  - [ ] `DashMap` — sharded concurrent map (low contention reads + writes)
  - [ ] `ArcSwap<HashMap>` — fully lock-free reads, atomic writer swap
  - [ ] `evmap` — left-right paired hash map
- [ ] Prototype both `DashMap` and `ArcSwap` and benchmark
- [ ] Pick the winner based on:
  - [ ] Read latency at 1, 4, 16, 64 threads
  - [ ] Write latency (less critical but tracked)
  - [ ] Memory overhead
  - [ ] Code complexity
- [ ] Replace cache layer in `Config` with chosen backend
- [ ] Remove `FastCache` tier or redesign it as a thread-local hot-cache (avoid the write-lock-on-read anti-pattern)
- [ ] Update cache statistics API to use atomic counters
- [ ] Write criterion benchmarks covering:
  - [ ] Single-key access (cold + warm)
  - [ ] Nested key access (`server.database.host`)
  - [ ] Concurrent reads at 4, 16, 64 threads
  - [ ] Mixed read/write at 4, 16 threads
- [ ] Commit `benches/baselines.json` with the new numbers
- [ ] Update `docs/PERFORMANCE.md` with methodology and results

### Exit criteria

- Cached single-key get under 50ns on a current dev machine (sustained across thread counts).
- Benchmark baselines committed.
- `docs/PERFORMANCE.md` documents methodology + numbers.

---

## Phase 0.9.6 — Event-driven hot reload

**Goal:** Replace polling-based hot-reload with OS-native event-driven file watching via `notify`.

**Estimated work:** 4-5 days.

### Tasks

- [ ] Add `notify = "6"` dependency (feature-gated behind `hot-reload` feature)
- [ ] Rewrite `hot_reload.rs` to use `notify` watcher:
  - [ ] Linux: inotify
  - [ ] macOS: FSEvents
  - [ ] Windows: ReadDirectoryChangesW
  - [ ] Debounce events (file editors often write+rename, generating multiple events)
- [ ] Preserve existing `ConfigChangeEvent` API for backward compatibility
- [ ] Add `HotReloadConfig::watch_with_debounce(Duration)` — configurable debounce window
- [ ] Add cross-platform integration tests:
  - [ ] File modified → reload event fires
  - [ ] File renamed → handled gracefully
  - [ ] File deleted → `FileDeleted` event fires
  - [ ] File temporarily missing during atomic write → no spurious failure
- [ ] Document platform-specific behavior in `docs/PLATFORM-NOTES.md`
- [ ] Update `docs/release-notes/v0.9.6.md`

### Exit criteria

- Reload latency drops from poll-interval seconds to <100ms.
- Cross-platform integration tests pass.
- No CPU usage when no file changes occur.

---

## Phase 0.9.7 — NOML/TOML opt-in + dependency hygiene

**Goal:** Lock down the 1.0 stability contract by making pre-1.0 dependencies opt-in.

**Estimated work:** 2-3 days.

### Tasks

- [ ] Move `noml` and `toml` features OUT of default features:
  ```toml
  default = ["conf", "ini", "properties", "json", "xml", "hcl"]
  noml = ["dep:noml"]
  toml = ["dep:noml"]   # still routes via noml crate
  ```
- [ ] Pin `noml = "=0.9.x"` (exact version pin while NOML is pre-1.0)
- [ ] Run `cargo audit` — no known vulnerabilities
- [ ] Run `cargo deny check` — no policy violations
- [ ] Verify all dependencies have MSRV ≤ 1.75
- [ ] Document the NOML dependency caveat in `docs/STABILITY-1.0.md`
- [ ] Add `deny.toml` if missing (it already exists; review and update)

### Exit criteria

- Default feature set has no pre-1.0 dependencies.
- NOML/TOML are clean opt-in features.
- All dependency checks pass.

---

## Phase 0.9.8 — Fuzz testing

**Goal:** Add `cargo-fuzz` harnesses for every parser. Run for >1 hour each. Fix any findings.

**Estimated work:** 3-4 days.

### Tasks

- [ ] Add `fuzz/` directory with `cargo-fuzz` setup
- [ ] Create fuzz targets:
  - [ ] `fuzz_targets/conf_parser.rs`
  - [ ] `fuzz_targets/ini_parser.rs`
  - [ ] `fuzz_targets/properties_parser.rs`
  - [ ] `fuzz_targets/json_parser.rs`
  - [ ] `fuzz_targets/xml_parser.rs`
  - [ ] `fuzz_targets/hcl_parser.rs`
- [ ] Run each target for at least 1 hour on the maintainer machine
- [ ] Fix any panics, infinite loops, OOMs
- [ ] Add corpus of interesting inputs found during fuzzing
- [ ] Document fuzz testing methodology in `docs/PLATFORM-NOTES.md` or new `docs/FUZZING.md`

### Exit criteria

- Six fuzz targets, all clean for at least 1 hour.
- Any findings during fuzzing fixed.

---

## Phase 0.9.9 — Documentation completeness + RC

**Goal:** Final documentation pass and release candidate cut.

**Estimated work:** 3-4 days.

### Tasks

- [ ] Write `docs/STABILITY-1.0.md` — the 1.0 contract:
  - [ ] API surface frozen
  - [ ] MSRV policy (1.75 baseline, MINOR releases may bump within 12-month window)
  - [ ] Feature flag stability (defaults won't change in 1.x)
  - [ ] NOML/TOML dependency caveat
  - [ ] Yank policy
  - [ ] Deprecation timeline
- [ ] Write `docs/ARCHITECTURE.md` — internal architecture:
  - [ ] Module structure
  - [ ] Data flow (file → parser → Value → cache)
  - [ ] Caching architecture
  - [ ] Hot reload architecture
  - [ ] Thread safety guarantees
- [ ] Write `docs/PERFORMANCE.md` — benchmark methodology + tuning guide
- [ ] Write `docs/PLATFORM-NOTES.md` — OS-specific behavior
- [ ] Verify every public item has rustdoc with at least one example
- [ ] Verify `cargo doc --no-deps --all-features` produces zero warnings
- [ ] Cut `1.0.0-rc.1` per RELEASE_WORKFLOW.md
- [ ] Solicit feedback from at least one external user (if available)

### Exit criteria

- All required docs in place.
- `1.0.0-rc.1` published to crates.io as pre-release.
- At least 1 week of RC soak time with no critical issues.

---

## Phase 1.0.0 — Stable release

**Goal:** Ship the canonical config library.

### Tasks

- [ ] Verify no critical issues from RC soak
- [ ] Final API freeze verification (no last-minute changes)
- [ ] Update `Cargo.toml` version to `1.0.0`
- [ ] Move `[Unreleased]` CHANGELOG to `[1.0.0] - <date>`
- [ ] Write `docs/release-notes/v1.0.0.md` per `_strategy/RELEASE_NOTES_TEMPLATE.md`
- [ ] Commit `Milestone Update v1.0.0`
- [ ] Push, verify CI green
- [ ] Tag `v1.0.0`
- [ ] Create GitHub release (NOT marked as pre-release)
- [ ] `cargo publish`
- [ ] Verify crates.io and docs.rs both show 1.0.0
- [ ] Announcement (project README, Hive DB README, optional blog post)

### Exit criteria

- `config-lib 1.0.0` live on crates.io.
- docs.rs builds clean.
- Hive DB consumes `config-lib = "1.0"` for its configuration layer.

---

## Post-1.0 backlog (deferred to 1.1+)

These are explicitly OUT of scope for 1.0 but tracked for future work:

- [ ] CST-based format preservation for CONF and INI (extensive work)
- [ ] Async file watcher integration with tokio (use `tokio::sync::watch` for change notifications)
- [ ] Typestate API for read-only / mutable distinction (compile-time enforcement)
- [ ] Distributed config sources (etcd, consul, Vault integration)
- [ ] Configuration diffing API (compute the diff between two `Config` instances)
- [ ] Encryption-at-rest for sensitive values
- [ ] `serde::Deserialize` impl for the `Value` type
- [ ] `proptest` invariant tests for parser round-tripping
- [ ] `loom` model checking for the cache + hot-reload interaction

---

## Quick reference

```
==============================================================
config-lib roadmap to 1.0
==============================================================
0.9.2  ─ Structure normalization              (1-2 days)
0.9.3  ─ REPS compliance + edition/MSRV       (2-3 days)
0.9.4  ─ Architectural consolidation          (1 week)
0.9.5  ─ Lock-free caching                    (1 week)
0.9.6  ─ Event-driven hot reload              (4-5 days)
0.9.7  ─ NOML/TOML opt-in + deps              (2-3 days)
0.9.8  ─ Fuzz testing                         (3-4 days)
0.9.9  ─ Docs + RC                            (3-4 days)
1.0.0  ─ Stable release                       (1 day)
==============================================================
Total: ~4-6 focused weeks
==============================================================
```

---

<sub>config-lib roadmap — Copyright &copy; 2026 James Gober. Apache-2.0 OR MIT (post-1.0 dual licensing).</sub>
