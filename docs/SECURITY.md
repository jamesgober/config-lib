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
  pre-read with a chroot or sandbox of their choice. Callers that
  pre-read should use `Config::from_string_strict`, so a NOML file
  cannot read a second file or the environment; see below.)
- **TOCTOU on hot-reloaded files.** The `notify` watcher detects
  *that* the file changed, not *what changed inside it*. A
  malicious peer with write access to the watched file can
  obviously change its contents.

---

## Reading only the given text (v1.3.0)

A caller that checks a configuration file before reading it (owner,
mode, size, refusing a FIFO) needs the parser to read nothing else.
The noml crate's resolver honours `include "path"`, which opens any
file relative to the working directory with none of those checks (a
FIFO blocks the reader), and `env("NAME")`, which copies any
environment variable into a value.

- **TOML never resolves NOML syntax.** The `toml` feature reads TOML
  through the noml crate. Since v1.3.0 the syntax tree is checked
  before it is resolved, and `include`, `env()` and any other function
  call, `@type(...)` values and bare `${...}` are refused with
  `Error::Parse` and the line and column of the first one. This
  applies to every TOML entry point and cannot be turned off; none of
  these is TOML. In 1.2.0 and earlier, a `.toml` file could include
  other files and read the environment.
- **Strict NOML is opt-in.** `Config::from_string_strict(text, format)`
  and `config_lib::parse_strict(text, format)` take the same arguments
  as `from_string` and `parse`. For NOML they refuse `include` and every
  function call, `env()` included, the same way; native types
  (`@size("10MB")`) and `${...}` references inside the document still
  work. For every other format they are the same as `from_string` and
  `parse`. Default NOML still resolves `include` and `env()`; since
  v1.4.0 includes are confined (see below), and files and hot reload
  have strict entry points too (`Config::from_file_strict`,
  `HotReloadConfig::from_file_strict`, or `LoadOptions::strict`).
- **The refusal does not quote the input.** The message names only
  the kind of node (`env() is not allowed in strict NOML`), not the
  path or variable name.

Cost: one pass over the syntax tree before resolving, with an
explicit stack. On a 4 KiB NOML document it measured about 12 µs on
top of a 340 µs parse (release build, one desktop machine; an
estimate for other hardware).

## Limits on every load (v1.4.0)

