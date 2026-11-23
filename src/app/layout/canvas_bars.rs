//! The bars around the drawing: the toolbar over it, the pointer and size line under it, and the
//! hints floating on it.

use crate::app::shortcuts::Action;
use crate::app::{icon_button, icon_button_tip, App, Icon, Tip, ViewMode};
use crate::theme::{hairline, ink};
use crate::widgets::SlideToggle;
use egui::{Pos2, Sense, Stroke, Vec2};
use shape_edit::selection_bounds;
use super::tool_column::GAP;
use super::widgets::toolbar_group;

impl App {
    /// The toolbar over the drawing area: snapping and grid on the left; 2D/3D and zoom at the
    /// right end, as in the original.
    pub(in crate::app) fn canvas_toolbar(&mut self, ui: &mut egui::Ui) {
        let (bar, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 68.0), Sense::hover());
        ui.painter().rect_filled(bar, 0.0, crate::theme::panel());
        ui.painter().line_segment([bar.left_bottom(), bar.right_bottom()], Stroke::new(1.0_f32, hairline()));
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(bar.shrink2(Vec2::new(8.0, 4.0))), |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = GAP;
                // the tools are in two groups, each in a thin box with its name under it (like the
                // ribbon of Fusion 360): what is drawn on the sheet, and the constraints
                // (the job is set up from the tool column only, as in the original: job_setup is
                // the last entry of its tool list, 0x10011d034)
                toolbar_group(ui, &t!("Dekoration"), |ui| {
                    let shown = self.origin_shown();
                    if icon_button_tip(ui, Icon::Origin, shown, Tip::new(t!("Nullpunkt"), t!("Zeigt oder versteckt das Achsenkreuz im Blatt und in der 3D-Ansicht."))).clicked() {
                        self.set_origin_shown(!shown);
                    }
                    let grid = icon_button_tip(ui, Icon::Grid, self.snap_grid, Tip::new(t!("Am Raster einrasten"), t!("Punkte springen beim Zeichnen und Verschieben auf das nächste Rasterkreuz.")));
                    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("cnccraft_grid_button"), grid.rect));
                    if grid.clicked() {
                        self.snap_grid = !self.snap_grid;
                    }
                    if icon_button_tip(ui, Icon::Snap, self.snap_geometry, Tip::new(t!("An Geometrie einrasten"), t!("Punkte springen auf Ecken, Mitten und Linien vorhandener Formen."))).clicked() {
                        self.snap_geometry = !self.snap_geometry;
                    }
                    if icon_button_tip(ui, Icon::Dimensions, self.show_dims, Tip::new(t!("Maße anzeigen"), t!("Blaue Maße sind Constraints. Linien: blau = frei, schwarz = Länge festgelegt."))).clicked() {
                        self.show_dims = !self.show_dims;
                    }
                    let rulers = self.rulers_shown();
                    if icon_button_tip(ui, Icon::Ruler, rulers, Tip::new(t!("Lineale"), t!("Zeigt die Lineale am Rand. Ein Klick darauf legt eine Hilfslinie an, die man zum Einrasten nutzen kann."))).clicked() {
                        self.set_rulers_shown(!rulers);
                    }
                });
                toolbar_group(ui, &t!("Constraints"), |ui| {
                    if icon_button_tip(ui, Icon::Distance, self.dim_mode.is_some(), Tip::new(t!("Abstand bemaßen"), t!("Zwei Punkte oder einen Punkt und eine Linie wählen, dann den Abstand eingeben.")).shortcut("D")).clicked() {
                        self.toggle_dim_mode();
                    }
                    if icon_button_tip(ui, Icon::Align, self.align_mode.is_some(), Tip::new(t!("Waagrecht / Senkrecht"), t!("Eine Linie oder zwei Punkte wählen; CncCraft legt sie waagrecht oder senkrecht fest.")).shortcut("H")).clicked() {
                        self.toggle_align_mode();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, Icon::ZoomSelection, false, &t!("Auf die Auswahl zoomen")).clicked() {
                        self.zoom_to_selection();
                    }
                    if icon_button(ui, Icon::ZoomFit, false, &t!("Ganze Ansicht")).clicked() {
                        self.run(Action::FitView);
                    }
                    // slide toggle: 2D / 3D
                    let mut idx = if self.mode == ViewMode::ThreeD { 1 } else { 0 };
                    let toggle = SlideToggle::new("view_toggle_slide", &["2D", "3D"]).width(72.0);
                    let hint = if idx == 1 { t!("Zur 2D-Ansicht wechseln") } else { t!("Zur 3D-Ansicht wechseln") };
                    toggle.show(ui, &mut idx).on_hover_text(hint);
                    self.mode = if idx == 1 { ViewMode::ThreeD } else { ViewMode::TwoD };
                });
            });
        });
    }

    /// Whether the origin is drawn in the view that is showing: the sheet's axis cross in 2D, the
    /// zero cross in 3D (the original's "Draw Origin" setting and its overlay switch,
    /// `Toggle_Origin_On/Off`).
    fn origin_shown(&mut self) -> bool {
        match self.mode {
            ViewMode::TwoD => self.sheet.switches().find(|(n, _)| *n == "Koordinatenkreuz").is_some_and(|(_, on)| *on),
            ViewMode::ThreeD => self.view_decos.switches().find(|(n, _)| *n == "Nullpunkt-Achsenkreuz").is_some_and(|(_, on)| *on),
        }
    }

    /// Show or hide the origin in both views at once.
    pub(in crate::app) fn set_origin_shown(&mut self, on: bool) {
        for (name, shown) in self.sheet.switches() {
            if name == "Koordinatenkreuz" {
                *shown = on;
            }
        }
        for (name, shown) in self.view_decos.switches() {
            if name == "Nullpunkt-Achsenkreuz" {
                *shown = on;
            }
        }
        self.status = if on { "Nullpunkt wird gezeigt".into() } else { "Nullpunkt ausgeblendet".into() };
    }

    /// Zoom the drawing so the selection fills it.
    fn zoom_to_selection(&mut self) {
        let Some((lo, hi)) = selection_bounds(&self.shapes, &self.edit.selection) else {
            self.run(Action::FitView);
            return;
        };
        let r = self.canvas_rect;
        let (w, h) = ((hi[0] - lo[0]).max(1.0) as f32, (hi[1] - lo[1]).max(1.0) as f32);
        self.mode = ViewMode::TwoD;
        self.zoom = ((r.width() - 120.0) / w).min((r.height() - 120.0) / h).clamp(0.05, 200.0);
        let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        self.origin = Pos2::new(r.center().x - c[0] as f32 * self.zoom, r.center().y + c[1] as f32 * self.zoom);
        self.fitted = true;
    }

    /// Let the worksheet's interactive decorations (the hints) react, over the 2D drawing and
    /// the 3D view alike, and do what they ask for.
    pub(in crate::app) fn sheet_interact(&mut self, ctx: &egui::Context) {
        // the decorations are set aside while they read the view, which borrows the app
        let mut sheet = std::mem::replace(&mut self.sheet, crate::sheet::Decorations::empty());
        let actions = sheet.interact(&self.sheet_view(self.canvas_rect), ctx);
        self.sheet = sheet;
        for action in actions {
            match action {
                crate::sheet::SheetAction::ShowToolHelp => {
                    self.status = self.tool.tip().into();
                    self.shortcuts_open = true;
                }
            }
        }
    }

    /// Pointer position, the selection's size and the layer, as the original's bottom line. While
    /// toolpaths are worked out in the background, a thin bar runs along its top edge: drawn over
    /// the line, not as a strip of its own, so the drawing never shifts when it comes and goes.
    pub(in crate::app) fn status_bar(&mut self, ctx: &egui::Context) {
        let panel = egui::TopBottomPanel::bottom("status").exact_height(24.0).frame(egui::Frame::none().fill(crate::theme::surface()).inner_margin(egui::Margin::symmetric(12.0, 3.0))).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let mut text = String::new();
                if let Some(p) = self.mouse_mm {
                    text.push_str(&format!("X[{:.4} mm] | Y[{:.4} mm]", p[0], p[1]));
                }
                if let Some((lo, hi)) = selection_bounds(&self.shapes, &self.edit.selection) {
                    text.push_str(&format!("   W[{:.3} mm] | H[{:.3} mm]", hi[0] - lo[0], hi[1] - lo[1]));
                }
                text.push_str(&format!(" | {}", self.layers.name(self.layers.active)));
                ui.label(egui::RichText::new(text).monospace().size(12.0).color(ink()));
            });
        });
        if self.bg_compute.is_some() {
            let line = panel.response.rect;
            let rect = egui::Rect::from_min_size(line.min, egui::Vec2::new(line.width(), PROGRESS_H));
            let t = ctx.input(|i| i.time) % 2.0;
            let frac = if t < 1.0 { t } else { 2.0 - t } as f32;
            let glow_w = rect.width() * 0.25;
            let cx = rect.left() + (rect.width() - glow_w) * frac + glow_w * 0.5;
            let bar = egui::Rect::from_x_y_ranges(cx - glow_w * 0.5..=cx + glow_w * 0.5, rect.y_range());
            let painter = ctx.layer_painter(panel.response.layer_id);
            painter.rect_filled(rect, 0.0, crate::theme::separator());
            painter.rect_filled(bar, 1.5, crate::app::ACCENT);
            ctx.request_repaint();
        }
    }
}

/// Height of the background-work bar on the status line's top edge.
const PROGRESS_H: f32 = 3.0;
