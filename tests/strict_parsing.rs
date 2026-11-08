//! Tests for the v1.3.0 parsing fixes:
//!
//! - the CONF, INI and Properties parsers work on byte offsets, so any
//!   `&str` parses without a panic, in linear time, with line and column
//!   numbers in characters;
//! - TOML refuses NOML's `include`, `env()` and other NOML-only syntax;
//! - strict NOML (`parse_strict`, `Config::from_string_strict`) refuses
//!   `include` and function calls, and default NOML still resolves them.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[cfg(any(feature = "toml", feature = "noml"))]
use config_lib::Error;
use config_lib::{Config, Value};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Byte offsets in the CONF, INI and Properties parsers
// ---------------------------------------------------------------------------

#[test]
fn conf_non_ascii_comment_before_section() {
    // Panicked in 1.2.0: the parser sliced at a character count
    let config = Config::from_string("#\u{e9}\u{e9}\u{e9}\n[a]\nk = 1\n", Some("conf")).unwrap();
    assert_eq!(config.get("a.k").unwrap().as_integer().unwrap(), 1);
}

#[test]
fn conf_non_ascii_round_trip() {
    let source = "# \u{4e2d}\u{6587} comment\n\
                  n\u{e4}me = J\u{fc}rgen\n\
                  greeting = \"gr\u{fc}\u{df} dich\"\n\
                  [se\u{e7}\u{e3}o]\n\
                  \u{43a}\u{43b}\u{44e}\u{447} = \u{3b1}\u{3b2}\u{3b3}\n\
                  list = \u{e9} \u{fc} \u{1F600}\n";
    let first = Config::from_string(source, Some("conf")).unwrap();
    assert_eq!(
        first.get("n\u{e4}me").unwrap().as_string().unwrap(),
        "J\u{fc}rgen"
    );
    assert_eq!(
        first.get("greeting").unwrap().as_string().unwrap(),
        "gr\u{fc}\u{df} dich"
    );
    let written = first.serialize().unwrap();
    let second = Config::from_string(&written, Some("conf")).unwrap();
    assert_eq!(first.as_value(), second.as_value());
}

#[test]
fn ini_and_properties_non_ascii() {
    // Both hung or refused in 1.2.0
    let ini = config_lib::parse("; caf\u{e9}\n[s]\nk = \u{e9}\n", Some("ini")).unwrap();
    let Value::Table(map) = ini else {
        panic!("expected a table");
    };
    assert_eq!(map.get("s.k").unwrap().as_string().unwrap(), "\u{e9}");

    let props = config_lib::parse("# \u{e9}\u{e9}\u{e9}\nk=v\u{e9}\n", Some("properties")).unwrap();
    assert_eq!(props.get("k").unwrap().as_string().unwrap(), "v\u{e9}");
}

/// A small deterministic generator (xorshift64), so the test needs no
/// extra dependency and fails the same way on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Characters that mean something to at least one parser, plus multi-byte
/// characters of every encoded length.
const ALPHABET: &[char] = &[
    '#',
    ';',
    '!',
    '[',
    ']',
    '{',
    '}',
    '(',
    ')',
    '=',
    ':',
    ',',
    '.',
    '"',
    '\'',
    '\\',
    '$',
    '@',
    '<',
    '>',
    '/',
    '-',
    '_',
    ' ',
    '\t',
    '\n',
    '\r',
    '\0',
    'a',
    'k',
    'e',
    'n',
    'v',
    'i',
    'c',
    'l',
    'u',
    'd',
    't',
    'r',
    '0',
    '1',
    '9',
    '\u{e9}',
    '\u{fc}',
    '\u{43a}',
    '\u{4e2d}',
    '\u{2028}',
    '\u{feff}',
    '\u{1F600}',
    '\u{10FFFF}',
];

/// Documents to cut up and mutate, so the generated input is close to
/// real syntax as well as random.
const SEEDS: &[&str] = &[
    "#\u{e9}\u{e9}\u{e9}\n[a]\nk = 1\n",
    "n\u{e4}me = \"J\u{fc}rgen\" # \u{2713}\n[se\u{e7}\u{e3}o]\nlist = [1, '\u{e9}', [2]]\n",
    "; \u{e9}\n[s]\nk=\u{e9}\nq=\"a\\\"b\"\n",
    "# \u{e9}\nk=v\\\n  \u{e9}\nu=\\u00e9\n",
    "a = { b = [1, 2] }\n[t]\nx = \"${a}\"\ny = env(\"X\", \"d\")\n",
    "<a><b>\u{e9}</b></a>",
    "{\"k\": [1, \"\u{e9}\"]}",
];

