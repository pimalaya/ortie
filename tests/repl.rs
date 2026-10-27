//! # REPL
//!
//! End-to-end coverage of a piped REPL session, driving the real binary
//! over stdin and stdout.

use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
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
