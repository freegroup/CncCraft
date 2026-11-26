//! The agent guide (`agent.md`, a "skill" file): everything an AI agent needs to write CncCraft
//! scripts on its own — what the app and the script language are for, the syntax, the coordinate
//! system, the pacing rules, every command with its parameters and return value, every
//! menu item and button, and a few example scripts. It is generated from the same catalogs the MCP
//! server and the editor use, so it always matches the program it came from.


use crate::mcp::tools::catalog;

/// Short example scripts, each a (title, script). They are checked by the tests: every line parses
/// and uses a command that exists.
pub const EXAMPLES: &[(&str, &str)] = &[
    (
        "Draw a part, cut it out and show it in 3D",
        "# a 300 x 200 mm plywood board
replay_speed speed=1
new_job width=300 height=200 thickness=18 material=Plywood
text_title text=\"CncCraft — CAM for the hobbyist\"\ntext_subtitle text=\"A round pocket in two steps\" duration=3
c = draw_circle cx=150 cy=100 radius=40
add_toolpath kind=pocket shape_ids=[$c.id]
r = draw_rectangle cx=150 cy=100 width=240 height=160 corner_radius=10
add_toolpath kind=cutout shape_ids=[$r.id]
view mode=3d
toolpath_preview
wait seconds=2",
    ),
    (
        "Snap a shape onto construction lines",
        "new_job width=300 height=200
g = guide_add axis=vertical offset=50
h = guide_add axis=horizontal offset=150
s = draw_rectangle cx=150 cy=100 width=60 height=40
text_subtitle text=\"Drag it onto the guides\"
drag_shape id=$s.id left=51 top=149     # lands on x=50 / y=150 and attaches
wait seconds=1
guide_move id=$g.id offset=80           # the attached shape follows the line
guides_lock_all",
    ),
    (
        "Use the menus and panels like a person",
        "panel_milling_open
edit_select_all
edit_duplicate
press_key key=z cmd=true                 # undo, shown as keycaps
panel_tools_close
text_subtitle text=\"\"               # remove the subtitle",
    ),
];

/// The whole guide as Markdown.
pub fn agent_md() -> String {
    let mut md = String::new();
    md.push_str(INTRO);
    md.push_str("\n## Commands\n\nEvery line of a script is one of these commands. Parameters are `name=value`; required ones are marked. *Returns* is what a later line can use through a variable.\n");
    for tool in catalog() {
        md.push_str(&format!("\n### `{}`\n\n{}\n\n", tool.name, tool.description));
        if tool.params.is_empty() {
            md.push_str("No parameters.\n\n");
        } else {
            md.push_str("| Parameter | Type | Required | Values | Meaning |\n|---|---|---|---|---|\n");
            for p in &tool.params {
                let values = if p.values.is_empty() { String::new() } else { p.values.join(" \\| ") };
                md.push_str(&format!("| `{}` | {} | {} | {} | {} |\n", p.name, p.kind, if p.required { "yes" } else { "no" }, values, p.description.replace('|', "\\|")));
            }
            md.push('\n');
        }
        md.push_str(&format!("*Returns:* {}\n\n", tool.returns));
        let mut line = tool.name.clone();
        for p in tool.params.iter().filter(|p| p.required) {
            line.push_str(&format!(" {}={}", p.name, sample_value(p)));
        }
        md.push_str(&format!("```\n{line}\n```\n"));
    }
    md.push_str("\n## Example scripts\n");
    for (title, script) in EXAMPLES {
        md.push_str(&format!("\n### {title}\n\n```\n{script}\n```\n"));
    }
    md
}

/// The full command reference as an HTML fragment (no `<html>`/`<body>`), for the help website's
/// scripting page. Generated from the same [`catalog`] the automation server advertises, so the
/// published list of commands can never drift from the program. The docs build injects it where it
/// finds the marker `<!--COMMANDS-->`.
pub fn command_reference_html() -> String {
    fn esc(s: &str) -> String {
        s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
    }
    let mut h = String::new();
    h.push_str("<p>Every line of a script is one of these commands. Parameters are written <code>name=value</code>; required ones are marked. <em>Returns</em> is what a later line can use through a variable (for example <code>$name.id</code>). This list is generated from the program itself, so it always matches your version.</p>\n");
    for tool in catalog() {
        h.push_str(&format!("<h3 id=\"cmd-{}\"><code>{}</code></h3>\n<p>{}</p>\n", esc(&tool.name), esc(&tool.name), esc(&tool.description)));
        if tool.params.is_empty() {
            h.push_str("<p><em>No parameters.</em></p>\n");
        } else {
            h.push_str("<table>\n<tr><th>Parameter</th><th>Type</th><th>Required</th><th>Values</th><th>Meaning</th></tr>\n");
            for p in &tool.params {
                let values = if p.values.is_empty() { String::new() } else { esc(&p.values.join(", ")) };
                h.push_str(&format!(
                    "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    esc(&p.name), esc(&p.kind), if p.required { "yes" } else { "no" }, values, esc(&p.description),
                ));
            }
            h.push_str("</table>\n");
        }
        h.push_str(&format!("<p><em>Returns:</em> {}</p>\n", esc(&tool.returns)));
    }
    h
}

