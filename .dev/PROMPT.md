# config-lib — Project Prompt

> Context document for AI editor sessions working on `config-lib`.
> Read this BEFORE writing any code on this crate.

---

## Read order (mandatory)

1. **`REPS.md`** at repo root — Rust Efficiency & Performance Standards. **SUPREME AUTHORITY.**
2. **`_strategy/UNIVERSAL_PROMPT.md`** — portfolio-wide engineering directives.
3. **`.dev/DIRECTIVES.md`** — this project's specific directives.
4. **This file** — project context.
5. **`.dev/ROADMAP.md`** — current phase, milestone targets, exit criteria.
6. **`.dev/AUDIT-0.9.1.md`** — the audit that motivates the current roadmap.

REPS is mandatory and overrides anything else in this repository.

---

## What this crate is

`config-lib` is an **enterprise-grade, multi-format configuration library** for Rust. It provides a single unified `Config` API for reading, modifying, and watching configuration files across 8 supported formats (CONF, INI, Properties, JSON, XML, HCL, NOML, TOML).

It is the **canonical configuration library for the Hive DB stack** — meaning the performance, reliability, and stability requirements are at the level required by a distributed database product.

## Why it exists

Most Rust config libraries:

- Support one format (toml, serde_json, etc.)
- Force a specific deserialization model (typed structs only)
- Lack hot reloading, audit logging, or environment overrides
- Have no caching layer for high-throughput access
- Are not designed for multi-instance use within a single process

`config-lib` consolidates all of these into a production-grade library that a real database product can depend on without needing to wrap it in a custom layer.

---

## Current state (as of this document)

**Version:** `0.9.1` (audit in progress)
**Code:** 6,606 lines across 19 source files
**Status:** Late beta, structurally mature, needs polish to reach 1.0

**Strengths:**
- Multi-format parser layer is real and works (8 formats supported)
- Hot reload, audit logging, env overrides all implemented
- Multi-instance support via `ConfigManager`
- Schema validation and rule-based validation systems exist
- Solid rustdoc throughout
- Recent (0.9.0) safety hardening pass eliminated `unwrap`/`expect` from shipping code

