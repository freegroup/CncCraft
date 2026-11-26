//! A tiny script language on top of the MCP tools, so a whole TikTok-style demo can be written out
//! and played back hands-free. It is deliberately small — like AutoHotkey or Robot Framework, one
//! command per line — because every line is just one of the MCP tools (see [`crate::mcp`]); the
//! player that runs them lives with the app (`src/app/script.rs`), this module turns text into
//! statements, resolves the variables between them, and describes the tools for the editor's
//! autocomplete, so it stays generic and easy to test.
//!
//! Grammar (one statement per line):
//! - `# …` or `// …` or a blank line — a comment, skipped.
//! - `tool key=value …` — run the MCP tool `tool` with those arguments.
//! - `name = tool key=value …` — the same, and keep the tool's result (JSON) under `name`.
//!
//! Every tool returns a small JSON object, so a later line can use an earlier result: write `$name`
//! for the whole value or `$name.field` for one field (e.g. the shape id a drawing tool returned).
//! References work inside lists too: `add_toolpath kind=profile shape_ids=[$c.id]`.
//!
//! ```text
//! c = draw_circle cx=150 cy=100 radius=40      # c = {"id": 2}
//! add_toolpath kind=profile shape_ids=[$c.id]  # uses the circle's id
//! g = guide_add axis=vertical offset=50
//! view mode=3d
//! ```

use serde_json::{json, Value};

pub mod client;
pub mod editor;
pub mod help;
pub mod skill;

/// A parsed statement: an optional variable to bind the result to, which tool to run, its raw
/// arguments (references still as `$name.field` text), and the line it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub line_no: usize,
    pub bind: Option<String>,
    pub tool: String,
    pub args: Value,
}

/// Parse a whole script into statements, with the error message for any line that doesn't parse.
pub fn parse(text: &str) -> Result<Vec<Statement>, (usize, String)> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line_no = i + 1;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        out.push(parse_line(line_no, line).map_err(|e| (line_no, e))?);
    }
    Ok(out)
}

/// Drop a trailing `# …` or `// …` comment (not inside quotes).
fn strip_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut in_quote = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_quote = !in_quote,
            b'#' if !in_quote => return &line[..i],
            b'/' if !in_quote && i + 1 < bytes.len() && bytes[i + 1] == b'/' => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// Parse one non-empty, non-comment line into a statement.
fn parse_line(line_no: usize, line: &str) -> Result<Statement, String> {
    // an optional "name = …" at the front binds the result (the "=" stands alone, spaced, so it is
    // not mistaken for an argument's key=value)
    let (bind, body) = split_binding(line);
    let tokens = tokenize(body)?;
    let mut it = tokens.into_iter();
    let tool = it.next().ok_or("empty statement")?;
    let mut map = serde_json::Map::new();
    for tok in it {
        let (key, value) = tok.split_once('=').ok_or_else(|| format!("expected key=value, got `{tok}`"))?;
        if key.is_empty() {
            return Err(format!("empty argument name in `{tok}`"));
        }
        map.insert(key.to_string(), parse_value(value));
    }
    Ok(Statement { line_no, bind, tool, args: Value::Object(map) })
}

/// Split off a leading `name =` binding, if present (the name is a bare identifier and the `=` is
/// surrounded by spaces, so it is not mistaken for an argument's `key=value`).
fn split_binding(line: &str) -> (Option<String>, &str) {
    if let Some(eq) = line.find(" = ") {
        let name = &line[..eq];
        if is_identifier(name) {
            return (Some(name.to_string()), &line[eq + 3..]);
        }
    }
    (None, line)
}

/// A bare variable name: letters, digits and underscores, starting with a letter.
fn is_identifier(s: &str) -> bool {
    let mut cs = s.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_alphabetic()) && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Split a line into whitespace-separated tokens, keeping `"quoted strings"` and `[1, 2]` lists in
