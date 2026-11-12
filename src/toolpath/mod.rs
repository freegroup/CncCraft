//! Toolpath tools. Each cutting strategy (profile, pocket, drill, V-carve) lives in its own
//! module and implements [`ToolpathKind`], so the geometry for one tool is isolated and a new
//! tool is added by writing one file and registering it in [`make_kind`].
//!
//! Z is 0 at the material surface and negative into the material. All shared helpers the tools
//! use to emit cutter moves (retract/plunge, ring cutting, tabs, raster fill) live here.

mod drill;
mod pocket;
mod profile;
mod vcarve;

use crate::geom::{bounds_of, dist, perimeter, signed_area, Contour, Item, Pt, Ring};
use crate::tools::{Tool, ToolKind, ToolLibrary};
use serde::{Deserialize, Serialize};

pub use drill::DrillTool;
pub use pocket::PocketTool;
pub use profile::ProfileTool;
pub use vcarve::VCarveTool;

// Values from MECHANICS.md
pub const SAFE_Z: f64 = 5.0; // estimated
pub const HOME_Z: f64 = 10.0; // estimated
pub const TAB_LENGTH: f64 = 6.0; // estimated
pub const TAB_THICKNESS: f64 = 3.0; // estimated
pub const RETRACT_GAP: f64 = 1.0; // estimated
pub const RAPID_RATE: f64 = 5000.0; // estimated, mm/min
/// Cut-out pass depth from ToolpathDefaults/CutOutMachine.default: 0.075 in.
pub const CUT_OUT_PASS_DEPTH_IN: f64 = 0.075;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ZZero {
    MaterialSurface,
    MachineBed,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Datum {
    LowerLeft,
    Center,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub width: f64,
    pub height: f64,
    pub thickness: f64,
    pub z_zero: ZZero,
    pub datum: Datum,
    pub material: String,
    pub safe_z: f64,
    pub home_z: f64,
    pub inches: bool,
}

impl Default for Job {
    fn default() -> Self {
        Job {
            width: 300.0,
            height: 200.0,
            thickness: 18.0,
            z_zero: ZZero::MaterialSurface,
            datum: Datum::LowerLeft,
            material: "Softwood".into(),
            safe_z: SAFE_Z,
            home_z: HOME_Z,
            inches: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Outside,
    Inside,
    On,
}

/// The per-tool parameters, stored with the toolpath and shown in the editor. Each variant maps
/// to the tool class of the same name.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Strategy {
    Profile { side: Side, climb: bool, tabs: bool, tab_count: u32 },
    Pocket { raster: bool, climb: bool, #[serde(default)] raster_angle: f64 },
    Drill,
    VCarve { max_depth: f64 },
}

impl Strategy {
    pub fn label(&self) -> &'static str {
        self.kind().label()
    }

    /// The tool class that computes this strategy.
    pub fn kind(&self) -> Box<dyn ToolpathKind> {
        match *self {
            Strategy::Profile { side, climb, tabs, tab_count } => Box::new(ProfileTool { side, climb, tabs, tab_count }),
            Strategy::Pocket { raster, climb, raster_angle } => Box::new(PocketTool { raster, climb, raster_angle }),
            Strategy::Drill => Box::new(DrillTool),
            Strategy::VCarve { max_depth } => Box::new(VCarveTool { max_depth }),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Move {
    pub to: [f64; 3],
    pub rapid: bool,
}

/// What a tool is told when it computes its cutter moves.
pub struct CutContext<'a> {
    pub tool: &'a Tool,
    pub start_depth: f64,
    pub cut_depth: f64,
    /// All contours of the selected shapes.
    pub contours: &'a [Contour],
}

impl CutContext<'_> {
    pub fn radius(&self) -> f64 {
        self.tool.diameter / 2.0
    }
    pub fn closed(&self) -> Vec<Contour> {
        self.contours.iter().filter(|c| c.closed).cloned().collect()
    }
    pub fn depths(&self) -> Vec<f64> {
        pass_depths(self.start_depth, self.cut_depth, self.tool.stepdown)
    }
}

/// One cutting strategy. Each tool class implements this; the big match is gone.
pub trait ToolpathKind {
    /// Short name shown in the UI.
    fn label(&self) -> &'static str;
    /// Emit the cutter moves into `out`. Returns a warning when the result is empty/invalid.
    fn cut(&self, out: &mut Out, cx: &CutContext) -> Option<String>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Toolpath {
    pub name: String,
    pub strategy: Strategy,
    pub tool: Tool,
    pub start_depth: f64,
    pub cut_depth: f64,
    /// The depths as typed when they are a formula or use variables (`platte_d/2`): the number
    /// above is worked out from it whenever the variables change. Old files have none.
    #[serde(default)]
    pub start_expr: Option<String>,
    #[serde(default)]
    pub cut_expr: Option<String>,
    pub shape_ids: Vec<u64>,
    pub visible: bool,
    #[serde(skip)]
    pub moves: Vec<Move>,
    #[serde(skip)]
    pub warning: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub cut_len: f64,
    pub rapid_len: f64,
    pub minutes: f64,
}

impl Toolpath {
    /// Work the depths out again from their formulas (a formula that can't be worked out leaves the
    /// number as it was). Returns whether a depth changed.
    pub fn resolve_depths(&mut self, lookup: &dyn shape_edit::calc::Lookup) -> bool {
        let mut changed = false;
        for (expr, value) in [(&self.start_expr, &mut self.start_depth), (&self.cut_expr, &mut self.cut_depth)] {
            if let Some(e) = expr {
                if let Ok(v) = shape_edit::calc::eval(e, lookup) {
                    let v = v.max(0.0);
                    if (v - *value).abs() > 1e-9 {
                        *value = v;
                        changed = true;
                    }
                }
            }
        }
        changed
    }

    /// A variable was renamed: the formulas use the new name. Returns whether one changed.
    pub fn rename_variable(&mut self, from: &str, to: &str) -> bool {
        let mut changed = false;
        for expr in [&mut self.start_expr, &mut self.cut_expr].into_iter().flatten() {
            if shape_edit::calc::names_in(expr).iter().any(|n| n == from) {
                *expr = shape_edit::calc::rename_in(expr, from, to);
                changed = true;
            }
        }
        changed
    }

    /// A variable was deleted: depths that used it become plain numbers (they keep their value).
    pub fn release_variable(&mut self, name: &str) {
        for expr in [&mut self.start_expr, &mut self.cut_expr] {
            if expr.as_ref().is_some_and(|e| shape_edit::calc::names_in(e).iter().any(|n| n == name)) {
                *expr = None;
            }
        }
    }

    pub fn compute(&mut self, job: &Job, shapes: &[Item]) {
        self.warning = None;
        let contours: Vec<Contour> = shapes.iter().filter(|s| !s.hidden && self.shape_ids.contains(&s.id)).flat_map(|s| s.shape.contours()).collect();
        let mut out = Out { moves: vec![], job };
        out.rapid([0.0, 0.0, job.home_z]);
        if contours.is_empty() {
            self.warning = Some("Keine Vektoren gewählt".into());
        } else {
            let cx = CutContext { tool: &self.tool, start_depth: self.start_depth, cut_depth: self.cut_depth, contours: &contours };
            if let Some(w) = self.strategy.kind().cut(&mut out, &cx) {
                self.warning = Some(w);
            }
        }
        out.retract();
        let c = out.cur();
        out.rapid([c[0], c[1], job.home_z]);
        out.rapid([0.0, 0.0, job.home_z]);
        self.moves = out.moves;
    }

    pub fn stats(&self) -> Stats {
        let mut s = Stats::default();
        let mut prev = [0.0, 0.0, self.moves.first().map(|m| m.to[2]).unwrap_or(0.0)];
        for m in &self.moves {
            let d = ((m.to[0] - prev[0]).powi(2) + (m.to[1] - prev[1]).powi(2) + (m.to[2] - prev[2]).powi(2)).sqrt();
            if m.rapid {
                s.rapid_len += d;
                s.minutes += d / RAPID_RATE;
            } else {
                s.cut_len += d;
                let plunging = (m.to[2] - prev[2]).abs() > 1e-6 && (m.to[0] - prev[0]).abs() < 1e-6 && (m.to[1] - prev[1]).abs() < 1e-6;
                let rate = if plunging { self.tool.plunge } else { self.tool.feed };
                s.minutes += d / rate.max(1.0);
            }
            prev = m.to;
        }
        s
    }
}

/// Collects the cutter moves a tool emits, and provides the safe retract/plunge/ring helpers
/// every tool shares.
pub struct Out<'a> {
    moves: Vec<Move>,
    job: &'a Job,
}

impl Out<'_> {
    pub fn cur(&self) -> [f64; 3] {
        self.moves.last().map(|m| m.to).unwrap_or([0.0, 0.0, self.job.home_z])
    }
    pub fn rapid(&mut self, p: [f64; 3]) {
        self.moves.push(Move { to: p, rapid: true });
    }
    pub fn feed(&mut self, p: [f64; 3]) {
        self.moves.push(Move { to: p, rapid: false });
    }
    /// Retract, travel above `p`, then plunge to it.
    pub fn go_to(&mut self, p: [f64; 3]) {
        let c = self.cur();
        let safe = self.job.safe_z;
        if (c[0] - p[0]).abs() > 1e-6 || (c[1] - p[1]).abs() > 1e-6 {
            if c[2] < safe {
                self.rapid([c[0], c[1], safe]);
            }
            self.rapid([p[0], p[1], safe]);
            // rapid down to just above where material may already be cut
            self.rapid([p[0], p[1], 0.5_f64.min(safe)]);
        }
        self.feed(p);
    }
    pub fn retract(&mut self) {
        let c = self.cur();
        if c[2] < self.job.safe_z {
            self.rapid([c[0], c[1], self.job.safe_z]);
        }
    }

    /// Cut one closed ring at depth `z`, optionally lifting to `tab_z` over the given spans.
    pub fn cut_ring(&mut self, pts: &[Pt], z: f64, tab_z: Option<(f64, &[(f64, f64)])>) {
        if pts.len() < 2 {
            return;
        }
        let start = pts[0];
        let z_at = |s: f64| -> f64 {
            if let Some((tz, tabs)) = tab_z {
                if tabs.iter().any(|(a, b)| s >= *a && s <= *b) {
                    return z.max(tz);
                }
            }
            z
        };
        self.go_to([start[0], start[1], z_at(0.0)]);
        let mut s = 0.0;
        let n = pts.len();
        for i in 0..n {
            let a = pts[i];
            let b = pts[(i + 1) % n];
            let l = dist(a, b);
            let mut cuts: Vec<f64> = vec![];
            if let Some((_, tabs)) = tab_z {
                for (ta, tb) in tabs {
                    for t in [*ta, *tb] {
                        if t > s && t < s + l {
                            cuts.push(t);
                        }
                    }
                }
            }
            cuts.sort_by(|x, y| x.partial_cmp(y).unwrap());
            for t in cuts {
                let f = (t - s) / l;
                let p = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
                self.feed([p[0], p[1], z_at(t - 1e-6)]);
                self.feed([p[0], p[1], z_at(t + 1e-6)]);
            }
            s += l;
            self.feed([b[0], b[1], z_at(s - 1e-6)]);
        }
    }

    /// Cut an open polyline at depth `z`.
    pub fn cut_open(&mut self, pts: &[Pt], z: f64) {
        if pts.is_empty() {
            return;
        }
        self.go_to([pts[0][0], pts[0][1], z]);
        for p in &pts[1..] {
            self.feed([p[0], p[1], z]);
        }
    }
}

/// The depths of each pass: stepdown increments, last one exactly at `depth`.
pub fn pass_depths(start: f64, depth: f64, stepdown: f64) -> Vec<f64> {
    let total = (depth - start).max(0.0);
    if total <= 1e-9 {
        return vec![];
    }
    let n = (total / stepdown.max(0.01) - 1e-9).ceil().max(1.0) as usize;
    (1..=n).map(|i| -(start + total * i as f64 / n as f64)).collect()
}

fn orient(mut pts: Vec<Pt>, ccw: bool) -> Vec<Pt> {
    if (signed_area(&pts) > 0.0) != ccw {
        pts.reverse();
    }
    pts
}

/// Ring direction for a cut: with a clockwise spindle, climb milling goes clockwise around
/// outer boundaries and anticlockwise around holes.
pub fn ring_dir(r: &Ring, climb: bool) -> Vec<Pt> {
    let ccw = r.hole == climb;
    orient(r.pts.clone(), ccw)
}

/// Even spans of `width` along the perimeter where holding tabs keep the part attached.
pub fn tab_spans(pts: &[Pt], count: u32, width: f64) -> Vec<(f64, f64)> {
    let per = perimeter(pts, true);
    if count == 0 || per < width * count as f64 * 3.0 {
        return vec![];
    }
    (0..count)
        .map(|i| {
            let c = per * (i as f64 + 0.5) / count as f64;
            (c - width / 2.0, c + width / 2.0)
        })
        .collect()
}

/// Boustrophedon (back-and-forth) horizontal fill segments clipped to the rings.
pub fn raster_segments(rings: &[Ring], step: f64) -> Vec<(Pt, Pt)> {
    let all: Vec<Contour> = rings.iter().map(|r| Contour { pts: r.pts.clone(), closed: true }).collect();
    let Some((lo, hi)) = bounds_of(&all) else { return vec![] };
    let mut segs = Vec::new();
    let mut y = lo[1] + step * 0.5;
    let mut flip = false;
    while y < hi[1] {
        let mut xs = Vec::new();
        for r in rings {
            let n = r.pts.len();
            for i in 0..n {
                let a = r.pts[i];
                let b = r.pts[(i + 1) % n];
                if (a[1] > y) != (b[1] > y) {
                    xs.push(a[0] + (y - a[1]) / (b[1] - a[1]) * (b[0] - a[0]));
                }
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut row: Vec<(Pt, Pt)> = xs.chunks(2).filter(|c| c.len() == 2).map(|c| ([c[0], y], [c[1], y])).collect();
        if flip {
            row.reverse();
            for s in row.iter_mut() {
                *s = (s.1, s.0);
            }
        }
        segs.extend(row);
        flip = !flip;
        y += step;
    }
    segs
}

/// Build a toolpath with the original's defaults for the given tool kind.
pub fn default_toolpath(kind: &str, lib: &ToolLibrary, job: &Job, ids: Vec<u64>) -> Toolpath {
    let (strategy, tool, depth) = match kind {
        "pocket" => (Strategy::Pocket { raster: false, climb: true, raster_angle: 0.0 }, lib.pick(&job.material, ToolKind::EndMill, 6.0, None), 3.0),
        "drill" => (Strategy::Drill, lib.pick(&job.material, ToolKind::Drill, 6.0, None), 10.0),
        "vcarve" => (Strategy::VCarve { max_depth: 0.0 }, lib.pick(&job.material, ToolKind::VBit, 12.0, Some(90.0)), 6.0),
        "cutout" => {
            let mut t = lib.pick(&job.material, ToolKind::EndMill, 6.0, None);
            // use the original cut-out default pass depth (0.075 in) as the starting value
            t.stepdown = CUT_OUT_PASS_DEPTH_IN * 25.4;
            (Strategy::Profile { side: Side::Outside, climb: true, tabs: true, tab_count: 4 }, t, job.thickness)
        }
        _ => (Strategy::Profile { side: Side::Outside, climb: true, tabs: false, tab_count: 4 }, lib.pick(&job.material, ToolKind::EndMill, 3.0, None), 3.0),
    };
    let name = match kind {
        "cutout" => "Ausschnitt".to_string(),
        _ => strategy.label().to_string(),
    };
    Toolpath { name, strategy, tool, start_depth: 0.0, cut_depth: depth, start_expr: None, cut_expr: None, shape_ids: ids, visible: true, moves: vec![], warning: None }
}

#[cfg(test)]
mod depth_formula_tests {
    use super::*;
    use shape_edit::calc::CalcError;

    fn lookup(n: &str) -> Result<f64, CalcError> {
        match n {
            "platte_d" => Ok(18.0),
            "rand" => Ok(2.0),
            _ => Err(CalcError::Unknown(n.to_string())),
        }
    }

    fn toolpath() -> Toolpath {
        let lib = ToolLibrary::load();
        default_toolpath("profile", &lib, &Job::default(), vec![])
    }

    #[test]
    fn a_depth_formula_follows_the_variables() {
        let mut tp = toolpath();
        tp.cut_expr = Some("platte_d/2".into());
        assert!(tp.resolve_depths(&lookup));
        assert_eq!(tp.cut_depth, 9.0);
        assert!(!tp.resolve_depths(&lookup), "nothing changed the second time");
        // a formula that can't be worked out leaves the number alone
        tp.cut_expr = Some("gibtsnicht".into());
        assert!(!tp.resolve_depths(&lookup));
        assert_eq!(tp.cut_depth, 9.0);
        // both depths
        tp.start_expr = Some("rand".into());
        tp.cut_expr = Some("platte_d - rand".into());
        tp.resolve_depths(&lookup);
        assert_eq!((tp.start_depth, tp.cut_depth), (2.0, 16.0));
    }

    #[test]
    fn renaming_and_deleting_a_variable_reach_the_depths() {
        let mut tp = toolpath();
        tp.cut_expr = Some("platte_d/2".into());
        assert!(tp.rename_variable("platte_d", "dicke"));
        assert_eq!(tp.cut_expr.as_deref(), Some("dicke/2"));
        assert!(!tp.rename_variable("nix", "x"));
        tp.release_variable("dicke");
        assert_eq!(tp.cut_expr, None, "the number stays, the formula goes");
    }

    #[test]
    fn old_files_without_formulas_still_load() {
        let mut v = serde_json::to_value(toolpath()).unwrap();
        v.as_object_mut().unwrap().remove("start_expr");
        v.as_object_mut().unwrap().remove("cut_expr");
        let back: Toolpath = serde_json::from_value(v).unwrap();
        assert!(back.start_expr.is_none() && back.cut_expr.is_none());
    }
}
