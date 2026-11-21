//! # Auth listen
//!
//! End-to-end coverage of the handed-off authorization code flow: a
//! piped `auth get` hands off its state, verifier and redirect URI, and
//! `auth listen` catches the redirection the test sends as the browser,
//! skipping a request that does not echo the state, then exchanges the
//! code against a mock token endpoint.

mod common;

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;
use tempfile::TempDir;
use url::Url;

use crate::common::*;

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
fn write_code_config(dir: &Path, addr: SocketAddr, extra: &str) -> (PathBuf, PathBuf) {
    let lines = format!(
        "client-id = \"app-id\"\nendpoints.authorization = \"http://127.0.0.1:1/authorize\"\nendpoints.token = \"http://{addr}/token\"\n{extra}"
    );

    write_config(dir, "code", &lines, "")
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
    let (config, _) = write_code_config(dir.path(), addr, "");

    let output = ortie(&config, &["--json", "auth", "get"]);
    assert!(output.status.success(), "{output:?}");

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let uri = Url::parse(json["authorizationUri"].as_str().unwrap()).unwrap();
    assert_eq!(json["redirectUri"], query(&uri, "redirect_uri"), "{json}");
    assert!(json.get("interactive").is_none(), "{json}");

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
    let (config, token) = write_code_config(dir.path(), addr, "");

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
    // first one accepted is the first request it reads.
    let target = Url::parse(&redirect_uri).unwrap();
    let target = (
        target.host_str().unwrap().to_owned(),
        target.port().unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut forger = loop {
        match TcpStream::connect((target.0.as_str(), target.1)) {
            Ok(stream) => break stream,
            Err(_) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            Err(err) => panic!("the listener never bound: {err}"),
        }
    };

    write!(
        forger,
        "GET /?code=forged-code&state=forged-state HTTP/1.1\r\nHost: x\r\n\r\n"
    )
    .unwrap();
    let mut page = String::new();
    let _ = forger.read_to_string(&mut page);
    assert!(page.starts_with("HTTP/1.0 404"), "{page}");

    let mut browser = TcpStream::connect((target.0.as_str(), target.1)).unwrap();
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with(r#"{"account":"code","#), "{stdout}");
    let receipt: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(receipt["expiresIn"], 3600, "{receipt}");

    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1, "{bodies:?}");
    assert_eq!(form(&bodies[0], "code").as_deref(), Some("the-code"));
    assert_eq!(form(&bodies[0], "redirect_uri"), Some(redirect_uri));
    assert_eq!(form(&bodies[0], "code_verifier"), Some(verifier));

    assert!(std::fs::read_to_string(token).unwrap().contains("at-test"));
}

#[test]
fn listen_is_refused_without_a_loopback_redirect_or_on_a_device_account() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();

    let (config, _) = write_code_config(dir.path(), addr, "");
    let output = ortie(
        &config,
        &["auth", "listen", "--redirect-uri=org.example.app:/cb"],
    );
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("loopback"), "{stdout}");

    let (config, _) = write_code_config(dir.path(), addr, r#"grant = "device""#);
    let output = ortie(&config, &["auth", "listen"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("device grant"), "{stdout}");
}

/// The listener speaks plain HTTP, so an https loopback redirect is one
/// it cannot serve: listening on it is refused, and the handoff offers
/// the resume alone, where the wait would never end.
#[test]
fn an_https_loopback_redirect_is_not_listened_for() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();
    let redirection = r#"endpoints.redirection = "https://127.0.0.1:1/cb""#;
    let (config, _) = write_code_config(dir.path(), addr, redirection);

    let output = ortie(&config, &["auth", "listen"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("not an http URI"), "{stdout}");

    let output = ortie(&config, &["auth", "get"]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains(" auth listen "), "{stdout}");
    assert!(stdout.contains(" auth resume "), "{stdout}");
}

/// Runs the binary over `config` with `args`, `input` on its stdin.
fn ortie_fed(config: &Path, args: &[&str], input: &[u8]) -> std::process::Child {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ortie"))
        .arg("-c")
        .arg(config)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(input).unwrap();
    child
}

/// The `auth get --json` handoff piped into `auth listen --handoff -`, no
/// flow value on its command line, completes the flow with the handoff's state,
/// verifier and redirect URI.
#[test]
fn a_handoff_on_stdin_is_listened_for_and_stored() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock();
    let (config, token) = write_code_config(dir.path(), addr, "");

    let output = ortie(&config, &["--json", "auth", "get"]);
    assert!(output.status.success(), "{output:?}");

    let handoff: Value = serde_json::from_slice(&output.stdout).unwrap();
    let state = handoff["state"].as_str().unwrap().to_owned();
    let verifier = handoff["pkceCodeVerifier"].as_str().unwrap().to_owned();
    let redirect_uri = handoff["redirectUri"].as_str().unwrap().to_owned();

    let args = ["--json", "auth", "listen", "--handoff", "-"];
    let listen = ortie_fed(&config, &args, &output.stdout);

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

    let bodies = bodies.lock().unwrap();
    assert_eq!(form(&bodies[0], "code").as_deref(), Some("the-code"));
    assert_eq!(form(&bodies[0], "redirect_uri"), Some(redirect_uri));
    assert_eq!(form(&bodies[0], "code_verifier"), Some(verifier));

    assert!(std::fs::read_to_string(token).unwrap().contains("at-test"));
}

