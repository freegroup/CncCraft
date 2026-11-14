//! Keyboard shortcuts in one table: which key does what, and where we know it from. The keys
//! come from the original's own hint texts (its message catalogues) where it names them; the
//! rest are the usual ones of a Mac program and are marked as estimated. The menus and the
//! "Tastenkürzel" window read the same table, so what they show is what the keys do.

use egui::{InputState, Key};

/// Something a key can do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    /// Switch the selected vector into node editing and back.
    NodeEdit,
    /// Cancel what is going on: a drag, the rotate mode, a line being drawn, the tool.
    Cancel,
    /// Finish the line being drawn.
    Apply,
    /// Close the line being drawn.
    ClosePolyline,
    /// Finish the line being drawn and start a new one.
    NextLine,
    Delete,
    Undo,
    Redo,
    SelectAll,
    /// Make the selection one group / take the outermost group apart.
    Group,
    Ungroup,
    Duplicate,
    Copy,
    Cut,
    Paste,
    New,
    Open,
    Save,
    SaveAs,
    View2D,
    View3D,
    FitView,
    Shortcuts,
    /// The dimension mode: constrain distances.
    Dimension,
    /// The level/upright mode: keep a line level or upright.
    Align,
    /// A drawing tool by one key: L line, R rectangle, C circle.
    Select,
    Line,
    SplineTool,
    Rectangle,
    Circle,
    /// O: the offset tool.
    Offset,
}

/// One key combination.
pub(super) struct Shortcut {
    pub action: Action,
    pub key: Key,
    pub command: bool,
    pub shift: bool,
    pub what: crate::lang::Tr,
    /// Where the original names this key, or "geschätzt".
    pub source: crate::lang::Tr,
}

const fn sc(action: Action, key: Key, command: bool, shift: bool, what: crate::lang::Tr, source: crate::lang::Tr) -> Shortcut {
    Shortcut { action, key, command, shift, what, source }
}

const HINT: crate::lang::Tr = tr!("Hinweistext des Originals (UvsT)");
const EST: crate::lang::Tr = tr!("geschätzt");

