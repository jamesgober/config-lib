# Performance

This document describes `config-lib`'s performance posture: what
targets the v1.0 stability contract commits to, how to verify
those targets locally, and the methodology behind the committed
baseline numbers.

For the **architectural** decisions behind the performance work
(why DashMap, why parent-directory watching, why `Arc<Value>`),
see [`docs/ARCHITECTURE.md`](ARCHITECTURE.md).

---

## Performance contract (v1.0 stability surface)

| Operation                                       | Target  | Verified by                                |
|-------------------------------------------------|---------|--------------------------------------------|
| Single-key cached `get_arc` (1 thread, warm)    | <50 ns  | `benches/cache_warm.rs::get_arc_warm_simple` |
| Single-key cached `get_arc` (16 threads, warm)  | <50 ns  | `benches/cache_concurrent.rs::concurrent_get_arc/16` |
| Single-key cached `get_arc` (cold miss)         | <5 µs   | `benches/cache_warm.rs::get_arc_cold` (forthcoming) |
| Nested-key cached `get_arc` (3 levels deep)     | <100 ns | `benches/cache_warm.rs::get_arc_warm_nested` |
| Borrowed `get` (1 thread, no cache)             | <50 ns  | `benches/cache_warm.rs::get_borrowed_warm_simple` |
| Typed accessor (`as_string` / `as_integer` etc.) | <10 ns  | `benches/value_accessors.rs`               |
| `Config::set` cached write (incl. invalidation) | <500 ns | `benches/cache_warm.rs::set_cached`        |
| Hot reload detection latency                    | <100 ms | `tests/hot_reload_modified.rs`             |
| `on_change` dispatch — empty handler list       | <10 ns  | `benches/notification.rs::dispatch_n_handlers/0` |
| `on_change` dispatch — single handler           | <50 ns  | `benches/notification.rs::dispatch_n_handlers/1` |
| `on_change` dispatch — ten handlers             | <500 ns | `benches/notification.rs::dispatch_n_handlers/10` |
| `on_change` dispatch — hundred handlers         | <5 µs   | `benches/notification.rs::dispatch_n_handlers/100` |
| Handler registration cost (incl. RCU update)    | <1 µs   | `benches/notification.rs::register_handler` |
| Cold parse — small CONF (~200 B)                | <1 µs   | `benches/parse_throughput.rs::parse_small_conf` |
| Cold parse — 1 KiB CONF                         | <10 µs  | `benches/parse_throughput.rs::parse_1kib_conf` |
| Cold parse — 100 KiB JSON                       | <500 µs | `benches/parse_throughput.rs::parse_100kib_json` |
| Memory — empty `Config`                         | <1 KiB  | `dhat` measurement (manual)                |
| Memory — `Config` with 1000 cached keys         | <128 KiB| `dhat` measurement (manual)                |

These targets are the **stability contract** — the v1.0 line
commits to them and any release that breaks one is a release that
needs a fix. The measured values (committed to
`benches/baselines.json` after the maintainer's reference-hardware
runs) may improve over time but the targets are the floor.

---

## Methodology

### Hardware

Performance numbers cited in this document and in the release notes
come from the maintainer's reference hardware. Numbers from other
hardware (developer laptops, CI ephemeral runners, cloud-VM
instances) are useful for trend detection but are **not** the
canonical figures. The 1.0 release notes will record the exact
hardware spec.

A typical developer laptop benchmark run reproduces the *shape* of
the canonical numbers (relative ordering, scaling-with-threads
trend) but absolute values will differ by 2-3× either direction
depending on CPU, memory, and thermal state.

### Tooling

- **`criterion = "0.5"`** — the de-facto Rust benchmarking crate.
  Each `benches/*.rs` file is a separate criterion harness. Run
  with `cargo bench --bench <name>`.
- **`dhat`** — for memory profiling. Configured per-benchmark
  on demand; not part of the default `cargo bench`.

### Process

1. Build with `--release` (criterion does this by default).
2. Close other processes; pin to a single core if measurement
   variance dominates ("`taskset -c 0 cargo bench ...`" on Linux,
   PowerShell `Start-Process -ProcessorAffinity 1 ...` on Windows).
3. Let criterion run to its default sample count (100 samples,
   ~10 seconds per benchmark). Override with
   `--measurement-time 30` for tighter intervals on noisy systems.
4. Compare against the previous run via criterion's built-in
   regression detection: `cargo bench -- --baseline <name>`.

### Interpreting the numbers

Criterion reports a confidence interval for each measurement, not a
single number. A successful run looks like:

```text
get_arc_warm_simple   time:   [29.6 ns 30.1 ns 30.7 ns]
                      change: [-0.3% -0.1% +0.4%] (p = 0.74 > 0.05)
                      No change in performance detected.
```

The performance contract cares about the **upper bound** of the
confidence interval, not the median. A 50 ns target is met when the
upper bound is below 50 ns.

---

## How the benchmarks are organised

The four bench files target the four classes of operation:

### `benches/cache_warm.rs`

Single-threaded warm-cache reads. The `get_arc_warm_simple` and
`get_arc_warm_nested` benchmarks pre-populate the cache and measure
a tight loop. The `get_borrowed_warm_simple` benchmark provides a
borrowed-return comparison (no cache, no `Arc::clone` — useful for
deciding which accessor to use in a given context). The `set_cached`
benchmark measures a `set()` call including the wholesale cache
invalidation cost.

### `benches/cache_concurrent.rs`

Multi-threaded throughput. The `concurrent_get_arc` parametric
benchmark spins {1, 2, 4, 8, 16} threads hammering the same warm
key. Reports total throughput per iteration; criterion
parametrises so each thread count gets its own measurement.

