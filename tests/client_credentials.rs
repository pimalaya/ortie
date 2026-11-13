//! # Client credentials grants
//!
//! End-to-end coverage of the two headless grants, driving the real
//! binary against a local mock authorization server: silent
//! re-acquisition on expiry, a fresh JWT assertion per mint, and the
//! certificate renewal hint.

mod common;

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use base64_codec::codec::{Base64Alphabet, Base64Codec};
use serde_json::Value;
use tempfile::TempDir;

use crate::common::*;

/// The fake DER certificate bytes whose x5t thumbprint is the known
/// constant asserted by the io-oauth test suite.
const CERT_DER: &[u8] = b"example-cert-der";

/// `base64url(sha1(CERT_DER))`, precomputed.
const CERT_X5T: &str = "G8982Odi4bIpR3fqlbW9jOaBYa0";

/// One captured token endpoint request: raw head and form body.
struct CapturedRequest {
    head: String,
    body: String,
}

/// Starts a mock token endpoint answering every POST with the given
/// status and body, capturing each request for later assertions.
fn start_mock(status: u16, response: &str) -> (SocketAddr, Arc<Mutex<Vec<CapturedRequest>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_t = Arc::clone(&requests);
    let response = response.to_owned();

    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut raw = Vec::new();
            let mut buf = [0u8; 8192];

            let (head, mut body) = loop {
                let Ok(n) = stream.read(&mut buf) else {
                    break (String::new(), Vec::new());
                };
                if n == 0 {
                    break (String::from_utf8_lossy(&raw).into_owned(), Vec::new());
                }
                raw.extend_from_slice(&buf[..n]);

                if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&raw[..pos]).into_owned();
                    let body = raw[pos + 4..].to_vec();
                    break (head, body);
                }
            };

            let content_length: usize = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(str::trim)
                        .map(str::to_owned)
                })
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);

            while body.len() < content_length {
                let Ok(n) = stream.read(&mut buf) else { break };
                if n == 0 {
                    break;
                }
                body.extend_from_slice(&buf[..n]);
            }

            requests_t.lock().unwrap().push(CapturedRequest {
                head,
                body: String::from_utf8_lossy(&body).into_owned(),
            });

            let resp = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    thread::sleep(Duration::from_millis(20));

    (addr, requests)
}

/// Writes a config bound to the mock token endpoint and file-backed
/// storage, returning the config and token file paths.
fn write_cc_config(dir: &Path, addr: SocketAddr, grant_lines: &str) -> (PathBuf, PathBuf) {
    let lines = format!(
        "client-id = \"app-id\"\n{grant_lines}\nendpoints.token = \"http://{addr}/token\"\nscopes = [\"https://graph.microsoft.com/.default\"]"
    );

    write_config(dir, "cc", &lines, "")
}

/// Seeds the storage file with an already expired token.
fn seed_expired_token(token: &Path) {
    std::fs::write(
        token,
        r#"{"access_token":"at-stale","token_type":"Bearer","expires_in":3600,"issued_at":1000}"#,
    )
    .unwrap();
}

/// The successful token response the mock server answers with.
const TOKEN_RESPONSE: &str =
    r#"{"access_token":"at-fresh","token_type":"Bearer","expires_in":3600}"#;

#[test]
fn token_show_reacquires_an_expired_client_credentials_token() {
    let (addr, requests) = start_mock(200, TOKEN_RESPONSE);
    let dir = TempDir::new().unwrap();
    let (config, token) = write_cc_config(
        dir.path(),
        addr,
        "grant = \"client-credentials\"\nclient-secret.raw = \"s3cret\"",
    );
    seed_expired_token(&token);

    let out = ortie(&config, &["token", "show", "--auto-refresh"]);
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("at-fresh"), "{stdout}");

    let stored = stored(&token);
    assert_eq!(stored["access_token"], "at-fresh");
    assert!(stored["refresh_token"].is_null());

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0].body.contains("grant_type=client_credentials"),
        "{}",
        requests[0].body
    );
    assert!(
        requests[0]
            .head
            .to_ascii_lowercase()
            .contains("authorization: basic"),
        "client secret must ride as Basic auth; head: {}",
        requests[0].head
    );
}

#[test]
fn token_show_acquires_when_storage_is_empty() {
    let (addr, requests) = start_mock(200, TOKEN_RESPONSE);
    let dir = TempDir::new().unwrap();
    let (config, token) = write_cc_config(
        dir.path(),
        addr,
        "grant = \"client-credentials\"\nclient-secret.raw = \"s3cret\"",
    );

    let out = ortie(&config, &["token", "show", "--auto-refresh"]);
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("at-fresh"));

    let stored = stored(&token);
    assert_eq!(stored["access_token"], "at-fresh");
    assert_eq!(requests.lock().unwrap().len(), 1);
}

