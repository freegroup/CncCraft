//! The rectangular **pattern** tool ("Vervielfältigen"): a parametric grid of copies of one
//! source shape. Unlike a plain duplicate, a pattern stays live:
//!
//! - It is a [`Pattern`] record (source id, group id, counts and centre distances as text). The
//!   source and its children share one group, so they pick, move and scale as one "PatternShape".
//! - The children are rebuilt ([`App::regenerate_pattern`]) whenever the source moves or changes,
//!   or a variable the counts/distances use changes.
//! - The source carries a small marker; a double-click on it reopens the dialog ([`App::edit_pattern`]).
//! - A toolpath put on the source is applied to every child too ([`App::pattern_toolpath_ids`]);
//!   children never own toolpaths of their own.
//! - Deleting the source deletes the children; `Ungroup` frees the children, copying the source's
//!   toolpaths onto each so they stand on their own.
//!
//! Everything goes through the shapes+patterns snapshot, so it is all undo/redo.

use super::*;
use crate::theme::{ink, muted};

/// The largest grid a pattern builds, so a stray formula can't flood the drawing.
const MAX_CELLS: usize = 2000;

/// Diameter of the round pattern marker button, in screen points.
const MARKER: f32 = 16.0;

impl App {
    /// Open the pattern dialog for shape `source`: pre-filled from its pattern if it has one, else
    /// with sensible starting values based on the shape's size (distance = size, so copies sit edge
    /// to edge to begin with).
    pub(super) fn open_pattern_dialog(&mut self, source: u64) {
        let existing = self.patterns.iter().find(|p| p.source == source).cloned();
        let dialog = match existing {
            Some(p) => PatternDialog { source, group: Some(p.group), cols: p.cols, rows: p.rows, dist_x: p.dist_x, dist_y: p.dist_y, error: None },
            None => {
                let (w, h) = self.shapes.iter().find(|s| s.id == source).and_then(|s| s.bounds()).map(|(lo, hi)| (hi[0] - lo[0], hi[1] - lo[1])).unwrap_or((20.0, 20.0));
                PatternDialog { source, group: None, cols: "3".into(), rows: "2".into(), dist_x: fmt_mm(w.max(1.0)), dist_y: fmt_mm(h.max(1.0)), error: None }
            }
        };
        self.pattern_dialog = Some(dialog);
    }

