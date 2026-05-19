# Security

This document describes the security posture of `config-lib` —
what we promise, how we verify it, and how to report a finding.

If you've found a security issue, please follow the **Reporting**
section at the bottom rather than opening a public GitHub issue.

---

## Threat model

`config-lib` parses **untrusted text** at runtime: configuration
files chosen by the operator, configuration strings from a remote
source, environment variable values, sometimes data assembled from
templates. A configuration library that crashes the host process
on a malformed input is an availability bug. A configuration
library that loops forever on a crafted input is a denial-of-
service vector. A configuration library that allocates unbounded
memory on adversarial nesting is a denial-of-service vector.

Any of these on untrusted input is a security finding in this
crate's context, even if it does not violate Rust's memory safety
guarantees.

### What we defend against

- **Panics on malformed input.** Every parser entry point returns
  `Result<Value, Error>`; producing an `Err` for unparseable input
  is correct, but producing a panic is a bug.
- **Infinite loops on crafted input.** Every parser must converge
  (return `Ok` or `Err`) in a bounded number of steps relative to
  input length.
- **Unbounded memory allocation on crafted input.** Adversarial
  nesting (e.g. 1 MiB of `{{{{...}}}}` in JSON) must either error
  cleanly or use bounded memory.

### What we do NOT defend against

- **Operator-supplied configuration values being inherently
  dangerous.** If your config sets `dangerous_mode = true` and your
  program acts on that, that is a deployment policy question, not
  a `config-lib` security issue.
- **Filesystem access by the operator.** `Config::from_file(path)`
  reads the file at `path`. If the operator controls the path,
  that's a path-traversal question at the caller, not in this
  crate. (We expose `Config::from_string` for callers that want to
  pre-read with a chroot or sandbox of their choice.)
- **TOCTOU on hot-reloaded files.** The `notify` watcher detects
  *that* the file changed, not *what changed inside it*. A
  malicious peer with write access to the watched file can
  obviously change its contents.

---

## Lint-enforced defenses

The REPS lint configuration (`src/lib.rs`, hardened in v0.9.3)
denies the following clippy lints crate-wide on shipping code:

| Lint                       | Rationale                                                                        |
|----------------------------|----------------------------------------------------------------------------------|
| `clippy::unwrap_used`      | Untrusted input must never reach `.unwrap()`.                                    |
| `clippy::expect_used`      | Same, with a fancier message.                                                    |
| `clippy::todo`             | `todo!()` panics. Production code cannot panic on untrusted input.               |
| `clippy::unimplemented`    | Same.                                                                            |
| `clippy::print_stdout`     | Leaked diagnostic output is a side channel; allowed only at audited sites.       |
| `clippy::print_stderr`     | Same; allowed only at audited sites (`audit.rs` last-resort fallback).           |
| `clippy::dbg_macro`        | `dbg!()` leaks to stderr.                                                        |
| `clippy::undocumented_unsafe_blocks` | Every `unsafe {}` must carry a `// SAFETY:` comment.                 |
| `clippy::missing_safety_doc`         | Every `unsafe fn` must have a `# Safety` rustdoc section.            |

Test-module code carries narrower allowances under `#![cfg_attr(test,
allow(...))]` for ergonomic assertions — test code never reaches a
shipped binary.

---

## Fuzz testing

`config-lib` ships with `cargo-fuzz` harnesses for every parser entry
point. The harnesses live at the repo root under `fuzz/` as a separate
cargo workspace (they require nightly toolchain because libFuzzer
ships out-of-tree).

### Targets

| Target                                  | Parser entry point                          |
|-----------------------------------------|---------------------------------------------|
| `conf_parser`                           | `parsers::conf::parse`                      |
| `ini_parser`                            | `parsers::ini_parser::parse`                |
| `properties_parser`                     | `parsers::properties_parser::parse`         |
| `json_parser`                           | `parsers::json_parser::parse`               |
| `xml_parser`                            | `parsers::xml_parser::parse`                |
| `hcl_parser`                            | `parsers::hcl_parser::parse`                |
| `format_detection`                      | `crate::parse(content, None)` (auto-detect) |