#[test]
fn jwt_kind_mints_a_fresh_assertion_per_reacquisition() {
    let (addr, requests) = start_mock(200, TOKEN_RESPONSE);
    let dir = TempDir::new().unwrap();

    let key = dir.path().join("key.pem");
    std::fs::write(&key, KEY_PEM).unwrap();

    // NOTE: PEM-armored flavor, covering the armor-to-DER decoding.
    let cert = dir.path().join("cert.pem");
    let mut cert_body = Vec::new();
    Base64Codec::default().encode_vec(CERT_DER, &mut cert_body);
    let cert_pem = format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        String::from_utf8(cert_body).unwrap()
    );
    std::fs::write(&cert, cert_pem).unwrap();

    let (config, token) = write_cc_config(
        dir.path(),
        addr,
        &format!(
            "grant = \"client-credentials-jwt\"\nclient-key = \"{}\"\nclient-certificate = \"{}\"",
            key.display(),
            cert.display()
        ),
    );

    let refresh1 = ortie(&config, &["token", "refresh"]);
    assert!(refresh1.status.success(), "{refresh1:?}");
    let refresh2 = ortie(&config, &["token", "refresh"]);
    assert!(refresh2.status.success(), "{refresh2:?}");

    let stored = stored(&token);
    assert_eq!(stored["access_token"], "at-fresh");

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);

    let mut assertions = Vec::new();
    for request in requests.iter() {
        let body = &request.body;
        assert!(body.contains("grant_type=client_credentials"), "{body}");
        assert!(body.contains("client_id=app-id"), "{body}");
        assert!(
            body.contains("client_assertion_type=urn%3Aietf%3Aparams%3Aoauth%3Aclient-assertion-type%3Ajwt-bearer"),
            "{body}"
        );
        assert!(
            !request.head.to_ascii_lowercase().contains("authorization:"),
            "the assertion authenticates, no Basic header; head: {}",
            request.head
        );
        assert!(!body.contains("client_secret"), "{body}");

        let assertion = body
            .split('&')
            .find_map(|pair| pair.strip_prefix("client_assertion="))
            .expect("body carries a client_assertion")
            .to_owned();
        assertions.push(assertion);
    }

    let url = Base64Codec {
        alphabet: Base64Alphabet::Url,
        ..Base64Codec::default()
    };
    let decode_json = |part: &str| -> Value {
        let mut json = Vec::new();
        assert!(url.decode_vec(part.as_bytes(), &mut json), "{part}");
        serde_json::from_slice(&json).unwrap()
    };

    let mut jtis = Vec::new();
    for assertion in &assertions {
        let mut parts = assertion.split('.');
        let header = decode_json(parts.next().unwrap());
        let claims = decode_json(parts.next().unwrap());
        assert!(parts.next().is_some(), "assertion carries a signature");

        assert_eq!(header["alg"], "RS256");
        assert_eq!(header["x5t"], CERT_X5T);

        assert_eq!(claims["iss"], "app-id");
        assert_eq!(claims["sub"], "app-id");
        assert_eq!(claims["aud"], format!("http://{addr}/token"));
        assert_eq!(
            claims["scope"],
            Value::Null,
            "scope rides the body, not the claims"
        );

        let iat = claims["iat"].as_u64().unwrap();
        let exp = claims["exp"].as_u64().unwrap();
        assert_eq!(exp - iat, 600, "10 minute assertion validity");

        jtis.push(claims["jti"].as_str().unwrap().to_owned());
    }

    assert_ne!(jtis[0], jtis[1], "each mint carries a unique jti");
    assert_ne!(assertions[0], assertions[1], "assertions are never reused");
}

#[test]
fn invalid_client_on_the_jwt_kind_hints_certificate_renewal() {
    let (addr, _requests) = start_mock(
        400,
        r#"{"error":"invalid_client","error_description":"AADSTS700027: client assertion failed signature validation"}"#,
    );
    let dir = TempDir::new().unwrap();

    let key = dir.path().join("key.pem");
    std::fs::write(&key, KEY_PEM).unwrap();
    let cert = dir.path().join("cert.der");
    std::fs::write(&cert, CERT_DER).unwrap();

    let (config, _token) = write_cc_config(
        dir.path(),
        addr,
        &format!(
            "grant = \"client-credentials-jwt\"\nclient-key = \"{}\"\nclient-certificate = \"{}\"",
            key.display(),
            cert.display()
        ),
    );

    let out = ortie(&config, &["auth", "get"]);
    assert!(!out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("InvalidClient"), "{stdout}");
    assert!(
        stdout.contains("The certificate credential may be expired and need renewal"),
        "{stdout}"
    );
}

#[test]
fn an_error_status_without_an_oauth_body_is_reported() {
    let (addr, _requests) = start_mock(503, "<html>Service Unavailable</html>");
    let dir = TempDir::new().unwrap();
    let (config, token) = write_cc_config(
        dir.path(),
        addr,
        "grant = \"client-credentials\"\nclient-secret.raw = \"s3cret\"",
    );

    let out = ortie(&config, &["auth", "get"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("status 503"), "{stdout}");
    assert!(std::fs::read_to_string(token).unwrap().is_empty());
}