fn random_input(rng: &mut Rng) -> String {
    if rng.below(2) == 0 {
        // Random characters
        let len = rng.below(96);
        (0..len)
            .map(|_| ALPHABET[rng.below(ALPHABET.len())])
            .collect()
    } else {
        // A seed with characters replaced, inserted and removed
        let seed = SEEDS[rng.below(SEEDS.len())];
        let mut chars: Vec<char> = seed.chars().collect();
        for _ in 0..1 + rng.below(6) {
            let at = rng.below(chars.len() + 1);
            let c = ALPHABET[rng.below(ALPHABET.len())];
            match rng.below(3) {
                0 => chars.insert(at, c),
                1 if at < chars.len() => chars[at] = c,
                _ if at < chars.len() => {
                    chars.remove(at);
                }
                _ => chars.push(c),
            }
        }
        chars.into_iter().collect()
    }
}

/// Every format this build can parse
fn formats() -> Vec<&'static str> {
    let mut formats = vec!["conf", "ini", "properties"];
    if cfg!(any(feature = "json", feature = "jsonx")) {
        formats.push("json");
    }
    if cfg!(feature = "xml") {
        formats.push("xml");
    }
    if cfg!(feature = "hcl") {
        formats.push("hcl");
    }
    if cfg!(feature = "toml") {
        formats.push("toml");
    }
    if cfg!(feature = "noml") {
        formats.push("noml");
    }
    formats
}

#[test]
fn arbitrary_strings_never_panic() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let formats = formats();
    for _ in 0..3000 {
        let input = random_input(&mut rng);
        for &format in &formats {
            // Strict, so a generated `include` or `env()` in NOML never
            // touches the file system or the environment
            let outcome =
                std::panic::catch_unwind(|| config_lib::parse_strict(&input, Some(format)));
            assert!(outcome.is_ok(), "{format} panicked on {input:?}");
        }
        let detected = std::panic::catch_unwind(|| config_lib::parse_strict(&input, None));
        assert!(detected.is_ok(), "format detection panicked on {input:?}");
    }
}

#[test]
fn every_prefix_of_non_ascii_documents_parses_or_errs() {
    // Cuts at every character boundary, so a parse error lands next to a
    // multi-byte character
    for seed in SEEDS {
        for (cut, _) in seed.char_indices() {
            let prefix = &seed[..cut];
            for format in formats() {
                let outcome =
                    std::panic::catch_unwind(|| config_lib::parse_strict(prefix, Some(format)));
                assert!(outcome.is_ok(), "{format} panicked on {prefix:?}");
            }
        }
    }
}

/// A document of at least `size` bytes, half of its lines carrying
/// non-ASCII text, with a section every 1000 lines
fn big_document(size: usize, line: impl Fn(usize) -> String) -> (String, usize) {
    let mut text = String::with_capacity(size + 256);
    let mut lines = 0;
    while text.len() < size {
        if lines % 1000 == 0 {
            text.push_str(&format!("[se\u{e7}\u{e3}o_{lines}]\n"));
        }
        text.push_str(&line(lines));
        lines += 1;
    }
    (text, lines)
}

/// The budget for one size test in a debug build. The old parsers took
/// time quadratic in the input (about 250 ms for 40 KB, so minutes for
/// 1 MiB); the linear ones take well under 200 ms on a desktop.
const PARSE_BUDGET: Duration = Duration::from_secs(1);

#[test]
fn one_mib_conf_parses_quickly() {
    let (text, lines) = big_document(1 << 20, |i| {
        if i % 2 == 0 {
            format!("k\u{e9}y_{i} = v\u{e4}lue_{i} # caf\u{e9}\n")
        } else {
            format!("key_{i} = \"quoted {i}\"\n")
        }
    });
    let start = Instant::now();
    let value = config_lib::parse(&text, Some("conf")).unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < PARSE_BUDGET, "1 MiB of CONF took {elapsed:?}");
    let last = format!("se\u{e7}\u{e3}o_{}", (lines - 1) / 1000 * 1000);
    assert!(value.get(&last).is_some());
}

