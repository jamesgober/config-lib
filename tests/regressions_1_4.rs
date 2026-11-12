//! Regression tests for the fixes in v1.4.0.
//!
//! Each test fails on v1.3.2.

use config_lib::hot_reload::{ConfigChangeEvent, HotReloadConfig};
use config_lib::{parse, Config, Error, LoadOptions, Value};
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn write_synced(path: &std::path::Path, body: &str) {
    let mut f = std::fs::File::create(path).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    f.sync_all().unwrap();
}

fn wait_for(limit: Duration, mut done: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + limit;
    while Instant::now() < deadline {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    done()
}

/// Display and Debug of an error must not contain `secret`
fn assert_quiet<T: std::fmt::Debug>(result: config_lib::Result<T>, secret: &str) {
    let err = result.expect_err("expected an error");
    let shown = format!("{err} {err:?}");
    assert!(
        !shown.contains(secret),
        "error text quotes the input: {shown}"
    );
}

// ---------------------------------------------------------------------------
// Hot reload: handlers run after the lock is released
// ---------------------------------------------------------------------------

#[test]
fn a_handler_can_read_the_config_after_a_background_reload() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.conf");
    write_synced(&path, "port = 1\n");

    let hot = HotReloadConfig::from_file(&path)
        .unwrap()
        .with_debounce(Duration::from_millis(25))
        .with_poll_interval(Duration::from_millis(50));
    let shared = hot.config();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let finished = Arc::new(AtomicBool::new(false));
    let (s, v, f) = (
        Arc::clone(&shared),
        Arc::clone(&seen),
        Arc::clone(&finished),
    );
    hot.on_change(move |event| {
        if matches!(event, ConfigChangeEvent::Reloaded { .. }) {
            // A read lock on the reloader's own thread: this deadlocked
            // while the reloader still held the write lock
            let port: u16 = s.read().map(|c| c.get_or("port", 0)).unwrap_or(0);
            v.lock().unwrap().push(port);
            f.store(true, Ordering::SeqCst);
        }
    })
    .forget();
    let handle = hot.start_watching();
    std::thread::sleep(Duration::from_millis(300));
    write_synced(&path, "port = 2\n");

    let done = wait_for(Duration::from_secs(10), || finished.load(Ordering::SeqCst));
    if !done {
        // The worker is stuck; joining it would hang the test
        std::mem::forget(handle);
        panic!("the handler never finished reading the configuration");
    }
    assert!(shared.try_read().is_ok(), "readers are blocked");
    assert_eq!(seen.lock().unwrap().last(), Some(&2));
    handle.stop().unwrap();
}

// ---------------------------------------------------------------------------
// Hot reload: truncated files, carried state, validation
// ---------------------------------------------------------------------------

#[test]
fn a_truncated_file_does_not_replace_the_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.conf");
    std::fs::write(&path, "auth_required = true\nport = 8443\n").unwrap();
    let hot = HotReloadConfig::from_file(&path)
        .unwrap()
        .with_debounce(Duration::from_millis(25));
    let shared = hot.config();
    let reloads = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(AtomicUsize::new(0));
    let (r, fl) = (Arc::clone(&reloads), Arc::clone(&failures));
    hot.on_change(move |event| match event {
        ConfigChangeEvent::Reloaded { .. } => {
            r.fetch_add(1, Ordering::SeqCst);
        }
        ConfigChangeEvent::ReloadFailed { .. } => {
            fl.fetch_add(1, Ordering::SeqCst);
        }
        _ => {}
    })
    .forget();
    let handle = hot.start_watching();
    std::thread::sleep(Duration::from_millis(300));

    // A writer truncates the file, then is slow to write the body
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap();
    file.sync_all().unwrap();
    assert!(wait_for(Duration::from_secs(5), || failures
        .load(Ordering::SeqCst)
        > 0));
    assert_eq!(reloads.load(Ordering::SeqCst), 0);
    assert!(shared.read().unwrap().contains_key("port"));
    assert!(shared.read().unwrap().get_or("auth_required", false));

    file.write_all(b"auth_required = true\nport = 9443\n")
        .unwrap();
    file.sync_all().unwrap();
    drop(file);
    assert!(wait_for(Duration::from_secs(5), || shared
        .read()
        .unwrap()
        .get_or("port", 0u16)
        == 9443));
    handle.stop().unwrap();
}

