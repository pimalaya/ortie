//! # Accounts
//!
//! The built binary's `account list`, `account show` and `account
//! check`, the latter against a local listener standing for the
//! authorization server and a token file standing for the store, and its
//! failure object for an account it cannot take or a key the
//! configuration no longer reads.

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

/// A token issued in 1970 for an hour, with no refresh token.
const EXPIRED: &str =
    r#"{"access_token":"at-test","token_type":"Bearer","expires_in":3600,"issued_at":1000}"#;

/// A token issued in 1970 for an hour, with a refresh token.
const EXPIRED_REFRESHABLE: &str = r#"{"access_token":"at-test","token_type":"Bearer","expires_in":3600,"issued_at":1000,"refresh_token":"rt-test"}"#;

/// What the listener answers a `HEAD` on an OAuth endpoint, usually.
const NOT_ALLOWED: &str =
    "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

/// Runs the binary with `args` over the configuration `toml`, in a home
/// of its own, its stdin closed.
fn run(home: &Path, toml: &str, args: &[&str]) -> Output {
    run_layers(home, &[toml], args)
}

/// Runs the binary with `args` over one `-c` per layer of `tomls`, the
/// first the base and the next ones merged on top, in a home of its own,
/// its stdin closed.
fn run_layers(home: &Path, tomls: &[&str], args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ortie"));

    for (i, toml) in tomls.iter().enumerate() {
        let config = home.join(format!("config-{i}.toml"));
        fs::write(&config, toml).unwrap();
        cmd.arg("-c").arg(config);
    }

    cmd.args(args)
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

/// What a `--json` `account check` printed, once it exited with `status`:
/// one `check-finished` event per row, in the order of the rows, each
/// carrying the account and that row, then the result.
fn checked(output: &Output, status: i32) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(status), "{stdout}");

    let mut lines: Vec<Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|err| panic!("{err}: {line}")))
        .collect();
    let result = lines.pop().unwrap();
    assert_eq!(result.get("event"), None, "{stdout}");

    let events: Vec<Value> = result["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let mut event = json!({ "event": "check-finished", "account": result["account"] });
            event
                .as_object_mut()
                .unwrap()
                .extend(row.as_object().unwrap().clone());
            event
        })
        .collect();
    assert_eq!(lines, events, "{stdout}");

    result
}

/// What `output` printed as text, once it exited with `status`.
fn printed_text(output: &Output, status: i32) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(output.status.code(), Some(status), "{stdout}");
    stdout
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

/// The rows of a check, by name.
fn rows(json: &Value) -> Vec<&str> {
    let checks = json["checks"].as_array().unwrap();
    checks
        .iter()
        .map(|c| c["check"].as_str().unwrap())
        .collect()
}

/// The row of a check named `name`.
fn row<'a>(json: &'a Value, name: &str) -> &'a Value {
    let checks = json["checks"].as_array().unwrap();
    let row = checks.iter().find(|c| c["check"] == name);
    row.unwrap_or_else(|| panic!("no {name} row: {json}"))
}

#[test]
fn account_list_prints_every_account_sorted_with_its_grant() {
    let home = TempDir::new().unwrap();
    let toml = account(home.path(), "work", "grant = \"device\"", TOKEN)
        + "[accounts.home]\nclient-id = \"c\"\n\
           storage.read.command = \"exit 127\"\n\
           storage.write.command = \"exit 127\"\n";

    let output = run(home.path(), &toml, &["--json", "account", "list"]);
    let expected = json!({
        "accounts": [
            { "name": "home", "default": false, "grant": "authorization-code" },
            { "name": "work", "default": true, "grant": "device" },
        ]
    });
    assert_eq!(printed(&output, 0), expected);

    let output = run(home.path(), &toml, &["account", "ls"]);
    let text = printed_text(&output, 0);
    let expected = "\
home  authorization-code  -
work  device              default
";
    assert_eq!(text, expected);
}