#[test]
fn ini_and_properties_parse_in_linear_time() {
    // 256 KiB: the quadratic parsers took several seconds here
    let (ini, _) = big_document(256 << 10, |i| {
        format!("k\u{e9}y_{i} = v\u{e4}lue_{i} ; \u{e9}\n")
    });
    let start = Instant::now();
    config_lib::parse(&ini, Some("ini")).unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < PARSE_BUDGET, "256 KiB of INI took {elapsed:?}");

    let mut props = String::new();
    let mut i = 0;
    while props.len() < 256 << 10 {
        props.push_str(&format!("# caf\u{e9}\nk\u{e9}y_{i}=v\u{e4}lue_{i}\n"));
        i += 1;
    }
    let start = Instant::now();
    config_lib::parse(&props, Some("properties")).unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed < PARSE_BUDGET,
        "256 KiB of Properties took {elapsed:?}"
    );
}

// ---------------------------------------------------------------------------
// TOML refuses NOML-only syntax
// ---------------------------------------------------------------------------

/// Check a parse error's position and that its message quotes none of
/// `secrets`
#[cfg(any(feature = "toml", feature = "noml"))]
fn assert_refused(
    result: config_lib::Result<impl std::fmt::Debug>,
    at: (usize, usize),
    secrets: &[&str],
) {
    match result {
        Err(Error::Parse {
            line,
            column,
            message,
            ..
        }) => {
            assert_eq!((line, column), at, "{message}");
            // The failure text names the index, never the value
            for (index, secret) in secrets.iter().enumerate() {
                assert!(
                    !message.contains(secret),
                    "the message quotes test value {index}"
                );
            }
        }
        other => panic!("expected Error::Parse at {at:?}, got {other:?}"),
    }
}

#[cfg(feature = "toml")]
mod toml {
    use super::*;

    #[test]
    fn include_is_refused_with_its_position() {
        // The included file exists, so a resolve would succeed
        let dir = tempfile::tempdir().unwrap();
        let other = dir.path().join("other.toml");
        std::fs::write(&other, "x = 1\n").unwrap();
        let path = other.display().to_string().replace('\\', "/");
        let source = format!("name = \"app\"\nshared = include \"{path}\"\n");
        assert_refused(
            config_lib::parse(&source, Some("toml")),
            (2, 10),
            &[&path, "other.toml"],
        );
        assert_refused(
            Config::from_string(&source, Some("toml")),
            (2, 10),
            &[&path],
        );
    }

    #[test]
    fn env_is_refused_with_its_position() {
        std::env::set_var("CONFIG_LIB_TOML_SECRET", "hunter2");
        let source = "[db]\nuser = \"app\"\npassword = env(\"CONFIG_LIB_TOML_SECRET\")\n";
        let secrets = ["CONFIG_LIB_TOML_SECRET", "hunter2"];
        assert_refused(config_lib::parse(source, Some("toml")), (3, 12), &secrets);
        assert_refused(
            config_lib::parse_strict(source, Some("toml")),
            (3, 12),
            &secrets,
        );
        // Inside an array and an inline table
        let nested = "list = [1, { p = env(\"CONFIG_LIB_TOML_SECRET\", \"d\") }]\n";
        assert_refused(config_lib::parse(nested, Some("toml")), (1, 18), &secrets);
    }

    #[test]
    fn other_noml_syntax_is_refused() {
        assert_refused(
            config_lib::parse("size = @size(\"10MB\")\n", Some("toml")),
            (1, 8),
            &[],
        );
        assert_refused(
            config_lib::parse("a = 1\nb = ${a}\n", Some("toml")),
            (2, 5),
            &[],
        );
    }

    #[test]
    fn toml_keys_named_include_and_env_still_work() {
        let value = config_lib::parse(
            "include = \"x\"\nenv = 'y'\nt = \"${HOME}\"\n",
            Some("toml"),
        )
        .unwrap();
        assert_eq!(value.get("include").unwrap().as_string().unwrap(), "x");
        assert_eq!(value.get("env").unwrap().as_string().unwrap(), "y");
        assert_eq!(value.get("t").unwrap().as_string().unwrap(), "${HOME}");
    }