/// A plausible value for a parameter in a generated example line.
fn sample_value(p: &crate::mcp::tools::ParamInfo) -> String {
    if let Some(first) = p.values.first() {
        return first.clone();
    }
    match (p.kind.as_str(), p.name.as_str()) {
        ("array", _) => "[$s.id]".into(),
        ("boolean", _) => "true".into(),
        (_, "id") => "$s.id".into(),
        ("number", _) => "10".into(),
        (_, "path") => "/tmp/out".into(),
        _ => "\"text\"".into(),
    }
}

const INTRO: &str = "# CncCraft agent guide (skill)

CncCraft is a CNC/CAD editor: you draw shapes on a material board (in millimetres), attach milling
operations (toolpaths) to them, preview the cut in 3D and export G-code. It can be **remote-controlled
for demo recordings**: a script — or an AI agent over MCP — drives the program the way a person would,
with a visible virtual mouse. Every move, click and key press is shown on screen (a large mouse in the
bottom-right corner shows the pressed button, keys appear as keycaps), so the result can be recorded
as a tutorial or TikTok-style video. Captions can be faded in: a standing `text_title` and a changing `text_subtitle`.

Use this guide to write scripts. Play them in CncCraft's script window (Help → Script…, then Play), or
send each line as a `tools/call` to the MCP server at `http://127.0.0.1:8787/` (JSON-RPC over HTTP,
localhost only, no sign-in; it always runs while the app is open).

## Script syntax

- One command per line: `command name=value name=value …`.
- Values: numbers (`120`, `2.5`), words (`Plywood`, `vertical`), quoted text (`\"Hello world\"`),
  lists (`[1, 2, 3]`), `true` / `false`.
- `# …` or `// …` starts a comment (also at the end of a line). Blank lines are ignored.
- `name = command …` keeps the command's result under `name`. A later line uses it as `$name` (the
  whole result) or `$name.field`, e.g. `$c.id` — also inside lists: `shape_ids=[$c.id, $r.id]`.
- Drawing commands return `{\"id\": N}`: keep it (`c = draw_circle …`) to cut, move or delete the shape later.
- A line that fails stops the script, and the log says why.

## Coordinates and units

- Shapes, guides and toolpaths use **millimetres on the board**: x from the left edge, y from the
  **bottom** edge (y grows upwards). The board is `width` × `height` from `new_job`.
- The pointer commands (`move_to`, `click`, `drag`, `long_press`, `double_click`) use **screen pixels**
  from the window's top-left; prefer the board-level commands (`drag_shape`, `guide_move`, …), which
  find the right screen place themselves.

## How a script runs

- Each line waits until the visible mouse has finished the previous one, so nothing happens faster
  than a viewer can follow. A change happens only once the mouse has arrived (it moves to a guide's
  handle, then the guide moves).
- `wait seconds=N` holds the scene; `replay_speed speed=N` (0.25–4) makes the mouse slower or faster.
- `text_subtitle` (and `text_title`) do not wait: show the caption, then do the thing it describes.
- Things must be on screen to be clicked: if a shape is outside the view, run
  `view_fit` first. `view mode=3d` shows the board in 3D; the cuts appear after `toolpath_preview`
  (the mouse opens the toolpaths panel on the right and clicks Preview).
- Locked construction lines (`guides_lock_all`) cannot be added, moved or deleted until
  `guides_unlock_all`.
- Read the current situation with `state` (shapes, toolpaths, selection), `guides_list` and `window`
  (window size, panels, view, replay speed, whether the mouse is still busy).
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::commands::COMMANDS;

    #[test]
    fn every_command_and_every_menu_item_is_in_the_guide() {
        let md = agent_md();
        for tool in catalog() {
            assert!(md.contains(&format!("### `{}`", tool.name)), "{} is missing", tool.name);
            assert!(!tool.returns.is_empty(), "{} does not say what it returns", tool.name);
        }
        // every menu item and button is a command of its own, under its script name
        for c in COMMANDS {
            let name = crate::mcp::commands::tool_name(c.id);
            assert!(md.contains(&format!("### `{name}`")), "{} ({}) is missing", name, c.id);
        }
    }

    #[test]
    fn the_example_scripts_parse_and_use_real_commands() {
        for (title, script) in EXAMPLES {
            let statements = crate::scripting::parse(script).unwrap_or_else(|(line, e)| panic!("{title}, line {line}: {e}"));
            for s in statements {
                assert!(catalog().iter().any(|t| t.name == s.tool), "{title}: unknown command {}", s.tool);
                let tool = catalog().iter().find(|t| t.name == s.tool).unwrap();
                for key in s.args.as_object().unwrap().keys() {
                    assert!(tool.param(key).is_some(), "{title}: {} has no parameter {key}", s.tool);
                }
            }
        }
    }

    #[test]
    fn the_generated_example_lines_parse_too() {
        let md = agent_md();
        let lines: Vec<&str> = md.split("```\n").skip(1).step_by(2).flat_map(|block| block.lines()).collect();
        for line in lines {
            assert!(crate::scripting::parse(line).is_ok(), "does not parse: {line}");
        }
    }
}
