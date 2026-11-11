//! # Token revocation
//!
//! End-to-end coverage of `token revoke`, driving the real binary
//! against a local mock revocation endpoint (RFC 7009).

mod common;

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use serde_json::Value;
use tempfile::TempDir;

use crate::common::*;

/// Starts a mock endpoint answering every POST with `status` and
/// `response`, capturing each form body.
fn start_mock(status: u16, response: &'static str) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let bodies_t = Arc::clone(&bodies);

    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut raw = Vec::new();
            let mut buf = [0u8; 8192];

            let (head, mut body) = loop {
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break (String::from_utf8_lossy(&raw).into_owned(), Vec::new());
                }
                raw.extend_from_slice(&buf[..n]);

                if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&raw[..pos]).into_owned();
                    break (head, raw[pos + 4..].to_vec());
                }
            };

            let content_length: usize = head
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|value| value.trim().parse().ok())
                })
                .unwrap_or(0);

            while body.len() < content_length {
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break;
                }
                body.extend_from_slice(&buf[..n]);
            }

            bodies_t
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&body).into_owned());

            let resp = format!(
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    (addr, bodies)
}

/// Writes a config revoking against the mock, with file-backed storage
/// seeded with `token`, returning the config and token paths.
fn write_revoke_config(dir: &Path, addr: SocketAddr, token: &str) -> (PathBuf, PathBuf) {
    let lines = format!(
        "client-id = \"app-id\"\nendpoints.token = \"http://{addr}/token\"\nendpoints.revocation = \"http://{addr}/revoke\""
    );

    write_config(dir, "revoke", &lines, token)
}

const TOKEN_WITH_REFRESH: &str =
    r#"{"access_token":"at","token_type":"Bearer","refresh_token":"rt","issued_at":1000}"#;

#[test]
fn the_refresh_token_is_revoked_and_storage_kept() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock(200, "");
    let (config, token) = write_revoke_config(dir.path(), addr, TOKEN_WITH_REFRESH);

    let output = ortie(&config, &["--json", "token", "revoke"]);
    assert!(output.status.success(), "{output:?}");

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json, serde_json::json!({ "revoked": "refreshToken" }));

    let bodies = bodies.lock().unwrap();
    assert_eq!(
        bodies.as_slice(),
        ["token=rt&token_type_hint=refresh_token&client_id=app-id"]
    );

    assert_eq!(std::fs::read_to_string(token).unwrap(), TOKEN_WITH_REFRESH);
}

#[test]
fn without_a_refresh_token_the_access_token_is_revoked() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock(200, "");
    let (config, _) = write_revoke_config(
        dir.path(),
        addr,
        r#"{"access_token":"at","token_type":"Bearer"}"#,
    );

    let output = ortie(&config, &["--json", "token", "revoke"]);
    assert!(output.status.success(), "{output:?}");

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["revoked"], "accessToken");
    assert_eq!(
        bodies.lock().unwrap().as_slice(),
        ["token=at&token_type_hint=access_token&client_id=app-id"]
    );
}

#[test]
fn a_rejected_revocation_fails_with_its_code() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock(400, r#"{"error":"unsupported_token_type"}"#);
    let (config, _) = write_revoke_config(dir.path(), addr, TOKEN_WITH_REFRESH);

    let output = ortie(&config, &["token", "revoke"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("UnsupportedTokenType"), "{stdout}");
}