    #[test]
    fn preserving_parse_refuses_too() {
        assert_refused(
            config_lib::parsers::toml_parser::parse_with_preservation(
                "k = env(\"CONFIG_LIB_TOML_SECRET\")\n",
            ),
            (1, 5),
            &["CONFIG_LIB_TOML_SECRET"],
        );
    }
}

// ---------------------------------------------------------------------------
// Strict NOML
// ---------------------------------------------------------------------------

#[cfg(feature = "noml")]
mod noml {
    use super::*;

    fn included_file() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let other = dir.path().join("shared.noml");
        std::fs::write(&other, "x = 1\n").unwrap();
        let path = other.display().to_string().replace('\\', "/");
        (dir, path)
    }

    #[test]
    fn default_noml_still_resolves_include_and_env() {
        let (_dir, path) = included_file();
        std::env::set_var("CONFIG_LIB_NOML_VAR", "from-env");
        let source = format!("shared = include \"{path}\"\nv = env(\"CONFIG_LIB_NOML_VAR\")\n");
        let config = Config::from_string(&source, Some("noml")).unwrap();
        assert_eq!(config.get("shared.x").unwrap().as_integer().unwrap(), 1);
        assert_eq!(config.get("v").unwrap().as_string().unwrap(), "from-env");
    }

    #[test]
    fn strict_noml_refuses_include() {
        let (_dir, path) = included_file();
        let source = format!("name = \"app\"\nshared = include \"{path}\"\n");
        let secrets = [path.as_str(), "shared.noml"];
        assert_refused(
            Config::from_string_strict(&source, Some("noml")),
            (2, 10),
            &secrets,
        );
        assert_refused(
            config_lib::parse_strict(&source, Some("noml")),
            (2, 10),
            &secrets,
        );
        assert_refused(
            config_lib::parsers::noml_parser::parse_strict(&source),
            (2, 10),
            &secrets,
        );
    }

    #[test]
    fn strict_noml_refuses_env() {
        std::env::set_var("CONFIG_LIB_NOML_SECRET", "hunter2");
        let source = "[db]\npassword = env(\"CONFIG_LIB_NOML_SECRET\", \"none\")\n";
        let secrets = ["CONFIG_LIB_NOML_SECRET", "hunter2"];
        assert_refused(
            Config::from_string_strict(source, Some("noml")),
            (2, 12),
            &secrets,
        );
        // Detected as NOML without a hint, and refused
        let unhinted = "user = \"app\"\npassword = env(\"CONFIG_LIB_NOML_SECRET\")\n";
        assert_refused(
            Config::from_string_strict(unhinted, None),
            (2, 12),
            &secrets,
        );
        // Nested in a native type's arguments
        assert_refused(
            Config::from_string_strict(
                "s = @size(env(\"CONFIG_LIB_NOML_SECRET\"))\n",
                Some("noml"),
            ),
            (1, 11),
            &secrets,
        );
    }

    #[test]
    fn strict_noml_keeps_native_types_and_references() {
        let source =
            "base = \"/srv\"\ndata = \"${base}/data\"\ncopy = ${base}\nsize = @size(\"1KB\")\n";
        let strict = Config::from_string_strict(source, Some("noml")).unwrap();
        let default = Config::from_string(source, Some("noml")).unwrap();
        assert_eq!(strict.as_value(), default.as_value());
        assert_eq!(
            strict.get("data").unwrap().as_string().unwrap(),
            "/srv/data"
        );
        assert_eq!(strict.get("size").unwrap().as_integer().unwrap(), 1024);
    }
}

#[test]
fn strict_is_the_same_as_default_for_other_formats() {
    let mut cases = vec![("conf", "a = 1\n[s]\nb = two words\n")];
    if cfg!(feature = "toml") {
        cases.push(("toml", "a = 1\n[s]\nb = \"${x}\"\n"));
    }
    if cfg!(any(feature = "json", feature = "jsonx")) {
        cases.push(("json", "{\"a\": 1, \"s\": {\"b\": \"env(X)\"}}"));
    }
    for (format, source) in cases {
        let strict = Config::from_string_strict(source, Some(format)).unwrap();
        let default = Config::from_string(source, Some(format)).unwrap();
        assert_eq!(strict.as_value(), default.as_value(), "{format}");
        assert_eq!(strict.format(), default.format(), "{format}");
    }
}