/// The handoff `auth resume --handoff` reads, from stdin or a file, gives
/// the state the
/// redirected URI is checked against, and the verifier and redirect URI
/// of the exchange.
#[test]
fn a_handoff_on_stdin_is_resumed() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock();
    let (config, _) = write_code_config(dir.path(), addr, "");

    let output = ortie(&config, &["--json", "auth", "get"]);
    let handoff: Value = serde_json::from_slice(&output.stdout).unwrap();
    let state = handoff["state"].as_str().unwrap().to_owned();
    let verifier = handoff["pkceCodeVerifier"].as_str().unwrap().to_owned();
    let redirect_uri = handoff["redirectUri"].as_str().unwrap().to_owned();

    let forged = "http://127.0.0.1:1/?code=forged&state=forged-state";
    let resume = ortie_fed(
        &config,
        &["--json", "auth", "resume", "--handoff", "-", forged],
        &output.stdout,
    );
    let refused = resume.wait_with_output().unwrap();
    let failure: Value = serde_json::from_slice(&refused.stdout).unwrap();
    assert_eq!(failure["code"], "state-mismatch", "{failure}");

    let mut older = handoff.clone();
    older["interactive"] = Value::Bool(false);
    let redirected = format!("{redirect_uri}?code=the-code&state={state}");
    let flow = dir.path().join("flow.json");
    std::fs::write(&flow, older.to_string()).unwrap();
    let flow = flow.to_str().unwrap();
    let output = ortie(&config, &["auth", "resume", "--handoff", flow, &redirected]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let issued = "Access token of account code successfully issued (expires in 1h 1s)\n";
    assert_eq!(stdout, issued);

    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1, "{bodies:?}");
    assert_eq!(form(&bodies[0], "redirect_uri"), Some(redirect_uri));
    assert_eq!(form(&bodies[0], "code_verifier"), Some(verifier));
}

/// A REPL session reads its lines from stdin, so `auth resume` there
/// takes no handoff from it and the next line still runs.
#[test]
fn a_repl_resume_takes_no_handoff_from_its_lines() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();
    let (config, _) = write_code_config(dir.path(), addr, "");

    let input = b"auth resume http://127.0.0.1:1/?code=c&state=s\naccount list\n";
    let repl = ortie_fed(&config, &["repl"], input);
    let output = repl.wait_with_output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.lines().any(|l| l.starts_with("code ")), "{stdout}");
}

/// A handoff asked for on stdin inside a REPL session is refused in the
/// family's words, coded `stdin-taken`, and the next line still runs.
#[test]
fn a_repl_handoff_on_stdin_is_coded_stdin_taken() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();
    let (config, _) = write_code_config(dir.path(), addr, "");

    let input = b"auth resume --handoff - http://127.0.0.1:1/?code=c&state=s\naccount list\n";
    let repl = ortie_fed(&config, &["--json", "repl"], input);
    let output = repl.wait_with_output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap());
    let failure = lines.next().unwrap();
    assert_eq!(failure["code"], "stdin-taken", "{stdout}");
    let error = failure["error"].as_str().unwrap();
    assert!(error.starts_with("Cannot read input from stdin"), "{error}");
    assert!(lines.next().unwrap()["accounts"].is_array(), "{stdout}");
}

