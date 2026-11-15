//! The "Tastenkürzel" window: every key in the table, then what keys do with the mouse.

use crate::app::shortcuts::*;

/// The "Tastenkürzel" window: every key in the table, then what keys do with the mouse.
pub(in crate::app) fn window(ctx: &egui::Context, open: &mut bool) {
    super::window("Tastenkürzel", ctx).default_width(420.0).show(ctx, |ui| {
        if super::header(ui, &t!("Tastenkürzel"), true) {
            *open = false;
        }
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
