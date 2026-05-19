# config-lib — Directives

> Project-specific engineering directives. Apply on top of REPS and the portfolio universal directives.

---

## Priority order

1. `REPS.md` at repo root — **SUPREME AUTHORITY**
2. `_strategy/UNIVERSAL_PROMPT.md` — portfolio-wide directives
3. This file — config-lib specific directives
4. `.dev/PROMPT.md` — project context
5. `.dev/ROADMAP.md` — current phase and tasks

REPS overrides everything else.

---

## Cross-platform first-class

`config-lib` MUST work identically on Linux, macOS, and Windows:

- File-watching uses `notify` (which handles all three platforms internally)
- Path handling uses `std::path::Path` and `PathBuf` — never raw strings
- File reads use `std::fs::read_to_string` or async equivalents — no raw FFI
- Line ending handling: parsers MUST accept `\n`, `\r\n`, and `\r` line endings
- Default file open mode is portable text mode

Platform-specific behavior MUST be documented in `docs/PLATFORM-NOTES.md` with the rationale.

---

## REPS compliance (non-negotiable)

`src/lib.rs` MUST contain (as a minimum, expanded from current state):

```rust
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
#![deny(unused_results)]
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
#![allow(clippy::module_name_repetitions)]
```

Any `unwrap` / `expect` in code requires:

- A `// REPS-AUDIT:` comment explaining why it's safe
- A test exercising the path
- Approval from the maintainer

This includes test code in `#[cfg(test)]` modules — REPS applies there too, with rare allowances for assertions like `Result::unwrap()` after explicit `is_ok()` check.

---

## Performance discipline

Performance claims in `README.md` and rustdoc MUST be backed by committed benchmarks. The portfolio standard is:

1. If you claim a number, the benchmark exists in `benches/`
2. The baseline number is committed to `benches/baselines.json` (or equivalent tracking)
3. CI regression detection runs on PR (or local manual run for solo work)

Hot paths in `config-lib`:

- `Config::get(key)` — cached value lookup. Target: <50ns under contention.
- `Config::set(key, value)` — cached value write. Target: <500ns.
- `Config::from_string(s, format)` — cold parse. Target: depends on file size; baseline established per format.
- `Config::reload_if_changed()` — change detection. Target: <100µs check overhead.
- `Value::as_string()` / `as_integer()` / etc. — typed accessor. Target: <10ns.

Optimization rules:

- **Profile before optimizing.** No "this should be faster" without `perf` or `flamegraph` evidence.
- **Benchmark every change to a hot path.** No exceptions.
- **Zero-allocation on hot paths.** `Config::get` must not allocate. `Value::as_*` must not allocate.
- **Use `Cow<'_, str>` for return types** when borrowing is sometimes possible.
- **Inline hot accessors explicitly.** `#[inline]` on small functions; `#[inline(always)]` only with measurement.

---

## Concurrency

`config-lib` is used in multi-threaded servers (Hive DB has many concurrent connections, each potentially reading config). Concurrency requirements:

- **All public types `Send + Sync`.** Document where not.
- **Read-heavy workload optimized.** 99%+ of operations will be reads.
- **No reader serialization.** Multiple threads reading must not block each other.
- **Writer-side latency may degrade gracefully** under high write contention. Writes are rare.

Implementation guidance:

- Use `DashMap` for the cache (sharded, lock-free reads under contention)
- Use `ArcSwap<Arc<Value>>` for whole-config swap on hot reload (atomic pointer swap)
- Use atomic counters (`AtomicU64::fetch_add` with `Ordering::Relaxed`) for statistics
- Use `Arc<str>` instead of `String` for keys to reduce clones on cache hits

---

## Error handling

- Domain-specific `Error` enum in `error.rs` (already exists)
- `thiserror` for the error type derivation
- `Result<T>` is `Result<T, Error>` (already defined in lib.rs)
- Every public function documents its error conditions in rustdoc `# Errors` section
- No `Box<dyn Error>` in public API
- No silent error swallowing (`let _ = ...` requires comment)
- Error messages are actionable — include the relevant context (file path, key name, format, etc.)

---

## API stability discipline

Pre-1.0 stability:

- `0.9.x` line: feature-frozen, only bug fixes and audit findings
- API additions allowed; API breaks documented in CHANGELOG with migration guide
- `EnterpriseConfig` type is being deprecated in favor of unified `Config` — `#[deprecated]` alias maintained through `0.9.x` and into `1.x`