pub(super) const SHORTCUTS: &[Shortcut] = &[
    sc(Action::NodeEdit, Key::N, false, false, tr!("Ausgewählten Vektor im Knotenmodus bearbeiten / zurück"), HINT),
    sc(Action::Dimension, Key::D, false, false, tr!("Bemaßen: Abstand zwischen Punkten und Linien festlegen"), tr!("geschätzt (wie in Fusion 360)")),
    sc(Action::Align, Key::H, false, false, tr!("Waagrecht/Senkrecht: eine Linie waagrecht oder senkrecht festlegen"), tr!("gewünscht (wie in Fusion 360)")),
    sc(Action::Select, Key::S, false, false, tr!("Auswählen (Selektionsmodus)"), tr!("gewünscht (wie in Fusion 360)")),
    sc(Action::Line, Key::L, false, false, tr!("Linie zeichnen"), tr!("gewünscht (wie in Fusion 360)")),
    sc(Action::SplineTool, Key::P, false, false, tr!("Spline zeichnen"), tr!("gewünscht")),
    sc(Action::Rectangle, Key::R, false, false, tr!("Rechteck zeichnen"), tr!("gewünscht (wie in Fusion 360)")),
    sc(Action::Circle, Key::C, false, false, tr!("Kreis zeichnen"), tr!("gewünscht (wie in Fusion 360)")),
    sc(Action::Offset, Key::O, false, false, tr!("Vektoren versetzen"), tr!("gewünscht (wie in Fusion 360)")),
    sc(Action::Cancel, Key::Escape, false, false, tr!("Abbrechen · Transformationsmodus verlassen"), HINT),
    sc(Action::Apply, Key::Enter, false, false, tr!("Übernehmen · Linie beenden"), HINT),
    sc(Action::ClosePolyline, Key::Tab, false, false, tr!("Linie schließen"), HINT),
    sc(Action::NextLine, Key::Space, false, false, tr!("Linie beenden und neue beginnen"), HINT),
    sc(Action::Delete, Key::Delete, false, false, tr!("Löschen"), EST),
    sc(Action::Delete, Key::Backspace, false, false, tr!("Löschen"), EST),
    sc(Action::Undo, Key::Z, true, false, tr!("Rückgängig"), EST),
    sc(Action::Redo, Key::Z, true, true, tr!("Wiederholen"), tr!("Text „Ctrl+Shift+Z“ im Programm (0x1004eea4c)")),
    sc(Action::SelectAll, Key::A, true, false, tr!("Alles auswählen"), EST),
    sc(Action::Group, Key::G, true, false, tr!("Gruppieren (auch Gruppen in Gruppen)"), EST),
    sc(Action::Ungroup, Key::G, true, true, tr!("Gruppe auflösen (eine Ebene)"), EST),
    sc(Action::Duplicate, Key::D, true, false, tr!("Duplizieren"), EST),
    sc(Action::Copy, Key::C, true, false, tr!("Kopieren"), EST),
    sc(Action::Cut, Key::X, true, false, tr!("Ausschneiden"), EST),
    sc(Action::Paste, Key::V, true, false, tr!("Einfügen"), EST),
    sc(Action::New, Key::N, true, false, tr!("Neuer Job"), EST),
    sc(Action::Open, Key::O, true, false, tr!("Öffnen"), EST),
    sc(Action::Save, Key::S, true, false, tr!("Speichern"), EST),
    sc(Action::SaveAs, Key::S, true, true, tr!("Speichern unter"), tr!("Text „Ctrl+Shift+S“ im Programm (0x1004f1cb0)")),
    sc(Action::View2D, Key::F2, false, false, tr!("2D-Ansicht"), EST),
    sc(Action::View3D, Key::F3, false, false, tr!("3D-Ansicht"), EST),
    sc(Action::FitView, Key::F4, false, false, tr!("Ganze Ansicht"), EST),
    sc(Action::Shortcuts, Key::Slash, true, false, tr!("Diese Liste"), EST),
];

/// What the mouse does with keys held, as the original's hint texts describe it.
pub(super) const WITH_MOUSE: &[(crate::lang::Tr, crate::lang::Tr)] = &[
    (tr!("⇧ + Klick"), tr!("Mehrere Objekte aus- und abwählen")),
    (tr!("⇧ beim Ziehen"), tr!("Einrasten aus")),
    (tr!("⌥ oder ⌃ beim Verschieben"), tr!("Kopie erstellen")),
    (tr!("⌘⇧ + Pfeiltasten"), tr!("Auswahl um den festen Abstand verschieben")),
    (tr!("Pfeiltasten"), tr!("Auswahl oder Knoten verschieben (⇧: zehnfach)")),
    (tr!("↑ ↓ in Maßfeldern"), tr!("Zwischen den Feldern wechseln")),
    (tr!("Formel und „=“"), tr!("Wert ausrechnen")),
];

impl Shortcut {
    fn pressed(&self, i: &InputState) -> bool {
        // a plain letter key (a tool key like S or D) also works with Shift held, as a capital letter
        let letter = !self.command && !self.shift && self.key.name().chars().count() == 1 && self.key.name().chars().all(|c| c.is_ascii_alphabetic());
        i.modifiers.command == self.command && (i.modifiers.shift == self.shift || letter) && !i.modifiers.alt && i.key_pressed(self.key)
    }

    /// How the keys are written in a menu: "⇧⌘Z" on a Mac, "Strg+Umschalt+Z" elsewhere (the
    /// original names them "Ctrl+Shift+Z", `sub_1004ee574`, and its toolkit shows ⌘ for Ctrl on a Mac).
    pub fn keys(&self) -> String {
        self.keys_as(cfg!(target_os = "macos"))
    }