/// The check runs the read command, reads the token it printed, and
/// sends each endpoint one `HEAD` carrying nothing but its target and
/// host, any HTTP answer counting; the write command never runs.
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
    let ok = |check: &str| json!({ "check": check, "ok": true, "code": null, "error": null, "tls": null });
    let expected = json!({
        "account": "work",
        "checks": [
            ok("storage"),
            ok("token"),
            ok("endpoints.authorization"),
            ok("endpoints.token"),
            ok("endpoints.revocation"),
        ]
    });
    assert_eq!(checked(&output, 0), expected);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first = r#"{"event":"check-finished","account":"work","check":"storage","ok":true,"code":null,"error":null,"tls":null}"#;
    assert_eq!(stdout.lines().next(), Some(first), "{stdout}");
    assert_eq!(stdout.lines().count(), 6, "{stdout}");

    let head = |target: &str| {
        format!("HEAD {target} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n")
    };
    let heads = heads.lock().unwrap().clone();
    assert_eq!(
        heads,
        [head("/authorize"), head("/token?tenant=x"), head("/revoke")]
    );

    assert!(!home.path().join("written.json").exists());

    let output = run(home.path(), &toml, &["account", "check"]);
    assert_eq!(printed_text(&output, 0), "All 5 checks of work passed\n");
}

/// Every row prints whatever the others did, and a failed one exits 3
/// with the report, each failure coded as any command would code it.
#[test]
fn a_failed_row_exits_3_and_keeps_the_others() {
    let home = TempDir::new().unwrap();
    let lines = "grant = \"client-credentials\"\n\
                 endpoints.token = \"http://127.0.0.1:1/token\"";
    let toml = account(home.path(), "work", lines, "");

    let output = run(home.path(), &toml, &["--json", "account", "check"]);
    let json = checked(&output, 3);
    assert_eq!(rows(&json), ["storage", "token", "endpoints.token"]);

    for name in ["storage", "token"] {
        let row = row(&json, name);
        assert_eq!(row["ok"], false, "{json}");
        assert_eq!(row["code"], "token-missing", "{json}");
    }

    let endpoint = row(&json, "endpoints.token");
    assert_eq!(endpoint["ok"], false, "{json}");
    assert_eq!(endpoint["code"], Value::Null, "{json}");
    let error = endpoint["error"].as_str().unwrap();
    let reach = "Cannot reach endpoints.token at http://127.0.0.1:1/token";
    assert!(error.starts_with(reach), "{error}");

    let output = run(home.path(), &toml, &["account", "check"]);
    let text = printed_text(&output, 3);
    assert!(
        text.starts_with("3 of 3 checks of work failed:\n"),
        "{text}"
    );
    assert!(
        text.contains("\nstorage:          No access token stored"),
        "{text}"
    );
    assert!(
        text.contains("\ntoken:            No access token stored"),
        "{text}"
    );
    assert!(text.contains("\nendpoints.token:  Cannot reach"), "{text}");
}

