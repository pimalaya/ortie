//! # Failure codes
//!
//! The codes of the failures Ortie meets before any server answers: a
//! token the account does not store, an endpoint it does not configure,
//! and an authorization redirect carrying another state than the request.

mod common;

use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::TempDir;

use crate::common::*;

/// Writes an authorization code account whose endpoints point at a
/// closed port, storing `token`.
fn write_account(dir: &Path, extra: &str, token: &str) -> (PathBuf, PathBuf) {
    let lines = format!("client-id = \"c\"\n{extra}");
    write_config(dir, "codes", &lines, token)
}

/// Runs `args` under `--json` against an account of `extra` lines
/// storing `token`, answering the failure object.
fn failure_of(extra: &str, token: &str, args: &[&str]) -> Value {
    let dir = TempDir::new().unwrap();
    let (config, _) = write_account(dir.path(), extra, token);
    let args: Vec<_> = ["--json"].iter().chain(args).copied().collect();

    json_failure(&ortie(&config, &args))
}

const TOKEN: &str = r#"{"access_token":"at","token_type":"Bearer"}"#;

#[test]
fn a_token_the_account_does_not_store_is_coded_token_missing() {
    let failure = failure_of("", "", &["token", "show"]);
    assert_eq!(failure["code"], "token-missing", "{failure}");

    let extra = "endpoints.token = \"http://127.0.0.1:1/token\"";
    let failure = failure_of(extra, TOKEN, &["token", "refresh"]);
    assert_eq!(failure["code"], "token-missing", "{failure}");
    assert_eq!(failure["error"], "Missing refresh token", "{failure}");
}

#[test]
fn an_endpoint_the_account_does_not_configure_is_coded_endpoint_missing() {
    let failure = failure_of("", TOKEN, &["token", "revoke"]);
    assert_eq!(failure["code"], "endpoint-missing", "{failure}");

    let message = "Missing endpoints.revocation in the account config";
    assert_eq!(failure["error"], message, "{failure}");
}

#[test]
fn a_redirect_of_another_state_is_coded_state_mismatch() {
    let extra = "endpoints.token = \"http://127.0.0.1:1/token\"";

    for uri in [
        "http://127.0.0.1:1/?code=c&state=other",
        "http://127.0.0.1:1/?code=c",
    ] {
        let args = ["auth", "resume", "--state=state", uri];
        let failure = failure_of(extra, "", &args);
        assert_eq!(failure["code"], "state-mismatch", "{uri}: {failure}");
    }
}