    pub fn keys_as(&self, mac: bool) -> String {
        let mut s = String::new();
        if !mac {
            if self.command {
                s.push_str(&t!("Strg+"));
            }
            if self.shift {
                s.push_str(&t!("Umschalt+"));
            }
        } else {
            if self.shift {
                s.push('⇧');
            }
            if self.command {
                s.push('⌘');
            }
        }
        s.push_str(&match self.key {
            Key::Escape => t!("Esc"),
            Key::Enter => "↩".to_string(),
            Key::Tab => "⇥".to_string(),
            Key::Space => t!("Leertaste"),
            Key::Delete => t!("Entf"),
            Key::Backspace => "⌫".to_string(),
            Key::Slash => "/".to_string(),
            k => k.name().to_string(),
        });
        s
    }
}

/// Every action whose keys went down this frame, in table order.
pub(super) fn pressed(i: &InputState) -> Vec<Action> {
    let mut out: Vec<Action> = SHORTCUTS.iter().filter(|s| s.pressed(i)).map(|s| s.action).collect();
    // on a Mac the system turns ⌘C, ⌘X and ⌘V into copy, cut and paste events and sends no key for
    // them, so the keys of the table never see those three
    for e in &i.events {
        match e {
            egui::Event::Copy => out.push(Action::Copy),
            egui::Event::Cut => out.push(Action::Cut),
            egui::Event::Paste(_) => out.push(Action::Paste),
            _ => {}
        }
    }
    out.dedup();
    out
}

/// The keys of an action, for its menu entry ("" when it has none).
pub(super) fn keys_for(action: Action) -> String {
    SHORTCUTS.iter().find(|s| s.action == action).map(Shortcut::keys).unwrap_or_default()
}

/// The "Tastenkürzel" window: every key in the table, then what keys do with the mouse.
pub(super) fn window(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new(t!("Tastenkürzel")).id(egui::Id::new("Tastenkürzel")).open(open).collapsible(false).resizable(false).default_width(420.0).show(ctx, |ui| {
        egui::Grid::new("shortcut-keys").num_columns(2).spacing([18.0, 4.0]).striped(true).show(ui, |ui| {
            let mut seen = vec![];
            for s in SHORTCUTS {
                if seen.contains(&s.action) {
                    continue;
                }
                seen.push(s.action);
                let keys: Vec<String> = SHORTCUTS.iter().filter(|o| o.action == s.action).map(Shortcut::keys).collect();
                ui.label(egui::RichText::new(keys.join("  ")).monospace().strong());
                ui.label(s.what.get()).on_hover_text(s.source.get());
                ui.end_row();
            }
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(t!("Mit der Maus")).strong());
        egui::Grid::new("shortcut-mouse").num_columns(2).spacing([18.0, 4.0]).show(ui, |ui| {
            for (keys, what) in WITH_MOUSE {
                ui.label(egui::RichText::new(keys.get()).monospace());
                ui.label(what.get());
                ui.end_row();
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_two_entries_share_keys() {
        for (i, a) in SHORTCUTS.iter().enumerate() {
            for b in &SHORTCUTS[i + 1..] {
                assert!(!(a.key == b.key && a.command == b.command && a.shift == b.shift), "{:?} and {:?}", a.action, b.action);
            }
        }
    }

    #[test]
    fn written_like_a_mac_menu() {
        rust_i18n::set_locale("de");
        let find = |a: Action| SHORTCUTS.iter().find(|s| s.action == a).unwrap();
        assert_eq!(find(Action::Redo).keys_as(true), "⇧⌘Z");
        assert_eq!(find(Action::Redo).keys_as(false), "Strg+Umschalt+Z");
        assert_eq!(find(Action::Copy).keys_as(true), "⌘C");
        assert_eq!(find(Action::NodeEdit).keys_as(true), "N");
        assert_eq!(find(Action::Delete).keys_as(true), "Entf");
    }

    #[test]
    fn n_alone_is_node_edit_and_with_command_a_new_job() {
        let mut i = InputState::default();
        i.events.push(egui::Event::Key { key: Key::N, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE });
        i.keys_down.insert(Key::N);
        assert_eq!(pressed(&i), vec![Action::NodeEdit]);
        i.modifiers = egui::Modifiers::COMMAND;
        assert_eq!(pressed(&i), vec![Action::New]);
    }
}
