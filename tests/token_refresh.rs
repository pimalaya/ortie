//! # Token refresh
//!
//! `token refresh` and the refresh `token show` makes on its own, against
//! a mock token endpoint refusing the refresh token as `invalid_grant`,
//! as a server rotating refresh tokens does once another process of the
//! same account refreshed first, and against no endpoint at all.

mod common;

use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    thread,
};

use tempfile::TempDir;

use crate::common::*;

/// The token the account read before refreshing, expired.
const STALE: &str = r#"{"access_token":"at-stale","token_type":"Bearer","expires_in":3600,"issued_at":1000,"refresh_token":"rt-stale"}"#;

/// Starts a token endpoint refusing every request as `invalid_grant`,
/// first writing `rotated` to `token` when given, as a concurrent
/// refresh of the same account would have.
fn start_mock(token: PathBuf, rotated: Option<&'static str>) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0u8; 8192];
            let _ = stream.read(&mut buf);

            if let Some(rotated) = rotated {
                fs::write(&token, rotated).unwrap();
            }

            let body = r#"{"error":"invalid_grant"}"#;
            let resp = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    addr
}

/// An account refreshing at `endpoint`, its store seeded with [`STALE`].
fn write_account(dir: &Path, endpoint: &str) -> (PathBuf, PathBuf) {
    let lines = format!("client-id = \"c\"\nendpoints.token = \"{endpoint}\"");
    write_config(dir, "a", &lines, STALE)
}

#[test]
fn a_refresh_another_process_won_takes_the_token_it_stored() {
    let dir = TempDir::new().unwrap();
    let (config, token) = write_account(dir.path(), "http://127.0.0.1:1/token");
    let rotated = r#"{"access_token":"at-rotated","token_type":"Bearer","expires_in":3600,"refresh_token":"rt-rotated"}"#;
    let addr = start_mock(token.clone(), Some(rotated));

    let config_text = fs::read_to_string(&config).unwrap();
    let config_text =
        config_text.replace("http://127.0.0.1:1/token", &format!("http://{addr}/token"));
    fs::write(&config, config_text).unwrap();

    let show = ortie(&config, &["token", "show"]);
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert_eq!(show.status.code(), Some(0), "{stdout}");
    assert_eq!(stdout, "at-rotated\n");

    fs::write(&token, STALE).unwrap();
    let refresh = ortie(&config, &["--json", "token", "refresh"]);
    let stdout = String::from_utf8_lossy(&refresh.stdout);
    assert_eq!(refresh.status.code(), Some(0), "{stdout}");
    assert_eq!(stored(&token)["access_token"], "at-rotated");
}

#[test]
fn a_refused_refresh_with_an_unchanged_store_stays_refused() {
    let dir = TempDir::new().unwrap();
    let (config, token) = write_account(dir.path(), "http://127.0.0.1:1/token");
    let addr = start_mock(token.clone(), None);

    let config_text = fs::read_to_string(&config).unwrap();
    let config_text =
        config_text.replace("http://127.0.0.1:1/token", &format!("http://{addr}/token"));
    fs::write(&config, config_text).unwrap();

    let failure = json_failure(&ortie(&config, &["--json", "token", "show"]));
    assert_eq!(failure["code"], "grant-invalid", "{failure}");
    assert_eq!(stored(&token)["access_token"], "at-stale");
}

/// A refresh `token show` makes on its own says so when the endpoint
/// cannot be reached, rather than a bare connection error.
#[test]
fn an_unreachable_refresh_is_named_a_refresh() {
    let dir = TempDir::new().unwrap();
    let (config, _) = write_account(dir.path(), "http://127.0.0.1:1/token");

    let failure = json_failure(&ortie(&config, &["--json", "token", "show"]));
    assert_eq!(failure["error"], "Refresh access token error", "{failure}");
}
