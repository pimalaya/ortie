//! # Token inspect
//!
//! The built binary's `token inspect` over a token file standing for the
//! store: when the token expires, in text relative to now and in JSON as
//! an instant, both read from the issue time and lifetime stored.

mod common;

use std::{
    process::Output,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use humantime::format_rfc3339_seconds;
use serde_json::Value;
use tempfile::TempDir;

use crate::common::*;

/// Runs `token inspect` over a store seeded with `token`, in text then
/// under `--json`.
fn inspect(token: &str) -> (Output, Output) {
    let dir = TempDir::new().unwrap();
    let (config, _) = write_config(dir.path(), "inspect", "client-id = \"c\"", token);

    let text = ortie(&config, &["token", "inspect"]);
    let json = ortie(&config, &["--json", "token", "inspect"]);

    (text, json)
}

/// The value of the `Expires` field of the text rendering.
fn expires_field(out: &Output) -> String {
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.lines().find(|line| line.starts_with("Expires:"));
    let line = line.unwrap_or_else(|| panic!("no Expires field: {stdout}"));
    line["Expires:".len()..].trim().to_owned()
}

/// The object the JSON rendering printed.
fn payload(out: &Output) -> Value {
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn a_token_past_its_lifetime_reads_expired_in_both_forms() {
    let (text, json) = inspect(
        r#"{"access_token":"at","token_type":"Bearer","expires_in":3600,"issued_at":1000}"#,
    );

    assert_eq!(expires_field(&text), "expired");

    let json = payload(&json);
    assert_eq!(json["issuedAt"], "1970-01-01T00:16:40Z", "{json}");
    assert_eq!(json["expiresAt"], "1970-01-01T01:16:40Z", "{json}");
    assert!(json.get("expiresIn").is_none(), "{json}");
}

#[test]
fn a_live_token_expires_its_lifetime_after_its_issue() {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let issued_at = now.as_secs() - 630;
    let token = format!(
        r#"{{"access_token":"at","token_type":"Bearer","expires_in":3600,"issued_at":{issued_at}}}"#
    );

    let (text, json) = inspect(&token);

    assert!(expires_field(&text).starts_with("in 49m"), "{text:?}");

    let expires_at = UNIX_EPOCH + Duration::from_secs(issued_at + 3600);
    let expires_at = format_rfc3339_seconds(expires_at).to_string();
    assert_eq!(payload(&json)["expiresAt"], expires_at);
}

#[test]
fn a_token_stored_without_its_issue_time_expires_unknown() {
    let (text, json) = inspect(r#"{"access_token":"at","token_type":"Bearer","expires_in":3600}"#);

    assert_eq!(expires_field(&text), "unknown");

    let json = payload(&json);
    assert_eq!(json["issuedAt"], Value::Null, "{json}");
    assert_eq!(json["expiresAt"], Value::Null, "{json}");
}