Up to 1.3.2 the NOML paths did not hold to the threat model above.
v1.4.0 fixes them; every load applies
[`LoadOptions`](https://docs.rs/config-lib/latest/config_lib/struct.LoadOptions.html),
whose defaults are listed here.

- **`${...}` expansion is bounded.** Each reference copies the value it
  names, so ten lines of `s1 = "${s0}${s0}..."` grew 424 bytes into
  100 MB, in strict mode too. config-lib now measures what every
  reference would copy before the noml resolver runs, and refuses the
  document with `Error::Parse` when the expanded size would pass 64 MiB,
  when it has more than 10,000 references, or when a chain of references
  is more than 32 deep. The limits apply in strict and default NOML and
  can be changed with `LoadOptions::max_expanded_size`,
  `max_expansions` and `max_expansion_depth`. The fix is in config-lib,
  in front of the noml crate (pinned at 0.9.2): the noml resolver itself
  has no limit.
- **Includes are confined.** config-lib reads included files itself and
  hands the noml resolver a tree with no `include` left, so noml never
  opens a file. An included file must be a regular file of at most
  16 MiB inside the directory of the file being loaded (the current
  directory for text), checked on the canonical path, so `..` and
  symbolic links cannot leave it. `LoadOptions::allow_include_dir` adds
  directories and `allow_includes_anywhere` turns the check off. At most
  64 files are included per load, nested at most 10 deep, and a file
  that includes itself is refused. Remote (`http://`) includes are
  refused.
- **Files are checked before they are read.** Every file load refuses
  anything that is not a regular file (a FIFO, a device, a directory)
  and anything over 16 MiB (`LoadOptions::max_file_size`), counted again
  while reading, so a file that grows in between is still refused.
- **Errors never quote the input.** TOML, NOML, INI, Properties and XML
  syntax errors are `Error::Parse` with a line, a column and fixed text.
  Before, TOML and NOML quoted the offending token (a password with a
  missing quote appeared in the error), INI quoted the whole line, and a
  file included by mistake had its first token quoted. Hot-reload
  `ReloadFailed` events carry the same text.
- **No format is guessed from content but XML, JSON and INI.**
  `parse(text, None)` and files with no extension used substring
  heuristics, so a comment mentioning `include` sent CONF text to the
  NOML parser, which resolves includes. Text is now read as CONF unless
  it starts like XML or JSON or has a `;` comment line (INI).

Remaining: the noml crate's parser is not ours and is fuzzed only
through the `noml_parser` and `toml_parser` targets. Hot reload checks
that a file did not change while it was read and refuses an empty file
in place of a non-empty configuration, but a writer that pauses in the
middle of an in-place rewrite can still leave a prefix that parses;
write a temporary file and rename it over the target
(`Config::save_to_file` does).

### Text positions in CONF, INI and Properties

Before v1.3.0 these three parsers counted their position in
characters and sliced the input with it as a byte offset. Non-ASCII
text could panic the CONF parser (`#ééé` before a section header),
made INI loop forever, and made valid files fail to parse; looking up
each character from the start made every parse quadratic (about
250 ms for 40 KB of CONF). They now keep a byte offset on a character
boundary and slice with checked `str::get`, so any `&str` parses or
returns an error without a panic, in linear time: 1 MiB of CONF
parses in about 24 ms (release) and well under a second in a debug
build (tested). Line and column numbers count characters from 1, as
before.

Also fixed in v1.3.0, found while testing these parsers:

- CONF arrays nest at most 128 levels. Arrays are parsed recursively,
  and a long run of `[` used to overflow the stack and abort the
  process.
- An INI line starting with `=` or `:` looped forever; it is now a
  parse error.

Fixed in v1.3.1:

- An INI value with an unclosed quote, or with an apostrophe inside
  text (`O'Brien`), read the rest of the file, across lines, into one
  value. A quote opens a value only as its first character, and an
  unclosed one is a parse error that names the line and does not quote
  the value.
- A CONF section named like a key above it lost all its keys without
  an error; it is a parse error now.

---

## Strict JSON (feature `jsonx`)

With the `jsonx` feature (since v1.2.0), JSON is read and written by
[`jsonx-lib`](https://docs.rs/jsonx-lib), and it fails closed:

- **Duplicate keys are refused.** `{"role": "user", "role": "admin"}`
  is a parse error. Readers that disagree about repeated keys are a
  known way to show a validator one value and the program another;
  refusing the document removes the question. This is the one
  behaviour difference from the `json` feature (serde_json), which
  keeps the last value. Keys are compared after unescaping, so
  `"ab"` and `"ab"` are the same key.
- **Budgets, checked before allocation.** jsonx-lib's defaults:
  nesting depth 128, input 64 MiB, one string or key 16 MiB, one
  number 1 KiB, 1 Mi values, and 64 MiB of memory for the parsed tree.
  A document over any of them is refused before the memory it would
  take is allocated.
- **No recursion on nesting depth.** jsonx-lib parses with an explicit
  stack, and config-lib converts its tree and writes JSON the same
  way, so a million levels of `[` is a parse error on a 256 KiB stack
  (tested), not a stack overflow.
- **Errors do not quote the input.** A parse error is `Error::Parse`
  with the line and column (from 1) of the mistake. The message names
  the kind of mistake and its byte offset and contains no text from
  the document, because configuration values can be secrets.
- **Strict grammar.** RFC 8259 only: no comments, no trailing commas,
  no byte-order mark, no lone surrogate escapes, no numbers outside
  the grammar or too large for `f64`.

The `jsonx` writer escapes U+2028 and U+2029 inside strings, which
JavaScript before ES2019 treats as line ends; serde_json writes them
raw. Both read back as the same string.

Cost: duplicate keys are found when each object closes, with a
64-bit filter for objects of up to 16 keys and a sort past that
(`O(n log n)` in the object's key count); jsonx-lib's own benchmarks
put `parse` at about 220 MiB/s on a 1 MiB document. For configuration
files of a few KiB that is well under a millisecond (an estimate, not
a config-lib measurement).

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
| `noml_parser`                           | `crate::parse_strict` and `crate::parse`, NOML (v1.4.0) |
| `toml_parser`                           | `crate::parse(content, Some("toml"))` (v1.4.0) |

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
goal. Continuous extended fuzzing (via
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
upstream `noml` crate, itself pinned to `=0.9.2` exactly to prevent
silent transitive bumps), or the `jsonx` feature (`jsonx-lib`,
pinned to `=0.5.1` exactly for the same reason; it has no
dependencies of its own and is `#![forbid(unsafe_code)]`).

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

<sub>Last reviewed: 2026-11-07 (v1.3.0).</sub>
