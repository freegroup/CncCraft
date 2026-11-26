//! The script engine as an MCP client: it runs a script by talking to the main window over the
//! loopback MCP port, exactly as an outside agent would — it does not reach into the app's state. A
//! background thread connects to `127.0.0.1:PORT`, sends each line as a `tools/call`, keeps each
//! result under its variable so later lines can use it, and waits (polling the `window` tool's
//! `busy` flag) until the visible cursor has finished a line before the next. Log lines come back
//! over a channel so the script window can show progress while the engine runs on its own.

use crate::scripting::{self, Statement};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

/// One line of progress from a running script.
#[derive(Clone, Debug)]
pub struct LogLine {
    pub line_no: usize,
    pub text: String,
    pub error: bool,
}

/// A handle to a running script: the log lines it sends, and a flag to tell it to stop.
pub struct Run {
    pub rx: Receiver<LogLine>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub done: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Run {
    /// Ask the running script to stop after the current line.
    pub fn stop(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    /// Whether the run has finished (drained, errored or stopped).
    pub fn is_done(&self) -> bool {
        self.done.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Start running a script over the MCP port on a background thread. Parse errors are reported as the
/// first (and only) log line. `port` is [`crate::mcp::PORT`].
pub fn start(text: &str, port: u16) -> Run {
    let (tx, rx) = std::sync::mpsc::channel();
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let statements = scripting::parse(text);
    let (stop_t, done_t) = (stop.clone(), done.clone());
    std::thread::Builder::new()
        .name("cnccraft-script".into())
        .spawn(move || {
            match statements {
                Ok(stmts) => run(stmts, port, &tx, &stop_t),
                Err((line_no, msg)) => {
                    let _ = tx.send(LogLine { line_no, text: format!("Zeile {line_no}: {msg}"), error: true });
                }
            }
            done_t.store(true, std::sync::atomic::Ordering::Relaxed);
        })
        .ok();
    Run { rx, stop, done }
}

/// Run every statement in order, pausing between lines until the app is idle.
fn run(stmts: Vec<Statement>, port: u16, tx: &Sender<LogLine>, stop: &std::sync::atomic::AtomicBool) {
    let mut conn = match Conn::connect(port) {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(LogLine { line_no: 0, text: format!("MCP nicht erreichbar: {e}"), error: true });
            return;
        }
    };
    if let Err(e) = conn.initialize() {
        let _ = tx.send(LogLine { line_no: 0, text: format!("MCP-Start fehlgeschlagen: {e}"), error: true });
        return;
    }
    let mut vars: HashMap<String, Value> = HashMap::new();
    let mut ok = true;
    for stmt in stmts {
        if stop.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = tx.send(LogLine { line_no: stmt.line_no, text: "gestoppt".into(), error: false });
            ok = false;
            break;
        }
        // fill in $variable references from earlier results
        let args = match scripting::resolve(&stmt.args, &vars) {
            Ok(v) => v,
            Err(e) => {
                let _ = tx.send(LogLine { line_no: stmt.line_no, text: format!("{}: {e}", stmt.tool), error: true });
                ok = false;
                break;
            }
        };
        match conn.call_tool(&stmt.tool, &args) {
            Ok((text, is_error)) => {
                if let Some(name) = &stmt.bind {
                    let value = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| Value::String(text.clone()));
                    vars.insert(name.clone(), value);
                }
                let _ = tx.send(LogLine { line_no: stmt.line_no, text: format!("{} → {}", stmt.tool, text), error: is_error });
                if is_error {
                    ok = false;
                    break; // a failed line stops the run, so a mistake is noticed
                }
            }
            Err(e) => {
                let _ = tx.send(LogLine { line_no: stmt.line_no, text: format!("{}: {e}", stmt.tool), error: true });
                ok = false;
                break;
            }
        }
        // wait until the visible cursor has finished this line's moves before the next
        conn.wait_until_idle(stop);
    }
    // a run that ended early (a failed line or Stop) leaves nothing half-done: close a recording the
    // script had started, so the next run can record again without "already running"
    if !ok {
        if let Ok((text, false)) = conn.call_tool("recording_stop", &json!({})) {
            let _ = tx.send(LogLine { line_no: 0, text: format!("recording stopped: {text}"), error: false });
        }
    }
    // when the run is over (any way it ended), take the captions off: a standing title must not
    // stay on screen after the script
    let _ = conn.call_tool("clear_overlays", &json!({}));
}

/// A keep-alive MCP connection over loopback: one TCP stream, HTTP/JSON-RPC like the server speaks.
struct Conn {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    id: u64,
}

impl Conn {
    fn connect(port: u16) -> std::io::Result<Conn> {
        let stream = TcpStream::connect(("127.0.0.1", port))?;
        stream.set_read_timeout(Some(Duration::from_secs(130)))?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Conn { stream, reader, id: 0 })
    }

    fn initialize(&mut self) -> Result<(), String> {
        self.rpc("initialize", json!({})).map(|_| ())
    }

    /// Call a tool and return (text, is_error) from its content.
    fn call_tool(&mut self, name: &str, args: &Value) -> Result<(String, bool), String> {
        let result = self.rpc("tools/call", json!({"name": name, "arguments": args}))?;
        let text = result.pointer("/content/0/text").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let is_error = result.get("isError").and_then(|v| v.as_bool()).unwrap_or(false);
        Ok((text, is_error))
    }

    /// Poll the `window` tool until its `busy` flag is false (the cursor has settled), or stop.
    fn wait_until_idle(&mut self, stop: &std::sync::atomic::AtomicBool) {
        for _ in 0..1200 {
            if stop.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            match self.call_tool("window", &json!({})) {
                Ok((text, _)) => {
                    let busy = serde_json::from_str::<Value>(&text).ok().and_then(|v| v.get("busy").and_then(|b| b.as_bool())).unwrap_or(false);
                    if !busy {
                        return;
                    }
                }
                Err(_) => return,
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// One JSON-RPC round trip, returning the `result` object.
    fn rpc(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.id += 1;
        let body = json!({"jsonrpc": "2.0", "id": self.id, "method": method, "params": params}).to_string();
        write!(self.stream, "POST / HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()).map_err(|e| e.to_string())?;
        self.stream.flush().map_err(|e| e.to_string())?;
        // read headers for the content length, then the body
        let mut len = 0usize;
        loop {
            let mut line = String::new();
            if self.reader.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
                return Err("connection closed".into());
            }
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                len = v.trim().parse().unwrap_or(0);
            }
        }
        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
        let msg: Value = serde_json::from_slice(&buf).map_err(|e| e.to_string())?;
        if let Some(err) = msg.get("error") {
            return Err(err.get("message").and_then(|m| m.as_str()).unwrap_or("error").to_string());
        }
        Ok(msg.get("result").cloned().unwrap_or(Value::Null))
    }
}
