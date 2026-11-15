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
/// How steeply the cutter ramps into the material along its path, in degrees (CncCraft's own
/// default for hobby machines; 0 plunges straight down).
pub const RAMP_ANGLE: f64 = 5.0;

fn default_ramp_angle() -> f64 {
    RAMP_ANGLE
}

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
    pub fn label(&self) -> String {
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
    /// Where the material still stands above the pass at depth index `i`: the pass before, or
    /// the start depth for the first.
    pub fn top_of(&self, depths: &[f64], i: usize) -> f64 {
        if i == 0 {
            -self.start_depth
        } else {
            depths[i - 1]
        }
    }
}

/// One cutting strategy. Each tool class implements this; the big match is gone.
pub trait ToolpathKind {
    /// Short name shown in the UI.
    fn label(&self) -> String;
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
    /// How steeply the cutter ramps into the material (degrees, 0 = straight down). Old files get
    /// the default.
    #[serde(default = "default_ramp_angle")]
    pub ramp_angle: f64,
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
        // A toolpath starts at its first cut and ends over its last one, lifted to safe_z. It never
        // visits home or 0/0: the way from home to the first toolpath, and from one toolpath to the
        // next, is the first move of the next one, at safe_z (the job's hooks put the machine at home
        // height at the start and at safe_z between toolpaths and tools).
        let mut out = Out { moves: vec![], job, ramp_tan: self.ramp_angle.clamp(0.0, 89.0).to_radians().tan(), top: 0.0, ring_end: None };
        if contours.is_empty() {
            self.warning = Some(t!("Keine Vektoren gewählt").into());
        } else {
            let cx = CutContext { tool: &self.tool, start_depth: self.start_depth, cut_depth: self.cut_depth, contours: &contours };
            if let Some(w) = self.strategy.kind().cut(&mut out, &cx) {
                self.warning = Some(w);
            }
        }
        out.retract();
        self.moves = out.moves;
    }

    /// How long the machine takes for move `i`, in seconds, by the same estimates as `stats`: the
    /// real time of the move at its feed rate (rapid moves at the estimated rapid rate).
    pub fn move_seconds(&self, i: usize) -> f64 {
        let (Some(m), Some(prev)) = (self.moves.get(i), i.checked_sub(1).and_then(|p| self.moves.get(p))) else { return 0.0 };
        let (a, b) = (prev.to, m.to);
        let d = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
        if m.rapid {
            return d / RAPID_RATE * 60.0;
        }
        let plunging = (b[2] - a[2]).abs() > 1e-6 && (b[0] - a[0]).abs() < 1e-6 && (b[1] - a[1]).abs() < 1e-6;
        let rate = if plunging { self.tool.plunge } else { self.tool.feed };
        d / rate.max(1.0) * 60.0
    }

    /// The same toolpath with other moves (for carving a part of it).
    pub fn with_moves(&self, moves: Vec<Move>) -> Toolpath {
        Toolpath { name: self.name.clone(), strategy: self.strategy.clone(), tool: self.tool.clone(), start_depth: self.start_depth, cut_depth: self.cut_depth, start_expr: None, cut_expr: None, shape_ids: vec![], visible: self.visible, ramp_angle: self.ramp_angle, moves, warning: None }
    }

    /// A clone without the computed moves (cheap to send to a background thread).
    pub fn without_moves(&self) -> Toolpath {
        Toolpath { name: self.name.clone(), strategy: self.strategy.clone(), tool: self.tool.clone(), start_depth: self.start_depth, cut_depth: self.cut_depth, start_expr: self.start_expr.clone(), cut_expr: self.cut_expr.clone(), shape_ids: self.shape_ids.clone(), visible: self.visible, ramp_angle: self.ramp_angle, moves: vec![], warning: None }
    }

    pub fn stats(&self) -> Stats {
        let mut s = Stats::default();
        let mut prev = self.moves.first().map(|m| m.to).unwrap_or([0.0; 3]);
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
    /// Tangent of the ramp angle (0: plunge straight down).
    ramp_tan: f64,
    /// Where the material still stands above the coming cuts (set by the tool for each pass).
    top: f64,
    /// Where the last ring cut ended: its first point, its length, the position along it and
    /// the point there, so the next pass on the same ring ramps on from there.
    ring_end: Option<(Pt, f64, f64, Pt)>,
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
    /// Retract to safe_z, travel in XY at safe_z, then come down just over the material and plunge
    /// to the target. The first move of a toolpath comes from wherever the machine is: at home
    /// height at the start of the job, at safe_z after another toolpath, so it only ever runs at
    /// safe_z or above.
    pub fn go_to(&mut self, p: [f64; 3]) {
        let safe = self.job.safe_z;
        let Some(c) = self.moves.last().map(|m| m.to) else {
            self.rapid([p[0], p[1], safe]);
            return self.come_down_and_plunge(p);
        };
        if (c[0] - p[0]).abs() > 1e-6 || (c[1] - p[1]).abs() > 1e-6 {
            if c[2] < safe {
                self.rapid([c[0], c[1], safe]);
            }
            self.rapid([p[0], p[1], c[2].max(safe)]);
        }
        self.come_down_and_plunge(p)
    }

    fn come_down_and_plunge(&mut self, p: [f64; 3]) {
        let safe = self.job.safe_z;
        if (self.cur()[2] - 0.5_f64.min(safe)).abs() > 1e-6 {
            self.rapid([p[0], p[1], 0.5_f64.min(safe)]);
        }
        self.feed(p);
    }
    /// Lift straight up to safe_z where the cutter is (nothing, if it has not moved yet).
    pub fn retract(&mut self) {
        let Some(c) = self.moves.last().map(|m| m.to) else { return };
        if c[2] < self.job.safe_z {
            self.rapid([c[0], c[1], self.job.safe_z]);
        }
    }

    /// The material above the coming cuts is cleared down to `z` (the pass before).
    pub fn set_top(&mut self, z: f64) {
        self.top = z;
    }

    /// Travel at safe height to `p` and come down to just over the material still standing there;
    /// returns the height reached.
    fn approach(&mut self, p: Pt) -> f64 {
        let safe = self.job.safe_z;
        match self.moves.last().map(|m| m.to) {
            None => self.rapid([p[0], p[1], safe]),
            Some(c) if (c[0] - p[0]).abs() > 1e-6 || (c[1] - p[1]).abs() > 1e-6 => {
                if c[2] < safe {
                    self.rapid([c[0], c[1], safe]);
                }
                self.rapid([p[0], p[1], c[2].max(safe)]);
            }
            _ => {}
        }
        let over = (self.top + 0.5).min(safe);
        if (self.cur()[2] - over).abs() > 1e-6 {
            self.rapid([p[0], p[1], over]);
        }
        over
    }

    /// Cut one closed ring at depth `z`, optionally lifting to `tab_z` over the given spans. With
    /// a ramp angle the cutter goes down along the ring instead of straight, then cuts one whole
    /// round at depth, which also cleans up the ramp.
    pub fn cut_ring(&mut self, pts: &[Pt], z: f64, tab_z: Option<(f64, &[(f64, f64)])>) {
        if pts.len() < 2 {
            return;
        }
        if self.ramp_tan > 1e-9 && perimeter(pts, true) > 1e-6 {
            return self.ramp_ring(pts, z, tab_z);
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

    fn ramp_ring(&mut self, pts: &[Pt], z: f64, tab_z: Option<(f64, &[(f64, f64)])>) {
        let n = pts.len();
        let mut cum = vec![0.0; n + 1];
        for i in 0..n {
            cum[i + 1] = cum[i] + dist(pts[i], pts[(i + 1) % n]);
        }
        let per = cum[n];
        let tab_level = |t: f64| -> f64 {
            let t = t.rem_euclid(per);
            match tab_z {
                Some((tz, tabs)) if tabs.iter().any(|(a, b)| t >= *a && t <= *b) => z.max(tz),
                _ => z,
            }
        };
        let breaks: Vec<f64> = tab_z.map(|(_, tabs)| tabs.iter().flat_map(|(a, b)| [a.rem_euclid(per), b.rem_euclid(per)]).collect()).unwrap_or_default();
        // go on from where the same ring's last pass ended, else come down over its first point
        let cur = self.cur();
        let (start, z_from) = match self.ring_end {
            Some((first, len, s, at)) if first == pts[0] && (len - per).abs() < 1e-9 && dist(at, [cur[0], cur[1]]) < 1e-6 && cur[2] > z + 1e-9 => (s, cur[2]),
            _ => (0.0, self.approach(pts[0])),
        };
        let emit = |out: &mut Self, p: Pt, s: f64, level: f64| {
            let (before, after) = (level.max(tab_level(s - 1e-6)), level.max(tab_level(s + 1e-6)));
            out.feed([p[0], p[1], before]);
            if (after - before).abs() > 1e-9 {
                out.feed([p[0], p[1], after]);
            }
        };
        let ramp_len = (z_from - z).max(0.0) / self.ramp_tan;
        let mut s = start;
        for (p, walked, at) in ring_walk(pts, &cum, s, ramp_len, &breaks) {
            emit(self, p, at, (z_from - walked * self.ramp_tan).max(z));
            s = at;
        }
        let s_end = s.rem_euclid(per);
        for (p, _, at) in ring_walk(pts, &cum, s_end, per, &breaks) {
            emit(self, p, at, z);
        }
        let end = self.cur();
        self.ring_end = Some((pts[0], per, s_end, [end[0], end[1]]));
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

/// Walk `len` along a closed ring from position `from` (arc length from its first point; `cum`
/// holds the arc length at each point and the total at the end), wrapping round as often as
/// needed. Yields every corner passed, every position in `breaks` and the end: the point, the
/// distance walked so far and the position along the ring.
fn ring_walk(pts: &[Pt], cum: &[f64], from: f64, len: f64, breaks: &[f64]) -> Vec<(Pt, f64, f64)> {
    let n = pts.len();
    let per = cum[n];
    let mut out = vec![];
    let mut s = from.rem_euclid(per);
    let mut i = (0..n).rev().find(|&k| cum[k] <= s).unwrap_or(0);
    let mut walked = 0.0;
    while walked < len - 1e-9 {
        let mut next = cum[i + 1];
        for &b in breaks {
            if b > s + 1e-9 && b < next {
                next = b;
            }
        }
        next = next.min(s + (len - walked));
        walked += next - s;
        s = next;
        let f = (s - cum[i]) / (cum[i + 1] - cum[i]).max(1e-12);
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        out.push(([a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f], walked, s));
        if s >= cum[i + 1] - 1e-9 {
            i += 1;
            if i == n {
                i = 0;
                s = 0.0;
            }
        }
    }
    out
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
        "cutout" => t!("Ausschnitt").to_string(),
        _ => strategy.label(),
    };
    Toolpath { name, strategy, tool, start_depth: 0.0, cut_depth: depth, start_expr: None, cut_expr: None, shape_ids: ids, visible: true, ramp_angle: RAMP_ANGLE, moves: vec![], warning: None }
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

#[cfg(test)]
mod ramp_tests {
    use super::*;
    use shape_edit::{Rect, Shape};

    fn cut(ramp_angle: f64, tabs: bool) -> (Toolpath, Job) {
        let lib = ToolLibrary::load();
        let job = Job::default();
        let mut tp = default_toolpath("profile", &lib, &job, vec![1]);
        tp.strategy = Strategy::Profile { side: Side::Outside, climb: true, tabs, tab_count: 2 };
        tp.cut_depth = 6.0;
        tp.tool.stepdown = 2.0;
        tp.ramp_angle = ramp_angle;
        tp.compute(&job, &[Item::new(1, Shape::Rect(Rect::new([100.0, 100.0], 60.0, 40.0)))]);
        (tp, job)
    }

    /// Feed moves that go down without moving sideways.
    fn straight_plunges_into_material(tp: &Toolpath) -> usize {
        tp.moves.windows(2).filter(|w| !w[1].rapid && w[1].to[2] < w[0].to[2] - 1e-6 && w[1].to[2] < -1e-6 && dist([w[0].to[0], w[0].to[1]], [w[1].to[0], w[1].to[1]]) < 1e-6).count()
    }

    #[test]
    fn with_a_ramp_the_cutter_never_plunges_straight_into_the_material() {
        let (straight, _) = cut(0.0, false);
        assert!(straight_plunges_into_material(&straight) >= 3, "without a ramp every pass plunges");
        let (ramped, _) = cut(5.0, false);
        assert_eq!(straight_plunges_into_material(&ramped), 0);
        // no move goes down more steeply than the ramp angle
        let tan = 5.0_f64.to_radians().tan();
        for w in ramped.moves.windows(2).filter(|w| !w[1].rapid && w[1].to[2] < w[0].to[2] - 1e-9) {
            let run = dist([w[0].to[0], w[0].to[1]], [w[1].to[0], w[1].to[1]]);
            assert!(w[0].to[2] - w[1].to[2] <= run * tan + 1e-6, "{w:?}");
        }
        // it still reaches the full depth
        let deepest = ramped.moves.iter().map(|m| m.to[2]).fold(0.0, f64::min);
        assert!((deepest + 6.0).abs() < 1e-9);
    }

    #[test]
    fn the_passes_follow_on_without_lifting_and_the_tabs_stay() {
        let (tp, job) = cut(5.0, true);
        // one lift to safe height at the end only: the passes on the same ring follow on
        let lifts = tp.moves.iter().filter(|m| m.rapid && (m.to[2] - job.safe_z).abs() < 1e-9).count();
        assert!(lifts <= 2, "{lifts} lifts");
        // the tabs keep their height: some moves at the last pass stay at the tab top
        let tab_top = -(6.0 - TAB_THICKNESS);
        assert!(tp.moves.iter().any(|m| (m.to[2] - tab_top).abs() < 1e-9));
    }

    #[test]
    fn old_files_get_the_default_ramp() {
        let (tp, _) = cut(5.0, false);
        let mut v = serde_json::to_value(&tp).unwrap();
        v.as_object_mut().unwrap().remove("ramp_angle");
        let back: Toolpath = serde_json::from_value(v).unwrap();
        assert_eq!(back.ramp_angle, RAMP_ANGLE);
    }
}
