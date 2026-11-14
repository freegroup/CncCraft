//! Decorations of the 3D view: everything drawn over the preview that isn't the preview itself
//! (for now the view cube). Like the worksheet's decorations (see `sheet`), each one is its own
//! component implementing [`ViewDecoration`]; a [`ViewDecorations`] list shows them in order,
//! lets each switch be turned off, and can gain new ones while the program runs.

mod cube;
mod tool;
mod zero;

pub use cube::ViewCube;
pub use tool::CutterOverlay;
pub use zero::{arrow_length, zero_point, ZeroCross};

use crate::sim::Camera;
use crate::tools::Tool;
use egui::Rect;

/// What a 3D-view decoration may know and change: where the view is on screen and the camera
/// (turn, tilt, zoom, perspective).
pub struct View3dContext<'a> {
    /// The 3D view on screen.
    pub area: Rect,
    pub cam: &'a mut Camera,
    /// How job points land on the screen (with the camera of this frame), where the machine's zero
    /// is (a job point) and how long its axis arrows are (mm).
    pub proj: &'a crate::sim::Projection,
    pub zero: [f32; 3],
    pub arrow_length: f32,
    /// Current cutter position and tool during simulation (set while animation runs).
    pub cutter: Option<([f64; 3], &'a Tool)>,
    /// Material thickness in mm, for sizing the shaft.
    pub thickness: f32,
    /// The 3D renderer, so decorations can inject GPU meshes.
    pub renderer: &'a crate::view3d::Renderer,
}

/// One decoration of the 3D view. It draws itself and reacts to the pointer; it may answer with
/// a message for the status line.
pub trait ViewDecoration {
    /// Its name, as shown where it can be switched on and off.
    fn name(&self) -> &'static str;
    /// Show it (each frame) and do what is clicked or dragged.
    fn show(&mut self, _ui: &mut egui::Ui, _view: &mut View3dContext) -> Option<String> {
        None
    }
}

/// The decorations of the 3D view, in painting order, each switchable.
pub struct ViewDecorations {
    items: Vec<(Box<dyn ViewDecoration>, bool)>,
}

impl Default for ViewDecorations {
    /// The standard 3D view: the zero cross, then the view cube on top of it.
    fn default() -> Self {
        let mut d = ViewDecorations { items: Vec::new() };
        d.add(Box::new(ZeroCross));
        d.add(Box::new(CutterOverlay::default()));
        d.add(Box::new(ViewCube));
        d
    }
}

impl ViewDecorations {
    /// Add a decoration (shown); it comes after those already there.
    #[allow(dead_code)]
    pub fn add(&mut self, decoration: Box<dyn ViewDecoration>) {
        self.items.push((decoration, true));
    }

    /// Show the switched-on decorations; returns the messages they gave, in order.
    pub fn show(&mut self, ui: &mut egui::Ui, view: &mut View3dContext) -> Vec<String> {
        self.items.iter_mut().filter(|(_, shown)| *shown).filter_map(|(d, _)| d.show(ui, view)).collect()
    }

    /// Each decoration's name with its on/off switch (for a menu).
    pub fn switches(&mut self) -> impl Iterator<Item = (&'static str, &mut bool)> {
        self.items.iter_mut().map(|(d, shown)| (d.name(), shown))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct Probe(Rc<RefCell<Vec<&'static str>>>);

    impl ViewDecoration for Probe {
        fn name(&self) -> &'static str {
            "Probe"
        }
        fn show(&mut self, _ui: &mut egui::Ui, _view: &mut View3dContext) -> Option<String> {
            self.0.borrow_mut().push("shown");
            Some("hallo".into())
        }
    }

    #[test]
    fn a_decoration_can_be_added_and_switched_off() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut d = ViewDecorations { items: Vec::new() };
        d.add(Box::new(Probe(seen.clone())));
        let names: Vec<&str> = d.switches().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["Probe"]);
        assert_eq!(ViewDecorations::default().switches().map(|(n, _)| n).collect::<Vec<_>>(), vec!["Nullpunkt-Achsenkreuz", "Fräser", "Ansichtswürfel"]);
        for (_, shown) in d.switches() {
            *shown = false;
        }
        // switched off: nothing is shown
        let ctx = egui::Context::default();
        let mut cam = Camera::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let job = crate::toolpath::Job::default();
                let proj = crate::sim::Projection::new(ui.max_rect(), &job, &Camera::default());
                let renderer = crate::view3d::Renderer::default();
                let mut view = View3dContext { area: ui.max_rect(), cam: &mut cam, proj: &proj, zero: zero_point(&job), arrow_length: arrow_length(&job), cutter: None, thickness: job.thickness as f32, renderer: &renderer };
                assert!(d.show(ui, &mut view).is_empty());
            });
        });
        assert!(seen.borrow().is_empty());
    }
}