Post-1.0:

- Public API frozen for the lifetime of `1.x`
- New items may be added in MINOR releases
- `#[non_exhaustive]` on enums likely to grow (Error variants, ConfigChangeEvent)
- Breaking changes require `2.0` MAJOR bump

---

## Documentation discipline

Every public item MUST have:

- A one-line summary
- A longer description if behavior is non-obvious
- A `# Examples` section with runnable code
- A `# Errors` section if the function returns `Result`
- A `# Panics` section if the function can panic (rare; library code shouldn't panic)
- A `# Safety` section for any `unsafe` function

The README MUST stay current with the public API. The CHANGELOG MUST be updated under `[Unreleased]` for every change in the same commit.

---

## Testing discipline

Each parser MUST have:

- Unit tests for happy paths
- Unit tests for malformed input (empty file, truncated, invalid syntax)
- Unit tests for edge cases (very large file, deeply nested structures, unicode in keys/values)
- Integration test in `tests/` for the full parse → access → modify → save round-trip
- Fuzz target in `fuzz/fuzz_targets/` for adversarial input

Caching MUST have:

- Unit tests for cache hits and misses
- Unit tests for cache eviction behavior
- Concurrency tests for race-free read/write under multiple threads

Hot reload MUST have:

- Integration tests for file-modified detection
- Integration tests for file-renamed (atomic write) handling
- Integration tests for file-deleted handling
- Cross-platform CI verification

---

## Format support policy

The 8 supported formats are:

| Format | Source | Default? | Notes |
|--------|--------|----------|-------|
| CONF | Built-in parser | Yes | Custom format — see `docs/FORMATS.md` |
| INI | Built-in parser | Yes (1.0 target) | Standard INI with sections and comments |
| Properties | Built-in parser | Yes (1.0 target) | Java .properties format |
| JSON | Built-in parser | Yes (1.0 target) | Standard JSON with edit capability |
| XML | `quick-xml` crate | Yes (1.0 target) | Java/.NET interop |
| HCL | Built-in parser | Yes (1.0 target) | Basic HashiCorp Configuration Language |
| NOML | `noml` crate (external) | **No (1.0 opt-in)** | Format preservation; pre-1.0 dependency |
| TOML | `noml` crate (external) | **No (1.0 opt-in)** | Routed via NOML for format preservation |

NOML and TOML being opt-in is a 1.0 stability requirement (see audit). They remain available, but not in the default feature set.

---

## Dependencies

Approved current dependencies:

- `thiserror = "1.0"` — error derivation
- `serde = "1.0"` (with derive) — serialization where needed

Optional dependencies (feature-gated):

- `tokio` (feature: `async`) — async file I/O
- `chrono` (feature: `chrono`) — DateTime support
- `serde_json` (feature: `json`) — JSON parsing
- `regex` (feature: `validation`) — pattern validation
- `quick-xml` (feature: `xml`) — XML parsing
- `noml = "=0.9.x"` (feature: `noml` and `toml`) — NOML/TOML support
- `notify = "6"` (feature: `hot-reload`) — file system events (to be added in 0.9.6)

New dependencies require:

- A justified reason in the PR description
- A `cargo audit` clean run with the new dep
- License compatibility check (Apache-2.0, MIT, or compatible OSS license)
- MSRV check (the new dep MUST support Rust 1.75)
- Documentation of why it can't be implemented in-house

---

## Out of scope (always)

Things NOT in this crate's responsibility:

- Configuration **discovery** (where is the config file located?) — leave to caller
- Configuration **distribution** (etcd, consul, Vault, ZooKeeper) — separate crate if needed
- Configuration **schemas registry** (centralized schema management) — separate concern
- **Encryption-at-rest** for sensitive values — separate concern
- **Secret management** — separate concern (HashiCorp Vault clients exist)
- **Web UI** for editing config — separate concern

If a user asks for one of these, point them to a different crate or recommend they build the integration in their application layer.

---

## When you must break a directive

If a directive in this file genuinely needs an exception:

1. STOP. Don't break it silently.
2. Document why in the PR description.
3. Get explicit maintainer approval.
4. Add a `// CONFIG-LIB-EXCEPTION:` comment at the violation point with the rationale.
5. Update this file or `.dev/PROMPT.md` if the exception reveals a flaw in the directive.

---

<sub>config-lib directives — Copyright &copy; 2026 James Gober. Apache-2.0 OR MIT (post-1.0 dual licensing).</sub>
