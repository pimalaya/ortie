//! # Auth listen
//!
//! End-to-end coverage of the handed-off authorization code flow: a
//! piped `auth get` hands off its state, verifier and redirect URI, and
//! `auth listen` catches the redirection the test sends as the browser,
//! then exchanges the code against a mock token endpoint.

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;
use tempfile::TempDir;
use url::Url;

/// Starts a mock token endpoint answering every POST with a token,
/// keeping each form body.
fn start_mock() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let bodies_t = Arc::clone(&bodies);

    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut raw = Vec::new();
            let mut buf = [0u8; 8192];

            let (head, mut body) = loop {
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break (String::from_utf8_lossy(&raw).into_owned(), Vec::new());
                }
                raw.extend_from_slice(&buf[..n]);

                if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&raw[..pos]).into_owned();
                    break (head, raw[pos + 4..].to_vec());
                }
            };

            let content_length: usize = head
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|value| value.trim().parse().ok())
                })
                .unwrap_or(0);

            while body.len() < content_length {
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break;
                }
                body.extend_from_slice(&buf[..n]);
            }

            bodies_t
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&body).into_owned());

            let response = r#"{"access_token":"at-test","token_type":"Bearer","expires_in":3600}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    (addr, bodies)
}

/// Writes an authorization code account reaching the mock at `addr`,
/// with file-backed storage and the given extra lines.
fn write_config(dir: &Path, addr: SocketAddr, extra: &str) -> (PathBuf, PathBuf) {
    let token = dir.join("token.json");
    std::fs::write(&token, "").unwrap();

    let config = dir.join("config.toml");
    std::fs::write(
        &config,
        format!(
            r#"
[accounts.code]
default = true
client-id = "app-id"
endpoints.authorization = "http://127.0.0.1:1/authorize"
endpoints.token = "http://{addr}/token"
storage.read.command = ["cat", "{t}"]
storage.write.command = ["tee", "{t}"]
{extra}
"#,
            t = token.display()
        ),
    )
    .unwrap();

    (config, token)
}

/// Runs the built binary against `config` with piped stdio.
fn ortie(config: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ortie"))
        .arg("-c")
        .arg(config)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// The query parameter `key` of `uri`.
fn query(uri: &Url, key: &str) -> String {
    uri.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| panic!("no {key} in {uri}"))
}

/// The value of `key` in a form body.
fn form(body: &str, key: &str) -> Option<String> {
    url::form_urlencoded::parse(body.as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

#[test]
fn a_handed_off_flow_carries_its_redirect_uri() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();
    let (config, _) = write_config(dir.path(), addr, "");

    let output = ortie(&config, &["--json", "auth", "get"]);
    assert!(output.status.success(), "{output:?}");

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let uri = Url::parse(json["authorizationUri"].as_str().unwrap()).unwrap();
    assert_eq!(json["redirectUri"], query(&uri, "redirect_uri"), "{json}");

    let output = ortie(&config, &["auth", "get"]);
    assert!(output.status.success(), "{output:?}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let flag = "--redirect-uri='http://127.0.0.1:";
    let listen = stdout.lines().find(|line| line.contains(" auth listen "));
    let resume = stdout.lines().find(|line| line.contains(" auth resume "));
    assert!(listen.is_some_and(|line| line.contains(flag)), "{stdout}");
    assert!(resume.is_some_and(|line| line.contains(flag)), "{stdout}");
}

#[test]
fn a_handed_off_flow_is_listened_for_and_stored() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock();
    let (config, token) = write_config(dir.path(), addr, "");

    let output = ortie(&config, &["--json", "auth", "get"]);
    assert!(output.status.success(), "{output:?}");

    let handoff: Value = serde_json::from_slice(&output.stdout).unwrap();
    let state = handoff["state"].as_str().unwrap().to_owned();
    let verifier = handoff["pkceCodeVerifier"].as_str().unwrap().to_owned();
    let redirect_uri = handoff["redirectUri"].as_str().unwrap().to_owned();

    let listen = Command::new(env!("CARGO_BIN_EXE_ortie"))
        .arg("-c")
        .arg(&config)
        .args(["--json", "auth", "listen"])
        .arg(format!("--state={state}"))
        .arg(format!("--pkce={verifier}"))
        .arg(format!("--redirect-uri={redirect_uri}"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // NOTE: a refused connection never reaches the listener, so the
    // first one accepted is the redirect itself.
    let target = Url::parse(&redirect_uri).unwrap();
    let target = (
        target.host_str().unwrap().to_owned(),
        target.port().unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut browser = loop {
        match TcpStream::connect((target.0.as_str(), target.1)) {
            Ok(stream) => break stream,
            Err(_) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            Err(err) => panic!("the listener never bound: {err}"),
        }
    };

    write!(
        browser,
        "GET /?code=the-code&state={state} HTTP/1.1\r\nHost: x\r\n\r\n"
    )
    .unwrap();
    let mut page = String::new();
    let _ = browser.read_to_string(&mut page);
    assert!(page.starts_with("HTTP/1.0 200 OK"), "{page}");

    let output = listen.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");

    let receipt: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(receipt["expiresIn"], 3600, "{receipt}");

    let bodies = bodies.lock().unwrap();
    assert_eq!(form(&bodies[0], "code").as_deref(), Some("the-code"));
    assert_eq!(form(&bodies[0], "redirect_uri"), Some(redirect_uri));
    assert_eq!(form(&bodies[0], "code_verifier"), Some(verifier));

    assert!(std::fs::read_to_string(token).unwrap().contains("at-test"));
}

#[test]
fn listen_is_refused_without_a_loopback_redirect_or_on_a_device_account() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();

    let (config, _) = write_config(dir.path(), addr, "");
    let output = ortie(
        &config,
        &["auth", "listen", "--redirect-uri=org.example.app:/cb"],
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("loopback"), "{stdout}");

    let (config, _) = write_config(dir.path(), addr, r#"grant = "device""#);
    let output = ortie(&config, &["auth", "listen"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("device grant"), "{stdout}");
}
