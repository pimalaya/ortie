//! # Fastmail live suite
//!
//! The binary against Fastmail, on the Fastmail section of
//! config.sample.toml. Only a browser consent issues a Fastmail token, so
//! the suite is `anonymous` only and touches no account.
//!
//! Run with `cargo test --test fastmail -- --ignored`.

mod common;

mod anonymous {
    use std::fs;

    use tempfile::TempDir;

    use crate::common::*;

    #[test]
    #[ignore = "requires network and --ignored"]
    fn auth_resume_rejected() {
        let dir = TempDir::new().unwrap();
        let section = sample_section("Fastmail", &[]);
        let (config, token) = write_config(dir.path(), "fastmail", &section, "");

        let args = [
            "--json",
            "auth",
            "resume",
            "--state=state",
            "--pkce=verifier-verifier-verifier-verifier-verifier",
            "--redirect-uri=http://localhost/",
            "http://localhost/?code=made-up&state=state",
        ];
        let error = json_failure(&ortie(&config, &args));

        // Fastmail tells its failures apart by description only.
        let sources = error["sources"].as_array().unwrap();
        assert!(sources.iter().any(|source| source != ""), "{error}");
        assert_eq!(fs::read_to_string(token).unwrap(), "");
    }
}
