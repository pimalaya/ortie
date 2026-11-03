//! # REPL
//!
//! End-to-end coverage of a piped REPL session, driving the real binary
//! over stdin and stdout.

use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

use serde_json::Value;
use tempfile::TempDir;

/// A piped `--json` session prints every result and every failure on
/// stdout as one JSON object per line, and keeps going after each
/// failure.
///
/// The token endpoint points at a closed port, so `auth resume` fails
/// without a server, and an unknown subcommand fails at parsing.
#[test]
fn piped_json_session_prints_errors_on_stdout() {
    let dir = TempDir::new().unwrap();
    let token = dir.path().join("token.json");
    std::fs::write(
        &token,
        r#"{"access_token":"at-test","token_type":"Bearer"}"#,
    )
    .unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(
        &config,
        format!(
            r#"
[accounts.device]
default = true
client-id = "c"
grant = "device"
endpoints.device-authorization = "http://127.0.0.1:1/devicecode"
endpoints.token = "http://127.0.0.1:1/token"
storage.read.command = ["cat", "{t}"]
storage.write.command = ["tee", "{t}"]
"#,
            t = token.display()
        ),
    )
    .unwrap();
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_ortie"));

    let mut child = Command::new(&bin)
        .args(["-c", config.to_str().unwrap(), "--json", "repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"token show\ntoken frobnicate\nauth resume dc-test\ntoken show\nquit\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();

    assert!(out.status.success(), "{out:?}");

    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(lines.len(), 4, "stdout: {stdout}");
    assert_eq!(lines[0]["accessToken"], "at-test");
    assert!(lines[1]["error"].is_string(), "stdout: {stdout}");
    assert!(lines[2]["error"].is_string(), "stdout: {stdout}");
    assert_eq!(lines[3]["accessToken"], "at-test");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("rror"), "stderr: {stderr}");
}

/// A refresh whose storage write fails still leaves the session on the
/// tokens the server issued: a server rotating refresh tokens has
/// already invalidated the stored one.
#[test]
fn a_failed_refresh_write_keeps_the_issued_token_in_session() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        let Some(Ok(mut stream)) = listener.incoming().next() else {
            return;
        };

        let mut raw = Vec::new();
        let mut buf = [0u8; 8192];

        // NOTE: the refresh body is small, so the head and the body
        // arrive before the answer is needed; read until the client
        // waits.
        while !raw.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = stream.read(&mut buf).unwrap_or(0);
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&buf[..n]);
        }

        let body =
            r#"{"access_token":"at-fresh","token_type":"Bearer","refresh_token":"rt-rotated"}"#;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(resp.as_bytes());
    });

    let dir = TempDir::new().unwrap();
    let token = dir.path().join("token.json");
    std::fs::write(
        &token,
        r#"{"access_token":"at-stale","token_type":"Bearer","refresh_token":"rt-old"}"#,
    )
    .unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(
        &config,
        format!(
            r#"
[accounts.rotating]
default = true
client-id = "c"
endpoints.token = "http://{addr}/token"
storage.read.command = ["cat", "{t}"]
storage.write.command = ["false"]
"#,
            t = token.display()
        ),
    )
    .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_ortie"))
        .args(["-c", config.to_str().unwrap(), "--json", "repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"token refresh\ntoken show\nquit\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(lines.len(), 2, "stdout: {stdout}");
    assert!(lines[0]["error"].is_string(), "stdout: {stdout}");
    assert_eq!(lines[1]["accessToken"], "at-fresh", "stdout: {stdout}");
}
