//! The in-process MCP server: a small JSON-RPC-over-HTTP endpoint bound to 127.0.0.1 only, so an AI
//! client (Claude) can drive CncCraft the way a person would. Unlike GridCraft's on-demand CLI, this
//! server **runs always, inside the app**: it is started at launch (see `App::new`), listens on a
//! fixed loopback port with no authentication, and hands each tool call to the app loop over a
//! channel; the app carries it out on the next frame (egui is single-threaded, so nothing touches
//! the UI from the thread) and sends a short result back. Pointer actions are played through the
//! visible [`crate::app::demo`] cursor, so a recording shows the mouse move and the clicks.
//!
//! Every command is a tool of its own, and the tools are exactly the script commands ("every menu
//! item, button and gesture is a command"): the strongly-typed ones that take parameters (drawing a
//! shape, creating a toolpath) and one per entry of the [`commands`] catalog of menu items and
//! buttons. The server here is only transport; the catalog and the handlers live with the app, so
//! this module stays generic.

pub mod commands;
mod tool;
pub(crate) mod tools;

pub use tool::ToolResult;

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};

/// A tool call forwarded to the app loop, with a channel to send the result back.
pub struct Call {
    pub tool: String,
    pub args: Value,
    pub reply: Sender<ToolResult>,
}

/// The app side of the server: the receiver of calls. The app drains it each frame.
pub struct Mcp {
    pub rx: Receiver<Call>,
    /// The port the server listens on, for the window title / logging.
    pub port: u16,
}

/// The fixed port the server listens on, so a client always knows where to reach it.
pub const PORT: u16 = 8787;

/// Start the server on the fixed loopback port. Returns the app-side handle, or `None` if it could
/// not bind (e.g. a second instance is already running); then the app simply runs without the MCP.
/// No authentication: access is limited to loopback (127.0.0.1) only.
pub fn start() -> Option<Mcp> {
    let listener = TcpListener::bind(("127.0.0.1", PORT)).ok()?;
    eprintln!("CncCraft MCP: http://127.0.0.1:{PORT}/");
    run(listener)
}

/// Serve on an already bound listener (the fixed port in the app, a free one in tests).
fn run(listener: TcpListener) -> Option<Mcp> {
    let port = listener.local_addr().ok()?.port();
    let (tx, rx) = std::sync::mpsc::channel::<Call>();
    std::thread::Builder::new()
        .name("cnccraft-mcp".into())
        .spawn(move || serve(listener, tx))
        .ok()?;
    Some(Mcp { rx, port })
}

/// The accept loop. Each connection is handled in its own thread, so one slow or idle client (an
/// MCP HTTP client keeps its socket open) can never block the others.
fn serve(listener: TcpListener, tx: Sender<Call>) {
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        // loopback only: refuse anything that is not 127.0.0.1 / ::1
        if stream.peer_addr().map(|a| !a.ip().is_loopback()).unwrap_or(true) {
            continue;
        }
        let tx = tx.clone();
        std::thread::spawn(move || { let _ = handle(stream, &tx); });
    }
}

/// Read HTTP requests on one keep-alive connection and answer each with its JSON-RPC message. The
/// `BufReader` is created once, outside the loop, so bytes of a pipelined next request are not lost.
fn handle(mut stream: TcpStream, tx: &Sender<Call>) -> std::io::Result<()> {
    let read = stream.try_clone()?;
    let mut reader = BufReader::new(read);
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(()); // connection closed
        }
        // request line "POST / HTTP/1.1" — we only serve POST to /; headers until a blank line
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            if reader.read_line(&mut h)? == 0 {
                return Ok(());
            }
            let h = h.trim_end();
            if h.is_empty() {
                break;
            }
            let lower = h.to_ascii_lowercase();
            if let Some(v) = lower.strip_prefix("content-length:") {
                len = v.trim().parse().unwrap_or(0);
            }
        }
        let mut body = vec![0u8; len];
        reader.read_exact(&mut body)?;
        let req: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        let resp = dispatch(&req, tx);
        write_http(&mut stream, &resp)?;
    }
}

/// Turn one JSON-RPC request into a response: `initialize`, `tools/list`, `tools/call`.
fn dispatch(req: &Value, tx: &Sender<Call>) -> Value {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "cnccraft", "version": env!("CARGO_PKG_VERSION")},
        }),
        "tools/list" => json!({"tools": tools::specs()}),
        "tools/call" => {
            let name = req.pointer("/params/name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let args = req.pointer("/params/arguments").cloned().unwrap_or(json!({}));
            let (reply_tx, reply_rx) = std::sync::mpsc::channel();
            if tx.send(Call { tool: name.clone(), args, reply: reply_tx }).is_err() {
                return rpc_err(id, "app is not running");
            }
            // the app answers within a bounded time (a long demo step can take a few seconds)
            match reply_rx.recv_timeout(std::time::Duration::from_secs(120)) {
                Ok(r) => json!({"content": [{"type": "text", "text": r.text}], "isError": r.is_error}),
                Err(_) => json!({"content": [{"type": "text", "text": "timed out"}], "isError": true}),
            }
        }
        "notifications/initialized" | "ping" => return json!({"jsonrpc": "2.0", "id": id, "result": {}}),
        other => return rpc_err(id, &format!("unknown method {other}")),
    };
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn rpc_err(id: Value, msg: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": msg}})
}

fn write_http(stream: &mut TcpStream, body: &Value) -> std::io::Result<()> {
    let s = serde_json::to_string(body).unwrap_or_default();
    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{s}", s.len())?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(stream: &mut TcpStream, body: &str) -> String {
        write!(stream, "POST / HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        stream.flush().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            reader.read_line(&mut h).unwrap();
            let h = h.trim_end().to_ascii_lowercase();
            if h.is_empty() {
                break;
            }
            if let Some(v) = h.strip_prefix("content-length:") {
                len = v.trim().parse().unwrap();
            }
        }
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn an_idle_open_connection_does_not_block_a_second_client() {
        let mcp = run(TcpListener::bind("127.0.0.1:0").unwrap()).unwrap();
        let addr = ("127.0.0.1", mcp.port);
        // a client that connects and then just keeps the socket open, as MCP clients do
        let _idle = TcpStream::connect(addr).unwrap();
        // a second client must still get its answer
        let mut second = TcpStream::connect(addr).unwrap();
        second.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
        let resp = post(&mut second, r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
        assert!(resp.contains("add_toolpath"), "second client got the tool list: {resp}");
    }

    #[test]
    fn several_requests_on_one_kept_alive_connection_all_get_answers() {
        let mcp = run(TcpListener::bind("127.0.0.1:0").unwrap()).unwrap();
        let mut s = TcpStream::connect(("127.0.0.1", mcp.port)).unwrap();
        s.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
        for id in 1..=3 {
            let resp = post(&mut s, &format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"initialize"}}"#));
            assert!(resp.contains("cnccraft"), "request {id} on the same connection answered: {resp}");
        }
    }
}
