//! # Google live suite
//!
//! The binary against Google, on the Google section of config.sample.toml
//! with a made-up client. Ortie has no headless grant Google accepts, so
//! the suite is `anonymous` only and touches no account.
//!
//! Run with `cargo test --test google -- --ignored`.

mod common;

mod anonymous {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use tempfile::TempDir;

    use crate::common::*;

    /// A token the server never issued.
    const MADE_UP: &str = r#"{"access_token":"made-up","token_type":"Bearer","refresh_token":"made-up","issued_at":1000}"#;

    /// Writes the sample's Google account on a made-up client, its
    /// storage seeded with `token`.
    fn write_account(dir: &Path, token: &str) -> (PathBuf, PathBuf) {
        let client = [
            (
                "client-id",
                "\"000000000000-made-up.apps.googleusercontent.com\"",
            ),
            ("client-secret.raw", "\"made-up\""),
        ];
        write_config(dir, "google", &sample_section("Google", &client), token)
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn auth_resume_rejected() {
        let dir = TempDir::new().unwrap();
        let (config, token) = write_account(dir.path(), "");

        let args = [
            "--json",
            "auth",
            "resume",
            "--state=state",
            "--pkce=verifier-verifier-verifier-verifier-verifier",
            "--redirect-uri=http://127.0.0.1:1/",
            "http://127.0.0.1:1/?code=made-up&state=state",
        ];
        let error = json_failure(&ortie(&config, &args));

        let message = error["error"].as_str().unwrap_or_default();
        assert!(message.contains("InvalidClient"), "{error}");
        assert_eq!(fs::read_to_string(token).unwrap(), "");
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn token_revoke_rejected() {
        let dir = TempDir::new().unwrap();
        let (config, token) = write_account(dir.path(), MADE_UP);

        let error = json_failure(&ortie(&config, &["--json", "token", "revoke"]));

        // Google answers `invalid_token`, a code outside RFC 7009's.
        let message = error["error"].as_str().unwrap_or_default();
        assert!(message.contains("Revoke token error"), "{error}");
        assert_eq!(fs::read_to_string(token).unwrap(), MADE_UP);
    }
}
