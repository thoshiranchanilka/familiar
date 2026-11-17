//! Localhost-only HTTP server receiving Claude Code hook and status-line POSTs.
//! Routes: `POST /hook` (hook JSON) and `POST /status` (status-line JSON),
//! both requiring `Authorization: Bearer <token>`.

use crate::normalize::{from_hook, from_status, MascotEvent};
use std::io::{ErrorKind, Read};
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Request, Response, Server};

pub const DEFAULT_PORT: u16 = 47653;
const MAX_BODY: usize = 64 * 1024;

#[derive(Debug, PartialEq)]
pub enum ServerError {
    PortInUse,
    Io(String),
}

/// Stops the server when dropped.
pub struct ServerHandle {
    addr: SocketAddr,
    server: Arc<Server>,
}

impl ServerHandle {
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

pub fn start(token: String, port: u16, sink: Sender<MascotEvent>) -> Result<ServerHandle, ServerError> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).map_err(|e| match e.kind() {
        ErrorKind::AddrInUse => ServerError::PortInUse,
        _ => ServerError::Io(e.to_string()),
    })?;
    let addr = listener.local_addr().map_err(|e| ServerError::Io(e.to_string()))?;
    let server = Arc::new(Server::from_listener(listener, None).map_err(|e| ServerError::Io(e.to_string()))?);

    let worker = Arc::clone(&server);
    std::thread::spawn(move || {
        for req in worker.incoming_requests() {
            handle(req, &token, &sink);
        }
    });
    Ok(ServerHandle { addr, server })
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Compares in time independent of where the first difference is.
fn token_matches(given: &[u8], expected: &[u8]) -> bool {
    if given.len() != expected.len() {
        return false;
    }
    given.iter().zip(expected).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

fn authorized(req: &Request, token: &str) -> bool {
    req.headers()
        .iter()
        .find(|h| h.field.equiv("Authorization"))
        .and_then(|h| h.value.as_str().strip_prefix("Bearer "))
        .is_some_and(|given| token_matches(given.trim().as_bytes(), token.as_bytes()))
}

fn handle(mut req: Request, token: &str, sink: &Sender<MascotEvent>) -> u16 {
    let route = match (req.method().as_str(), req.url()) {
        ("POST", "/hook") => Some(true),
        ("POST", "/status") => Some(false),
        _ => None,
    };
    let code = match route {
        None => 404,
        Some(_) if !authorized(&req, token) => 401,
        Some(_) if req.body_length().is_some_and(|n| n > MAX_BODY) => 413,
        Some(is_hook) => {
            let mut body = Vec::new();
            let read = req.as_reader().take(MAX_BODY as u64 + 1).read_to_end(&mut body);
            if read.is_err() {
                400
            } else if body.len() > MAX_BODY {
                413
            } else {
                match serde_json::from_slice::<serde_json::Value>(&body) {
                    Err(_) => 400,
                    Ok(v) => {
                        let ev = if is_hook { from_hook(&v, now_ms()) } else { from_status(&v, now_ms()) };
                        if let Some(ev) = ev {
                            let _ = sink.send(ev);
                        }
                        200
                    }
                }
            }
        }
    };
    let _ = req.respond(Response::empty(code));
    code
}
