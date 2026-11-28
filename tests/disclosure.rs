//! # Google data disclosure
//!
//! The built binary's `auth get` on an account using Pimalaya's Google
//! application: the agreement asked in a terminal, given by
//! `--agree-data-use`, and refused coded `agreement-missing` before
//! anything is sent where it can be neither.

mod common;

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use serde_json::Value;
use tempfile::TempDir;
use url::Url;

use crate::common::*;

/// The client id of Pimalaya's Google Desktop application.
const PIMALAYA_GOOGLE: &str =
    "803561575535-41415ghqi04uibrl6t5umg3p27stdhqr.apps.googleusercontent.com";

/// Starts a listener counting the connections it accepts, standing for
/// the authorization server.
fn start_counter() -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let count_t = Arc::clone(&count);

    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            count_t.fetch_add(1, Ordering::SeqCst);
            drop(stream);
        }
    });

    (addr, count)
}

/// Writes an account of `client_id` running `grant` against the server
/// at `addr`, asking Gmail and Tasks scopes.
fn write_account(dir: &Path, client_id: &str, grant: &str, addr: SocketAddr) -> PathBuf {
    let lines = format!(
        "client-id = \"{client_id}\"\ngrant = \"{grant}\"\n\
         endpoints.authorization = \"http://{addr}/authorize\"\n\
         endpoints.device-authorization = \"http://{addr}/device\"\n\
         endpoints.token = \"http://{addr}/token\"\n\
         scopes = [\"https://mail.google.com/\", \"https://www.googleapis.com/auth/tasks\"]"
    );

    write_config(dir, "google", &lines, "").0
}