#[test]
fn an_empty_file_replaces_the_config_when_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.conf");
    std::fs::write(&path, "port = 1\n").unwrap();
    let mut hot = HotReloadConfig::from_file(&path)
        .unwrap()
        .allow_empty_reload();
    std::fs::write(&path, "").unwrap();
    assert!(hot.reload().unwrap());
    assert!(!hot.config().read().unwrap().contains_key("port"));

    let mut strict = HotReloadConfig::from_file(&path).unwrap();
    std::fs::write(&path, "port = 1\n").unwrap();
    assert!(strict.reload().unwrap());
    std::fs::write(&path, "").unwrap();
    assert!(strict.reload().is_err());
    assert!(strict.config().read().unwrap().contains_key("port"));
}

#[test]
fn a_reload_keeps_read_only_mode_and_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.conf");
    std::fs::write(&path, "port = 1\n").unwrap();
    let mut hot = HotReloadConfig::from_file(&path).unwrap();
    let shared = hot.config();
    {
        let mut c = shared.write().unwrap();
        c.make_read_only();
        c.set_default("timeout", 30).unwrap();
    }
    std::fs::write(&path, "port = 2\nother = 1\n").unwrap();
    assert!(hot.reload().unwrap());
    let c = shared.read().unwrap();
    assert_eq!(c.get_or("port", 0u16), 2);
    assert!(c.is_read_only());
    assert_eq!(c.get_or_default("timeout"), Some(Value::Integer(30)));
}

#[test]
fn a_validator_refuses_a_reload_and_keeps_the_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.conf");
    std::fs::write(&path, "port = 1\n").unwrap();
    let mut hot = HotReloadConfig::from_file(&path)
        .unwrap()
        .with_validator(|c| {
            if c.contains_key("port") {
                Ok(())
            } else {
                Err(Error::validation("port is required"))
            }
        });
    std::fs::write(&path, "host = xyz\n").unwrap();
    let err = hot.reload().unwrap_err();
    assert!(err.to_string().contains("port is required"), "{err}");
    assert_eq!(hot.config().read().unwrap().get_or("port", 0u16), 1);
}

#[cfg(feature = "validation")]
#[test]
fn validation_rules_carry_over_and_critical_errors_refuse_a_reload() {
    use config_lib::validation::{
        ValidationError, ValidationResult, ValidationRule, ValidationRuleSet, ValidationSeverity,
    };

    /// Refuses the text "bad" anywhere, as a critical error
    struct NoBad;
    impl ValidationRule for NoBad {
        fn name(&self) -> &str {
            "no_bad"
        }
        fn validate(&self, path: &str, value: &Value) -> ValidationResult {
            if value.as_string().map(|s| s == "bad").unwrap_or(false) {
                ValidationResult::Invalid(
                    ValidationError::new(path, "no_bad", "bad value")
                        .with_severity(ValidationSeverity::Critical),
                )
            } else {
                ValidationResult::Valid
            }
        }
    }

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.conf");
    std::fs::write(&path, "[server]\nmode = good\n").unwrap();
    let mut hot = HotReloadConfig::from_file(&path).unwrap();
    hot.config()
        .write()
        .unwrap()
        .set_validation_rules(ValidationRuleSet::new().add_rule(NoBad));

    // Nested values are checked: the reload is refused, the old config kept
    std::fs::write(&path, "[server]\nmode = bad\n").unwrap();
    assert!(hot.reload().is_err());
    let mode = hot
        .config()
        .read()
        .unwrap()
        .get_or("server.mode", String::new());
    assert_eq!(mode, "good");

    // The rules are still in place for the next reload
    std::fs::write(&path, "[server]\nmode = fine\nother = 1\n").unwrap();
    assert!(hot.reload().unwrap());
    std::fs::write(&path, "[server]\nmode = bad\nother = 2\n").unwrap();
    assert!(hot.reload().is_err());
    let mode = hot
        .config()
        .read()
        .unwrap()
        .get_or("server.mode", String::new());
    assert_eq!(mode, "fine");
}

