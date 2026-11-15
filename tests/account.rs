//! # Accounts
//!
//! The built binary's `account list` and `account check`, the latter
//! against a local listener standing for the authorization server and a
//! token file standing for the store, and its failure object for an
//! account it cannot take or a key the configuration no longer reads.

use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::Path,
    process::{Command, Output, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use serde_json::{Value, json};
use tempfile::TempDir;

/// The token the store holds when the test seeds it.
const TOKEN: &str = r#"{"access_token":"at-test","token_type":"Bearer"}"#;

/// What the listener answers a `HEAD` on an OAuth endpoint, usually.
const NOT_ALLOWED: &str =
    "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

/// Runs the binary with `args` over the configuration `toml`, in a home
/// of its own, its stdin closed.
fn run(home: &Path, toml: &str, args: &[&str]) -> Output {
    let config = home.join("config.toml");
    fs::write(&config, toml).unwrap();

    Command::new(env!("CARGO_BIN_EXE_ortie"))
        .args(["-c", config.to_str().unwrap()])
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// What `output` printed as JSON, once it exited with `status`.
fn printed(output: &Output, status: i32) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(status), "{stdout}");
    serde_json::from_str(&stdout).unwrap()
}

/// An account `name` of `lines`, its token read from token.json in
/// `home`, seeded with `token`, and written to written.json there.
fn account(home: &Path, name: &str, lines: &str, token: &str) -> String {
    let token_path = home.join("token.json");
    fs::write(&token_path, token).unwrap();

    let read = token_path.display();
    let write = home.join("written.json");
    let write = write.display();

    format!(
        "[accounts.{name}]\ndefault = true\nclient-id = \"c\"\n\
         client-secret.raw = \"s3cret\"\n{lines}\n\
         storage.read.command = [\"cat\", \"{read}\"]\n\
         storage.write.command = [\"tee\", \"{write}\"]\n"
    )
}

/// A listener answering every connection with `answer` once it read the
/// request head, which it keeps.
fn listen(answer: &'static str) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let heads = Arc::new(Mutex::new(Vec::new()));
    let kept = heads.clone();

    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            let mut head = Vec::new();
            let mut buf = [0u8; 512];

            while !head.windows(4).any(|w| w == b"\r\n\r\n") {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => head.extend_from_slice(&buf[..n]),
                }
            }

            kept.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&head).into_owned());
            let _ = stream.write_all(answer.as_bytes());
        }
    });

    (addr, heads)
}

