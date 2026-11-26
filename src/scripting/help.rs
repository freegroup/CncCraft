//! The command help: every script/MCP command with what it does and each of its parameters — type,
//! whether it must be given, the values it takes and what it means. A search field narrows the list,
//! and "In Skript einfügen" hands back a ready line (the command with its required parameters) for
//! the script. Only the contents live here; the window around it is hosted by the app
//! (`src/app/script_ui.rs`).

use crate::mcp::tools::{catalog, ToolInfo};

/// The help's contents, searching with `filter`; returns the line to put into the script when
/// "In Skript einfügen" was pressed.
pub fn help_ui(ui: &mut egui::Ui, filter: &mut String) -> Option<String> {
    let mut insert = None;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(t!("Suchen")).color(crate::theme::ink()));
        ui.add(egui::TextEdit::singleline(filter).hint_text(t!("Befehl oder Parameter …")).desired_width(280.0));
    });
    ui.label(egui::RichText::new(t!("Jede Skriptzeile ist ein Befehl mit Parametern als name=wert. „c = befehl …“ merkt sich das Ergebnis, „$c.id“ nutzt es in einer späteren Zeile.")).small().color(crate::theme::muted()));
    ui.add_space(4.0);
    ui.separator();
    let search = filter.trim().to_lowercase();
    let tools: Vec<&ToolInfo> = catalog().iter().filter(|t| matches(t, &search)).collect();
    egui::ScrollArea::vertical().id_salt("command_help_scroll").auto_shrink([false, false]).show(ui, |ui| {
        if tools.is_empty() {
            ui.label(egui::RichText::new(t!("Kein Befehl passt zur Suche.")).color(crate::theme::muted()));
        }
        for tool in tools {
            if tool_card(ui, tool) {
                insert = Some(template(tool));
            }
            ui.add_space(6.0);
        }
    });
    insert
}

/// One command's card: name, "In Skript einfügen", description, its ready line and its parameters.
/// Returns whether "In Skript einfügen" was pressed.
fn tool_card(ui: &mut egui::Ui, tool: &ToolInfo) -> bool {
    let mut pressed = false;
    egui::Frame::none()
        .fill(crate::theme::surface())
        .stroke(egui::Stroke::new(1.0_f32, crate::theme::hairline()))
        .rounding(5.0)
        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&tool.name).monospace().size(15.0).strong().color(crate::theme::ink()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    pressed = ui.button(t!("In Skript einfügen")).clicked();
                });
            });
            ui.add(egui::Label::new(egui::RichText::new(&tool.description).color(crate::theme::ink())).wrap());
            ui.label(egui::RichText::new(template(tool)).monospace().small().color(crate::theme::muted()));
            if tool.params.is_empty() {
                return;
            }
            ui.add_space(4.0);
            egui::Grid::new(("command_help_params", &tool.name)).num_columns(2).spacing([14.0, 6.0]).show(ui, |ui| {
                for p in &tool.params {
                    ui.label(egui::RichText::new(&p.name).monospace().color(crate::theme::ACCENT_DEEP));
                    ui.vertical(|ui| {
                        let need = if p.required { t!("Pflicht") } else { t!("optional") };
                        let mut meta = format!("{} · {need}", p.kind);
                        if !p.values.is_empty() {
                            meta.push_str(&format!(" · {}", p.values.join(" | ")));
                        }
                        ui.label(egui::RichText::new(meta).small().color(crate::theme::muted()));
                        ui.add(egui::Label::new(egui::RichText::new(&p.description).color(crate::theme::ink())).wrap());
                    });
                    ui.end_row();
                }
            });
        });
    pressed
}

/// Whether `tool` matches the (lower-case) search: by its name, its description or a parameter.
fn matches(tool: &ToolInfo, search: &str) -> bool {
    search.is_empty()
        || tool.name.to_lowercase().contains(search)
        || tool.description.to_lowercase().contains(search)
        || tool.params.iter().any(|p| p.name.to_lowercase().contains(search))
}

/// A ready script line for `tool`: its name and its required parameters, values left to fill in.
fn template(tool: &ToolInfo) -> String {
    let mut line = tool.name.clone();
    for p in tool.params.iter().filter(|p| p.required) {
        line.push_str(&format!(" {}=", p.name));
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_template_holds_the_required_parameters_only() {
        let tool = catalog().iter().find(|t| t.name == "draw_rectangle").unwrap();
        assert_eq!(template(tool), "draw_rectangle cx= cy= width= height=");
    }

    #[test]
    fn every_parameter_says_what_it_means() {
        for tool in catalog() {
            for p in &tool.params {
                assert!(!p.description.is_empty(), "{}.{} has no description", tool.name, p.name);
            }
        }
    }

    #[test]
    fn the_search_finds_a_command_by_a_parameter() {
        let hits: Vec<&str> = catalog().iter().filter(|t| matches(t, "shape_ids")).map(|t| t.name.as_str()).collect();
        assert_eq!(hits, vec!["add_toolpath"]);
    }
}