#[test]
fn save_to_file_replaces_the_file_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.conf");
    std::fs::write(&path, "port = 1\n").unwrap();
    let mut config = Config::from_file(&path).unwrap();
    config.set("port", 2).unwrap();
    config.save().unwrap();
    let back = Config::from_file(&path).unwrap();
    assert_eq!(back.get_or("port", 0u16), 2);
    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 1, "no temporary file left behind: {names:?}");

    // A reader never sees a partly written file while saves run
    let big = |c: char| {
        let mut config = Config::new();
        config.set("k", c.to_string().repeat(400_000)).unwrap();
        config
    };
    let (a, b) = (big('a'), big('b'));
    a.save_to_file(&path).unwrap();
    let full = std::fs::metadata(&path).unwrap().len();
    let stop = Arc::new(AtomicBool::new(false));
    let short_reads = Arc::new(AtomicUsize::new(0));
    let reader = {
        let (path, stop, short) = (path.clone(), Arc::clone(&stop), Arc::clone(&short_reads));
        std::thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if (text.len() as u64) < full {
                        short.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        })
    };
    for i in 0..40 {
        if i % 2 == 0 { &b } else { &a }
            .save_to_file(&path)
            .unwrap();
    }
    stop.store(true, Ordering::SeqCst);
    reader.join().unwrap();
    assert_eq!(
        short_reads.load(Ordering::SeqCst),
        0,
        "a reader saw a partial file"
    );
}

// ---------------------------------------------------------------------------
// NOML expansion limits
// ---------------------------------------------------------------------------

#[cfg(feature = "noml")]
mod noml {
    use super::*;

    fn bomb(levels: usize) -> String {
        let mut doc = String::from("s0 = \"AAAAAAAAAA\"\n");
        for i in 1..=levels {
            let r = format!("${{s{}}}", i - 1);
            doc.push_str(&format!("s{i} = \"{}\"\n", r.repeat(10)));
        }
        doc
    }

    #[test]
    fn exponential_expansion_is_refused_strict_and_default() {
        let doc = bomb(7);
        let t = Instant::now();
        for result in [
            config_lib::parse_strict(&doc, Some("noml")),
            config_lib::parse(&doc, Some("noml")),
        ] {
            match result {
                Err(Error::Parse { message, .. }) => {
                    assert!(message.contains("size limit"), "{message}");
                }
                other => panic!("expected a size-limit error, got {other:?}"),
            }
        }
        assert!(t.elapsed() < Duration::from_secs(5), "{:?}", t.elapsed());
    }

    #[test]
    fn bare_copies_of_tables_are_bounded_too() {
        let mut doc = String::from("[t0]\na = \"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\"\n");
        for i in 1..=8 {
            doc.push_str(&format!("[t{i}]\n"));
            for j in 0..10 {
                doc.push_str(&format!("c{j} = ${{t{}}}\n", i - 1));
            }
        }
        assert!(config_lib::parse(&doc, Some("noml")).is_err());
    }

    #[test]
    fn ordinary_references_still_resolve() {
        let doc = "name = \"app\"\nlog = \"/var/log/${name}.log\"\n[server]\nport = 80\n[copy]\nserver = ${server}\nalso = \"${server.port}\"\n";
        let v = config_lib::parse(doc, Some("noml")).unwrap();
        assert_eq!(
            v.get("log").unwrap().as_string().unwrap(),
            "/var/log/app.log"
        );
        assert_eq!(v.get("copy.server.port").unwrap().as_integer().unwrap(), 80);
        assert_eq!(v.get("copy.also").unwrap().as_string().unwrap(), "80");
    }

