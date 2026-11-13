//! Tool groups: the toolpath list shows which tool cuts from a toolpath on, and the export can
//! write one file for each stretch of toolpaths that uses the same tool (a tool change starts a
//! new stretch). Everything here is plain logic without a window, so it can be tested.

use crate::gcode::{self, PostProcessor};
use crate::toolpath::{Job, Toolpath};
use crate::tools::Tool;

/// Whether two toolpaths are cut with the same tool, so no tool change is needed between them.
pub fn same_tool(a: &Tool, b: &Tool) -> bool {
    a.name == b.name && a.kind == b.kind && a.number == b.number && (a.diameter - b.diameter).abs() < 1e-9 && (a.angle - b.angle).abs() < 1e-9
}

/// For each toolpath in list order: whether the list shows a tool entry above it, which is the case
/// for the first one and wherever the tool differs from the toolpath before.
pub fn headers(tools: &[&Tool]) -> Vec<bool> {
    tools.iter().enumerate().map(|(i, t)| i == 0 || !same_tool(tools[i - 1], t)).collect()
}

/// Splits toolpaths (in the order they are cut) into stretches of the same tool; each stretch is the
/// list of positions in `tools`.
pub fn groups(tools: &[&Tool]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    for (i, t) in tools.iter().enumerate() {
        match out.last_mut() {
            Some(last) if same_tool(tools[*last.last().unwrap()], t) => last.push(i),
            _ => out.push(vec![i]),
        }
    }
    out
}

/// A tool's name as a part of a file name: letters and digits only, umlauts spelled out, anything
/// else a single dash ("Schaftfräser (6 mm)" becomes "Schaftfraeser-6-mm").
pub fn file_part(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        match c {
            'ä' => out.push_str("ae"),
            'ö' => out.push_str("oe"),
            'ü' => out.push_str("ue"),
            'Ä' => out.push_str("Ae"),
            'Ö' => out.push_str("Oe"),
            'Ü' => out.push_str("Ue"),
            'ß' => out.push_str("ss"),
            c if c.is_ascii_alphanumeric() => out.push(c),
            _ => {
                if !out.ends_with('-') {
                    out.push('-');
                }
            }
        }
    }
    out.trim_matches('-').to_string()
}

/// One machine file per stretch of the same tool: `<stem>_<nn>_<tool>.<ext>` with the running number
/// counted from 01 in the order the stretches are cut, and its G-code.
pub fn files(job: &Job, paths: &[&Toolpath], pp: &PostProcessor, stem: &str) -> Vec<(String, String)> {
    let tools: Vec<&Tool> = paths.iter().map(|t| &t.tool).collect();
    let width = groups(&tools).len().to_string().len().max(2);
    groups(&tools)
        .iter()
        .enumerate()
        .map(|(n, group)| {
            let part: Vec<&Toolpath> = group.iter().map(|&i| paths[i]).collect();
            let name = format!("{stem}_{:0width$}_{}.{}", n + 1, file_part(&paths[group[0]].tool.name), pp.ext);
            (name, gcode::write(job, &part, pp))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcode::builtins;
    use crate::toolpath::{default_toolpath, Job};
    use crate::tools::{ToolKind, ToolLibrary};

    fn path(lib: &ToolLibrary, kind: &str, tool: Option<(ToolKind, f64)>) -> Toolpath {
        let job = Job::default();
        let mut t = default_toolpath(kind, lib, &job, vec![]);
        if let Some((k, d)) = tool {
            t.tool = lib.pick(&job.material, k, d, Some(90.0));
        }
        t.moves = vec![crate::toolpath::Move { to: [0.0, 0.0, 0.0], rapid: true }, crate::toolpath::Move { to: [5.0, 0.0, -1.0], rapid: false }];
        t
    }

    #[test]
    fn a_header_stands_only_where_the_tool_changes() {
        let lib = ToolLibrary::load();
        let a = path(&lib, "profile", Some((ToolKind::EndMill, 3.0)));
        let b = path(&lib, "pocket", Some((ToolKind::EndMill, 3.0)));
        let c = path(&lib, "vcarve", None);
        let d = path(&lib, "profile", Some((ToolKind::EndMill, 3.0)));
        let tools = [&a.tool, &b.tool, &c.tool, &d.tool];
        assert_eq!(headers(&tools), vec![true, false, true, true]);
        assert_eq!(groups(&tools), vec![vec![0, 1], vec![2], vec![3]]);
    }

    #[test]
    fn no_toolpaths_have_no_groups() {
        assert!(headers(&[]).is_empty());
        assert!(groups(&[]).is_empty());
    }

    #[test]
    fn file_names_are_plain() {
        assert_eq!(file_part("Schaftfräser (6 mm)"), "Schaftfraeser-6-mm");
        assert_eq!(file_part("V-Fräser 90°"), "V-Fraeser-90");
        assert_eq!(file_part("  "), "");
    }

    #[test]
    fn each_stretch_of_one_tool_gets_a_numbered_file() {
        let lib = ToolLibrary::load();
        let a = path(&lib, "profile", Some((ToolKind::EndMill, 3.0)));
        let b = path(&lib, "pocket", Some((ToolKind::EndMill, 3.0)));
        let c = path(&lib, "vcarve", None);
        let pp = &builtins()[0];
        let list = files(&Job::default(), &[&a, &b, &c], pp, "schild");
        assert_eq!(list.len(), 2);
        assert!(list[0].0.starts_with("schild_01_Schaftfraeser") && list[0].0.ends_with(".nc"), "{}", list[0].0);
        assert!(list[1].0.starts_with("schild_02_V-Fraeser") && list[1].0.ends_with(".nc"), "{}", list[1].0);
        assert!(list[0].1.contains("( CncCraft – 2 Werkzeugweg(e) )"));
        assert!(list[1].1.contains("( CncCraft – 1 Werkzeugweg(e) )"));
        assert!(list[0].1.contains("M6") && list[1].1.contains("M6"));
    }

    #[test]
    fn grbl_files_are_numbered_too() {
        let lib = ToolLibrary::load();
        let a = path(&lib, "profile", Some((ToolKind::EndMill, 3.0)));
        let c = path(&lib, "vcarve", None);
        let pp = &builtins()[1]; // GRBL
        let list = files(&Job::default(), &[&a, &c], pp, "x");
        assert_eq!(list.iter().map(|f| f.0.rsplit('.').next().unwrap()).collect::<Vec<_>>(), vec!["gcode", "gcode"]);
    }
}
