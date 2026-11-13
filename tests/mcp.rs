//! # MCP
//!
//! `ortie mcp` run as the built binary against a mock authorization
//! server, fed JSON-RPC requests on stdin one at a time. The test plays
//! the browser: it reads the authorization URI a tool answers and sends
//! the redirect the server listens for. Every response is kept, so each
//! test can assert that no secret ever reached stdout.

mod common;

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use tempfile::TempDir;
use url::Url;

use crate::common::*;

const TOKEN: &str = r#"{"access_token":"at-test","token_type":"Bearer","expires_in":3600,"refresh_token":"rt-test"}"#;

/// A mock authorization server: `/devicecode` answers a device code,
/// `/token` answers `authorization_pending` for the first `pending`
/// requests and a token after. Each token request body is kept.
fn start_mock(pending: usize) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    start_mock_with(pending, r#""expires_in":60,"interval":1"#)
}

/// The mock of [`start_mock`], its device code answer carrying `timing`
/// as its lifetime and interval members.
fn start_mock_with(pending: usize, timing: &'static str) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let bodies_t = Arc::clone(&bodies);
    let polls = AtomicUsize::new(0);

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

            let path = head.split_whitespace().nth(1).unwrap_or("/").to_owned();

            let (status, response) = if path.starts_with("/devicecode") {
                (
                    200,
                    format!(
                        r#"{{"device_code":"dc-test","user_code":"USER","verification_uri":"http://{addr}/d",{timing}}}"#
                    ),
                )
            } else if path.starts_with("/token") {
                bodies_t
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&body).into_owned());

                if polls.fetch_add(1, Ordering::SeqCst) < pending {
                    (400, String::from(r#"{"error":"authorization_pending"}"#))
                } else {
                    (200, String::from(TOKEN))
                }
            } else {
                (404, String::from(r#"{"error":"not_found"}"#))
            };

            let resp = format!(
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    (addr, bodies)
}

/// Writes an account config reaching the mock at `addr`, with
/// file-backed storage and the given extra lines, returning the config
/// and token paths.
fn write_mcp_config(dir: &Path, addr: SocketAddr, extra: &str) -> (PathBuf, PathBuf) {
    let lines = format!(
        "client-id = \"app-id\"\nendpoints.authorization = \"http://127.0.0.1:1/authorize\"\nendpoints.device-authorization = \"http://{addr}/devicecode\"\nendpoints.token = \"http://{addr}/token\"\n{extra}"
    );

    write_config(dir, "mcp", &lines, "")
}

/// A running server, answering one request per line.
struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    /// Every line the server printed.
    printed: String,
}

impl Server {
    fn start(config: &Path, flags: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ortie"))
            .args(["-c", config.to_str().unwrap(), "mcp"])
            .args(flags)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());

        Self {
            child,
            stdin,
            stdout,
            next_id: 0,
            printed: String::new(),
        }
    }

    /// Sends one request and returns its response.
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let request =
            json!({ "jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params });
        writeln!(self.stdin, "{request}").unwrap();

        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        self.printed.push_str(&line);

        serde_json::from_str(&line).unwrap_or_else(|err| panic!("{err}: {line}"))
    }

    /// Calls a tool and returns its result.
    fn call(&mut self, name: &str, arguments: Value) -> Value {
        let response = self.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        );
        response["result"].clone()
    }

    /// Calls `auth_status` until the flow leaves `pending`.
    fn settle(&mut self) -> Value {
        let deadline = Instant::now() + Duration::from_secs(10);

        loop {
            let status = self.call("auth_status", json!({}))["structuredContent"].clone();
            if status["status"] != "pending" || Instant::now() > deadline {
                return status;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Closes stdin and returns everything the server printed.
    fn finish(mut self) -> String {
        drop(self.stdin);

        let mut rest = String::new();
        self.stdout.read_to_string(&mut rest).unwrap();
        self.printed.push_str(&rest);

        let status = self.child.wait().unwrap();
        assert!(status.success(), "{status:?}");

        self.printed
    }
}

/// The query parameter `key` of `uri`.
fn query(uri: &Url, key: &str) -> String {
    uri.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| panic!("no {key} in {uri}"))
}

/// Plays the browser: sends the redirect carrying `code` and `state` to
/// the loopback `redirect_uri`, and returns the page it answered.
fn redirect(redirect_uri: &str, code: &str, state: &str) -> String {
    let redirect_uri = Url::parse(redirect_uri).unwrap();
    let addr = (
        redirect_uri.host_str().unwrap(),
        redirect_uri.port().unwrap(),
    );

    let mut stream = TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "GET {}?code={code}&state={state} HTTP/1.1\r\nHost: x\r\n\r\n",
        redirect_uri.path()
    )
    .unwrap();

    let mut page = String::new();
    let _ = stream.read_to_string(&mut page);
    page
}

/// The value of `key` in a form body.
fn form(body: &str, key: &str) -> Option<String> {
    url::form_urlencoded::parse(body.as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

#[test]
fn tools_are_listed_with_their_schemas_and_revocation_is_opt_in() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock(0);
    let (config, _) = write_mcp_config(dir.path(), addr, "");

    let mut server = Server::start(&config, &[]);
    let init = server.request("initialize", json!({}));
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");

    let tools = server.request("tools/list", json!({}))["result"]["tools"].clone();
    let names: Vec<&str> = tools
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "token_inspect",
            "auth_get",
            "auth_status",
            "auth_resume",
            "token_refresh"
        ]
    );

    for tool in tools.as_array().unwrap() {
        assert_eq!(tool["inputSchema"]["additionalProperties"], false, "{tool}");
        assert_eq!(tool["outputSchema"]["type"], "object", "{tool}");
    }
    server.finish();

    let mut server = Server::start(&config, &["--allow-revoke"]);
    let tools = server.request("tools/list", json!({}))["result"]["tools"].clone();
    assert!(
        tools
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "token_revoke")
    );
    let unknown = server.call("token_revoke", json!({}));
    assert_eq!(unknown["isError"], true, "{unknown}");
    server.finish();
}