/// Runs the binary with `line` in a terminal util-linux's `script`
/// allocates, `input` typed into it, answering what it printed.
fn in_terminal(config: &Path, line: &str, input: &[u8]) -> (Option<i32>, String) {
    let line = format!(
        "{} -c {} {line}",
        env!("CARGO_BIN_EXE_ortie"),
        config.display()
    );
    let mut script = Command::new("script")
        .args(["-qec", &line, "/dev/null"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut stdin = script.stdin.take().unwrap();
    let mut stdout = BufReader::new(script.stdout.take().unwrap());
    let mut printed = String::new();

    while !printed.contains("[y/N]") {
        let mut chunk = Vec::new();
        let read = stdout.read_until(b' ', &mut chunk).unwrap();
        assert_ne!(read, 0, "{printed}");
        printed.push_str(&String::from_utf8_lossy(&chunk));
    }

    stdin.write_all(input).unwrap();
    stdout.read_to_string(&mut printed).unwrap();
    let status = script.wait().unwrap();
    drop(stdin);

    (status.code(), printed)
}

#[test]
fn a_json_sign_in_without_the_flag_is_refused_before_anything_is_sent() {
    for grant in ["authorization-code", "device"] {
        let dir = TempDir::new().unwrap();
        let (addr, count) = start_counter();
        let config = write_account(dir.path(), PIMALAYA_GOOGLE, grant, addr);

        let failure = json_failure(&ortie(&config, &["--json", "auth", "get"]));

        assert_eq!(failure["code"], "agreement-missing", "{failure}");
        let error = failure["error"].as_str().unwrap();
        assert!(error.contains("your Gmail messages, labels"), "{error}");
        assert!(error.contains("your Google task lists"), "{error}");
        assert!(!error.contains("calendars"), "{error}");
        assert!(error.contains("https://pimalaya.org/privacy/"), "{error}");
        assert!(
            error.ends_with("--agree-data-use to agree to it"),
            "{error}"
        );
        assert_eq!(count.load(Ordering::SeqCst), 0, "{grant}");
    }
}

#[test]
fn the_flag_agrees_and_the_json_handoff_is_unchanged() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_counter();
    let config = write_account(dir.path(), PIMALAYA_GOOGLE, "authorization-code", addr);

    let output = ortie(&config, &["--json", "auth", "get", "--agree-data-use"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{stdout}");

    let handoff: Value = serde_json::from_str(&stdout).unwrap();
    let uri = Url::parse(handoff["authorizationUri"].as_str().unwrap()).unwrap();
    assert!(
        uri.query_pairs()
            .any(|(k, v)| k == "client_id" && v == PIMALAYA_GOOGLE)
    );
}

#[test]
fn a_piped_text_sign_in_prints_the_disclosure_before_the_request() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_counter();
    let config = write_account(dir.path(), PIMALAYA_GOOGLE, "authorization-code", addr);

    let refused = ortie(&config, &["auth", "get"]);
    let stdout = String::from_utf8_lossy(&refused.stdout);
    assert_eq!(refused.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.starts_with("Error: Pimalaya's Google application"),
        "{stdout}"
    );

    let agreed = ortie(&config, &["auth", "get", "--agree-data-use"]);
    let stdout = String::from_utf8_lossy(&agreed.stdout);
    assert!(agreed.status.success(), "{stdout}");
    assert!(
        stdout.starts_with("Pimalaya and your Google data\n"),
        "{stdout}"
    );

    let disclosed = stdout.find("https://pimalaya.org/privacy/").unwrap();
    let requested = stdout.find("Open the link in your browser").unwrap();
    assert!(disclosed < requested, "{stdout}");
}

#[test]
fn another_client_signs_in_without_the_disclosure() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_counter();
    let own = "000000000000-own.apps.googleusercontent.com";
    let config = write_account(dir.path(), own, "authorization-code", addr);

    let output = ortie(&config, &["auth", "get"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{stdout}");
    assert!(!stdout.contains("Google data"), "{stdout}");
}

#[test]
fn a_repl_line_is_refused_and_the_next_one_runs() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_counter();
    let config = write_account(dir.path(), PIMALAYA_GOOGLE, "authorization-code", addr);

    let input = b"auth get\naccount list\n";
    let output: Output = ortie_with_input(&config, &["--json", "repl"], input);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut lines = stdout
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap());
    assert_eq!(
        lines.next().unwrap()["code"],
        "agreement-missing",
        "{stdout}"
    );
    assert!(lines.next().unwrap()["accounts"].is_array(), "{stdout}");
}

/// A terminal is asked, and anything but a yes cancels the sign-in.
#[cfg(target_os = "linux")]
#[test]
fn a_terminal_is_asked_and_a_no_cancels() {
    let dir = TempDir::new().unwrap();
    let (addr, count) = start_counter();
    let config = write_account(dir.path(), PIMALAYA_GOOGLE, "device", addr);

    let (status, printed) = in_terminal(&config, "auth get", b"n\n");

    assert_eq!(status, Some(1), "{printed}");
    assert!(
        printed.starts_with("Pimalaya and your Google data"),
        "{printed}"
    );
    assert!(printed.contains("Google Tasks:"), "{printed}");
    assert!(printed.contains("Error: Sign-in cancelled"), "{printed}");
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

/// A yes goes on to the request, here a device authorization the counter
/// drops.
#[cfg(target_os = "linux")]
#[test]
fn a_terminal_yes_goes_on_to_the_request() {
    let dir = TempDir::new().unwrap();
    let (addr, count) = start_counter();
    let config = write_account(dir.path(), PIMALAYA_GOOGLE, "device", addr);

    let (status, printed) = in_terminal(&config, "auth get", b"y\n");

    assert_eq!(status, Some(1), "{printed}");
    assert!(!printed.contains("Sign-in cancelled"), "{printed}");
    assert!(count.load(Ordering::SeqCst) >= 1, "{printed}");
}

/// A session reads its lines off stdin, so a REPL in a terminal asks
/// nothing and refuses the line.
#[cfg(target_os = "linux")]
#[test]
fn a_repl_in_a_terminal_is_refused_without_asking() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_counter();
    let config = write_account(dir.path(), PIMALAYA_GOOGLE, "authorization-code", addr);

    let line = format!(
        "{} -c {} repl",
        env!("CARGO_BIN_EXE_ortie"),
        config.display()
    );
    let mut script = Command::new("script")
        .args(["-qec", &line, "/dev/null"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut stdin = script.stdin.take().unwrap();
    stdin.write_all(b"auth get\nquit\n").unwrap();

    let mut printed = String::new();
    script
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut printed)
        .unwrap();
    script.wait().unwrap();
    drop(stdin);

    assert!(!printed.contains("[y/N]"), "{printed}");
    assert!(
        printed.contains("--agree-data-use to agree to it"),
        "{printed}"
    );
}