/// An interactive `auth get` interrupted while it waits for the
/// redirection prints the `auth resume` command it hid, then exits 130.
///
/// The binary runs in a terminal util-linux's `script` allocates.
#[cfg(target_os = "linux")]
#[test]
fn an_interrupted_interactive_wait_prints_the_resume() {
    use std::io::BufRead;

    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();
    let (config, _) = write_code_config(dir.path(), addr, "");

    let line = format!(
        "{} -c {} auth get",
        env!("CARGO_BIN_EXE_ortie"),
        config.display()
    );
    let mut script = Command::new("script")
        .args(["-qec", &line, "/dev/null"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut stdout = std::io::BufReader::new(script.stdout.take().unwrap());
    let mut printed = String::new();

    while !printed.contains("Wait for redirection") {
        let mut line = String::new();
        assert_ne!(stdout.read_line(&mut line).unwrap(), 0, "{printed}");
        assert!(!line.contains("State:"), "{line}");
        printed.push_str(&line);
    }

    let pid = script.id();
    let children = format!("/proc/{pid}/task/{pid}/children");
    let ortie = std::fs::read_to_string(children).unwrap();
    let ortie = ortie.split_whitespace().next().unwrap().to_owned();
    let killed = Command::new("kill")
        .args(["-INT", &ortie])
        .status()
        .unwrap();
    assert!(killed.success());

    stdout.read_to_string(&mut printed).unwrap();
    let status = script.wait().unwrap();

    let resume = printed.lines().find(|l| l.contains(" auth resume "));
    let resume = resume.unwrap_or_else(|| panic!("{printed}"));
    assert!(resume.contains("--state="), "{resume}");
    assert!(resume.contains("--pkce="), "{resume}");
    assert!(resume.contains("--redirect-uri="), "{resume}");
    assert_eq!(status.code(), Some(130), "{printed}");
}

/// In a REPL session, an interrupted interactive wait ends the session as
/// a signal does, with 130 and no handoff, and installs no handler that
/// would swallow a later Ctrl-C at the prompt.
#[cfg(target_os = "linux")]
#[test]
fn an_interrupted_repl_wait_ends_the_session() {
    use std::io::BufRead;

    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();
    let (config, _) = write_code_config(dir.path(), addr, "");

    let line = format!(
        "{} -c {} repl",
        env!("CARGO_BIN_EXE_ortie"),
        config.display()
    );
    let mut script = Command::new("script")
        .args(["-qec", &line, "/dev/null"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut stdin = script.stdin.take().unwrap();
    stdin.write_all(b"auth get\n").unwrap();

    let mut stdout = std::io::BufReader::new(script.stdout.take().unwrap());
    let mut printed = String::new();

    while !printed.contains("Wait for redirection") {
        let mut line = String::new();
        assert_ne!(stdout.read_line(&mut line).unwrap(), 0, "{printed}");
        printed.push_str(&line);
    }

    let pid = script.id();
    let children = format!("/proc/{pid}/task/{pid}/children");
    let ortie = std::fs::read_to_string(children).unwrap();
    let ortie = ortie.split_whitespace().next().unwrap().to_owned();
    let killed = Command::new("kill")
        .args(["-INT", &ortie])
        .status()
        .unwrap();
    assert!(killed.success());

    stdout.read_to_string(&mut printed).unwrap();
    let status = script.wait().unwrap();
    drop(stdin);

    assert!(!printed.contains(" auth resume "), "{printed}");
    assert_eq!(status.code(), Some(130), "{printed}");
}

/// Without `--handoff` the commands never read stdin, so a script whose
/// stdin stays open does not wait on it.
#[test]
fn without_the_handoff_flag_an_open_stdin_is_left_alone() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();
    let (config, token) = write_code_config(dir.path(), addr, "");

    let mut resume = Command::new(env!("CARGO_BIN_EXE_ortie"))
        .arg("-c")
        .arg(&config)
        .args(["auth", "resume", "http://127.0.0.1:1/?code=c&state=s"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let stdin = resume.stdin.take().unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = resume.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            resume.kill().unwrap();
            panic!("auth resume waited on its open stdin");
        }
        thread::sleep(Duration::from_millis(20));
    };
    drop(stdin);

    assert!(status.success(), "{status:?}");
    assert!(std::fs::read_to_string(token).unwrap().contains("at-test"));
}

/// A listen given no redirection to bind exits 2 coded `usage-invalid`,
/// the redirect URI `auth get` printed being the fix; one whose
/// configured redirection cannot be bound is a configuration fault: exit
/// 1, coded `redirection-unsupported`.
#[test]
fn a_redirection_the_config_lacks_or_cannot_listen_on_is_refused() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock();

    let cases: &[(&str, i32, &str, &str)] = &[
        (
            "",
            2,
            "usage-invalid",
            "Missing --redirect-uri, pass the one auth get printed",
        ),
        (
            r#"endpoints.redirection = "https://127.0.0.1:1/cb""#,
            1,
            "redirection-unsupported",
            "Cannot listen on endpoints.redirection https://127.0.0.1:1/cb, not an \
             http URI on a loopback host, run auth resume with the redirected URI \
             instead",
        ),
        (
            r#"endpoints.redirection = "org.example.app:/cb""#,
            1,
            "redirection-unsupported",
            "Cannot listen on endpoints.redirection org.example.app:/cb, not an \
             http URI on a loopback host, run auth resume with the redirected URI \
             instead",
        ),
    ];

    for (extra, status, code, message) in cases {
        let (config, _) = write_code_config(dir.path(), addr, extra);
        let output = ortie(&config, &["--json", "auth", "listen", "--state=abc"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(*status), "{extra}: {stdout}");

        let failure: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(failure["code"], *code, "{extra}: {failure}");
        assert_eq!(failure["error"], *message, "{extra}: {failure}");
    }

    assert!(bodies.lock().unwrap().is_empty());
}

/// A handoff on an account whose grant takes none is a command line
/// Ortie refuses after clap parsed it: coded `usage-invalid`, exit 2, as
/// clap's own refusals.
#[test]
fn a_command_line_refused_after_parsing_exits_2_coded() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();

    let (config, _) = write_code_config(dir.path(), addr, r#"grant = "device""#);
    let args = ["--json", "auth", "resume", "--handoff", "flow.json", "CODE"];
    let output = ortie(&config, &args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(2), "{stdout}");
    let failure: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(failure["code"], "usage-invalid", "{failure}");
    let message = "The --handoff flag is only valid for the authorization-code grant";
    assert_eq!(failure["error"], message, "{failure}");
}

/// A command or a flag the account's grant has no use for, an empty
/// device code and a redirected URI that does not parse are command lines
/// too, refused after parsing: coded `usage-invalid`, exit 2, nothing
/// sent.
#[test]
fn a_command_line_the_grant_refuses_exits_2_coded() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock();

    let cases: &[(&str, &[&str], &str)] = &[
        (
            r#"grant = "device""#,
            &["auth", "listen"],
            "The device grant has no redirection to listen for, run auth resume instead",
        ),
        (
            r#"grant = "device""#,
            &["auth", "resume", "--state=abc", "CODE"],
            "The --state, --pkce and --redirect-uri flags are only valid for the \
             authorization-code grant",
        ),
        (
            r#"grant = "device""#,
            &["auth", "resume", "--", " "],
            "Missing device code",
        ),
        (
            r#"grant = "client-credentials""#,
            &["auth", "listen"],
            "The client credentials grants complete in a single auth get, nothing to listen for",
        ),
        (
            r#"grant = "client-credentials""#,
            &["auth", "resume", "x"],
            "The client credentials grants complete in a single auth get, nothing to resume",
        ),
        (
            "",
            &["auth", "listen", "--redirect-uri=https://127.0.0.1:1/cb"],
            "Cannot listen on https://127.0.0.1:1/cb, not an http URI on a loopback host, \
             run auth resume with the redirected URI instead",
        ),
        (
            "",
            &["auth", "resume", "not a uri"],
            "Invalid redirected URI: relative URL without a base",
        ),
    ];

    for (extra, args, message) in cases {
        let (config, _) = write_code_config(dir.path(), addr, extra);
        let args: Vec<_> = ["--json"].iter().chain(*args).copied().collect();
        let output = ortie(&config, &args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {stdout}");

        let failure: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(failure["code"], "usage-invalid", "{args:?}: {failure}");
        assert_eq!(failure["error"], *message, "{args:?}: {failure}");
    }

    assert!(bodies.lock().unwrap().is_empty());
}

/// The handoff gives the state and the verifier, so `--handoff` beside
/// `--state` or `--pkce` is a command line the grammar refuses.
#[test]
fn the_handoff_flag_refuses_the_state_and_pkce_flags() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock();
    let (config, _) = write_code_config(dir.path(), addr, "");

    for args in [
        ["--json", "auth", "listen", "--handoff=-", "--state=s"].as_slice(),
        &[
            "--json",
            "auth",
            "resume",
            "--handoff=-",
            &format!("--pkce={}", "v".repeat(43)),
            "x",
        ],
    ] {
        let output = ortie(&config, args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(2), "{stdout}");
        assert!(stdout.contains("usage-invalid"), "{stdout}");
        assert!(stdout.contains("cannot be used with"), "{stdout}");
    }
}