Each target accepts a `&[u8]` from the fuzzer, transcodes to `&str`
(non-UTF-8 inputs are valid `&[u8]` but not valid `&str` and the
caller is responsible for the conversion, so they're skipped here),
and calls the corresponding parser. The fuzzer wins by producing
input that panics, infinitely loops, or causes the process to be
OOM-killed.

### Running

```bash
# One-time setup
rustup install nightly
cargo install cargo-fuzz

# From the repo root:
cd fuzz
cargo +nightly fuzz run conf_parser -- -max_total_time=3600
cargo +nightly fuzz run ini_parser -- -max_total_time=3600
cargo +nightly fuzz run properties_parser -- -max_total_time=3600
cargo +nightly fuzz run json_parser -- -max_total_time=3600
cargo +nightly fuzz run xml_parser -- -max_total_time=3600
cargo +nightly fuzz run hcl_parser -- -max_total_time=3600
cargo +nightly fuzz run format_detection -- -max_total_time=3600
```

Each invocation runs that target for one CPU-hour and stops. The
fuzzer's working corpus accumulates in `fuzz/corpus/<target>/`
(gitignored — the corpus is build artifact, not committed source);
any findings land in `fuzz/artifacts/<target>/` with a reproducer
filename.

### Pre-release contract

Before a `0.9.x` → `1.0.0` cut, every fuzz target must have a
documented 1-CPU-hour clean run on the maintainer's reference
hardware. The exit criterion: zero panics, zero hangs, zero OOMs.
A clean run is recorded in the release notes for whichever version
ships the gate.

### Continuous fuzzing

Phase 0.9.8 introduced the harness infrastructure. A short CI fuzz
pass (10 CPU-minutes per target) on every PR is a v0.9.9 polish
goal — see `.dev/ROADMAP.md`. Continuous extended fuzzing (via
e.g. `oss-fuzz` or a dedicated workflow on a long-running runner)
is a post-1.0 ambition.

### Triage workflow

When `cargo fuzz run` reports a finding:

1. The fuzzer dumps the offending input to
   `fuzz/artifacts/<target>/crash-<hash>`.
2. Reproduce locally:
   ```bash
   cargo +nightly fuzz run <target> fuzz/artifacts/<target>/crash-<hash>
   ```
3. Classify:
   - **Panic** — fix the parser to return `Err` cleanly.
   - **Hang** — add an iteration cap with a clear error.
   - **OOM** — add an input-size or nesting-depth limit with a
     clear error.
4. Promote the reproducer into a regression test under
   `tests/parser_corpus.rs` so the input is exercised on every
   `cargo test` from then on. (The regression-test file is
   populated lazily — empty until the first finding lands.)
5. If the finding is a confidentiality or integrity issue (not just
   availability), follow the **Reporting** section instead of
   committing the reproducer to the public corpus.

---

## Dependency hygiene

Every release runs `cargo audit` and `cargo deny check` and must
exit clean on both:

- `cargo audit` checks the Cargo.lock against the [`RustSec`
  advisory database](https://rustsec.org/).
- `cargo deny check` checks license compatibility (allow-list in
  `deny.toml`), advisories, license sources, and dependency bans.

The default feature set has **zero pre-1.0 dependencies** as of
v0.9.7. Pre-1.0 transitive deps reach the build only when the user
opts into the `noml` or `toml` features (which depend on the
upstream `noml` crate, itself pinned to `=0.9.0` exactly to prevent
silent transitive bumps).

---

## Unsafe code

The crate's shipping `src/` directory contains **zero `unsafe`
blocks**. The REPS lint configuration (`#![deny(unsafe_op_in_unsafe_fn)]`)
catches accidental introduction. The audit posture matches
`#![forbid(unsafe_code)]` in spirit, without explicitly forbidding
(so a deliberate future `unsafe` block — with a `// SAFETY:`
comment and Miri exercise — would be possible).

Transitive dependencies do contain `unsafe` code, of course
(`notify`'s kernel-API bindings, `serde_json`'s SIMD acceleration,
`quick-xml`'s zero-copy slicing). Those crates carry their own
audits; we do not re-audit them, but we do track them via
`cargo audit`.

---

## Reporting a finding

Please report security issues by email to
[`security@hivedb.com`](mailto:security@hivedb.com) rather than
opening a public GitHub issue. Include:

- The fuzz target name (if applicable) or a minimal reproducer
- The output the parser produced (panic message, stack trace, etc.)
- The version of `config-lib` and the feature flags in use
- Your contact information for follow-up

We aim to acknowledge reports within two business days and to ship
a fix within thirty days for confirmed findings. Credit is offered
in the release notes for the fix, unless you prefer to remain
anonymous.

---

<sub>Last reviewed: 2026-05-19 (v0.9.8).</sub>