/// An endpoint the grant needs fails its row before any request, coded
/// as `auth get` codes it; the other rows still run, and an endpoint the
/// grant has no use for is not checked.
#[test]
fn an_endpoint_the_grant_needs_is_coded_endpoint_missing() {
    let home = TempDir::new().unwrap();
    let (addr, heads) = listen(NOT_ALLOWED);
    let lines = format!(
        "grant = \"device\"\n\
         endpoints.authorization = \"http://{addr}/authorize\"\n\
         endpoints.token = \"http://{addr}/token\""
    );
    let toml = account(home.path(), "work", &lines, TOKEN);

    let output = run(home.path(), &toml, &["--json", "account", "check"]);
    let json = checked(&output, 3);
    let names = [
        "storage",
        "token",
        "endpoints.device-authorization",
        "endpoints.token",
    ];
    assert_eq!(rows(&json), names);

    let missing = row(&json, "endpoints.device-authorization");
    assert_eq!(missing["code"], "endpoint-missing", "{json}");
    let message = "Missing endpoints.device-authorization in the account config";
    assert_eq!(missing["error"], message, "{json}");
    assert_eq!(missing["tls"], Value::Null, "{json}");
    assert_eq!(row(&json, "endpoints.token")["ok"], true, "{json}");

    let head = format!("HEAD /token HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    assert_eq!(*heads.lock().unwrap(), [head]);
}

/// A server answering something else than HTTP is not reached.
#[test]
fn an_answer_that_is_no_http_fails_the_endpoint() {
    let home = TempDir::new().unwrap();
    let (addr, _) = listen("SSH-2.0-OpenSSH_9.9\r\n");
    let lines =
        format!("grant = \"client-credentials\"\nendpoints.token = \"http://{addr}/token\"");
    let toml = account(home.path(), "work", &lines, TOKEN);

    let output = run(home.path(), &toml, &["--json", "account", "check"]);
    let json = checked(&output, 3);
    let error = row(&json, "endpoints.token")["error"].as_str().unwrap();
    assert!(
        error.ends_with("The answer is no HTTP status line"),
        "{error}"
    );
}

/// The token row reads what the storage returned, sending nothing: an
/// expired token passes when a refresh token or the grant renews it, and
/// fails coded `token-expired` otherwise.
#[test]
fn the_token_row_tells_an_expired_token_apart() {
    let home = TempDir::new().unwrap();
    let cases = [
        ("", EXPIRED, false),
        ("", EXPIRED_REFRESHABLE, true),
        ("grant = \"client-credentials\"", EXPIRED, true),
        ("auto-refresh = false", EXPIRED_REFRESHABLE, true),
    ];

    for (lines, token, ok) in cases {
        let toml = account(home.path(), "work", lines, token);
        let output = run(home.path(), &toml, &["--json", "account", "check"]);
        let json = checked(&output, 3);
        let row = row(&json, "token");

        assert_eq!(row["ok"], ok, "{lines} {token}: {json}");
        assert_eq!(row["tls"], Value::Null, "{json}");

        if ok {
            assert_eq!(row["code"], Value::Null, "{json}");
            assert_eq!(row["error"], Value::Null, "{json}");
        } else {
            assert_eq!(row["code"], "token-expired", "{json}");
            let error = "Expired access token with no refresh token, run `auth get` to issue one";
            assert_eq!(row["error"], error, "{json}");
            assert_eq!(output.status.code(), Some(3), "{json}");
        }
    }
}

/// A storage command failing for another reason than an empty store
/// leaves the token row nothing to read, uncoded.
#[test]
fn a_failed_storage_fails_the_token_row_uncoded() {
    let home = TempDir::new().unwrap();
    let toml = "[accounts.a]\ndefault = true\nclient-id = \"c\"\n\
                storage.read.command = \"echo locked >&2; exit 1\"\n\
                storage.write.command = \"true\"\n";

    let output = run(home.path(), toml, &["--json", "account", "check"]);
    let json = checked(&output, 3);
    let storage = row(&json, "storage");
    let token = row(&json, "token");

    assert_eq!(storage["code"], Value::Null, "{json}");
    assert!(
        storage["error"].as_str().unwrap().contains("locked"),
        "{json}"
    );
    assert_eq!(token["ok"], false, "{json}");
    assert_eq!(token["code"], Value::Null, "{json}");
    let error = "No access token to check, the storage check failed";
    assert_eq!(token["error"], error, "{json}");
}

#[test]
fn an_unknown_account_name_is_coded_account_missing() {
    let home = TempDir::new().unwrap();
    let toml = account(home.path(), "work", "", TOKEN);

    for command in ["check", "show"] {
        let output = run(
            home.path(),
            &toml,
            &["--json", "-a", "nope", "account", command],
        );
        let json = printed(&output, 1);
        assert_eq!(json["code"], "account-missing", "{json}");
    }
}

/// The flat `tls` string of 2.x is refused by serde as any value of the
/// wrong type, naming no removed key.
#[test]
fn the_tls_string_is_refused_as_a_wrong_type() {
    let home = TempDir::new().unwrap();
    let toml = account(home.path(), "work", "tls = \"rustls-ring\"", TOKEN);

    let output = run(home.path(), &toml, &["--json", "account", "list"]);
    let json = printed(&output, 1).to_string();
    assert!(json.contains("invalid type: string"), "{json}");
    assert!(!json.contains("MIGRATION.md"), "{json}");
}

/// An `https` endpoint row names the TLS the account configures, a plain
/// HTTP one and the storage and token rows none.
#[test]
fn an_https_endpoint_row_names_the_tls_configured() {
    let home = TempDir::new().unwrap();
    let lines = "grant = \"client-credentials\"\n\
                 endpoints.token = \"https://127.0.0.1:1/token\"\n\
                 endpoints.revocation = \"http://127.0.0.1:1/revoke\"\n\
                 tls.provider = \"rustls\"\n\
                 tls.cert = \"/etc/ortie/ca.pem\"\n\
                 tls.client.cert = \"/etc/ortie/me.pem\"\n\
                 tls.client.key = \"/etc/ortie/me.key.pem\"";
    let toml = account(home.path(), "work", lines, TOKEN);

    let output = run(home.path(), &toml, &["--json", "account", "check"]);
    let json = checked(&output, 3);

    for name in ["storage", "token", "endpoints.revocation"] {
        assert_eq!(row(&json, name)["tls"], Value::Null, "{name}: {json}");
    }

    let tls = json!({
        "provider": "rustls",
        "crypto": null,
        "cert": "/etc/ortie/ca.pem",
        "clientCert": "/etc/ortie/me.pem",
    });
    assert_eq!(row(&json, "endpoints.token")["tls"], tls, "{json}");
}

/// A storage command failing after it printed the token is reported by
/// its stderr, or by its program and exit status when it wrote none,
/// never by its stdout, which holds the token.
#[test]
fn a_failing_read_command_never_reports_its_stdout() {
    let home = TempDir::new().unwrap();
    let token = home.path().join("token.json");
    fs::write(
        &token,
        r#"{"access_token":"at-leak","refresh_token":"rt-leak"}"#,
    )
    .unwrap();
    let token = token.display();

    for (read, reason) in [
        (format!("cat {token}; exit 1"), "failed with exit status: 1"),
        (format!("cat {token}; echo locked >&2; exit 1"), "locked"),
    ] {
        let toml = format!(
            "[accounts.a]\ndefault = true\nclient-id = \"c\"\n\
             storage.read.command = \"{read}\"\nstorage.write.command = \"true\"\n"
        );

        let inspect = run(home.path(), &toml, &["--json", "token", "inspect"]);
        let check = run(home.path(), &toml, &["--json", "account", "check"]);

        for output in [inspect, check] {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains(reason), "{stdout}");

            for secret in ["at-leak", "rt-leak"] {
                assert!(!stdout.contains(secret), "{secret} printed: {stdout}");
            }
        }
    }
}

