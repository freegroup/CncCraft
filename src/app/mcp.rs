//! The in-process MCP server: a tiny JSON-RPC-over-HTTP endpoint bound to 127.0.0.1 only, so an AI
//! client (Claude) can drive CncCraft the way a person would, for feature demos. It runs in a
//! background thread and hands each tool call to the app loop over a channel; the app carries it out
//! on the next frame (egui is single-threaded, so nothing touches the UI from the thread) and sends
//! a short result text back. Every pointer action is played through the visible [`super::demo`]
//! cursor, so the recording shows the mouse move, the button press and any long press.
//!
//! Only loopback is accepted and a random token (printed at start and also in `MCP_PORT`/`MCP_TOKEN`
//! files under the config dir) must be sent, so nothing outside the machine can reach it.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};

/// A tool call forwarded to the app loop, with a channel to send the one-line result back.
pub struct Call {
    pub tool: String,
    pub args: Value,
    pub reply: Sender<Result<String, String>>,
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
        "tools/list" => json!({"tools": tool_specs()}),
        "tools/call" => {
            let name = req.pointer("/params/name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let args = req.pointer("/params/arguments").cloned().unwrap_or(json!({}));
            let (reply_tx, reply_rx) = std::sync::mpsc::channel();
            if tx.send(Call { tool: name.clone(), args, reply: reply_tx }).is_err() {
                return rpc_err(id, "app is not running");
            }
            // the app answers within a bounded time (a long demo step can take a few seconds)
            match reply_rx.recv_timeout(std::time::Duration::from_secs(120)) {
                Ok(Ok(text)) => json!({"content": [{"type": "text", "text": text}]}),
                Ok(Err(e)) => json!({"content": [{"type": "text", "text": e}], "isError": true}),
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

/// The tools the server advertises. Kept small and powerful: a few low-level pointer/keyboard
/// actions (which together can do anything the user can) plus a handful of high-level shortcuts.
fn tool_specs() -> Vec<Value> {
    let num = || json!({"type": "number"});
    let obj = |props: Value, req: Value| json!({"type": "object", "properties": props, "required": req});
    vec![
        json!({"name": "cursor", "description": "Show or hide the virtual mouse cursor.", "inputSchema": obj(json!({"on": {"type": "boolean"}}), json!(["on"]))}),
        json!({"name": "move_to", "description": "Glide the cursor to a screen point (pixels from the top-left of the window).", "inputSchema": obj(json!({"x": num(), "y": num()}), json!(["x", "y"]))}),
        json!({"name": "click", "description": "Click at a screen point with the left (default) or right button.", "inputSchema": obj(json!({"x": num(), "y": num(), "button": {"type": "string", "enum": ["left", "right"]}}), json!(["x", "y"]))}),
        json!({"name": "double_click", "description": "Double-click at a screen point.", "inputSchema": obj(json!({"x": num(), "y": num()}), json!(["x", "y"]))}),
        json!({"name": "long_press", "description": "Long-press at a screen point (to open a value for editing, like a person holding the mouse).", "inputSchema": obj(json!({"x": num(), "y": num()}), json!(["x", "y"]))}),
        json!({"name": "drag", "description": "Press at (x1,y1), drag to (x2,y2) and release.", "inputSchema": obj(json!({"x1": num(), "y1": num(), "x2": num(), "y2": num()}), json!(["x1", "y1", "x2", "y2"]))}),
        json!({"name": "type_text", "description": "Type text into the focused field.", "inputSchema": obj(json!({"text": {"type": "string"}}), json!(["text"]))}),
        json!({"name": "press_key", "description": "Press a key (Enter, Escape, Tab, Delete, Backspace, Space, or a single letter/digit) with optional modifiers cmd/shift/alt/ctrl.", "inputSchema": obj(json!({"key": {"type": "string"}, "cmd": {"type": "boolean"}, "shift": {"type": "boolean"}, "alt": {"type": "boolean"}, "ctrl": {"type": "boolean"}}), json!(["key"]))}),
        json!({"name": "wait", "description": "Pause for a number of seconds (a beat for the viewer).", "inputSchema": obj(json!({"seconds": num()}), json!(["seconds"]))}),
        json!({"name": "new_job", "description": "Start a new job with a size (mm), thickness (mm) and material key (e.g. Softwood, Hardwood, Plywood, MDF, Acrylic, Aluminium).", "inputSchema": obj(json!({"width": num(), "height": num(), "thickness": num(), "material": {"type": "string"}}), json!(["width", "height"]))}),
        json!({"name": "list_materials", "description": "List the available material keys for a job (e.g. for new_job).", "inputSchema": obj(json!({}), json!([]))}),
        json!({"name": "list_tools", "description": "List the cutters available for the current job's material (name, kind, diameter in mm).", "inputSchema": obj(json!({}), json!([]))}),
        json!({"name": "state", "description": "Report the current job (size, material), the shapes (id, kind, centre, size) and the toolpaths (name, kind, depth). Use this to find shape ids before adding toolpaths.", "inputSchema": obj(json!({}), json!([]))}),
        json!({"name": "draw_rectangle", "description": "Add a rectangle by its centre (mm) and width/height (mm); optional corner_radius. Returns its shape id.", "inputSchema": obj(json!({"cx": num(), "cy": num(), "width": num(), "height": num(), "corner_radius": num()}), json!(["cx", "cy", "width", "height"]))}),
        json!({"name": "draw_circle", "description": "Add a circle by centre (mm) and radius (mm). Returns its shape id.", "inputSchema": obj(json!({"cx": num(), "cy": num(), "radius": num()}), json!(["cx", "cy", "radius"]))}),
        json!({"name": "draw_ellipse", "description": "Add an ellipse by centre (mm) and full width/height (mm). Returns its shape id.", "inputSchema": obj(json!({"cx": num(), "cy": num(), "width": num(), "height": num()}), json!(["cx", "cy", "width", "height"]))}),
        json!({"name": "draw_polygon", "description": "Add a regular polygon by centre (mm), radius (mm) and number of sides (>=3). Returns its shape id.", "inputSchema": obj(json!({"cx": num(), "cy": num(), "radius": num(), "sides": num()}), json!(["cx", "cy", "radius", "sides"]))}),
        json!({"name": "add_text", "description": "Add text by centre (mm), the text and the letter height (mm). Returns its shape id.", "inputSchema": obj(json!({"cx": num(), "cy": num(), "text": {"type": "string"}, "height": num()}), json!(["cx", "cy", "text"]))}),
        json!({"name": "delete_shapes", "description": "Delete shapes by their ids.", "inputSchema": obj(json!({"ids": {"type": "array", "items": num()}}), json!(["ids"]))}),
        json!({"name": "select", "description": "Select shapes by their ids (clears the previous selection).", "inputSchema": obj(json!({"ids": {"type": "array", "items": num()}}), json!(["ids"]))}),
        json!({"name": "set_variable", "description": "Create or change a document variable: a name and an expression (a number or formula). Shapes and toolpaths bound to it follow.", "inputSchema": obj(json!({"name": {"type": "string"}, "expression": {"type": "string"}}), json!(["name", "expression"]))}),
        json!({"name": "add_toolpath", "description": "Create a milling toolpath on the given shapes. kind is profile, cutout, pocket, slot, drill or vcarve; optional cut_depth and start_depth (mm) and tool_diameter (mm). Returns the toolpath name.", "inputSchema": obj(json!({"kind": {"type": "string", "enum": ["profile", "cutout", "pocket", "slot", "drill", "vcarve"]}, "shape_ids": {"type": "array", "items": num()}, "cut_depth": num(), "start_depth": num(), "tool_diameter": num(), "name": {"type": "string"}}), json!(["kind", "shape_ids"]))}),
        json!({"name": "delete_toolpath", "description": "Delete a toolpath by its index in the list (0-based).", "inputSchema": obj(json!({"index": num()}), json!(["index"]))}),
        json!({"name": "save_project", "description": "Save the project to an absolute .cnccraft path.", "inputSchema": obj(json!({"path": {"type": "string"}}), json!(["path"]))}),
        json!({"name": "export_gcode", "description": "Write the visible toolpaths as a G-code file to an absolute path.", "inputSchema": obj(json!({"path": {"type": "string"}}), json!(["path"]))}),
        json!({"name": "view", "description": "Switch to the 2D or 3D view.", "inputSchema": obj(json!({"mode": {"type": "string", "enum": ["2d", "3d"]}}), json!(["mode"]))}),
        json!({"name": "window", "description": "Report the window size and whether a demo is still playing, so the client can place clicks and wait.", "inputSchema": obj(json!({}), json!([]))}),
        json!({"name": "demo_window", "description": "Open or close the in-app Demo & remote-control window.", "inputSchema": obj(json!({"open": {"type": "boolean"}}), json!([]))}),
        json!({"name": "screenshot", "description": "Save a PNG of the window to the given absolute path.", "inputSchema": obj(json!({"path": {"type": "string"}}), json!(["path"]))}),
    ]
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
