# The `config-lib` 1.0 Stability Contract

This document is the canonical reference for what `config-lib` 1.0
promises its users. It lives next to the code so it cannot drift from
the implementation, and it is the single source of truth when answers
to "is this API stable?", "what's the MSRV policy?", or "what's *not*
guaranteed?" matter.

This contract is **in force as of `v1.0.0`**. Every promise below
applies to the v1.x SemVer line. Items in force during the `0.9.x`
build-up (e.g. the `#[non_exhaustive]` markers, the MSRV 1.75
commitment) are unchanged.

---

## 1. What is part of the stability contract

### 1.1 Frozen public items

Every `pub` item in the crate root and in `pub` modules is part of
the SemVer contract. That covers:

- `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`,
  `pub const`, `pub static`
- The signatures of `pub fn` items (argument types, return types,
  generic bounds, `where` clauses)
- The field set of `pub` (or `pub(crate)`-with-public-getter)
  structs, *modulo* `#[non_exhaustive]` markers (see §1.2)
- The variant set of `pub enum`s, *modulo* `#[non_exhaustive]`
  markers (see §1.2)
- The bound set of `pub trait`s and the signatures of their methods
- The public re-exports at the crate root

Breaking any of these requires a major-version bump (`2.0`).

### 1.2 `#[non_exhaustive]` policy

The following public enums and structs are marked `#[non_exhaustive]`
so the v1.x SemVer line can add variants / fields without breaking
SemVer:

| Item                 | Module               | Why                                                                       |
|----------------------|----------------------|---------------------------------------------------------------------------|
| `Error`              | `error.rs`           | New error categories arrive with every new feature.                       |
| `ConfigChangeEvent`  | `hot_reload.rs`      | New file-system event types may be added.                                 |
| `ValidationSeverity` | `validation.rs`      | Severity tiers may grow.                                                  |
| `AuditEventType`     | `audit.rs`           | New operation types arrive with every audited subsystem.                  |
| `AuditSeverity`      | `audit.rs`           | Parallel to `ValidationSeverity`.                                         |
| `FieldType`          | `schema.rs`          | Schema field types grow with format support.                              |
| `CacheStats`         | `config.rs`          | Born non-exhaustive in 0.9.5; new counter fields planned for 0.9.5.x.     |
| `ConfigOptions`      | `config.rs`          | Born non-exhaustive in 0.9.4; new knobs may be added (eviction policy, etc.).      |

Adding a new variant / field to any `#[non_exhaustive]` type is **not**
a breaking change; v1.x users are required to use wildcard match arms
and `..Default::default()` struct-update syntax against these types.

**`Value` and `ValueType` are intentionally *not* `#[non_exhaustive]`.**
They form the core type system; exhaustive matching against every
variant is a feature for users writing format converters and type
dispatchers. Adding a new `Value` variant in the future would be a
deliberate breaking change requiring a `2.0` bump.

### 1.3 Public symbol enumeration

