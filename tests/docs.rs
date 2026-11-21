//! # README terminal block
//!
//! The command line the README shows under its description, run against
//! a stored token, prints the JSON the README shows after it (readme-006).

mod common;

use std::fs;

use serde_json::Value;
use tempfile::TempDir;

use crate::common::*;

/// The token the README's block inspects, issued at 2026-11-21T08:00:00Z
/// for an hour.
const TOKEN: &str = r#"{"access_token":"made-up","token_type":"Bearer","expires_in":3600,"issued_at":1795248000,"refresh_token":"made-up","scope":"https://mail.google.com/ https://www.googleapis.com/auth/calendar"}"#;

/// The content of the first fenced block of `lang` in `text`, and the
/// rest of `text` after it.
fn fenced<'a>(text: &'a str, lang: &str) -> (&'a str, &'a str) {
    let open = format!("```{lang}\n");
    let start = text.find(&open).expect("a fenced block") + open.len();
    let end = start + text[start..].find("\n```").expect("a closing fence");
    (&text[start..end], &text[end + 4..])
}

#[test]
fn the_readme_header_block_prints_what_it_shows() {
    let readme = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    let (command, rest) = fenced(&readme, "sh");
    let (shown, _) = fenced(rest, "json");
    let args: Vec<&str> = command
        .strip_prefix("$ ortie ")
        .expect("an ortie command line")
        .split_whitespace()
        .collect();

    let dir = TempDir::new().unwrap();
    let (config, _) = write_config(dir.path(), "example", "client-id = \"c\"", TOKEN);
    let out = ortie(&config, &args);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stdout}");

    let printed: Value = serde_json::from_slice(&out.stdout).unwrap();
    let shown: Value = serde_json::from_str(shown).unwrap();
    assert_eq!(printed, shown);
}