    #[test]
    fn the_limits_can_be_set() {
        let doc = bomb(3); // 10 KB
        assert!(config_lib::parse(&doc, Some("noml")).is_ok());
        let tight = LoadOptions::new().max_expanded_size(4096);
        assert!(Config::from_string_with(&doc, Some("noml"), &tight).is_err());
        let few = LoadOptions::new().max_expansions(5);
        assert!(Config::from_string_with(&doc, Some("noml"), &few).is_err());

        let mut chain = String::from("a0 = \"x\"\n");
        for i in 1..=40 {
            chain.push_str(&format!("a{i} = \"${{a{}}}\"\n", i - 1));
        }
        assert!(
            config_lib::parse(&chain, Some("noml")).is_err(),
            "deeper than 32"
        );
        let deep = LoadOptions::new().max_expansion_depth(64);
        let v = Config::from_string_with(&chain, Some("noml"), &deep).unwrap();
        assert_eq!(v.get("a40").unwrap().as_string().unwrap(), "x");
    }

    #[test]
    fn a_reference_cycle_is_an_error() {
        assert!(config_lib::parse("a = \"${b}\"\nb = \"${a}\"\n", Some("noml")).is_err());
        assert!(config_lib::parse("a = ${a}\n", Some("noml")).is_err());
    }

    // -----------------------------------------------------------------------
    // Includes
    // -----------------------------------------------------------------------

    fn fwd(p: &std::path::Path) -> String {
        p.display().to_string().replace('\\', "/")
    }

    #[test]
    fn includes_outside_the_config_directory_are_refused() {
        let secret_dir = tempfile::tempdir().unwrap();
        let secret = secret_dir.path().join("secret.noml");
        std::fs::write(&secret, "token = \"s3cr3t\"\n").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("app.noml");
        std::fs::write(&cfg, format!("stolen = include \"{}\"\n", fwd(&secret))).unwrap();

        assert_quiet(Config::from_file(&cfg), "s3cr3t");
        assert!(HotReloadConfig::from_file(&cfg).is_err());

        let allowed = LoadOptions::new().allow_include_dir(secret_dir.path());
        let c = Config::from_file_with(&cfg, &allowed).unwrap();
        assert_eq!(
            c.get("stolen.token").unwrap().as_string().unwrap(),
            "s3cr3t"
        );
        let anywhere = LoadOptions::new().allow_includes_anywhere();
        assert!(Config::from_file_with(&cfg, &anywhere).is_ok());
    }

    #[test]
    fn includes_inside_the_directory_work_and_dot_dot_cannot_escape() {
        let outer = tempfile::tempdir().unwrap();
        std::fs::write(outer.path().join("secret.noml"), "token = \"s3cr3t\"\n").unwrap();
        let dir = outer.path().join("conf");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join("db.noml"),
            "host = \"db\"\nurl = \"pg://${host}\"\n",
        )
        .unwrap();
        let cfg = dir.join("app.noml");
        std::fs::write(&cfg, "db = include \"db.noml\"\nhost = \"app\"\n").unwrap();
        let c = Config::from_file(&cfg).unwrap();
        // A reference in the included file finds its own value first
        assert_eq!(c.get("db.url").unwrap().as_string().unwrap(), "pg://db");

        std::fs::write(&cfg, "x = include \"../secret.noml\"\n").unwrap();
        assert_quiet(Config::from_file(&cfg), "s3cr3t");
    }

    #[test]
    fn an_included_file_that_is_not_noml_is_not_quoted() {
        let dir = tempfile::tempdir().unwrap();
        let creds = dir.path().join("creds.txt");
        std::fs::write(
            &creds,
            "aws_secret = wJalrXUtnFEMIK7MDENGbPxRfiCYEXAMPLEKEY\n",
        )
        .unwrap();
        let cfg = dir.path().join("app.noml");
        // An absolute path, so 1.3.2 (which resolved relative includes
        // against the working directory) reads the file too
        std::fs::write(&cfg, format!("x = include \"{}\"\n", fwd(&creds))).unwrap();
        assert_quiet(
            Config::from_file(&cfg),
            "wJalrXUtnFEMIK7MDENGbPxRfiCYEXAMPLEKEY",
        );
    }

    #[test]
    fn strict_file_and_hot_reload_loads_refuse_include_and_env() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("db.noml"), "host = \"db\"\n").unwrap();
        let cfg = dir.path().join("app.noml");
        std::fs::write(&cfg, "db = include \"db.noml\"\n").unwrap();
        assert!(Config::from_file(&cfg).is_ok());
        assert!(matches!(
            Config::from_file_strict(&cfg),
            Err(Error::Parse { .. })
        ));
        assert!(HotReloadConfig::from_file_strict(&cfg).is_err());

        std::fs::write(&cfg, "port = 1\n").unwrap();
        let mut hot = HotReloadConfig::from_file_strict(&cfg).unwrap();
        std::fs::write(&cfg, "port = env(\"PATH\")\n").unwrap();
        assert!(hot.reload().is_err());
        assert_eq!(hot.config().read().unwrap().get_or("port", 0u16), 1);
    }

    #[test]
    fn noml_syntax_errors_are_not_quoted() {
        assert_quiet(parse("password = hunter2\n", Some("noml")), "hunter2");
    }
}

