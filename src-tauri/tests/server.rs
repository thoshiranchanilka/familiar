use familiar_lib::server::{start, ServerError};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

const TOKEN: &str = "test-token-123";

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn post(addr: SocketAddr, path: &str, auth: Option<&str>, body: &str) -> u16 {
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let auth_line = auth.map(|t| format!("Authorization: Bearer {t}\r\n")).unwrap_or_default();
    write!(
        s,
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n{auth_line}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut resp = String::new();
    let _ = s.read_to_string(&mut resp);
    resp.split_whitespace().nth(1).and_then(|c| c.parse().ok()).unwrap_or(0)
}

fn setup() -> (familiar_lib::server::ServerHandle, Receiver<familiar_lib::normalize::MascotEvent>) {
    let (tx, rx) = channel();
    let h = start(TOKEN.into(), free_port(), tx).expect("server starts");
    (h, rx)
}

#[test]
fn accepts_authorized_hook_and_emits_event() {
    let (h, rx) = setup();
    let code = post(h.local_addr(), "/hook", Some(TOKEN), r#"{"hook_event_name":"UserPromptSubmit","session_id":"s1"}"#);
    assert_eq!(code, 200);
    let e = rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(e.event, "prompt");
    assert_eq!(e.session_id.as_deref(), Some("s1"));
}

#[test]
fn accepts_status_line_payload() {
    let (h, rx) = setup();
    let code = post(h.local_addr(), "/status", Some(TOKEN), r#"{"model":{"display_name":"Opus 5.5"}}"#);
    assert_eq!(code, 200);
    assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap().model.as_deref(), Some("Opus 5.5"));
}

#[test]
fn rejects_missing_or_wrong_token() {
    let (h, rx) = setup();
    let body = r#"{"hook_event_name":"Stop"}"#;
    assert_eq!(post(h.local_addr(), "/hook", None, body), 401);
    assert_eq!(post(h.local_addr(), "/hook", Some("wrong"), body), 401);
    assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn rejects_malformed_json() {
    let (h, rx) = setup();
    assert_eq!(post(h.local_addr(), "/hook", Some(TOKEN), "{not json"), 400);
    assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn unknown_hook_is_accepted_but_not_emitted() {
    let (h, rx) = setup();
    assert_eq!(post(h.local_addr(), "/hook", Some(TOKEN), r#"{"hook_event_name":"CwdChanged"}"#), 200);
    assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn rejects_oversized_body() {
    let (h, _rx) = setup();
    let big = format!(r#"{{"x":"{}"}}"#, "a".repeat(70 * 1024));
    assert_eq!(post(h.local_addr(), "/hook", Some(TOKEN), &big), 413);
}

#[test]
fn unknown_path_is_404() {
    let (h, _rx) = setup();
    assert_eq!(post(h.local_addr(), "/nope", Some(TOKEN), "{}"), 404);
}

#[test]
fn binds_loopback_only() {
    let (h, _rx) = setup();
    assert!(h.local_addr().ip().is_loopback());
}

#[test]
fn second_server_on_same_port_reports_port_in_use() {
    let port = free_port();
    let (tx, _rx) = channel();
    let _first = start(TOKEN.into(), port, tx.clone()).unwrap();
    assert_eq!(start(TOKEN.into(), port, tx).err(), Some(ServerError::PortInUse));
}