/// `account show` fills the defaults of every property the account
/// leaves out, and leaves out the optional ones it does not set, `tls`
/// included when no key of it is set.
#[test]
fn account_show_fills_the_defaults() {
    let home = TempDir::new().unwrap();
    let token = home.path().join("token.json");
    let token = token.display();
    let written = home.path().join("written.json");
    let written = written.display();
    let toml = format!(
        "[accounts.work]\nclient-id = \"c\"\n\
         storage.read.command = [\"cat\", \"{token}\"]\n\
         storage.write.command = \"tee {written}\"\n"
    );

    let output = run(
        home.path(),
        &toml,
        &["--json", "-a", "work", "account", "show"],
    );
    let expected = json!({
        "account": {
            "name": "work",
            "default": false,
            "clientId": "c",
            "grant": "authorization-code",
            "endpoints": {},
            "alpn": [],
            "scopes": [],
            "resources": [],
            "pkce": "s256",
            "extras": {},
            "autoRefresh": true,
            "storage": {
                "read": { "command": ["cat", token.to_string()] },
                "write": { "command": format!("tee {written}") },
            },
        }
    });
    assert_eq!(printed(&output, 0), expected);

    let output = run(home.path(), &toml, &["-a", "work", "account", "show"]);
    let expected = format!(
        "\
Account:                work
default:                false
client-id:              c
grant:                  authorization-code
alpn:
scopes:
resources:
pkce:                   s256
extras:
auto-refresh:           true
storage.read.command:   cat {token}
storage.write.command:  tee {written}
"
    );
    assert_eq!(printed_text(&output, 0), expected);
    assert!(!home.path().join("written.json").exists());
}