// ---------------------------------------------------------------------------
// Error text never quotes the input
// ---------------------------------------------------------------------------

#[cfg(feature = "toml")]
#[test]
fn toml_syntax_errors_are_not_quoted() {
    assert_quiet(parse("password = hunter2\n", Some("toml")), "hunter2");
    assert_quiet(parse("a = 1\na = 2 # hunter2\n", Some("toml")), "hunter2");
}

#[test]
fn ini_conf_and_properties_errors_are_not_quoted() {
    assert_quiet(
        parse("[db]\nhunter2-secret-token\n", Some("ini")),
        "hunter2-secret-token",
    );
    assert_quiet(
        parse("key = \"unterminated hunter2", Some("conf")),
        "hunter2",
    );
    assert_quiet(parse("key = [hunter2", Some("conf")), "hunter2");
}

#[test]
fn file_errors_do_not_quote_the_contents() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.ini");
    std::fs::write(&path, "[db]\nhunter2-secret-token\n").unwrap();
    assert_quiet(Config::from_file(&path), "hunter2-secret-token");
}

#[test]
fn file_size_limit_and_non_regular_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("big.conf");
    std::fs::write(&path, "a = 1\n".repeat(1000)).unwrap();
    assert!(Config::from_file(&path).is_ok());
    let small = LoadOptions::new().max_file_size(100);
    assert!(Config::from_file_with(&path, &small).is_err());
    assert!(
        Config::from_file(dir.path()).is_err(),
        "a directory is refused"
    );
}

// ---------------------------------------------------------------------------
// INI and environment typing
// ---------------------------------------------------------------------------

#[test]
fn ini_one_and_zero_are_integers_that_read_as_booleans_too() {
    let c = Config::from_string(
        "[pool]\nworkers = 1\nretries = 0\nflag = yes\n",
        Some("ini"),
    )
    .unwrap();
    assert_eq!(c.get_or("pool.workers", 16u16), 1);
    assert_eq!(c.get_or("pool.retries", 5u32), 0);
    assert!(c.get_or("pool.workers", false));
    assert!(!c.get_or("pool.retries", true));
    assert!(c.get_or("pool.flag", false));
}

#[cfg(feature = "env-override")]
#[test]
fn env_overrides_keep_the_type_of_the_value_they_replace() {
    use config_lib::env_override::EnvOverrideSystem;
    std::env::set_var("CL14_POOL_WORKERS", "1");
    std::env::set_var("CL14_DB_PASSWORD", "ab,cd");
    std::env::set_var("CL14_DB_HOSTS", "a,b");
    let base = Config::from_string(
        "[pool]\nworkers = 8\n[db]\npassword = x\nhosts = [one, two]\n",
        Some("conf"),
    )
    .unwrap();
    let v = EnvOverrideSystem::with_prefix("CL14_")
        .apply_overrides(base.as_value().clone())
        .unwrap();
    let c = Config::from(v);
    assert_eq!(c.get_or("pool.workers", 8u16), 1);
    assert_eq!(c.get("db.password").unwrap().as_string().unwrap(), "ab,cd");
    assert!(c.get("db.hosts").unwrap().is_array());
}

// ---------------------------------------------------------------------------
// Format detection
// ---------------------------------------------------------------------------