The 16-thread number is the headline figure for the "scales to 16
threads" claim. If the per-thread time at 16 threads is materially
worse than at 1 thread (more than ~2× slower), the cache backend
choice needs revisiting before v1.0.

### `benches/value_accessors.rs`

Typed accessor cost in isolation. `Value::as_integer`,
`Value::as_string`, `Value::as_bool`, `Value::as_float` are each
called in a tight loop on a pre-constructed `Value`. The
single-digit-nanosecond target is for the accessor itself, not the
preceding `Config::get` call.

### `benches/notification.rs`

Lock-free notification dispatch (v1.0.0+). The
`dispatch_n_handlers` parametric benchmark exercises
`HandlerList::dispatch` for `n ∈ {0, 1, 10, 100}` registered
handlers. The empty-list case isolates the irreducible per-event
overhead (one `ArcSwap::load` + zero-length iteration). The non-
empty cases add `n × (Arc::clone + catch_unwind + closure call)`
where each closure increments an `AtomicUsize` — the smallest
meaningful body.

The `register_handler` benchmark measures `HandlerList::register`
+ `Subscription::drop` round trip — useful as an upper bound on
"how expensive is it to come and go from the handler list", e.g.
in short-lived RAII guards.

These numbers are the v1.0.0 headline perf wins: where v0.9.x
notified subscribers via `mpsc::Sender::send` (queue node
allocation + atomic CAS + cross-thread wake = ~150–250 ns per
send), v1.0.0 dispatches inline for ~5 ns + per-handler cost.
The savings compound with subscriber count: ten handlers via
mpsc fan-out costs ~1.6 µs; ten handlers via `on_change` costs
~50–100 ns plus closure work.

### `benches/parse_throughput.rs`

Cold-parse time-to-`Config`. Three benchmarks: a small (~200 B)
CONF from `tests/fixtures/test.ini`, a synthesised 1 KiB CONF
(twenty sections × ~50 B each), and a synthesised 100 KiB JSON
(thousand entries × ~100 B each — gated on the `json` feature).

These numbers anchor "how long does it take to load a config at
startup?" — the canonical claim being that even a 100 KiB JSON
file parses in well under a millisecond.

---

## Baselines

`benches/baselines.json` (when committed) holds the canonical numbers
for each benchmark on the maintainer's reference hardware. The
file's schema:

```json
{
  "hardware": "...",
  "rustc": "...",
  "date": "YYYY-MM-DD",
  "measurements": {
    "<benchmark_name>": {
      "median_ns": ...,
      "ci_lower_ns": ...,
      "ci_upper_ns": ...
    }
  }
}
```

A CI regression check (forthcoming as a Phase 1.x deliverable)
compares each PR's run against this baseline. A measurement that
exceeds the previous baseline by more than the contract target
fails the gate.

**At v0.9.9 the baselines.json file is not yet committed** — the
maintainer's reference-hardware runs land alongside the v1.0.0 cut.
v0.9.9 publishes the harness; v1.0.0 publishes the verified numbers
that the stability contract cites.

---

## Tuning guidance for users

If `config-lib` is showing up hot in your profile, in order:

### Use `get_arc` instead of `get` when you'd otherwise re-resolve

```rust
// Anti-pattern: same key resolved on every iteration.
for _ in 0..1_000_000 {
    let port = config.get("server.port").and_then(|v| v.as_integer().ok());
    do_work(port);
}

// Better: get the Arc once outside the loop.
let port = config
    .get_arc("server.port")
    .and_then(|v| v.as_integer().ok());
for _ in 0..1_000_000 {
    do_work(port);
}
```

Both patterns are fast, but the first allocates work for the parser
tree-walk on every iteration; the second reuses the cached value.

### Disable the cache when configs are write-heavy

The cache is invalidated wholesale on every `set` / `remove` /
`merge`. If your workload writes more often than it reads, the
cache is pure overhead — disable it:

```rust
use config_lib::{Config, ConfigOptions};

let opts = ConfigOptions::new().cache_enabled(false);
let cfg = Config::with_options(opts);
```

`get_arc` still works in this mode; it just walks the tree and
allocates a fresh `Arc<Value>` each call.

### Avoid re-parsing on every hot-reload event

The `notify`-backed watcher already debounces (default 100 ms). If
your editor save pattern produces wider bursts, raise the
debounce:

```rust
let cfg = HotReloadConfig::from_file("app.conf")?
    .with_debounce(Duration::from_millis(500));
```

### Reach for `dhat` if memory is the bottleneck

```rust
let _profiler = dhat::Profiler::new_heap();
let cfg = Config::from_file("big-config.json")?;
// ...
drop(_profiler); // dump heap snapshot
```

The dhat output identifies which `Value` variants and which cache
entries are dominating the footprint.

---

## What the contract does NOT promise

- **Exact ns numbers in the README.** The README cites typical
  values for user-facing context, not stability commitments. The
  *targets* in §1 are what the v1.0 line commits to.
- **Identical numbers across architectures.** ARM64, x86_64,
  RISC-V, and POWER all have different memory subsystem
  characteristics. The targets are met on commodity x86_64 server
  hardware; other architectures may meet or beat them but are not
  individually verified.
- **Identical numbers across allocators.** Numbers cited use the
  system default allocator. `jemalloc` / `mimalloc` typically
  improve allocation-heavy benchmarks (`parse_*`) by 10-30%.
- **No regression between MINOR releases.** A patch release that
  trades a small constant slowdown for a correctness fix is
  expected and shippable. The performance contract is the floor,
  not the historical best.

---

<sub>Last reviewed: 2026-05-19 (v0.9.9).</sub>
