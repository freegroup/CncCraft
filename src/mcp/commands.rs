//! The command catalog: the menu items, tool buttons and view switches, each of which is its own
//! MCP and script command — "every menu item, button and gesture is a command". Every entry becomes
//! a tool of its own (see `mcp/tools/catalog_command.rs`) named after its id in script style
//! ([`tool_name`]: `edit.selectAll` → `edit_select_all`), so it shows in the editor's autocomplete,
//! the command help and the agent guide like every other command; nothing hides behind a generic
//! "run this id" command.
//!
//! This table is pure metadata: a stable id (`edit.undo`, `view.3d`, `toolpath.recomputeAll` …), a
//! human label, the menu path it lives under, its keyboard keys and a note on what it does. It
//! deliberately knows nothing about the app's own types — the host binds each id to behaviour in
//! `App::run_command` (through the visible menu, or the panel the button is in), so the catalog stays
//! generic. Commands that take arguments (drawing a shape, a construction line) are written as their
//! own tools instead.

use crate::lang::Tr;

/// One command in the catalog: its stable id, label, the menu path it lives under, keys and params.
pub struct Command {
    /// A stable, machine-facing id, e.g. "edit.undo" (never translated).
    pub id: &'static str,
    /// A short human label (translated for display).
    pub label: Tr,
    /// Where it lives in the menus, for discovery (translated), e.g. "Bearbeiten".
    pub path: Tr,
    /// The keyboard keys, Mac style ("⌘Z"), or "" when it has none. Kept as text so the catalog
    /// stays independent of the app's shortcut table.
    pub keys: &'static str,
    /// A one-line note on what it does beyond its label (where the mouse goes), or "".
    pub params: &'static str,
}

const fn c(id: &'static str, label: Tr, path: Tr, keys: &'static str, params: &'static str) -> Command {
    Command { id, label, path, keys, params }
}

/// Every command the program exposes by name, grouped roughly by menu. The ids are stable; the
/// labels and paths are translated only when shown.
pub const COMMANDS: &[Command] = &[
    // ----- Datei -----
    c("file.new", tr!("Neuer Job"), tr!("Datei"), "⌘N", ""),
    c("file.open", tr!("Öffnen…"), tr!("Datei"), "⌘O", ""),
    c("file.save", tr!("Speichern"), tr!("Datei"), "⌘S", ""),
    c("file.saveAs", tr!("Speichern unter…"), tr!("Datei"), "⇧⌘S", ""),
    c("file.exportToolpaths", tr!("Werkzeugwege speichern…"), tr!("Datei"), "", ""),
    c("file.startPage", tr!("Startseite"), tr!("Datei"), "", ""),
    // ----- Bearbeiten -----
    c("edit.undo", tr!("Rückgängig"), tr!("Bearbeiten"), "⌘Z", ""),
    c("edit.redo", tr!("Wiederholen"), tr!("Bearbeiten"), "⇧⌘Z", ""),
    c("edit.cut", tr!("Ausschneiden"), tr!("Bearbeiten"), "⌘X", ""),
    c("edit.copy", tr!("Kopieren"), tr!("Bearbeiten"), "⌘C", ""),
    c("edit.paste", tr!("Einfügen"), tr!("Bearbeiten"), "⌘V", ""),
    c("edit.duplicate", tr!("Duplizieren"), tr!("Bearbeiten"), "⌘D", ""),
    c("edit.delete", tr!("Löschen"), tr!("Bearbeiten"), "⌫", ""),
    c("edit.group", tr!("Gruppieren"), tr!("Bearbeiten"), "⌘G", ""),
    c("edit.ungroup", tr!("Gruppe auflösen"), tr!("Bearbeiten"), "⇧⌘G", ""),
    c("edit.selectAll", tr!("Alles auswählen"), tr!("Bearbeiten"), "⌘A", ""),
    c("edit.nodeEdit", tr!("Knoten bearbeiten"), tr!("Bearbeiten"), "N", ""),
    c("edit.jobSetup", tr!("Werkstück einrichten…"), tr!("Bearbeiten"), "", ""),
    c("edit.cancel", tr!("Abbrechen"), tr!("Bearbeiten"), "Esc", ""),
    // ----- Werkzeuge (Zeichenwerkzeuge) -----
    c("tool.select", tr!("Auswählen"), tr!("Werkzeuge"), "S", ""),
    c("tool.line", tr!("Linie zeichnen"), tr!("Werkzeuge"), "L", ""),
    c("tool.spline", tr!("Spline zeichnen"), tr!("Werkzeuge"), "P", ""),
    c("tool.rectangle", tr!("Rechteck zeichnen"), tr!("Werkzeuge"), "R", ""),
    c("tool.circle", tr!("Kreis zeichnen"), tr!("Werkzeuge"), "C", ""),
    c("tool.offset", tr!("Vektoren versetzen"), tr!("Werkzeuge"), "O", ""),
    c("tool.dimension", tr!("Bemaßen"), tr!("Werkzeuge"), "D", ""),
    c("tool.align", tr!("Waagrecht/Senkrecht"), tr!("Werkzeuge"), "H", ""),
    // ----- Ansicht -----
    c("view.2d", tr!("2D-Ansicht"), tr!("Ansicht"), "F2", ""),
    c("view.3d", tr!("3D-Ansicht"), tr!("Ansicht"), "F3", ""),
    c("view.fit", tr!("Ganze Ansicht"), tr!("Ansicht"), "F4", ""),
    c("view.toggleDimensions", tr!("Maße anzeigen"), tr!("Ansicht"), "", ""),
    // ----- Werkzeugwege -----
    c("toolpath.profile", tr!("Profil"), tr!("Werkzeugwege"), "", "opens the Profil toolpath editor for the selected shapes"),
    c("toolpath.pocket", tr!("Tasche"), tr!("Werkzeugwege"), "", "opens the Tasche (pocket) toolpath editor"),
    c("toolpath.drill", tr!("Bohren"), tr!("Werkzeugwege"), "", "opens the Bohren (drill) toolpath editor"),
    c("toolpath.vcarve", tr!("V-Carving"), tr!("Werkzeugwege"), "", "opens the V-Carving toolpath editor"),
    c("toolpath.cutout", tr!("Ausschnitt"), tr!("Werkzeugwege"), "", "opens the Ausschnitt (cutout) toolpath editor"),
    c("toolpath.recomputeAll", tr!("Alle neu berechnen"), tr!("Werkzeugwege"), "", ""),
    c("toolpath.preview", tr!("Vorschau"), tr!("Werkzeugwege-Leiste"), "", "the mouse opens the toolpaths panel on the right if needed and clicks Preview: the finished cut in 3D"),
    c("toolpath.simulate", tr!("Simulieren"), tr!("Werkzeugwege-Leiste"), "", "the mouse opens the toolpaths panel on the right if needed and clicks Simulate: the cut animated in 3D"),
    c("toolpath.gcodeMotionOn", tr!("G-Code-Fahrt einblenden"), tr!("Werkzeugwege-Leiste"), "", "the mouse clicks the \"G-code motion in 3D\" checkbox so the machine's moves are drawn over the 3D view (does nothing if already on)"),
    c("toolpath.gcodeMotionOff", tr!("G-Code-Fahrt ausblenden"), tr!("Werkzeugwege-Leiste"), "", "the mouse clicks the \"G-code motion in 3D\" checkbox so the moves are hidden (does nothing if already off)"),
];

