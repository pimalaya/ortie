//! # Exit contract
//!
//! The built binary on a command line the grammar refuses: exit 2 before
//! any configuration is read, the failure object coded `usage-invalid` on
//! stdout under `--json`, clap's rendering on stderr otherwise.

use std::{
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::{Value, json};
use tempfile::TempDir;

/// Runs the binary with `args` in `home`, its stdin closed.
fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ortie"))
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn a_refused_command_line_under_json_prints_the_usage_failure_object() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["--json", "token", "show", "--nope"]);
    let stdout = text(&output.stdout);

    assert_eq!(output.status.code(), Some(2), "{stdout}");
    assert!(output.stderr.is_empty(), "{}", text(&output.stderr));

    let failure: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(failure["code"], "usage-invalid", "{failure}");
    assert_eq!(failure["sources"], json!([]), "{failure}");
    assert_eq!(failure["backtrace"], Value::Null, "{failure}");

    let error = failure["error"].as_str().unwrap();
    assert!(error.starts_with("unexpected argument '--nope'"), "{error}");
    assert!(!error.contains('\x1b'), "{error}");
}

#[test]
fn a_refused_command_line_without_json_is_clap_rendering_on_stderr() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["token", "show", "--nope"]);
    let stderr = text(&output.stderr);

    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
    assert!(stderr.starts_with("error: unexpected argument"), "{stderr}");
}

/// A `--json` after `--` is an argument, not the flag.
#[test]
fn a_json_word_after_the_separator_leaves_the_refusal_on_stderr() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["token", "show", "--nope", "--", "--json"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
    assert!(!output.stderr.is_empty());
}

/// Help is no failure, `--json` or not: clap prints it on stdout and
/// exits 0.
#[test]
fn help_under_json_is_no_failure() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["--json", "token", "show", "--help"]);
    let stdout = text(&output.stdout);

    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(stdout.contains("Usage: ortie token show"), "{stdout}");
}

/// A refused PKCE verifier is a secret of the flow, so the refusal names
/// its length and the flag, never its value.
#[test]
fn a_refused_pkce_verifier_is_not_echoed() {
    let home = TempDir::new().unwrap();
    let secret = "pkce-secret-value-too-short";

    let output = run(home.path(), &["auth", "resume", "--pkce", secret, "x"]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("length 27"), "{stderr}");
    assert!(!stderr.contains(secret), "{stderr}");

    let output = run(home.path(), &["--json", "auth", "listen", "--pkce", secret]);
    let stdout = text(&output.stdout);
    assert_eq!(output.status.code(), Some(2), "{stdout}");
    assert!(stdout.contains("usage-invalid"), "{stdout}");
    assert!(!stdout.contains(secret), "{stdout}");
}

/// A log file Ortie cannot open fails the command as any other failure,
/// with the failure object and exit 1, instead of a panic.
#[test]
fn a_log_file_that_cannot_be_opened_is_a_failure_object() {
    let home = TempDir::new().unwrap();
    let path = home.path().join("missing").join("ortie.log");
    let path = path.to_str().unwrap();

    let output = run(
        home.path(),
        &["--json", "--log-file", path, "account", "list"],
    );
    let stdout = text(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{stdout}");

    let failure: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(failure["error"], format!("Cannot open log file {path}"));
}
