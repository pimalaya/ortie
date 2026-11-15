//! # Failure codes
//!
//! The codes of the failures Ortie meets before any server answers: an
//! account it cannot resolve, a token the account does not store, an
//! endpoint it does not configure, and an authorization redirect carrying
//! another state than the request.

mod common;

use std::{
    fs,
    path::{Path, PathBuf},
};

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

/// Runs `token show` under `--json` against `config` with the `-a`
/// arguments `account`, answering the failure object.
fn account_failure(config: &str, account: &[&str]) -> Value {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, config).unwrap();

    let args: Vec<_> = ["--json"]
        .iter()
        .chain(account)
        .chain(&["token", "show"])
        .copied()
        .collect();

    json_failure(&ortie(&path, &args))
}

/// An account of `name`, marked default when `default` is set, storing
/// no token.
fn account(name: &str, default: bool) -> String {
    account_reading(name, default, "\"true\"")
}

/// An account of `name`, marked default when `default` is set, reading
/// its token with the TOML command `read`.
fn account_reading(name: &str, default: bool, read: &str) -> String {
    format!(
        "[accounts.{name}]\ndefault = {default}\nclient-id = \"c\"\n\
         storage.read.command = {read}\nstorage.write.command = \"true\"\n"
    )
}

#[test]
fn an_account_resolution_failure_carries_its_code() {
    let config = account("a", true) + &account("b", false);
    let failure = account_failure(&config, &["-a", "nope"]);
    assert_eq!(failure["code"], "account-missing", "{failure}");

    let config = account("a", false) + &account("b", false);
    let failure = account_failure(&config, &[]);
    assert_eq!(failure["code"], "account-default-missing", "{failure}");

    let config = account("a", true) + &account("b", true);
    let failure = account_failure(&config, &[]);
    assert_eq!(failure["code"], "account-default-several", "{failure}");
}

/// `default` names the account called so, never the one marked default.
#[test]
fn default_is_an_account_name_like_any_other() {
    let config = account("a", true);
    let failure = account_failure(&config, &["-a", "default"]);
    assert_eq!(failure["code"], "account-missing", "{failure}");

    let message = "Account `default` not found, the configuration holds: a, omit `-a` \
                   to take the account marked default";
    assert_eq!(failure["error"], message, "{failure}");

    // NOTE: `a` stores a token, so only taking `default` fails.
    let read = format!("[\"echo\", '{TOKEN}']");
    let config = account("default", false) + &account_reading("a", true, &read);
    let failure = account_failure(&config, &["-a", "default"]);
    assert_eq!(failure["code"], "token-missing", "{failure}");
}