#[test]
fn a_code_flow_is_captured_exchanged_and_stored() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock(0);
    let (config, token) = write_mcp_config(dir.path(), addr, "");

    let mut server = Server::start(&config, &[]);
    assert_eq!(server.settle()["status"], "none");

    let get = server.call("auth_get", json!({}))["structuredContent"].clone();
    assert_eq!(get["kind"], "authorization", "{get}");
    assert_eq!(get["redirectCaptured"], true, "{get}");

    let uri = Url::parse(get["authorizationUri"].as_str().unwrap()).unwrap();
    let redirect_uri = query(&uri, "redirect_uri");
    let page = redirect(&redirect_uri, "the-code", &query(&uri, "state"));
    assert!(page.starts_with("HTTP/1.0 200 OK"), "{page}");

    let status = server.settle();
    assert_eq!(status["status"], "issued", "{status}");
    assert_eq!(status["receipt"]["withRefreshToken"], true, "{status}");

    let inspect = server.call("token_inspect", json!({}))["structuredContent"].clone();
    assert_eq!(inspect["tokenType"], "Bearer", "{inspect}");

    let printed = server.finish();

    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1, "{bodies:?}");
    assert_eq!(form(&bodies[0], "code").as_deref(), Some("the-code"));
    assert_eq!(form(&bodies[0], "redirect_uri"), Some(redirect_uri));
    let verifier = form(&bodies[0], "code_verifier").expect("a PKCE verifier");

    assert!(std::fs::read_to_string(token).unwrap().contains("at-test"));

    for secret in ["at-test", "rt-test", verifier.as_str()] {
        assert!(!printed.contains(secret), "{secret} printed: {printed}");
    }
}

#[test]
fn a_redirect_no_listener_catches_is_resumed_from_its_uri() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock(0);
    let (config, _) = write_mcp_config(
        dir.path(),
        addr,
        r#"endpoints.redirection = "org.example.app:/cb""#,
    );

    let mut server = Server::start(&config, &[]);

    let get = server.call("auth_get", json!({}))["structuredContent"].clone();
    assert_eq!(get["redirectCaptured"], false, "{get}");

    let uri = Url::parse(get["authorizationUri"].as_str().unwrap()).unwrap();
    let redirected = format!("org.example.app:/cb?code=c2&state={}", query(&uri, "state"));

    let resume = server.call("auth_resume", json!({ "uri": redirected }));
    assert_eq!(
        resume["structuredContent"]["withRefreshToken"], true,
        "{resume}"
    );
    assert_eq!(server.settle()["status"], "issued");

    server.finish();
    assert_eq!(
        form(&bodies.lock().unwrap()[0], "code").as_deref(),
        Some("c2")
    );
}

#[test]
fn a_device_flow_is_polled_to_the_token() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock(1);
    let (config, _) = write_mcp_config(dir.path(), addr, r#"grant = "device""#);

    let mut server = Server::start(&config, &[]);

    let get = server.call("auth_get", json!({}))["structuredContent"].clone();
    assert_eq!(get["kind"], "device", "{get}");
    assert_eq!(get["userCode"], "USER", "{get}");

    let status = server.settle();
    assert_eq!(status["status"], "issued", "{status}");

    let printed = server.finish();
    assert!(bodies.lock().unwrap().len() >= 2);

    for secret in ["dc-test", "at-test", "rt-test"] {
        assert!(!printed.contains(secret), "{secret} printed: {printed}");
    }
}

