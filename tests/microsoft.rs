//! # Microsoft live suite
//!
//! The binary against Entra. `anonymous` runs the public client of
//! config.sample.toml and made-up secrets, touching no account.
//! `credentialed` runs a confidential registration of the test tenant,
//! read from `ORTIE_MICROSOFT_TENANT_ID`, `ORTIE_MICROSOFT_CLIENT_ID` and
//! `ORTIE_MICROSOFT_CLIENT_SECRET`, the JWT kind from
//! `ORTIE_MICROSOFT_CLIENT_KEY` and `ORTIE_MICROSOFT_CLIENT_CERT` (PEM) or
//! their `_FILE` paths.
//!
//! Run with `cargo test --test microsoft -- --ignored`. A failure never
//! shows a token: the credentialed assertions print stderr and the exit
//! code only.

mod common;

mod anonymous {
    use std::fs;

    use serde_json::Value;
    use tempfile::TempDir;

    use crate::common::*;

    /// The device authorization endpoint of the sample's tenant.
    const DEVICE_AUTHORIZATION: &str =
        "https://login.microsoftonline.com/common/oauth2/v2.0/devicecode";

    #[test]
    #[ignore = "requires network and --ignored"]
    fn device_auth_get_json() {
        let dir = TempDir::new().unwrap();
        let section = sample_section("Microsoft Graph", &[]);
        let lines = format!(
            "{section}grant = \"device\"\nendpoints.device-authorization = \"{DEVICE_AUTHORIZATION}\""
        );
        let (config, token) = write_config(dir.path(), "microsoft", &lines, "");

        let out = ortie(&config, &["--json", "auth", "get"]);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{stdout}");

        let json: Value = serde_json::from_slice(&out.stdout).unwrap();
        for key in ["deviceCode", "userCode", "verificationUri"] {
            assert!(json[key].is_string(), "no {key} in {json}");
        }

        assert_eq!(fs::read_to_string(token).unwrap(), "");
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn client_credentials_rejected() {
        let dir = TempDir::new().unwrap();
        let client_id = sample_section("Microsoft Graph", &[])
            .lines()
            .find_map(|line| line.strip_prefix("client-id = "))
            .unwrap()
            .to_owned();
        let lines = format!(
            "grant = \"client-credentials\"\nclient-id = {client_id}\nclient-secret.raw = \"made-up\"\nendpoints.token = \"https://login.microsoftonline.com/pimalaya.onmicrosoft.com/oauth2/v2.0/token\"\nscopes = [\"https://graph.microsoft.com/.default\"]"
        );
        let (config, token) = write_config(dir.path(), "microsoft", &lines, "");

        let error = json_failure(&ortie(&config, &["--json", "auth", "get"]));

        let message = error["error"].as_str().unwrap_or_default();
        assert!(message.contains("InvalidClient"), "{error}");
        assert_eq!(fs::read_to_string(token).unwrap(), "");
    }
}

mod credentialed {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Output,
        time::{SystemTime, UNIX_EPOCH},
    };

    use tempfile::TempDir;

    use crate::common::*;

    /// A token the server never issued, expired since 1970.
    const EXPIRED: &str =
        r#"{"access_token":"expired","token_type":"Bearer","expires_in":3600,"issued_at":1000}"#;

    /// Writes a client credentials account of the test registration,
    /// authenticated by `credential`, returning the config and token
    /// paths.
    fn write_account(dir: &Path, grant: &str, credential: &str) -> (PathBuf, PathBuf) {
        let tenant = require_env("ORTIE_MICROSOFT_TENANT_ID");
        let client_id = require_env("ORTIE_MICROSOFT_CLIENT_ID");
        let lines = format!(
            "grant = \"{grant}\"\nclient-id = \"{client_id}\"\n{credential}\nendpoints.token = \"https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token\"\nscopes = [\"https://graph.microsoft.com/.default\"]"
        );

        write_config(dir, "microsoft", &lines, "")
    }

    /// Writes the account authenticated by the client secret, which
    /// reaches the binary through `printenv`.
    fn write_secret_account(dir: &Path) -> (PathBuf, PathBuf) {
        require_env("ORTIE_MICROSOFT_CLIENT_SECRET");
        let credential =
            "client-secret.command = [\"printenv\", \"ORTIE_MICROSOFT_CLIENT_SECRET\"]";
        write_account(dir, "client-credentials", credential)
    }