#[test]
fn detection_only_leaves_conf_for_clear_signatures() {
    let v = parse("msg = \"line1\\nline2\"\n", None).unwrap();
    assert_eq!(v.get("msg").unwrap().as_string().unwrap(), "line1\nline2");
    let v = parse("TZ = UTC\n", None).unwrap();
    assert_eq!(v.get("TZ").unwrap().as_string().unwrap(), "UTC");
    let v = parse("data = /var/lib/app\n", None).unwrap();
    assert_eq!(v.get("data").unwrap().as_string().unwrap(), "/var/lib/app");
    let v = parse("# include defaults\nport = 1\n", None).unwrap();
    assert_eq!(v.get("port").unwrap().as_integer().unwrap(), 1);
    let v = parse("[server]\nport = 1\n", None).unwrap();
    assert_eq!(v.get("server.port").unwrap().as_integer().unwrap(), 1);
    assert_eq!(config_lib::parsers::detect_format("; c\n[a]\nb=1"), "ini");
    assert_eq!(config_lib::parsers::detect_format("{\"a\": 1}"), "json");
    assert_eq!(
        config_lib::parsers::detect_format("<?xml version=\"1.0\"?><a/>"),
        "xml"
    );
}

// ---------------------------------------------------------------------------
// CONF
// ---------------------------------------------------------------------------

#[test]
fn conf_save_round_trips() {
    let mut c = Config::new();
    c.set("a.b.c", 1).unwrap();
    c.set("db.pool.size", 10).unwrap();
    c.set("db.name", "main").unwrap();
    c.set("list_like", "x,y").unwrap();
    c.set("word", "true").unwrap();
    c.set("empty", "").unwrap();
    c.set("secret", "p#ss").unwrap();
    c.set("path", "C:\\dir with space\\").unwrap();
    c.set("quote", "say \"hi\"\n\ttab").unwrap();
    c.set("nothing", Value::Null).unwrap();
    c.set("ratio", 1.0).unwrap();
    c.set("big", 1e20).unwrap();
    c.set(
        "items",
        Value::array(vec![
            Value::from("two words"),
            Value::from(3),
            Value::array(vec![Value::from(true)]),
        ]),
    )
    .unwrap();
    let text = c.serialize().unwrap();
    let back = Config::from_string(&text, Some("conf")).unwrap();
    assert_eq!(back.as_value(), c.as_value(), "{text}");
}

#[test]
fn conf_save_refuses_what_it_cannot_write() {
    let mut c = Config::new();
    c.set("ratio", f64::NAN).unwrap();
    assert!(c.serialize().is_err());
}

#[test]
fn conf_inline_comments_need_whitespace() {
    let c = Config::from_string(
        "password = abc#def\nurl = http://h/#frag\nport = 80 # web\n",
        Some("conf"),
    )
    .unwrap();
    assert_eq!(c.get("password").unwrap().as_string().unwrap(), "abc#def");
    assert_eq!(c.get("url").unwrap().as_string().unwrap(), "http://h/#frag");
    assert_eq!(c.get_or("port", 0u16), 80);
    let i =
        Config::from_string("[db]\npassword = abc;def\nhost = h ; note\n", Some("ini")).unwrap();
    assert_eq!(
        i.get("db.password").unwrap().as_string().unwrap(),
        "abc;def"
    );
    assert_eq!(i.get("db.host").unwrap().as_string().unwrap(), "h");
}

#[test]
fn conf_refuses_two_assignments_on_one_line() {
    assert!(Config::from_string("tls = \"on\" tls = off\n", Some("conf")).is_err());
}

#[test]
fn conf_numbers_are_decimal_only() {
    let c = Config::from_string(
        "a = nan\nb = inf\nc = 99999999999999999999\nd = 1e3\n",
        Some("conf"),
    )
    .unwrap();
    assert_eq!(c.get("a").unwrap().as_string().unwrap(), "nan");
    assert_eq!(c.get("b").unwrap().as_string().unwrap(), "inf");
    assert!(c.get("c").unwrap().is_string());
    assert_eq!(c.get("d").unwrap().as_float().unwrap(), 1000.0);
}

