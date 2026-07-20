//! Device grant e2e via the real binary and a local mock AS.

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::PathBuf,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use serde_json::Value;
use tempfile::TempDir;

fn start_mock() -> (SocketAddr, Arc<AtomicUsize>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let polls = Arc::new(AtomicUsize::new(0));
    let polls_t = Arc::clone(&polls);
    let handle = thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0u8; 8192];
            let Ok(n) = stream.read(&mut buf) else {
                continue;
            };
            let req = String::from_utf8_lossy(&buf[..n]);
            let path = req
                .lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1))
                .unwrap_or("/");
            let (status, body): (u16, String) = if path.starts_with("/devicecode") {
                (
                    200,
                    format!(
                        r#"{{"device_code":"dc-test","user_code":"USER","verification_uri":"http://{addr}/d","expires_in":60,"interval":1}}"#
                    ),
                )
            } else if path.starts_with("/token") {
                if polls_t.fetch_add(1, Ordering::SeqCst) == 0 {
                    (400, r#"{"error":"authorization_pending"}"#.into())
                } else {
                    (
                        200,
                        r#"{"access_token":"at-test","token_type":"Bearer","expires_in":3600}"#
                            .into(),
                    )
                }
            } else {
                (404, r#"{"error":"not_found"}"#.into())
            };
            let resp = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    thread::sleep(Duration::from_millis(20));
    (addr, polls, handle)
}

#[test]
fn auth_get_json_then_resume_stores_token() {
    let (addr, polls, _h) = start_mock();
    let dir = TempDir::new().unwrap();
    let token = dir.path().join("token.json");
    std::fs::write(&token, b"").unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(
        &config,
        format!(
            r#"
[accounts.device]
default = true
client-id = "c"
grant = "device"
endpoints.device-authorization = "http://{addr}/devicecode"
endpoints.token = "http://{addr}/token"
storage.read.command = ["cat", "{t}"]
storage.write.command = ["tee", "{t}"]
"#,
            t = token.display()
        ),
    )
    .unwrap();
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_ortie"));

    let get = Command::new(&bin)
        .args(["-c", config.to_str().unwrap(), "--json", "auth", "get"])
        .output()
        .unwrap();
    assert!(get.status.success(), "{get:?}");
    let v: Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(v["device_code"], "dc-test");
    assert_eq!(polls.load(Ordering::SeqCst), 0);

    let resume = Command::new(&bin)
        .args(["-c", config.to_str().unwrap(), "auth", "resume", "dc-test"])
        .output()
        .unwrap();
    assert!(resume.status.success(), "{resume:?}");
    assert!(polls.load(Ordering::SeqCst) >= 2);
    let stored: Value = serde_json::from_str(&std::fs::read_to_string(&token).unwrap()).unwrap();
    assert_eq!(stored["access_token"], "at-test");
}