/// `account show` prints the account every `-c` layer merges into, a
/// list of the overlay appended to the base's as the loader merges it.
#[test]
fn account_show_prints_the_merged_layers() {
    let home = TempDir::new().unwrap();
    let base = account(home.path(), "work", "scopes = [\"a\", \"b\"]", TOKEN);
    let overlay = "[accounts.work]\ngrant = \"device\"\n\
                   endpoints.token = \"https://example.com/token\"\n\
                   scopes = [\"c\"]\n";

    let output = run_layers(
        home.path(),
        &[&base, overlay],
        &["--json", "account", "show"],
    );
    let json = printed(&output, 0);
    let account = &json["account"];

    assert_eq!(account["clientId"], "c", "{json}");
    assert_eq!(account["grant"], "device", "{json}");
    assert_eq!(account["scopes"], json!(["a", "b", "c"]), "{json}");
    let endpoints = json!({ "token": "https://example.com/token" });
    assert_eq!(account["endpoints"], endpoints, "{json}");
    assert_eq!(account["default"], true, "{json}");

    let output = run_layers(home.path(), &[&base, overlay], &["account", "show"]);
    let text = printed_text(&output, 0);
    let lines = [
        "\ngrant:                  device\n",
        "\nendpoints.token:        https://example.com/token\n",
        "\nscopes:                 a\n                        b\n                        c\n",
    ];

    for line in lines {
        assert!(text.contains(line), "{line:?}: {text}");
    }
}

/// No raw secret is ever printed: the text writes it `<redacted>` and the
/// JSON `{"raw": null, "redacted": true}`, while a secret command prints
/// as written, as a shell would read it in the text.
#[test]
fn account_show_never_prints_a_raw_secret() {
    let home = TempDir::new().unwrap();
    let secret = "s3cret-never-printed";
    let toml = account(home.path(), "work", "", TOKEN).replace("s3cret", secret)
        + "[accounts.home]\nclient-id = \"c\"\n\
           client-secret.command = [\"pass\", \"show\", \"ortie/my home\"]\n\
           storage.read.command = \"true\"\nstorage.write.command = \"true\"\n";

    let output = run(home.path(), &toml, &["--json", "account", "show"]);
    let json = printed(&output, 0);
    let redacted = json!({ "raw": null, "redacted": true });
    assert_eq!(json["account"]["clientSecret"], redacted, "{json}");
    assert!(!json.to_string().contains(secret), "{json}");

    let output = run(home.path(), &toml, &["account", "show"]);
    let text = printed_text(&output, 0);
    assert!(
        text.contains("\nclient-secret.raw:      <redacted>\n"),
        "{text}"
    );
    assert!(!text.contains(secret), "{text}");

    let args = ["--json", "-a", "home", "account", "show"];
    let json = printed(&run(home.path(), &toml, &args), 0);
    let command = json!({ "command": ["pass", "show", "ortie/my home"] });
    assert_eq!(json["account"]["clientSecret"], command, "{json}");

    let output = run(home.path(), &toml, &["-a", "home", "account", "show"]);
    let text = printed_text(&output, 0);
    let line = "\nclient-secret.command:  pass show 'ortie/my home'\n";
    assert!(text.contains(line), "{text}");
}