/// A lifetime no clock can add and an interval of zero: the flow still
/// polls once a second, to the token.
#[cfg(target_pointer_width = "64")]
#[test]
fn a_device_flow_bounds_the_lifetime_and_the_interval_of_the_server() {
    let dir = TempDir::new().unwrap();
    let timing = r#""expires_in":18446744073709551615,"interval":0"#;
    let (addr, bodies) = start_mock_with(1, timing);
    let (config, _) = write_mcp_config(dir.path(), addr, r#"grant = "device""#);

    let mut server = Server::start(&config, &[]);
    let start = Instant::now();

    let get = server.call("auth_get", json!({}))["structuredContent"].clone();
    assert_eq!(get["kind"], "device", "{get}");

    let status = server.settle();
    assert_eq!(status["status"], "issued", "{status}");
    assert!(
        start.elapsed() >= Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );

    server.finish();
    assert_eq!(bodies.lock().unwrap().len(), 2);
}

#[test]
fn a_request_not_echoing_the_state_does_not_end_the_flow() {
    let dir = TempDir::new().unwrap();
    let (addr, bodies) = start_mock(0);
    let (config, _) = write_mcp_config(dir.path(), addr, "");

    let mut server = Server::start(&config, &[]);

    let get = server.call("auth_get", json!({}))["structuredContent"].clone();
    let uri = Url::parse(get["authorizationUri"].as_str().unwrap()).unwrap();
    let redirect_uri = query(&uri, "redirect_uri");

    let page = redirect(&redirect_uri, "forged-code", "forged-state");
    assert!(page.starts_with("HTTP/1.0 404"), "{page}");

    let status = server.call("auth_status", json!({}))["structuredContent"].clone();
    assert_eq!(status["status"], "pending", "{status}");

    let page = redirect(&redirect_uri, "the-code", &query(&uri, "state"));
    assert!(page.starts_with("HTTP/1.0 200 OK"), "{page}");
    assert_eq!(server.settle()["status"], "issued");

    server.finish();

    let bodies = bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1, "{bodies:?}");
    assert_eq!(form(&bodies[0], "code").as_deref(), Some("the-code"));
}

#[test]
fn a_new_flow_stops_the_listener_of_the_old_one() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock(0);
    let (config, _) = write_mcp_config(dir.path(), addr, "");

    let mut server = Server::start(&config, &[]);

    let first = server.call("auth_get", json!({}))["structuredContent"].clone();
    let first = Url::parse(first["authorizationUri"].as_str().unwrap()).unwrap();
    let first = Url::parse(&query(&first, "redirect_uri")).unwrap();
    let first = (first.host_str().unwrap().to_owned(), first.port().unwrap());

    server.call("auth_get", json!({}));

    let deadline = Instant::now() + Duration::from_secs(2);
    while TcpStream::connect((first.0.as_str(), first.1)).is_ok() {
        assert!(Instant::now() < deadline, "the first listener still runs");
        thread::sleep(Duration::from_millis(20));
    }

    let status = server.call("auth_status", json!({}))["structuredContent"].clone();
    assert_eq!(status["status"], "pending", "{status}");
    server.finish();
}

#[test]
fn a_new_flow_binds_the_configured_port_again() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock(0);

    let free = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = free.local_addr().unwrap().port();
    drop(free);

    let redirection = format!(r#"endpoints.redirection = "http://127.0.0.1:{port}/cb""#);
    let (config, _) = write_mcp_config(dir.path(), addr, &redirection);

    let mut server = Server::start(&config, &[]);

    for _ in 0..3 {
        let get = server.call("auth_get", json!({}));
        assert_eq!(get["structuredContent"]["redirectCaptured"], true, "{get}");
    }

    server.finish();
}

#[test]
fn an_unknown_argument_or_tool_is_refused() {
    let dir = TempDir::new().unwrap();
    let (addr, _) = start_mock(0);
    let (config, _) = write_mcp_config(dir.path(), addr, "");

    let mut server = Server::start(&config, &[]);

    let resume = server.call("auth_resume", json!({ "url": "x" }));
    assert_eq!(resume["isError"], true, "{resume}");
    assert!(resume.to_string().contains("takes `uri`"), "{resume}");

    let inspect = server.call("token_inspect", json!({ "verbose": true }));
    assert_eq!(inspect["isError"], true, "{inspect}");
    assert!(inspect.to_string().contains("takes none"), "{inspect}");

    let resume = server.call("auth_resume", json!({ "uri": "http://127.0.0.1/" }));
    assert_eq!(resume["isError"], true, "{resume}");
    assert!(
        resume.to_string().contains("call auth_get first"),
        "{resume}"
    );

    let unknown = server.request(
        "tools/call",
        json!({ "name": "token_show", "arguments": {} }),
    );
    assert_eq!(unknown["error"]["code"], -32602, "{unknown}");

    server.finish();
}