/// one piece.
fn tokenize(line: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut depth = 0i32;
    for c in line.chars() {
        match c {
            '"' => {
                in_quote = !in_quote;
                cur.push(c);
            }
            '[' if !in_quote => {
                depth += 1;
                cur.push(c);
            }
            ']' if !in_quote => {
                depth -= 1;
                cur.push(c);
            }
            c if c.is_whitespace() && !in_quote && depth == 0 => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if in_quote {
        return Err("unclosed quote".into());
    }
    if depth != 0 {
        return Err("unbalanced [ ]".into());
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    Ok(tokens)
}

/// Read one argument value, keeping `$ref` text as a string so it can be resolved at run time: a
/// list `[a, b, …]` is read element by element (so a reference inside it survives), otherwise a
/// JSON number/bool/string where it parses, else a `$ref` or a bare word kept as a string.
fn parse_value(s: &str) -> Value {
    let s = s.trim();
    if s.starts_with('$') {
        return json!(s); // a reference, resolved later
    }
    // a list: parse each element on its own, so "[$c.id, 3]" keeps the reference
    if let (Some(inner), true) = (s.strip_prefix('['), s.ends_with(']')) {
        let inner = &inner[..inner.len() - 1];
        if inner.trim().is_empty() {
            return json!([]);
        }
        return Value::Array(inner.split(',').map(|e| parse_value(e.trim())).collect());
    }
    if let Ok(v) = serde_json::from_str::<Value>(s) {
        return v;
    }
    let trimmed = s.trim_matches('"');
    json!(trimmed)
}

/// Replace every `$name` / `$name.field` reference in `args` with the captured value from `vars`.
/// A reference to something missing is an error (so a mistyped variable is caught, not silently
/// sent as the literal text).
pub fn resolve(args: &Value, vars: &std::collections::HashMap<String, Value>) -> Result<Value, String> {
    match args {
        Value::String(s) if s.starts_with('$') => lookup(s, vars),
        Value::Array(a) => Ok(Value::Array(a.iter().map(|v| resolve(v, vars)).collect::<Result<_, _>>()?)),
        Value::Object(o) => {
            let mut out = serde_json::Map::new();
            for (k, v) in o {
                out.insert(k.clone(), resolve(v, vars)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other.clone()),
    }
}

/// Resolve one `$name` or `$name.field.field` against the captured variables.
fn lookup(reference: &str, vars: &std::collections::HashMap<String, Value>) -> Result<Value, String> {
    let path = reference.trim_start_matches('$');
    let mut parts = path.split('.');
    let name = parts.next().unwrap_or("");
    let mut cur = vars.get(name).ok_or_else(|| format!("unknown variable ${name}"))?;
    for field in parts {
        cur = cur.get(field).ok_or_else(|| format!("{reference}: no field \"{field}\""))?;
    }
    Ok(cur.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn comments_and_blanks_are_skipped() {
        let s = parse("# a comment\n\n// another\nstate\n").unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].tool, "state");
    }

    #[test]
    fn key_values_become_json_of_the_right_type() {
        let s = parse("draw_circle cx=150 cy=100 radius=40").unwrap();
        assert_eq!(s[0].args, json!({"cx": 150, "cy": 100, "radius": 40}));
        assert_eq!(s[0].bind, None);
    }

    #[test]
    fn a_binding_keeps_the_variable_name() {
        let s = parse("c = draw_circle cx=10 cy=10 radius=5").unwrap();
        assert_eq!(s[0].bind.as_deref(), Some("c"));
        assert_eq!(s[0].tool, "draw_circle");
    }

    #[test]
    fn an_argument_key_value_is_not_mistaken_for_a_binding() {
        let s = parse("view mode=3d").unwrap();
        assert_eq!(s[0].bind, None);
        assert_eq!(s[0].tool, "view");
        assert_eq!(s[0].args["mode"], json!("3d"));
    }

    #[test]
    fn references_resolve_from_earlier_results() {
        let s = parse("add_toolpath kind=profile shape_ids=[$c.id]").unwrap();
        let mut vars = HashMap::new();
        vars.insert("c".to_string(), json!({"id": 7}));
        let resolved = resolve(&s[0].args, &vars).unwrap();
        assert_eq!(resolved, json!({"kind": "profile", "shape_ids": [7]}));
    }

    #[test]
    fn a_trailing_comment_after_a_command_is_dropped() {
        let s = parse("draw_circle cx=1 cy=2 radius=3   # make a dot").unwrap();
        assert_eq!(s[0].args, json!({"cx": 1, "cy": 2, "radius": 3}));
    }

    #[test]
    fn an_unknown_variable_is_an_error() {
        let s = parse("select ids=[$nope.id]").unwrap();
        assert!(resolve(&s[0].args, &HashMap::new()).is_err());
    }

    #[test]
    fn a_bad_line_reports_its_number() {
        let err = parse("state\ndraw_circle oops").unwrap_err();
        assert_eq!(err.0, 2);
    }
}