**Gaps to 1.0** (see `.dev/AUDIT-0.9.1.md` for full detail):
- REPS lint discipline weak (only `missing_docs` warn, no other denies)
- Lock-based caching limits sub-50ns claim under contention
- Polling-based hot reload (not event-driven via OS APIs)
- Dual `Config` / `EnterpriseConfig` API needs consolidation
- NOML/TOML default features create pre-1.0 dependency risk
- No fuzz testing
- Missing standard portfolio docs (REPS.md, LICENSE-MIT, .dev/*, docs/STABILITY-1.0.md)

---

## Skill areas

Working on this crate requires comfort with:

- **Configuration parsing** — multiple syntaxes (CONF, INI, JSON, XML, HCL, TOML, NOML, Java properties)
- **Lock-free data structures** — `DashMap`, `ArcSwap`, atomic counters
- **File system event monitoring** — `notify` crate, inotify, FSEvents, ReadDirectoryChangesW
- **Concurrency** — `Arc`, `RwLock`, lock-free alternatives, proper synchronization
- **Performance benchmarking** — `criterion`, baseline tracking, statistical rigor
- **Cross-platform development** — Linux, macOS, Windows file system nuances
- **Fuzz testing** — `cargo-fuzz`, libFuzzer integration
- **Schema validation patterns** — visitor patterns, rule engines
- **Audit logging** — structured logs, sink abstractions, severity filtering

---

## Scope (1.0)

The 1.0 scope is defined in `.dev/ROADMAP.md`. Summarized:

**In scope for 1.0:**
- Single unified `Config` API (consolidate `Config` + `EnterpriseConfig`)
- Lock-free cached reads (verified sub-50ns under contention)
- Event-driven hot reload (`notify` integration)
- All 8 format parsers (CONF, INI, Properties, JSON, XML, HCL built-in; NOML, TOML opt-in)
- Format preservation for NOML/TOML (via the `noml` crate; other formats document honestly)
- Audit logging, environment variable overrides, schema validation — all retained
- Multi-instance `ConfigManager`
- Write support (`config.set()` + `config.save()`)
- Cross-platform parity (Linux, macOS, Windows)
- Fuzz-tested parsers
- Full REPS compliance
- Complete portfolio documentation set

**Out of scope (deferred to 1.1+):**
- CST-based format preservation for CONF/INI (substantial work, separate effort)
- Typestate API for read-only / mutable distinction
- Async hot reload integration with tokio
- Distributed configuration sources (etcd, consul, Vault)
- Encryption-at-rest for sensitive values
- `serde::Deserialize` for `Value` (post-1.0 add-on)

---

## Architectural constraints

### MUST preserve

- **Existing public API** through the 0.9.x line. `EnterpriseConfig` becomes a deprecated alias, not a hard break.
- **8-format parser surface.** Don't drop a format silently.
- **`ConfigManager` multi-instance pattern.** It's a useful primitive.
- **Audit logging surface.** Compliance-grade requirement for Hive DB.
- **Environment variable override system.** Heavily used in deployment.

### MUST change

- **Default features.** NOML/TOML out of default.
- **Caching layer.** Lock-free replacement.
- **Hot reload.** Event-driven replacement.
- **Lint configuration.** Full REPS denies.
- **License.** Dual Apache-2.0 + MIT.
- **MSRV.** Down to 1.75 (or up to match portfolio if portfolio bumps).
- **Edition.** 2024.

### MUST NOT do

- Pull in async runtime as a hard dependency. Async support stays feature-gated.
- Add heavy framework dependencies (no `tower`, no `axum`, etc.).
- Break the `Config::from_string()` and `Config::from_file()` entry points.
- Silently drop a format mid-roadmap.

---

## Performance targets (verified by benchmark before claiming)

| Operation | Target | Current claim | How to verify |
|-----------|--------|---------------|---------------|
| Single-key cached get (warm) | <50ns | 24.9ns (claimed) | `criterion`, single-thread tight loop, current dev machine |
| Single-key cached get (1 thread vs 16 threads) | <50ns at both | unverified | `criterion`, contention benchmark |
| Cold parse → ready-to-read | <100µs for 1KB file | unverified | `criterion`, end-to-end load |
| Hot reload detection latency | <100ms | poll-based, ~seconds | `notify`-based detection |
| Memory overhead per `Config` instance | <1MB baseline | unverified | `dhat` or manual measurement |
| Concurrent reads (16 threads) | No measurable contention slowdown | likely degrades | `criterion` parametric |

**Rule:** if a target is not yet verified, claim it as a goal in docs, not as a fact.

---

## How to develop on this crate

1. Read this document, REPS, DIRECTIVES, ROADMAP.
2. Check current phase in `.dev/ROADMAP.md`.
3. Pick the next unchecked task in that phase.
4. Implement with REPS discipline:
   - No `unwrap`, no `expect`, no `todo!`, no `unimplemented!`
   - No `print_stdout`, no `print_stderr`, no `dbg!`
   - Every new public item: rustdoc + at least one example
   - Every new error path: test
   - Every hot path change: benchmark
5. Update `CHANGELOG.md` under `[Unreleased]` in the same commit.
6. Run the full CI gate locally before pushing:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all-features`
   - `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"`
7. Mark the task done in `.dev/ROADMAP.md` in the same commit.
8. Push.

When a phase is complete:

- Write `.dev/release/v0.9.X.md` with notes
- Bump version in `Cargo.toml`
- Run the full release workflow per `_strategy/RELEASE_WORKFLOW.md`

---

## Hive DB integration context

Hive DB will depend on `config-lib` for:

- Loading server configuration on startup
- Hot-reloading limits, quotas, and operational parameters without restart
- Multi-instance configs (one per database, one per service, one global)
- Audit logging of config changes for compliance
- Environment variable overrides in container deployments

Performance matters because:

- Hive DB may read config values on hot paths (per-query limits, etc.)
- Hot reload should not stall the database
- Audit logging should not slow down config reads

Reliability matters because:

- Config corruption could prevent a database from starting
- A hot reload failure should not bring down a running database
- Multi-instance bugs could affect tenant isolation

**Translation:** every decision on this crate should be evaluated against "is this acceptable in a production database?" If the answer is no, fix it before 1.0.

---

## When in doubt

- Read REPS first.
- Read the audit document.
- Check the roadmap for the current phase's exit criteria.
- If a feature isn't in the roadmap, propose it (update the roadmap first) before implementing.
- If performance is contested, write a benchmark.
- If correctness is contested, write a test.
- If documentation feels unclear, the API needs revision before docs.

---

<sub>config-lib — Copyright &copy; 2026 James Gober. Apache-2.0 OR MIT (post-1.0 dual licensing).</sub>