#[test]
fn account_list_prints_every_account_sorted_with_no_backend() {
    let home = TempDir::new().unwrap();
    let toml = account(home.path(), "work", "", TOKEN)
        + "[accounts.home]\nclient-id = \"c\"\n\
           storage.read.command = \"exit 127\"\n\
           storage.write.command = \"exit 127\"\n";

    let output = run(home.path(), &toml, &["--json", "account", "list"]);
    let expected = json!({
        "accounts": [
            { "name": "home", "default": false, "backends": [], "notBuilt": [] },
            { "name": "work", "default": true, "backends": [], "notBuilt": [] },
        ]
    });
    assert_eq!(printed(&output, 0), expected);

    let output = run(home.path(), &toml, &["account", "ls"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{text}");

    let rows: Vec<Vec<&str>> = text
        .lines()
        .map(|l| l.split_whitespace().collect())
        .collect();
    assert_eq!(rows, [vec!["home", "-", "-"], vec!["work", "-", "default"]]);
}

/// The check runs the read command and sends each endpoint one `HEAD`
/// carrying nothing but its target and host, any HTTP answer counting;
/// the write command never runs.
#[test]
fn account_check_reads_the_storage_and_sends_each_endpoint_a_head() {
    let home = TempDir::new().unwrap();
    let (addr, heads) = listen(NOT_ALLOWED);
    let lines = format!(
        "endpoints.authorization = \"http://{addr}/authorize\"\n\
         endpoints.token = \"http://{addr}/token?tenant=x\"\n\
         endpoints.revocation = \"http://{addr}/revoke\"\n\
         endpoints.redirection = \"http://127.0.0.1:1/callback\""
    );
    let toml = account(home.path(), "work", &lines, TOKEN);

    let output = run(home.path(), &toml, &["--json", "account", "check"]);
    let expected = json!({
        "account": "work",
        "backends": [
            { "backend": "storage", "ok": true, "code": null, "error": null },
            { "backend": "endpoints", "ok": true, "code": null, "error": null },
        ]
    });
    assert_eq!(printed(&output, 0), expected);

    let head = |target: &str| {
        format!("HEAD {target} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n")
    };
    let heads = heads.lock().unwrap().clone();
    assert_eq!(
        heads,
        [head("/authorize"), head("/token?tenant=x"), head("/revoke")]
    );

    assert!(!home.path().join("written.json").exists());
}

/// Both rows print whatever the first one did, and a failed one exits 3
/// with the report, each failure coded as any command would code it.
#[test]
fn a_failed_row_exits_3_and_keeps_the_other() {
    let home = TempDir::new().unwrap();
    let lines = "grant = \"client-credentials\"\n\
                 endpoints.token = \"http://127.0.0.1:1/token\"";
    let toml = account(home.path(), "work", lines, "");

    let output = run(home.path(), &toml, &["--json", "account", "check"]);
    let json = printed(&output, 3);
    let storage = &json["backends"][0];
    let endpoints = &json["backends"][1];

    assert_eq!(storage["ok"], false, "{json}");
    assert_eq!(storage["code"], "token-missing", "{json}");

    assert_eq!(endpoints["ok"], false, "{json}");
    assert_eq!(endpoints["code"], Value::Null, "{json}");
    let error = endpoints["error"].as_str().unwrap();
    let reach = "Cannot reach endpoints.token at http://127.0.0.1:1/token";
    assert!(error.starts_with(reach), "{error}");

    let output = run(home.path(), &toml, &["account", "check"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(3), "{text}");
    assert!(text.contains("storage:"), "{text}");
    assert!(text.contains("FAIL (No access token stored"), "{text}");
}

/// An endpoint the grant needs is checked configured before any request,
/// coded as `auth get` codes it.
#[test]
fn an_endpoint_the_grant_needs_is_coded_endpoint_missing() {
    let home = TempDir::new().unwrap();
    let (addr, heads) = listen(NOT_ALLOWED);
    let lines = format!("grant = \"device\"\nendpoints.token = \"http://{addr}/token\"");
    let toml = account(home.path(), "work", &lines, TOKEN);

    let output = run(home.path(), &toml, &["--json", "account", "check"]);
    let json = printed(&output, 3);
    let endpoints = &json["backends"][1];

    assert_eq!(json["backends"][0]["ok"], true, "{json}");
    assert_eq!(endpoints["code"], "endpoint-missing", "{json}");
    let message = "Missing endpoints.device-authorization in the account config";
    assert_eq!(endpoints["error"], message, "{json}");
    assert!(heads.lock().unwrap().is_empty());
}

/// A server answering something else than HTTP is not reached.
#[test]
fn an_answer_that_is_no_http_fails_the_endpoints() {
    let home = TempDir::new().unwrap();
    let (addr, _) = listen("SSH-2.0-OpenSSH_9.9\r\n");
    let lines =
        format!("grant = \"client-credentials\"\nendpoints.token = \"http://{addr}/token\"");
    let toml = account(home.path(), "work", &lines, TOKEN);

    let output = run(home.path(), &toml, &["--json", "account", "check"]);
    let json = printed(&output, 3);
    let error = json["backends"][1]["error"].as_str().unwrap();
    assert!(
        error.ends_with("The answer is no HTTP status line"),
        "{error}"
    );
}

#[test]
fn an_unknown_account_name_is_coded_account_missing() {
    let home = TempDir::new().unwrap();
    let toml = account(home.path(), "work", "", TOKEN);

    let output = run(
        home.path(),
        &toml,
        &["--json", "-a", "nope", "account", "check"],
    );
    let json = printed(&output, 1);
    assert_eq!(json["code"], "account-missing", "{json}");
}

/// The flat `tls` string of 2.x is refused by name, the sentence giving
/// the keys its value becomes.
#[test]
fn the_tls_string_is_refused_by_name() {
    let home = TempDir::new().unwrap();
    let toml = account(home.path(), "work", "tls = \"rustls-ring\"", TOKEN);

    let output = run(home.path(), &toml, &["--json", "account", "list"]);
    let json = printed(&output, 1).to_string();
    let sentence = "The `tls` key is removed, the TLS options being a table as \
                    in every Pimalaya CLI: write `tls.provider = \\\"rustls\\\"` \
                    and `tls.rustls.crypto = \\\"ring\\\"`, see MIGRATION.md";
    assert!(json.contains(sentence), "{json}");
}
