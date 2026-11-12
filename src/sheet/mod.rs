//! Worksheet decorations: everything drawn around and under the design that isn't the design
//! itself (the board, its grid, the rulers, the axis cross, the hints…). Each decoration is its
//! own component implementing [`Decoration`]; a [`Decorations`] list paints them in order, lets
//! the interactive ones react and report what the program should do, and can gain new ones, or
//! switch any of them off, while the program runs.

mod axes;
mod board;
mod grid;
mod hints;
mod rulers;

pub use axes::AxisCross;
pub use board::Board;
pub use grid::Grid;
pub use hints::Hints;
pub use rulers::Rulers;

/// What an interactive decoration asks the program to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetAction {
    /// Show help for the tool in use.
    ShowToolHelp,
}

use egui::{Painter, Pos2, Rect};

/// Whether a decoration lies under the design (board, grid) or over everything (rulers, axes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Under,
    Over,
}

/// What a decoration may know about the worksheet: where it is on screen and how millimetres
/// map to it. Y points up in millimetres and down on screen.
#[derive(Clone, Copy, Debug)]
pub struct SheetView<'a> {
    /// The drawing area on screen.
    pub canvas: Rect,
    /// Screen pixels per millimetre.
    pub zoom: f32,
    /// Screen position of 0/0 mm.
    pub origin: Pos2,
    /// The board's size in mm.
    pub width: f64,
    pub height: f64,
    /// The job's zero point in mm.
    pub datum: [f64; 2],
    /// The pointer in mm, when it is over the drawing.
    pub pointer: Option<[f64; 2]>,
    /// What the person should do next (the prompt line).
    pub prompt: &'a str,
}

impl SheetView<'_> {
    pub fn to_screen(&self, p: [f64; 2]) -> Pos2 {
        Pos2::new(self.origin.x + p[0] as f32 * self.zoom, self.origin.y - p[1] as f32 * self.zoom)
    }

    /// The board on screen.
    pub fn board(&self) -> Rect {
        Rect::from_two_pos(self.to_screen([0.0, 0.0]), self.to_screen([self.width, self.height]))
    }

    /// A round step in mm (1, 2, 5 × 10ⁿ) at least `min` mm long.
    pub fn nice_step(min: f64) -> f64 {
        [0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0].into_iter().find(|s| *s >= min).unwrap_or(2000.0)
    }
}

/// One worksheet decoration. It paints, reacts to the pointer, or both; both have defaults
/// that do nothing.
pub trait Decoration {
    /// Its name, as shown where it can be switched on and off.
    fn name(&self) -> &'static str;
    fn layer(&self) -> Layer;
    fn paint(&self, _view: &SheetView, _painter: &Painter) {}
    /// Show interactive parts (each frame) and say what the program should do.
    fn interact(&mut self, _view: &SheetView, _ctx: &egui::Context) -> Option<SheetAction> {
        None
    }
}

/// The decorations of a worksheet, in painting order, each switchable.
pub struct Decorations {
    items: Vec<(Box<dyn Decoration>, bool)>,
}

impl Default for Decorations {
    /// The standard worksheet: board and grid under the design; rulers, axes and the hints
    /// over it.
    fn default() -> Self {
        let mut d = Decorations::empty();
        d.add(Box::new(Board));
        d.add(Box::new(Grid));
        d.add(Box::new(Rulers));
        d.add(Box::new(AxisCross));
        d.add(Box::new(Hints::default()));
        d
    }
}

impl Decorations {
    /// No decorations at all.
    pub const fn empty() -> Self {
        Decorations { items: Vec::new() }
    }

    /// Let the shown decorations react; returns what they ask for, in order.
    pub fn interact(&mut self, view: &SheetView, ctx: &egui::Context) -> Vec<SheetAction> {
        self.items.iter_mut().filter(|(_, shown)| *shown).filter_map(|(d, _)| d.interact(view, ctx)).collect()
    }

    /// Add a decoration (shown); it paints after those already in its layer.
    pub fn add(&mut self, decoration: Box<dyn Decoration>) {
        self.items.push((decoration, true));
    }

    /// Paint the shown decorations of one layer, in order, kept inside the drawing area.
    pub fn paint(&self, layer: Layer, view: &SheetView, painter: &Painter) {
        let painter = painter.with_clip_rect(view.canvas);
        for (d, shown) in &self.items {
            if *shown && d.layer() == layer {
                d.paint(view, &painter);
            }
        }
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

    /// Records when it is painted.
    struct Probe(&'static str, Layer, Rc<RefCell<Vec<&'static str>>>);

    impl Decoration for Probe {
        fn name(&self) -> &'static str {
            self.0
        }
        fn layer(&self) -> Layer {
            self.1
        }
        fn paint(&self, _view: &SheetView, _painter: &Painter) {
            self.2.borrow_mut().push(self.0);
        }
    }

    fn view() -> SheetView<'static> {
        SheetView { canvas: Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0)), zoom: 2.0, origin: Pos2::new(100.0, 500.0), width: 300.0, height: 200.0, datum: [0.0, 0.0], pointer: None, prompt: "" }
    }

    #[test]
    fn decorations_that_only_paint_never_ask_for_anything() {
        let mut d = Decorations::empty();
        d.add(Box::new(Board));
        d.add(Box::new(Rulers));
        assert!(d.interact(&view(), &egui::Context::default()).is_empty());
    }

    #[test]
    fn decorations_paint_by_layer_in_order_and_can_be_switched_off() {
        let log = Rc::new(RefCell::new(vec![]));
        let mut d = Decorations::empty();
        d.add(Box::new(Probe("a", Layer::Under, log.clone())));
        d.add(Box::new(Probe("b", Layer::Over, log.clone())));
        d.add(Box::new(Probe("c", Layer::Under, log.clone())));
        let ctx = egui::Context::default();
        let painter = Painter::new(ctx, egui::LayerId::background(), Rect::EVERYTHING);
        d.paint(Layer::Under, &view(), &painter);
        assert_eq!(*log.borrow(), vec!["a", "c"]);
        for (name, shown) in d.switches() {
            if name == "a" {
                *shown = false;
            }
        }
        log.borrow_mut().clear();
        d.paint(Layer::Under, &view(), &painter);
        d.paint(Layer::Over, &view(), &painter);
        assert_eq!(*log.borrow(), vec!["c", "b"]);
        let shown: Vec<(&str, bool)> = d.switches().map(|(name, on)| (name, *on)).collect();
        assert_eq!(shown, vec![("a", false), ("b", true), ("c", true)]);
    }

    #[test]
    fn the_view_maps_millimetres_to_the_screen() {
        let v = view();
        assert_eq!(v.to_screen([0.0, 0.0]), Pos2::new(100.0, 500.0));
        assert_eq!(v.to_screen([10.0, 10.0]), Pos2::new(120.0, 480.0));
        assert_eq!(v.board(), Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(700.0, 500.0)));
        assert_eq!(SheetView::nice_step(3.0), 5.0);
    }
}
