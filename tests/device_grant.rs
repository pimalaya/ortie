//! # Device grant
//!
//! End-to-end coverage of the device authorization grant, driving the
//! real binary against a local mock authorization server.

mod common;

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use serde_json::Value;
use tempfile::TempDir;

use crate::common::*;

/// Starts a mock authorization server issuing a device code.
///
/// It answers the first token poll with `authorization_pending` and the
/// next with a token, counting the polls it received.
fn start_mock() -> (SocketAddr, Arc<AtomicUsize>, thread::JoinHandle<()>) {
    start_mock_with(r#""expires_in":60,"interval":1"#, None)
}

/// The mock of [`start_mock`], its device code answer carrying `timing`
/// as its lifetime and interval members, and every token poll answered
/// with the error body `refusal` when given.
fn start_mock_with(
    timing: &'static str,
    refusal: Option<&'static str>,
) -> (SocketAddr, Arc<AtomicUsize>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let polls = Arc::new(AtomicUsize::new(0));
    let polls_t = Arc::clone(&polls);
    let handle = thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0u8; 8192];
            let Ok(n) = stream.read(&mut buf) else {
                continue;
            };
            let req = String::from_utf8_lossy(&buf[..n]);
            let path = req
                .lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1))
                .unwrap_or("/");
            let (status, body): (u16, String) = if path.starts_with("/devicecode") {
                (
                    200,
                    format!(
                        r#"{{"device_code":"dc-test","user_code":"USER","verification_uri":"http://{addr}/d",{timing}}}"#
                    ),
                )
            } else if path.starts_with("/token") {
                let first = polls_t.fetch_add(1, Ordering::SeqCst) == 0;
                if let Some(refusal) = refusal {
                    (400, refusal.into())
                } else if first {
                    (400, r#"{"error":"authorization_pending"}"#.into())
                } else {
                    (
                        200,
                        r#"{"access_token":"at-test","token_type":"Bearer","expires_in":3600}"#
                            .into(),
                    )
                }
            } else {
                (404, r#"{"error":"not_found"}"#.into())
            };
            let resp = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    thread::sleep(Duration::from_millis(20));
    (addr, polls, handle)
}

/// Writes a device account reaching the mock at `addr`, with
/// file-backed storage, returning the config and token paths.
fn write_device_config(dir: &Path, addr: SocketAddr) -> (PathBuf, PathBuf) {
    let lines = format!(
        "client-id = \"c\"\ngrant = \"device\"\nendpoints.device-authorization = \"http://{addr}/devicecode\"\nendpoints.token = \"http://{addr}/token\""
    );

    write_config(dir, "device", &lines, "")
}

#[test]
fn auth_get_json_then_resume_stores_token() {
    let (addr, polls, _h) = start_mock();
    let dir = TempDir::new().unwrap();
    let (config, token) = write_device_config(dir.path(), addr);

    let get = ortie(&config, &["--json", "auth", "get"]);
    assert!(get.status.success(), "{get:?}");
    let v: Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(v["deviceCode"], "dc-test");
    assert_eq!(polls.load(Ordering::SeqCst), 0);

    let resume = ortie(&config, &["auth", "resume", "dc-test"]);
    assert!(resume.status.success(), "{resume:?}");
    assert!(polls.load(Ordering::SeqCst) >= 2);
    assert_eq!(stored(&token)["access_token"], "at-test");
}

#[test]
fn auth_get_reports_the_lifetime_and_the_interval_bounded() {
    let (addr, _, _h) = start_mock_with(r#""expires_in":86400,"interval":0"#, None);
    let dir = TempDir::new().unwrap();
    let (config, _) = write_device_config(dir.path(), addr);

    let get = ortie(&config, &["--json", "auth", "get"]);
    assert!(get.status.success(), "{get:?}");
    let v: Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(v["expiresIn"], 3600, "{v}");
    assert_eq!(v["interval"], 1, "{v}");
}

/// The whole grant runs in one REPL session.
///
/// A non-interactive auth get prints the device response and hands off,
/// auth resume polls and stores, and token show serves the freshly
/// issued token from the in-memory account.
#[test]
fn repl_auth_get_resume_then_token_show() {
    let (addr, polls, _h) = start_mock();
    let dir = TempDir::new().unwrap();
    let (config, _) = write_device_config(dir.path(), addr);

    let input = b"auth get\nauth resume dc-test\ntoken show\nquit\n";
    let out = ortie_with_input(&config, &["repl"], input);

    assert!(out.status.success(), "{out:?}");
    assert!(polls.load(Ordering::SeqCst) >= 2);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("at-test"),
        "token show should print the issued token; stdout: {stdout}"
    );
}

/// Runs `auth resume` on a device code whose every poll the server
/// refuses with `refusal`, answering the failure object.
fn resume_refused(refusal: &'static str) -> Value {
    let (addr, _, _h) = start_mock_with(r#""expires_in":60,"interval":1"#, Some(refusal));
    let dir = TempDir::new().unwrap();
    let (config, token) = write_device_config(dir.path(), addr);

    let failure = json_failure(&ortie(&config, &["--json", "auth", "resume", "dc-test"]));
    assert!(std::fs::read_to_string(token).unwrap().is_empty());
    failure
}

/// The user denying the device in the browser is coded
/// `permission-denied`.
#[test]
fn a_denied_device_is_coded_permission_denied() {
    let failure = resume_refused(r#"{"error":"access_denied"}"#);

    assert_eq!(failure["code"], "permission-denied", "{failure}");
    let message = "Issue access token error (code access_denied)";
    assert_eq!(failure["error"], message, "{failure}");
}

/// A device code the server reports expired is coded
/// `device-code-expired`.
#[test]
fn an_expired_device_code_is_coded_device_code_expired() {
    let failure = resume_refused(r#"{"error":"expired_token"}"#);

    assert_eq!(failure["code"], "device-code-expired", "{failure}");
    let message = "Issue access token error (code expired_token)";
    assert_eq!(failure["error"], message, "{failure}");
}

/// The printed resume ends its options with `--`, so a device code
/// beginning with `-` is still read as the code.
#[test]
fn the_printed_resume_takes_a_code_beginning_with_a_dash() {
    let (addr, _polls, _h) = start_mock();
    let dir = TempDir::new().unwrap();
    let (config, token) = write_device_config(dir.path(), addr);

    let get = ortie(&config, &["auth", "get"]);
    assert!(get.status.success(), "{get:?}");
    let stdout = String::from_utf8_lossy(&get.stdout);
    let line = stdout.lines().find(|l| l.contains("auth resume")).unwrap();
    assert!(line.ends_with(" auth resume -- 'dc-test'"), "{line}");

    let resume = ortie(&config, &["auth", "resume", "--", "-dc-test"]);
    assert!(resume.status.success(), "{resume:?}");
    assert_eq!(stored(&token)["access_token"], "at-test");
}