    /// Writes the account authenticated by a JWT signed with `key`,
    /// naming `certificate`.
    fn write_jwt_account(dir: &Path, key: &Path, certificate: &Path) -> (PathBuf, PathBuf) {
        let credential = format!(
            "client-key = \"{}\"\nclient-certificate = \"{}\"",
            key.display(),
            certificate.display()
        );
        write_account(dir, "client-credentials-jwt", &credential)
    }

    /// Runs `token show`, failing on stderr and the exit code only, and
    /// returns the token it printed.
    fn token_show(config: &Path) -> String {
        let out = ortie(config, &["token", "show"]);
        assert!(out.status.success(), "{}", failure(&out));
        String::from_utf8(out.stdout).unwrap().trim_end().to_owned()
    }

    /// What a failed run shows: its exit code and stderr, never stdout,
    /// which holds the token on success.
    fn failure(out: &Output) -> String {
        let stderr = String::from_utf8_lossy(&out.stderr);
        format!("exit {:?}, stderr: {stderr}", out.status.code())
    }

    /// Whether `token` holds the access token `shown`, issued within the
    /// last minute.
    fn holds_fresh(token: &Path, shown: &str) -> bool {
        let stored = stored(token);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let issued_at = stored["issued_at"].as_u64().unwrap_or_default();
        stored["access_token"] == shown && now.as_secs().abs_diff(issued_at) < 60
    }

    #[test]
    #[ignore = "requires Microsoft credentials and --ignored"]
    fn client_credentials_token_show() {
        let dir = TempDir::new().unwrap();
        let (config, token) = write_secret_account(dir.path());

        let shown = token_show(&config);
        assert!(holds_fresh(&token, &shown), "the token is not stored");

        let snapshot = fs::read(&token).unwrap();
        let again = token_show(&config);
        assert!(again == shown, "a valid token is not served again");
        assert!(fs::read(&token).unwrap() == snapshot, "storage was written");

        fs::write(&token, EXPIRED).unwrap();
        let fresh = token_show(&config);
        assert!(fresh != "expired", "the expired token is served");
        assert!(holds_fresh(&token, &fresh), "the token is not acquired");
    }

    #[test]
    #[ignore = "requires Microsoft credentials and --ignored"]
    fn client_credentials_jwt_token_show() {
        let dir = TempDir::new().unwrap();
        let key = env_file(dir.path(), "ORTIE_MICROSOFT_CLIENT_KEY");
        let certificate = env_file(dir.path(), "ORTIE_MICROSOFT_CLIENT_CERT");
        let (Some(key), Some(certificate)) = (key, certificate) else {
            panic!("set ORTIE_MICROSOFT_CLIENT_KEY and ORTIE_MICROSOFT_CLIENT_CERT");
        };
        let (config, token) = write_jwt_account(dir.path(), &key, &certificate);

        let shown = token_show(&config);
        assert!(holds_fresh(&token, &shown), "the token is not stored");

        // A second mint signs a new assertion, which Entra must accept.
        let out = ortie(&config, &["token", "refresh"]);
        assert!(out.status.success(), "{}", failure(&out));
    }

    #[test]
    #[ignore = "requires Microsoft credentials and --ignored"]
    fn client_credentials_jwt_renewal_hint() {
        let dir = TempDir::new().unwrap();
        let key = dir.path().join("key.pem");
        fs::write(&key, KEY_PEM).unwrap();
        let certificate = dir.path().join("certificate.der");
        fs::write(&certificate, b"unregistered-certificate").unwrap();
        let (config, token) = write_jwt_account(dir.path(), &key, &certificate);

        let error = json_failure(&ortie(&config, &["--json", "auth", "get"]));

        let message = error["error"].as_str().unwrap_or_default();
        assert!(message.contains("InvalidClient"), "{error}");

        let sources = error["sources"].as_array().unwrap();
        let hint = "The certificate credential may be expired and need renewal";
        assert!(sources.iter().any(|source| source == hint), "{error}");
        assert_eq!(fs::read_to_string(token).unwrap(), "");
    }
}