#[test]
fn as_integer_follows_the_checked_conversion() {
    assert!(Value::Float(f64::NAN).as_integer().is_err());
    assert!(Value::Float(1e20).as_integer().is_err());
    assert!(Value::Float(100.9).as_integer().is_err());
    assert_eq!(Value::Float(100.0).as_integer().unwrap(), 100);
}

#[test]
fn get_mut_respects_read_only_and_clears_the_cache() {
    let mut c = Config::from_string("port = 1\n", Some("conf")).unwrap();
    let _ = c.get_arc("port");
    *c.get_mut("port").unwrap() = Value::Integer(2);
    assert_eq!(*c.get_arc("port").unwrap(), Value::Integer(2));
    c.make_read_only();
    assert!(c.get_mut("port").is_err());
}

#[test]
fn conf_sections_nest_at_dots() {
    let c = Config::from_string("[a.b]\nc = 1\n[a]\nd = 2\n", Some("conf")).unwrap();
    assert_eq!(c.get("a.b.c").unwrap().as_integer().unwrap(), 1);
    assert_eq!(c.get("a.d").unwrap().as_integer().unwrap(), 2);
}

#[cfg(feature = "validation")]
#[test]
fn validation_checks_nested_values_and_non_finite_numbers() {
    use config_lib::validation::{RangeValidator, ValidationRuleSet};
    let mut c = Config::from_string("[server]\nport = 99999\n", Some("conf")).unwrap();
    c.set_validation_rules(
        ValidationRuleSet::new().add_rule(RangeValidator::new(Some(1.0), Some(65535.0))),
    );
    let errors = c.validate().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path, "server.port");

    let mut c = Config::new();
    c.set("ratio", f64::NAN).unwrap();
    c.set("limit", 100.9).unwrap();
    c.set_validation_rules(
        ValidationRuleSet::new().add_rule(RangeValidator::new(Some(-1.0), Some(100.0))),
    );
    assert_eq!(c.validate().unwrap().len(), 2);
}

// ---------------------------------------------------------------------------
// HCL
// ---------------------------------------------------------------------------

#[cfg(feature = "hcl")]
#[test]
fn hcl_parses_nesting_lists_and_comments_and_refuses_the_rest() {
    let v = parse(
        "server {\n  tags = {\n    env = \"prod\"\n  }\n  port = 8080 # web\n  hosts = [\"a\", \"b\"]\n}\n",
        Some("hcl"),
    )
    .unwrap();
    assert_eq!(
        v.get("server.tags.env").unwrap().as_string().unwrap(),
        "prod"
    );
    assert_eq!(v.get("server.port").unwrap().as_integer().unwrap(), 8080);
    assert_eq!(v.get("server.hosts").unwrap().as_array().unwrap().len(), 2);

    let v = parse(
        "resource \"aws_instance\" \"web\" {\n  ami = \"x\"\n}\n",
        Some("hcl"),
    )
    .unwrap();
    assert_eq!(
        v.get("resource.aws_instance.web.ami")
            .unwrap()
            .as_string()
            .unwrap(),
        "x"
    );

    assert!(parse("this is not hcl at all\n}}}\n", Some("hcl")).is_err());
    assert!(parse("server {\n  port = 1\n", Some("hcl")).is_err());
    assert!(parse("region = var.region\n", Some("hcl")).is_err());
    assert!(parse("n = max(1, 2)\n", Some("hcl")).is_err());
}

// ---------------------------------------------------------------------------
// Audit lines
// ---------------------------------------------------------------------------

#[test]
fn audit_lines_redact_values_and_escape_fields() {
    use config_lib::audit::{AuditEvent, AuditEventType, AuditSeverity};
    let event = AuditEvent::new(AuditEventType::Modification, AuditSeverity::Warning)
        .with_key("db.password\n[0] Reload:Info forged")
        .with_old_value(Value::from("old-hunter2"))
        .with_new_value(Value::from("new-hunter2"))
        .with_user_context("ops");
    let line = event.to_string();
    assert!(!line.contains('\n'), "{line}");
    assert!(!line.contains("hunter2"), "{line}");
    assert!(line.contains("<string, 11 bytes>"), "{line}");
}