    /// The pattern dialog window: counts and centre distances (each a number, variable or formula),
    /// a live copy count, and OK/Cancel. OK creates or updates the pattern and regenerates it.
    pub(super) fn pattern_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.pattern_dialog.take() else { return };
        if !self.shapes.iter().any(|s| s.id == d.source) {
            return; // the source is gone
        }
        let mut ok = false;
        let mut cancel = false;
        let title = if d.group.is_some() { t!("Muster ändern") } else { t!("Rechteckiges Muster") };
        dialogs::window("pattern_dialog", ctx).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(300.0);
            if dialogs::header(ui, &title) {
                cancel = true;
            }
            ui.label(egui::RichText::new(t!("Anzahl und Abstand (Mitte zu Mitte) – je als Zahl, Variable oder Formel.")).small().color(muted()));
            ui.add_space(6.0);
            let field = |ui: &mut egui::Ui, label: String, text: &mut String, id: &str| {
                ui.horizontal(|ui| {
                    ui.add_sized(Vec2::new(90.0, 0.0), egui::Label::new(egui::RichText::new(label).color(ink())));
                    ui.add(egui::TextEdit::singleline(text).id_salt(id).desired_width(150.0).font(egui::TextStyle::Monospace));
                });
            };
            dialog_group(ui, &t!("Achse 1 (X)"), |ui| {
                field(ui, t!("Anzahl"), &mut d.cols, "pat_cols");
                field(ui, t!("Abstand"), &mut d.dist_x, "pat_dist_x");
            });
            dialog_group(ui, &t!("Achse 2 (Y)"), |ui| {
                field(ui, t!("Anzahl"), &mut d.rows, "pat_rows");
                field(ui, t!("Abstand"), &mut d.dist_y, "pat_dist_y");
            });
            // the live result, or why it can't be worked out
            let probe = Pattern { source: d.source, group: 0, cols: d.cols.clone(), rows: d.rows.clone(), dist_x: d.dist_x.clone(), dist_y: d.dist_y.clone() };
            match probe.resolved(&self.vars_resolved) {
                Some((c, r, _, _)) if c.saturating_mul(r) <= MAX_CELLS => {
                    d.error = None;
                    ui.label(egui::RichText::new(t!("%{n} Kopien (%{c} × %{r})", n = (c * r).to_string(), c = c.to_string(), r = r.to_string())).small().color(muted()));
                }
                Some(_) => {
                    d.error = Some(t!("Höchstens %{n} Kopien", n = MAX_CELLS.to_string()));
                    ui.label(egui::RichText::new(d.error.clone().unwrap()).small().color(crate::theme::error()));
                }
                None => {
                    d.error = Some(t!("Eine Eingabe lässt sich nicht berechnen").into());
                    ui.label(egui::RichText::new(d.error.clone().unwrap()).small().color(crate::theme::error()));
                }
            }
            dialog_buttons(ui, |ui| {
                let primary = if d.group.is_some() { t!("Übernehmen") } else { t!("Erstellen") };
                let button = egui::Button::new(egui::RichText::new(&primary).strong().color(INK)).fill(ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0);
                if ui.add_enabled(d.error.is_none(), button).clicked() {
                    ok = true;
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                    cancel = true;
                }
            });
        });
        if ok {
            self.apply_pattern_dialog(&d);
        } else if !cancel {
            self.pattern_dialog = Some(d);
        }
    }

    /// Create or update the pattern from the dialog, then rebuild its children. One undo step.
    pub(super) fn apply_pattern_dialog(&mut self, d: &PatternDialog) {
        let before = self.snapshot();
        let group = d.group.unwrap_or_else(|| self.new_id());
        let pattern = Pattern { source: d.source, group, cols: d.cols.clone(), rows: d.rows.clone(), dist_x: d.dist_x.clone(), dist_y: d.dist_y.clone() };
        // put the source into the pattern group (new patterns), and replace any existing record
        if let Some(src) = self.shapes.iter_mut().find(|s| s.id == d.source) {
            if !src.groups.contains(&group) {
                src.groups.push(group);
            }
        }
        match self.patterns.iter_mut().find(|p| p.source == d.source) {
            Some(p) => *p = pattern,
            None => self.patterns.push(pattern),
        }
        self.regenerate_pattern(group);
        self.edit.selection = vec![d.source];
        self.push_undo_from(before);
        self.recompute_all();
        self.status = t!("Muster angelegt").into();
    }

    /// Which pattern a shape is a child of (not the source), if any.
    pub(super) fn pattern_of_child(&self, id: u64) -> Option<&Pattern> {
        let item = self.shapes.iter().find(|s| s.id == id)?;
        self.patterns.iter().find(|p| p.source != id && item.groups.contains(&p.group))
    }

    /// If `id` is a pattern child, the id of its source; otherwise `None`.
    pub(super) fn pattern_source_of(&self, id: u64) -> Option<u64> {
        self.pattern_of_child(id).map(|p| p.source)
    }

    /// The ids of every pattern child (a group member that is not the source). Pattern children are
    /// passive ghosts: frozen so only the source is picked, selected and shows edit handles.
    pub(super) fn pattern_child_ids(&self) -> std::collections::HashSet<u64> {
        self.patterns
            .iter()
            .flat_map(|p| self.shapes.iter().filter(move |s| s.id != p.source && s.groups.contains(&p.group)).map(|s| s.id))
            .collect()
    }

    /// Mark every pattern child frozen now (used where there is no layer sync to carry it, e.g. a
    /// test). In the running app the freezing happens inside [`crate::layers::Layers::sync_with`].
    pub(super) fn freeze_pattern_children(&mut self) {
        let children = self.pattern_child_ids();
        for item in self.shapes.iter_mut().filter(|s| children.contains(&s.id)) {
            item.frozen = true;
        }
    }

    /// The source of the pattern child nearest the point `p` on screen (within the view tolerance),
    /// so a click on a ghost child can select the pattern through its source.
    pub(super) fn pattern_child_source_at(&self, p: Pt, view: shape_edit::View) -> Option<u64> {
        // a frozen child is skipped by the editor's hit test, so test the children directly here
        let tol = view.tolerance;
        self.shapes
            .iter()
            .filter(|s| !s.hidden && self.pattern_of_child(s.id).is_some())
            .find(|s| s.hit(p, tol))
            .and_then(|s| self.pattern_source_of(s.id))
    }

    /// Draw a small marker on every pattern source: a 2×2 grid of dots just outside its top-left
    /// corner, in a round button that lights up when the pointer is over it. A double-click on the
    /// marker (or on the source) opens the dialog. Remembers each marker's rect for the hit test.
    pub(super) fn draw_pattern_markers(&mut self, painter: &egui::Painter, hover: Option<egui::Pos2>) {
        use crate::theme::{ACCENT, CONSTRAINT};
        let mut rects = Vec::new();
        for p in &self.patterns {
            let Some(item) = self.shapes.iter().find(|s| s.id == p.source) else { continue };
            if item.hidden {
                continue;
            }
            let Some((lo, hi)) = item.bounds() else { continue };
            // centre of the marker, just outside the source's top-left corner (Y points down)
            let centre = self.to_screen([lo[0], hi[1]]) + egui::vec2(-9.0, -9.0);
            let rect = egui::Rect::from_center_size(centre, egui::vec2(MARKER, MARKER));
            let hot = hover.is_some_and(|h| rect.contains(h));
            // a round button behind the dots; brighter and ringed while hovered
            painter.circle_filled(centre, MARKER / 2.0, crate::theme::surface().gamma_multiply(if hot { 1.0 } else { 0.85 }));
            painter.circle_stroke(centre, MARKER / 2.0, egui::Stroke::new(1.0_f32, if hot { ACCENT } else { CONSTRAINT.gamma_multiply(0.6) }));
            let dot = if hot { ACCENT } else { CONSTRAINT };
            for (dx, dy) in [(-2.0, -2.0), (2.0, -2.0), (-2.0, 2.0), (2.0, 2.0)] {
                painter.circle_filled(centre + egui::vec2(dx, dy), 1.4, dot);
            }
            rects.push((p.source, rect));
        }
        self.pattern_markers = rects;
    }

    /// A press or double-click on a pattern marker: a double-click opens its dialog. Returns whether
    /// the marker took the event, so the canvas does not also select or move underneath it. Also
    /// shows the hand cursor while the pointer is over a marker.
    pub(super) fn pattern_marker_interact(&mut self, resp: &egui::Response) -> bool {
        let Some(p) = resp.interact_pointer_pos().or_else(|| resp.hover_pos()) else { return false };
        let Some((source, _)) = self.pattern_markers.iter().find(|(_, r)| r.contains(p)).copied() else { return false };
        resp.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
        if resp.double_clicked() {
            self.open_pattern_dialog(source);
            return true;
        }
        // a single press on the marker is swallowed too, so it never starts a drag of the shape
        resp.clicked() || resp.drag_started_by(egui::PointerButton::Primary)
    }

    /// Rebuild the children of the pattern with group `group` so each is the current source shape,
    /// translated to its cell. Children behave like references to the source: on every rebuild they
    /// take the source's shape afresh (its type, size and nodes), so changing the source changes
    /// every child. Child ids are kept across a rebuild as long as the cell still exists, so their
    /// toolpath bindings and selection survive a source edit; cells added by a larger count get new
    /// ids, cells removed by a smaller count drop out. Assumes the caller wraps this in an undo
    /// step. Does not recompute toolpaths.
    pub(super) fn regenerate_pattern(&mut self, group: u64) {
        let Some(pattern) = self.patterns.iter().find(|p| p.group == group).cloned() else { return };
        let Some((cols, rows, dist_x, dist_y)) = pattern.resolved(&self.vars_resolved) else { return };
        if cols.saturating_mul(rows) > MAX_CELLS {
            return; // a variable change blew the grid up past the cap: leave the children as they were
        }
        let Some(src) = self.shapes.iter().find(|s| s.id == pattern.source).cloned() else { return };
        // the ids of the current children, in reading order, to reuse cell by cell
        let mut free: std::collections::VecDeque<u64> = self.shapes.iter().filter(|s| s.groups.contains(&group) && s.id != pattern.source).map(|s| s.id).collect();
        // drop the current children; we add them back (reusing ids) so positions stay in order
        self.shapes.retain(|s| !(s.groups.contains(&group) && s.id != pattern.source));
        for r in 0..rows {
            for c in 0..cols {
                if c == 0 && r == 0 {
                    continue; // cell (0, 0) is the source itself
                }
                // rows grow downward (−Y), columns to the right (+X), centre to centre
                let shift = [c as f64 * dist_x, -(r as f64) * dist_y];
                let id = free.pop_front().unwrap_or_else(|| self.new_id());
                let mut child = src.translated(shift[0], shift[1]);
                child.id = id;
                child.source = None; // a child is not an imported shape
                child.bound.clear(); // its size is the source's, not its own binding
                child.dim_offsets.clear();
                if !child.groups.contains(&group) {
                    child.groups.push(group);
                }
                self.shapes.push(child);
            }
        }
        // any ids left over (the count shrank) belonged to cells that no longer exist; their
        // toolpaths fall away with them on the next recompute
        let _ = free;
    }

    /// A copy of `tp` whose `shape_ids` also list the pattern children of each source it cuts, so
    /// one toolpath covers the whole grid when its moves are worked out.
    pub(super) fn with_pattern_children(&self, mut tp: Toolpath) -> Toolpath {
        tp.shape_ids = Self::toolpath_ids_with_children(&tp.shape_ids, &self.patterns, &self.shapes);
        tp
    }

    /// The shapes without any pattern children, for saving: children are references to their
    /// source, so they are not stored but rebuilt from the source and the pattern on load.
    pub(super) fn shapes_without_pattern_children(&self) -> Vec<Item> {
        self.shapes.iter().filter(|s| self.pattern_of_child(s.id).is_none()).cloned().collect()
    }

    /// `ids` with every pattern source's children added (no duplicates). A free function so it can
    /// run on the background compute thread, which has only the patterns and shapes, not the App.
    pub(super) fn toolpath_ids_with_children(ids: &[u64], patterns: &[Pattern], shapes: &[Item]) -> Vec<u64> {
        let mut out = ids.to_vec();
        for &id in ids {
            if let Some(p) = patterns.iter().find(|p| p.source == id) {
                for s in shapes.iter().filter(|s| s.id != id && s.groups.contains(&p.group)) {
                    if !out.contains(&s.id) {
                        out.push(s.id);
                    }
                }
            }
        }
        out
    }

    /// When the source of a pattern has moved or changed, rebuild every pattern whose source still
    /// exists; patterns whose source is gone are dropped (with their children). Call after a shape
    /// edit. Returns whether anything changed.
    pub(super) fn sync_patterns(&mut self) -> bool {
        let groups: Vec<u64> = self.patterns.iter().map(|p| p.group).collect();
        // drop patterns whose source vanished, and their children
        let gone: Vec<u64> = self.patterns.iter().filter(|p| !self.shapes.iter().any(|s| s.id == p.source)).map(|p| p.group).collect();
        if !gone.is_empty() {
            self.shapes.retain(|s| !s.groups.iter().any(|g| gone.contains(g)));
            self.patterns.retain(|p| !gone.contains(&p.group));
        }
        let before = self.shapes.clone();
        for g in groups {
            if self.patterns.iter().any(|p| p.group == g) {
                self.regenerate_pattern(g);
            }
        }
        self.shapes != before || !gone.is_empty()
    }

    /// Free the pattern with group `group`: the source and all its children stop being one group
    /// and the pattern record goes away. Each former child gets the source's toolpaths copied onto
    /// it, so every piece now stands on its own and can take its own milling operations. Assumes
    /// the caller wraps this in an undo step.
    pub(super) fn unlink_pattern(&mut self, group: u64) {
        let Some(pattern) = self.patterns.iter().find(|p| p.group == group).cloned() else { return };
        let children: Vec<u64> = self.shapes.iter().filter(|s| s.id != pattern.source && s.groups.contains(&group)).map(|s| s.id).collect();
        // the source's toolpaths, now copied onto every former child
        let source_tps: Vec<Toolpath> = self.toolpaths.iter().filter(|tp| tp.shape_ids.contains(&pattern.source)).cloned().collect();
        for child in &children {
            for tp in &source_tps {
                let mut copy = tp.clone();
                copy.shape_ids = vec![*child];
                copy.moves.clear();
                self.toolpaths.push(copy);
            }
        }
        // drop the shared group tag from source and children (keeping any other groups they are in)
        for item in self.shapes.iter_mut().filter(|s| s.groups.contains(&group)) {
            item.groups.retain(|g| *g != group);
        }
        self.patterns.retain(|p| p.group != group);
    }
}

/// A millimetre value as short text for a field (no trailing zeros).
fn fmt_mm(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}
