//! Client credentials grants e2e via the real binary and a local
//! mock authorization server: silent re-acquisition on expiry, fresh
//! JWT assertion per mint, and the certificate renewal hint.

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use base64::{Engine, prelude::BASE64_STANDARD, prelude::BASE64_URL_SAFE_NO_PAD};
use serde_json::Value;
use tempfile::TempDir;

/// Deterministic 2048-bit test key, the RustCrypto pkcs8 crate's
/// rsa2048-priv.pem example (mirrors the io-oauth test fixture).
const KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC2xCxRXxCmqvKC
xj7b4kJDoXDz+iYzvUgzY39Hyk9vNuA6XSnvwxkayA85DYdLOeMPQU/Owfyg7YHl
R+3CzTgsdvYckBiXPbn6U3lyp8cB9rd+CYLfwV/AGSfuXnzZS09Zn/BwE6fIKBvf
Ity8mtfKu3xDEcmC9Y7bchOtRVizMiZtdDrtgZLRiEytuLFHOaja2mbclwgG2ces
RQyxPQ18V1+xmFNPxhvEG8DwV04OATDHu7+9/cn2puLj4q/xy+rIm6V4hFKNVc+w
gyeh6MifTgA88oiOkzJB2daVvLus3JC0Tj4JX6NwWOolsT9eKVy+rG3oOKuMUK9h
4piXW4cvAgMBAAECggEAfsyDYsDtsHQRZCFeIvdKudkboGkAcAz2NpDlEU2O5r3P
uy4/lhRpKmd6CD8Wil5S5ZaOZAe52XxuDkBk+C2gt1ihTxe5t9QfX0jijWVRcE9W
5p56qfpjD8dkKMBtJeRV3PxVt6wrT3ZkP97T/hX/eKuyfmWsxKrQvfbbJ+9gppEM
XEoIXtQydasZwdmXoyxu/8598tGTX25gHu3hYaErXMJ8oh+B0smcPR6gjpDjBTqw
m++nJN7w0MOjwel0DA2fdhJqFJ7Aqn2AeCBUhCVNlR2wfEz5H7ZFTAlliP1ZJNur
6zWcogJSaNAE+dZus9b3rcETm61A8W3eY54RZHN2wQKBgQDcwGEkLU6Sr67nKsUT
ymW593A2+b1+Dm5hRhp+92VCJewVPH5cMaYVem5aE/9uF46HWMHLM9nWu+MXnvGJ
mOQi7Ny+149Oz9vl9PzYrsLJ0NyGRzypvRbZ0jjSH7Xd776xQ8ph0L1qqNkfM6CX
eQ6WQNvJEIXcXyY0O6MTj2stZwKBgQDT8xR1fkDpVINvkr4kI2ry8NoEo0ZTwYCv
Z+lgCG2T/eZcsj79nQk3R2L1mB42GEmvaM3XU5T/ak4G62myCeQijbLfpw5A9/l1
ClKBdmR7eI0OV3eiy4si480mf/cLTzsC06r7DhjFkKVksDGIsKpfxIFWsHYiIUJD
vRIn76fy+QKBgQDOaLesGw0QDWNuVUiHU8XAmEP9s5DicF33aJRXyb2Nl2XjCXhh
fi78gEj0wyQgbbhgh7ZU6Xuz1GTn7j+M2D/hBDb33xjpqWPE5kkR1n7eNAQvLibj
06GtNGra1rm39ncIywlOYt7p/01dZmmvmIryJV0c6O0xfGp9hpHaNU0S2wKBgCX2
5ZRCIChrTfu/QjXA7lhD0hmAkYlRINbKeyALgm0+znOOLgBJj6wKKmypacfww8oa
sLxAKXEyvnU4177fTLDvxrmO99ulT1aqmaq85TTEnCeUfUZ4xRxjx4x84WhyMbTI
61h65u8EgMuvT8AXPP1Yen5nr1FfubnedREYOXIpAoGAMZlUBtQGIHyt6uo1s40E
DF+Kmhrggn6e0GsVPYO2ghk1tLNqgr6dVseRtYwnJxpXk9U6HWV8CJl5YLFDPlFx
mH9FLxRKfHIwbWPh0//Atxt1qwjy5FpILpiEUcvkeOEusijQdFbJJLZvbO0EjYU/
Uz4xpoYU8cPObY7JmDznKvc=
-----END PRIVATE KEY-----
";

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
/// status and body, capturing each request (headers plus the full
/// content-length body) for later assertions.
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
fn write_config(dir: &Path, addr: SocketAddr, grant_lines: &str) -> (PathBuf, PathBuf) {
    let token = dir.join("token.json");
    std::fs::write(&token, b"").unwrap();

    let config = dir.join("config.toml");
    std::fs::write(
        &config,
        format!(
            r#"
[accounts.cc]
default = true
client-id = "app-id"
{grant_lines}
endpoints.token = "http://{addr}/token"
scopes = ["https://graph.microsoft.com/.default"]
storage.read.command = ["cat", "{t}"]
storage.write.command = ["tee", "{t}"]
"#,
            t = token.display()
        ),
    )
    .unwrap();

    (config, token)
}

/// Seeds the storage file with an already expired token.
fn seed_expired_token(token: &Path) {
    std::fs::write(
        token,
        r#"{"access_token":"at-stale","token_type":"Bearer","expires_in":3600,"issued_at":1000}"#,
    )
    .unwrap();
}

fn ortie(config: &Path, args: &[&str]) -> std::process::Output {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_ortie"));
    Command::new(&bin)
        .arg("-c")
        .arg(config)
        .args(args)
        .output()
        .unwrap()
}

const TOKEN_RESPONSE: &str =
    r#"{"access_token":"at-fresh","token_type":"Bearer","expires_in":3600}"#;

#[test]
fn token_show_reacquires_an_expired_client_credentials_token() {
    let (addr, requests) = start_mock(200, TOKEN_RESPONSE);
    let dir = TempDir::new().unwrap();
    let (config, token) = write_config(
        dir.path(),
        addr,
        "grant = \"client-credentials\"\nclient-secret.raw = \"s3cret\"",
    );
    seed_expired_token(&token);

    let out = ortie(&config, &["token", "show", "--auto-refresh"]);
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("at-fresh"), "{stdout}");

    let stored: Value = serde_json::from_str(&std::fs::read_to_string(&token).unwrap()).unwrap();
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
    let (config, token) = write_config(
        dir.path(),
        addr,
        "grant = \"client-credentials\"\nclient-secret.raw = \"s3cret\"",
    );

    let out = ortie(&config, &["token", "show", "--auto-refresh"]);
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("at-fresh"));

    let stored: Value = serde_json::from_str(&std::fs::read_to_string(&token).unwrap()).unwrap();
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
    let cert_pem = format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        BASE64_STANDARD.encode(CERT_DER)
    );
    std::fs::write(&cert, cert_pem).unwrap();

    let (config, token) = write_config(
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

    let stored: Value = serde_json::from_str(&std::fs::read_to_string(&token).unwrap()).unwrap();
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

    let decode_json = |part: &str| -> Value {
        serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(part).unwrap()).unwrap()
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

    let (config, _token) = write_config(
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