/// Find a command by its id.
pub fn find(id: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.id == id)
}

/// The MCP/script command name of catalog id `id`: the dot and every capital become `_` + lower
/// case, so `edit.selectAll` is `edit_select_all` and `view.3d` is `view_3d`.
pub fn tool_name(id: &str) -> String {
    let mut name = String::with_capacity(id.len() + 4);
    for c in id.chars() {
        match c {
            '.' => name.push('_'),
            c if c.is_ascii_uppercase() => {
                name.push('_');
                name.push(c.to_ascii_lowercase());
            }
            c => name.push(c),
        }
    }
    name
}

/// The place (`path` key) of the commands that are buttons of the toolpaths panel on the right
/// rather than menu items: they are carried out by clicking the button there.
pub const TOOLPATHS_PANEL: &str = "Werkzeugwege-Leiste";

/// The real top-level menus, by their (untranslated) name key. Commands whose `path` is one of
/// these can be driven through the visible menu; the others (the tool column, the construction
/// lines) have no menu of their own.
pub const MENUS: &[&str] = &["Datei", "Bearbeiten", "Ansicht", "Werkzeugwege", "Hilfe"];

/// The top-level menu a command lives in, if it is one of the real menus (so it can be opened with
/// the cursor); `None` for commands that are not in a menu.
pub fn menu_name(cmd: &Command) -> Option<&'static str> {
    MENUS.iter().copied().find(|m| *m == cmd.path.key())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        for (i, a) in COMMANDS.iter().enumerate() {
            for b in &COMMANDS[i + 1..] {
                assert_ne!(a.id, b.id, "duplicate command id {}", a.id);
            }
        }
    }

    #[test]
    fn ids_are_dotted_and_lowercase_start() {
        for c in COMMANDS {
            assert!(c.id.contains('.'), "{} should be namespaced like group.name", c.id);
            assert!(c.id.chars().next().unwrap().is_ascii_lowercase(), "{} should start lowercase", c.id);
        }
    }

    #[test]
    fn find_gets_a_known_command() {
        assert_eq!(find("edit.undo").map(|c| c.id), Some("edit.undo"));
        assert!(find("nope.nope").is_none());
    }

    #[test]
    fn command_names_are_script_style() {
        assert_eq!(tool_name("edit.selectAll"), "edit_select_all");
        assert_eq!(tool_name("toolpath.simulate"), "toolpath_simulate");
        assert_eq!(tool_name("view.3d"), "view_3d");
        for c in COMMANDS {
            let name = tool_name(c.id);
            assert!(name.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'), "{name}");
        }
    }
}