The exact set of `pub` items at the `v1.0.0` tag is the contract. A
machine-readable diff is producible via [`cargo
public-api`](https://crates.io/crates/cargo-public-api). Every PR that
changes a `pub` signature on the `1.x` line is required to attach the
`cargo public-api diff` output to its description; reviewers gate on
"breaking change?" before merge.

---

## 2. What is **not** part of the stability contract

- **Exact performance numbers.** The README and rustdoc may quote
  measured latencies (e.g. "single-key cached get: 30 ns"). These are
  *targets and recent measurements*, not stability promises. The
  *targets* in the Performance Contract (§5) are what matter; the
  measured values are advisory and may improve or regress within the
  target envelopes across minor releases.
- **Exact error message text.** `Error::Display` strings are
  human-readable diagnostics, not parser inputs. Comparing them
  against literal strings in user code is unsupported.
- **Transitive dependency versions.** A `cargo update` may pull in
  different patch versions of `serde`, `notify`, etc. Only the
  direct-dependency *requirements* in `Cargo.toml` are part of the
  contract.
- **Private modules and items.** Anything not marked `pub` (or
  reachable through a `pub` re-export) is internal scaffolding and
  may change at any time without notice.
- **Iteration ordering of `keys()` and `Value::Table` traversal.**
  `BTreeMap` is the current backing store and iterates in sorted
  order; the contract does not promise to keep using `BTreeMap`. If
  your code depends on iteration order, sort the result yourself.
- **Cargo feature internals.** Which transitive deps a feature
  pulls in, what `Cargo.toml`'s `[features]` keys look like
  *internally* — only the named, documented features themselves
  are part of the contract.

---

## 3. MSRV policy

### 3.1 Baseline MSRV: Rust 1.75

The 1.0 stability contract commits to **Rust 1.75** as the minimum
supported toolchain for the default feature set.

This is delivered as of `v0.9.7`: `Cargo.toml` declares
`rust-version = "1.75"`, and the default build (`cargo build` with
no extra features) compiles cleanly on `1.75.0`. This is exercised
by CI on every commit.

### 3.2 Feature-flag MSRV asymmetry

The **`noml` and `toml` features have a higher MSRV — Rust 1.82.**

This is because the upstream `noml = "=0.9.0"` crate itself declares
`rust-version = "1.82"`. Users enabling either feature must be on
Rust 1.82 or newer; the rest of `config-lib` remains MSRV-1.75. The
mismatch resolves naturally when an upstream `noml` release lowers
its own MSRV; until then, the asymmetry is documented here as a
known constraint.

### 3.3 MSRV bumps within v1.x

- **PATCH releases** (`1.0.x`) **never bump MSRV.**
- **MINOR releases** (`1.x.0`) may bump MSRV, subject to a moving
  window: the new MSRV must be within the last 12 stable Rust
  releases at the time of the bump. (Concretely: when this contract
  was written in May 2026, stable Rust is 1.95, so the MSRV bump
  window is `1.83+`. The MSRV may not jump above stable.)
- **MAJOR releases** (`2.x.0`) may bump MSRV freely.

The intent: deployment environments locked to a specific Rust
release see only PATCH-level updates for the duration of the v1.x
line; minor-release upgrades require a small toolchain bump check
but never a giant leap.

---

## 4. Feature flag stability

### 4.1 Stable named features

The named features below are part of the v1.0 contract. Their *names*
and *enabling behavior* are stable; their transitive dependencies may
change between releases.

#### Default features
- **`conf`** — Built-in CONF format parser. Default.
- **`hot-reload`** — Event-driven file watching via `notify`. Default.

#### Opt-in format features
- **`json`** — JSON parsing via `serde_json`.
- **`xml`** — XML parsing via `quick-xml`.
- **`hcl`** — HashiCorp Configuration Language parsing (built-in).
- **`noml`** — NOML parsing via the upstream `noml` crate (see §4.3).
- **`toml`** — TOML parsing routed through the upstream `noml` crate
  for format preservation (see §4.3).

#### Opt-in capability features
- **`async`** — Async file I/O via `tokio`.
- **`chrono`** — `DateTime` support via `chrono`.
- **`schema`** — Schema validation framework.
- **`validation`** — Rule-based validation via `regex`.
- **`env-override`** — Environment-variable override system.

### 4.2 Future feature additions

New features may be added in MINOR releases. Removing a feature, or
changing what an existing feature enables in an observably breaking
way, is a MAJOR-release event.

### 4.3 NOML / TOML pre-1.0 dependency caveat

The `noml` and `toml` features depend on the `noml` crate, which is
itself pre-1.0 (`0.9.x`). config-lib pins to `noml = "=0.9.0"`
exactly — a `noml 0.9.1` release does **not** automatically reach
config-lib users via `cargo update`; the maintainer makes a
deliberate pin bump after validating the new noml release.

When `noml 1.0.0` ships upstream, config-lib's NOML/TOML features
will be revisited and the pin loosened to `noml = "1.x"`. Until
then, the features remain opt-in and the maintainer reserves the
right to change `noml`-only behavior with each pin bump.

---

## 5. Performance contract

Every number below is the target the v1.x line commits to. Each
target is backed by a committed `criterion` benchmark.

| Operation                                | Target  | Verified by                                          |
|------------------------------------------|---------|------------------------------------------------------|
| Single-key cached `get` (1 thread, warm) | <50 ns  | `benches/cache_warm.rs`                              |
| Single-key cached `get` (16 threads)     | <50 ns  | `benches/cache_concurrent.rs`                        |
| Single-key cached `get` (cold miss)      | <5 µs   | `benches/cache_cold.rs`                              |
| Nested-key cached `get` (3 levels)       | <100 ns | `benches/cache_warm.rs`                              |
| Typed accessor (`as_string`, etc.)       | <10 ns  | `benches/value_accessors.rs`                         |
| `config.set()` cached write              | <500 ns | `benches/cache_warm.rs`                              |
| Hot reload detection latency             | <100 ms | Integration test, `tests/hot_reload_*.rs`            |
| Cold parse — 1 KiB CONF                  | <10 µs  | `benches/parse_throughput.rs`                        |
| Cold parse — 100 KiB JSON                | <500 µs | `benches/parse_throughput.rs`                        |
| Memory — empty `Config`                  | <1 KiB  | `dhat` measurement (manual)                          |
| Memory — `Config` with 1000 cached keys  | <128 KiB| `dhat` measurement (manual)                          |

The lock-free caching backend that delivers the `get` numbers is
Phase 0.9.5 Implementation work, currently pending canonical-hardware
benchmarks; the foundation API (`Config::cache_stats()`) is in place
as of v0.9.5.

---

## 6. Security contract

- **Zero unsafe code** in the public API. Internal `unsafe` blocks
  (if any) carry `// SAFETY:` comments and are exercised by Miri.
- **Every parser fuzzed** for at least 1 CPU-hour without finding a
  panic, infinite loop, or OOM. Phase 0.9.8 delivers this.
- **No untrusted input reaches `.unwrap()` / `.expect()`.** Enforced
  at compile time by the REPS lint configuration in `src/lib.rs`.
- **`cargo audit` clean** at every release. CI runs it.
- **`cargo deny check` clean** at every release. CI runs it.
- **No secrets logged.** The audit logging system's redaction
  policy is documented and tested.

---

## 7. Deprecation policy

Items marked `#[deprecated]` keep working for at least **one full
MINOR cycle** (six months minimum at the documented release cadence)
before they may be removed in the next MAJOR release. The deprecation
note on each item names the replacement.

Items currently `#[deprecated]` (as of v0.9.7):

- `EnterpriseConfig` (since 0.9.4) — use `Config` directly.
- `ConfigManager` (since 0.9.4) — return-type shape changes in
  0.9.5.x once `Config` absorbs the cached/thread-safe surface.
- `enterprise::direct::parse_string` (since 0.9.4) — use `crate::parse`.
- `enterprise::direct::parse_file` (since 0.9.4) — use `crate::parse_file`.

These are scheduled for removal in `v2.0.0`. They will continue to
compile through the entire v1.x line.

---

## 8. Yank policy

- **Critical correctness bugs** (data loss, memory safety, security)
  trigger yank + same-day patch.
- **Performance regressions** do not trigger yank. They are
  addressed in the next PATCH.
- **Deprecation removals** never trigger yank — they only happen on
  MAJOR bumps with full release notes.

---

## 9. Release process

config-lib releases follow a transparent cadence visible in the
`.dev/ROADMAP.md` document. The flow:

1. Each `0.9.x` patch ships a single roadmap Phase's deliverables
   (or the foundation half of a phase, with the implementation half
   following on canonical CI hardware).
2. `0.9.9` is the **final pre-1.0 polish** — documentation pass,
   external review window, no functional changes.
3. **`v1.0.0` ships directly from `0.9.9`**. There is no
   `1.0.0-rc.1` cut; soak time happens during the `0.9.9` polish
   release itself.
4. Post-1.0, PATCH (`1.0.x`) releases ship as needed; MINOR
   (`1.x.0`) releases ship roughly quarterly with new features.

---

## 10. Out-of-scope items (kept in the post-1.0 backlog)

The following are **not** part of the 1.0 contract and may or may not
ship in future v1.x MINOR releases. Their absence in 1.0 is not a
bug:

- CST-based format preservation for CONF and INI (separate effort).
- Typestate API for read-only / mutable `Config` distinction
  (compile-time enforcement of `ConfigOptions::read_only`).
- Async hot reload integration with `tokio::sync::watch`.
- `serde::Deserialize` for `Value` (post-1.0 convenience layer).
- Distributed configuration sources (`etcd`, `Consul`, `Vault`
  adapters — separate crates).
- Encryption-at-rest for sensitive values.
- Prometheus metrics integration.

---

<sub>This document is canonical. If it conflicts with anything in the
README, the rustdoc, or `.dev/`, this document wins. Updates to it
require a release notes entry under "Stability contract changes".</sub>