/// The text of `account show` is one `<key>: <value>` field per property,
/// labelled by its configuration key and aligned, a list one value per
/// line, every value defused of control and bidi characters; the JSON
/// carries `tls` once a key of it is set.
#[test]
fn account_show_text_labels_each_property_by_its_key() {
    let home = TempDir::new().unwrap();
    let lines = "grant = \"client-credentials-jwt\"\n\
                 client-key = \"/etc/ortie/key.pem\"\n\
                 client-cert = \"/etc/ortie/cert.pem\"\n\
                 endpoints.authorization = \"https://example.com/authorize\"\n\
                 endpoints.device-authorization = \"https://example.com/device\"\n\
                 endpoints.token = \"https://example.com/token?tenant=x\"\n\
                 endpoints.revocation = \"https://example.com/revoke\"\n\
                 endpoints.redirection = \"http://localhost\"\n\
                 tls.provider = \"rustls\"\n\
                 tls.rustls.crypto = \"aws\"\n\
                 tls.cert = \"/etc/ortie/ca.pem\"\n\
                 tls.client.cert = \"/etc/ortie/me.pem\"\n\
                 tls.client.key = \"/etc/ortie/me.key.pem\"\n\
                 alpn = [\"http/1.1\"]\n\
                 scopes = [\"openid\", \"quote\\\"d \\u202e\\u0007 bidi\"]\n\
                 resources = [\"https://example.com/jmap\"]\n\
                 pkce = false\n\
                 extras.access_type = \"offline\"\n\
                 extras.\"odd key\" = \"tab\\there\"\n\
                 auto-refresh = false";
    let toml = account(home.path(), "my.work", lines, TOKEN)
        .replace("[accounts.my.work]", "[accounts.\"my.work\"]");

    let shown = printed(&run(home.path(), &toml, &["--json", "account", "show"]), 0);
    assert_eq!(shown["account"]["name"], "my.work", "{shown}");
    assert_eq!(shown["account"]["pkce"], false, "{shown}");
    let tls = json!({
        "provider": "rustls",
        "rustls": { "crypto": "aws" },
        "cert": "/etc/ortie/ca.pem",
        "client": { "cert": "/etc/ortie/me.pem", "key": "/etc/ortie/me.key.pem" },
    });
    assert_eq!(shown["account"]["tls"], tls, "{shown}");

    let text = printed_text(&run(home.path(), &toml, &["account", "show"]), 0);
    let token = home.path().join("token.json");
    let token = token.display();
    let written = home.path().join("written.json");
    let written = written.display();
    let expected = format!(
        "\
Account:                         my.work
default:                         true
client-id:                       c
client-secret.raw:               <redacted>
client-key:                      /etc/ortie/key.pem
client-cert:                     /etc/ortie/cert.pem
grant:                           client-credentials-jwt
endpoints.authorization:         https://example.com/authorize
endpoints.device-authorization:  https://example.com/device
endpoints.token:                 https://example.com/token?tenant=x
endpoints.revocation:            https://example.com/revoke
endpoints.redirection:           http://localhost/
tls.provider:                    rustls
tls.rustls.crypto:               aws
tls.cert:                        /etc/ortie/ca.pem
tls.client.cert:                 /etc/ortie/me.pem
tls.client.key:                  /etc/ortie/me.key.pem
alpn:                            http/1.1
scopes:                          openid
                                 quote\"d \u{fffd}\u{fffd} bidi
resources:                       https://example.com/jmap
pkce:                            false
extras.access_type:              offline
extras.\"odd key\":                tab\u{fffd}here
auto-refresh:                    false
storage.read.command:            cat {token}
storage.write.command:           tee {written}
"
    );
    assert_eq!(text, expected);
}
