//! Reading DXF drawings (ASCII, AutoCAD R12 and R14 or newer) into shapes. Plain logic without a
//! window, so it can be tested.
//!
//! Lines, arcs, polylines (with bulges), ellipses and splines become open or closed curves, circles
//! stay circles, blocks (INSERT) are placed, and loose lines and arcs that meet end to end are
//! joined into one curve (closed when the ends meet). Other things (text, hatches, dimensions…)
//! are left out and counted, so the person can be told.

use super::Import;
use shape_edit::{Circle, Path, Shape};
use std::collections::{BTreeMap, HashMap};
use std::f64::consts::{PI, TAU};

type Pt = [f64; 2];

/// Largest gap (mm) between two ends that still counts as "they meet".
const JOIN_TOL: f64 = 0.01;
/// Largest distance (mm) a drawn arc may stray from the true curve.
const ARC_TOL: f64 = 0.02;

enum Prim {
    Circle { c: Pt, r: f64 },
    Path { pts: Vec<Pt>, closed: bool },
}

/// One entity (or section marker) of the file: its type and its group-code/value pairs.
struct Rec {
    kind: String,
    codes: Vec<(i32, String)>,
}

impl Rec {
    fn s(&self, code: i32) -> Option<&str> {
        self.codes.iter().find(|(c, _)| *c == code).map(|(_, v)| v.as_str())
    }

    fn num(&self, code: i32, default: f64) -> f64 {
        self.s(code).and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    fn flag(&self, code: i32, bit: i32) -> bool {
        (self.num(code, 0.0) as i32) & bit != 0
    }

    fn all(&self, code: i32) -> Vec<f64> {
        self.codes.iter().filter(|(c, _)| *c == code).filter_map(|(_, v)| v.parse().ok()).collect()
    }

    /// The points written as x (`cx`) followed by y (`cy`), in file order.
    fn pts(&self, cx: i32, cy: i32) -> Vec<Pt> {
        let mut out: Vec<Pt> = Vec::new();
        for (c, v) in &self.codes {
            let n = v.parse().unwrap_or(0.0);
            if *c == cx {
                out.push([n, 0.0]);
            } else if *c == cy {
                if let Some(last) = out.last_mut() {
                    last[1] = n;
                }
            }
        }
        out
    }
}

struct Block {
    base: Pt,
    recs: Vec<Rec>,
}

struct Ctx<'a> {
    blocks: &'a HashMap<String, Block>,
    skipped: BTreeMap<String, usize>,
}

/// Group code / value pairs: the file alternates a code line and a value line.
fn pairs(text: &str) -> Vec<(i32, String)> {
    let mut out = Vec::new();
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    while let (Some(c), Some(v)) = (lines.next(), lines.next()) {
        if let Ok(code) = c.trim().parse::<i32>() {
            out.push((code, v.trim().to_string()));
        }
    }
    out
}

fn version_name(code: &str) -> String {
    match code {
        "AC1006" => "R10",
        "AC1009" => "R12",
        "AC1012" => "R13",
        "AC1014" => "R14",
        "AC1015" => "2000",
        "AC1018" => "2004",
        "AC1021" => "2007",
        "AC1024" => "2010",
        "AC1027" => "2013",
        "AC1032" => "2018",
        other => other,
    }
    .to_string()
}

/// Read a DXF file.
pub fn read(bytes: &[u8]) -> Result<Import, String> {
    if bytes.starts_with(b"AutoCAD Binary DXF") {
        return Err(t!("Binäre DXF-Dateien werden nicht gelesen. Bitte als ASCII-DXF (R12 oder R14 und neuer) speichern."));
    }
    let text = String::from_utf8_lossy(bytes);
    let mut recs: Vec<Rec> = Vec::new();
    for (code, value) in pairs(&text) {
        if code == 0 {
            recs.push(Rec { kind: value, codes: Vec::new() });
        } else if let Some(r) = recs.last_mut() {
            r.codes.push((code, value));
        }
    }

    let mut section = String::new();
    let mut version = String::new();
    let (mut scale, mut unit) = (1.0, "mm");
    let mut entities: Vec<Rec> = Vec::new();
    let mut blocks: HashMap<String, Block> = HashMap::new();
    let mut open_block: Option<(String, Block)> = None;
    let mut saw_entities = false;
    for rec in recs {
        match rec.kind.as_str() {
            "SECTION" => {
                section = rec.s(2).unwrap_or("").to_string();
                match section.as_str() {
                    "HEADER" => {
                        let var = |name: &str| rec.codes.iter().position(|(c, v)| *c == 9 && v == name).and_then(|i| rec.codes.get(i + 1)).map(|(_, v)| v.clone());
                        version = var("$ACADVER").map(|v| version_name(&v)).unwrap_or_default();
                        match var("$INSUNITS").and_then(|v| v.parse::<i32>().ok()) {
                            Some(1) => (scale, unit) = (25.4, "Zoll"),
                            Some(2) => (scale, unit) = (304.8, "Fuß"),
                            Some(5) => (scale, unit) = (10.0, "cm"),
                            Some(6) => (scale, unit) = (1000.0, "m"),
                            _ => {}
                        }
                    }
                    "ENTITIES" => saw_entities = true,
                    _ => {}
                }
            }
            "ENDSEC" => section.clear(),
            "EOF" => break,
            "BLOCK" if section == "BLOCKS" => {
                open_block = Some((rec.s(2).unwrap_or("").to_string(), Block { base: [rec.num(10, 0.0), rec.num(20, 0.0)], recs: Vec::new() }));
            }
            "ENDBLK" => {
                if let Some((name, block)) = open_block.take() {
                    blocks.insert(name, block);
                }
            }
            _ => match section.as_str() {
                "ENTITIES" => entities.push(rec),
                "BLOCKS" => {
                    if let Some((_, block)) = &mut open_block {
                        block.recs.push(rec);
                    }
                }
                _ => {}
            },
        }
    }
    if !saw_entities {
        return Err(t!("Das ist keine gültige DXF-Datei (kein ENTITIES-Abschnitt gefunden)."));
    }

    let mut cx = Ctx { blocks: &blocks, skipped: BTreeMap::new() };
    let mut prims = Vec::new();
    convert(&entities, &mut cx, 0, &mut prims);
    for prim in &mut prims {
        match prim {
            Prim::Circle { c, r } => {
                *c = [c[0] * scale, c[1] * scale];
                *r *= scale;
            }
            Prim::Path { pts, .. } => pts.iter_mut().for_each(|p| *p = [p[0] * scale, p[1] * scale]),
        }
    }
    Ok(Import { shapes: shapes_from(prims), version, unit, skipped: cx.skipped.into_iter().collect() })
}

fn open(pts: Vec<Pt>) -> Prim {
    Prim::Path { pts, closed: false }
}

fn mirror_x(prims: &mut [Prim]) {
    for prim in prims {
        match prim {
            Prim::Circle { c, .. } => c[0] = -c[0],
            Prim::Path { pts, .. } => pts.iter_mut().for_each(|p| p[0] = -p[0]),
        }
    }
}

/// Turn entities into curves and circles. `depth` guards against blocks that contain themselves.
fn convert(recs: &[Rec], cx: &mut Ctx, depth: u8, out: &mut Vec<Prim>) {
    let mut i = 0;
    while i < recs.len() {
        let rec = &recs[i];
        i += 1;
        // things in the paper-space layout are not part of the drawing
        if rec.flag(67, 1) {
            continue;
        }
        let start = out.len();
        let mut mirrored = rec.num(230, 1.0) < 0.0;
        let at = |a: i32, b: i32| [rec.num(a, 0.0), rec.num(b, 0.0)];
        match rec.kind.as_str() {
            "LINE" => out.push(open(vec![at(10, 20), at(11, 21)])),
            "CIRCLE" => out.push(Prim::Circle { c: at(10, 20), r: rec.num(40, 0.0) }),
            "ARC" => {
                let a0 = rec.num(50, 0.0).to_radians();
                let mut sweep = (rec.num(51, 0.0).to_radians() - a0).rem_euclid(TAU);
                if sweep < 1e-9 {
                    sweep = TAU;
                }
                out.push(open(arc_points(at(10, 20), rec.num(40, 0.0), a0, sweep)));
            }
            "LWPOLYLINE" => {
                let mut verts: Vec<(Pt, f64)> = Vec::new();
                for (code, v) in &rec.codes {
                    let n = v.parse().unwrap_or(0.0);
                    match code {
                        10 => verts.push(([n, 0.0], 0.0)),
                        20 => {
                            if let Some(last) = verts.last_mut() {
                                last.0[1] = n;
                            }
                        }
                        42 => {
                            if let Some(last) = verts.last_mut() {
                                last.1 = n;
                            }
                        }
                        _ => {}
                    }
                }
                let closed = rec.flag(70, 1);
                out.push(Prim::Path { pts: bulge_path(&verts, closed), closed });
            }
            "POLYLINE" => {
                let flags = rec.num(70, 0.0) as i32;
                let mut verts: Vec<(Pt, f64)> = Vec::new();
                while i < recs.len() && recs[i].kind == "VERTEX" {
                    let v = &recs[i];
                    // 16 marks a control point of a spline frame, not a point of the curve
                    if !v.flag(70, 16) {
                        verts.push(([v.num(10, 0.0), v.num(20, 0.0)], v.num(42, 0.0)));
                    }
                    i += 1;
                }
                if i < recs.len() && recs[i].kind == "SEQEND" {
                    i += 1;
                }
                if flags & (16 | 64) != 0 {
                    *cx.skipped.entry("POLYLINE (Netz)".into()).or_default() += 1;
                } else {
                    let closed = flags & 1 != 0;
                    out.push(Prim::Path { pts: bulge_path(&verts, closed), closed });
                }
                // a 3D polyline's points are world coordinates already
                mirrored = mirrored && flags & 8 == 0;
            }
            "ELLIPSE" => {
                let (c, major, ratio) = (at(10, 20), at(11, 21), rec.num(40, 1.0));
                let side = if rec.num(230, 1.0) < 0.0 { -1.0 } else { 1.0 };
                let minor = [-major[1] * ratio * side, major[0] * ratio * side];
                let a0 = rec.num(41, 0.0);
                let mut sweep = (rec.num(42, TAU) - a0).rem_euclid(TAU);
                if sweep < 1e-9 {
                    sweep = TAU;
                }
                let full = (sweep - TAU).abs() < 1e-9;
                let n = ((sweep / TAU * 128.0).ceil() as usize).max(8);
                let pts = (0..if full { n } else { n + 1 })
                    .map(|k| {
                        let t = a0 + sweep * k as f64 / n as f64;
                        [c[0] + major[0] * t.cos() + minor[0] * t.sin(), c[1] + major[1] * t.cos() + minor[1] * t.sin()]
                    })
                    .collect();
                out.push(Prim::Path { pts, closed: full });
            }
            "SPLINE" => out.push(open(spline_points(rec))),
            "INSERT" => {
                let blocks = cx.blocks;
                match blocks.get(rec.s(2).unwrap_or("")) {
                    Some(block) if depth < 8 => {
                        let mut inner = Vec::new();
                        convert(&block.recs, cx, depth + 1, &mut inner);
                        let (sx, sy) = (rec.num(41, 1.0), rec.num(42, 1.0));
                        let (sin, cos) = rec.num(50, 0.0).to_radians().sin_cos();
                        let (cols, rows) = ((rec.num(70, 1.0) as usize).clamp(1, 100), (rec.num(71, 1.0) as usize).clamp(1, 100));
                        let (dc, dr) = (rec.num(44, 0.0), rec.num(45, 0.0));
                        let ins = at(10, 20);
                        for col in 0..cols {
                            for row in 0..rows {
                                let (u, v) = (col as f64 * dc, row as f64 * dr);
                                let to = [ins[0] + u * cos - v * sin, ins[1] + u * sin + v * cos];
                                for prim in &inner {
                                    out.push(place(prim, block.base, [sx, sy], (sin, cos), to));
                                }
                            }
                        }
                    }
                    _ => *cx.skipped.entry("INSERT".into()).or_default() += 1,
                }
            }
            "VERTEX" | "SEQEND" | "ATTRIB" | "ATTDEF" | "VIEWPORT" => {}
            other => *cx.skipped.entry(other.to_string()).or_default() += 1,
        }
        // circles, arcs and 2D polylines are drawn in their own plane; one facing down is mirrored
        if mirrored && matches!(rec.kind.as_str(), "CIRCLE" | "ARC" | "LWPOLYLINE" | "POLYLINE" | "INSERT") {
            mirror_x(&mut out[start..]);
        }
    }
}

/// A block's content in its place in the drawing: moved so the block's base point lands on `to`,
/// scaled and turned.
fn place(prim: &Prim, base: Pt, scale: [f64; 2], (sin, cos): (f64, f64), to: Pt) -> Prim {
    let xf = |p: Pt| {
        let (x, y) = ((p[0] - base[0]) * scale[0], (p[1] - base[1]) * scale[1]);
        [to[0] + x * cos - y * sin, to[1] + x * sin + y * cos]
    };
    match prim {
        Prim::Path { pts, closed } => Prim::Path { pts: pts.iter().map(|p| xf(*p)).collect(), closed: *closed },
        // stretched unevenly, a circle is no circle any more
        Prim::Circle { c, r } if (scale[0].abs() - scale[1].abs()).abs() > 1e-9 => {
            let pts = (0..72).map(|k| {
                let a = TAU * k as f64 / 72.0;
                xf([c[0] + r * a.cos(), c[1] + r * a.sin()])
            });
            Prim::Path { pts: pts.collect(), closed: true }
        }
        Prim::Circle { c, r } => Prim::Circle { c: xf(*c), r: r * scale[0].abs() },
    }
}

/// Points along an arc from angle `a0` turning by `sweep` (counter-clockwise when positive),
/// both ends included.
fn arc_points(c: Pt, r: f64, a0: f64, sweep: f64) -> Vec<Pt> {
    let step = if r > ARC_TOL { 2.0 * (1.0 - ARC_TOL / r).acos() } else { PI / 2.0 };
    let n = ((sweep.abs() / step).ceil() as usize).clamp(2, 720);
    (0..=n)
        .map(|k| {
            let a = a0 + sweep * k as f64 / n as f64;
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        })
        .collect()
}

/// The points of a polyline whose segments may be arcs: a bulge is the tangent of a quarter of the
/// arc's angle, positive counter-clockwise.
fn bulge_path(verts: &[(Pt, f64)], closed: bool) -> Vec<Pt> {
    let n = verts.len();
    let segments = if closed { n } else { n.saturating_sub(1) };
    let mut out = Vec::new();
    for (i, (p1, bulge)) in verts.iter().enumerate() {
        out.push(*p1);
        if i < segments && bulge.abs() > 1e-9 {
            let p2 = verts[(i + 1) % n].0;
            let theta = 4.0 * bulge.atan();
            let (dx, dy) = (p2[0] - p1[0], p2[1] - p1[1]);
            let chord = dx.hypot(dy);
            if chord < 1e-12 {
                continue;
            }
            // the centre lies left of the chord for a counter-clockwise arc
            let h = chord / 2.0 / (theta / 2.0).tan();
            let c = [(p1[0] + p2[0]) / 2.0 - dy / chord * h, (p1[1] + p2[1]) / 2.0 + dx / chord * h];
            let r = (p1[0] - c[0]).hypot(p1[1] - c[1]);
            let arc = arc_points(c, r, (p1[1] - c[1]).atan2(p1[0] - c[0]), theta);
            out.extend_from_slice(&arc[1..arc.len() - 1]);
        }
    }
    out
}

/// Points along a spline: the B-spline through its control points when the file has the knots
/// for it, else its fit points, else the control polygon.
fn spline_points(rec: &Rec) -> Vec<Pt> {
    let degree = rec.num(71, 3.0) as usize;
    let knots = rec.all(40);
    let ctrl = rec.pts(10, 20);
    let n = ctrl.len();
    if degree >= 1 && n > degree && knots.len() == n + degree + 1 {
        let weights = rec.all(41);
        let weight = |i: usize| if weights.len() == n { weights[i] } else { 1.0 };
        let (t0, t1) = (knots[degree], knots[n]);
        let samples = (n * 16).clamp(32, 2000);
        let mut out = Vec::with_capacity(samples + 1);
        for s in 0..=samples {
            let t = t0 + (t1 - t0) * s as f64 / samples as f64;
            let mut k = degree;
            while k + 1 < n && t >= knots[k + 1] {
                k += 1;
            }
            // De Boor's algorithm on weighted points (x·w, y·w, w)
            let mut d: Vec<[f64; 3]> = (0..=degree).map(|j| {
                let i = j + k - degree;
                [ctrl[i][0] * weight(i), ctrl[i][1] * weight(i), weight(i)]
            }).collect();
            for r in 1..=degree {
                for j in (r..=degree).rev() {
                    let i = j + k - degree;
                    let denom = knots[i + degree - r + 1] - knots[i];
                    let a = if denom.abs() < 1e-12 { 0.0 } else { (t - knots[i]) / denom };
                    for m in 0..3 {
                        d[j][m] = (1.0 - a) * d[j - 1][m] + a * d[j][m];
                    }
                }
            }
            let w = d[degree][2];
            out.push(if w.abs() < 1e-12 { [d[degree][0], d[degree][1]] } else { [d[degree][0] / w, d[degree][1] / w] });
        }
        return out;
    }
    let fit = rec.pts(11, 21);
    if fit.len() >= 2 {
        fit
    } else {
        ctrl
    }
}

fn near(a: Pt, b: Pt, tol: f64) -> bool {
    (a[0] - b[0]).hypot(a[1] - b[1]) <= tol
}

fn cell(p: Pt) -> (i64, i64) {
    ((p[0] / JOIN_TOL).floor() as i64, (p[1] / JOIN_TOL).floor() as i64)
}

/// Drop repeated points; a closed curve does not repeat its first point at the end.
fn clean(mut pts: Vec<Pt>, closed: bool) -> Vec<Pt> {
    pts.dedup_by(|a, b| near(*a, *b, 1e-9));
    if closed && pts.len() > 1 && near(pts[0], pts[pts.len() - 1], 1e-9) {
        pts.pop();
    }
    pts
}

/// Chain open curves that meet end to end; a chain whose ends meet is closed.
fn join(open: Vec<Vec<Pt>>) -> Vec<(Vec<Pt>, bool)> {
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, p) in open.iter().enumerate() {
        grid.entry(cell(p[0])).or_default().push(i);
        grid.entry(cell(p[p.len() - 1])).or_default().push(i);
    }
    let find = |at: Pt, used: &[bool]| -> Option<usize> {
        let (cx, cy) = cell(at);
        for gx in cx - 1..=cx + 1 {
            for gy in cy - 1..=cy + 1 {
                let Some(list) = grid.get(&(gx, gy)) else { continue };
                for &j in list {
                    if !used[j] && (near(open[j][0], at, JOIN_TOL) || near(open[j][open[j].len() - 1], at, JOIN_TOL)) {
                        return Some(j);
                    }
                }
            }
        }
        None
    };
    let mut used = vec![false; open.len()];
    let mut out = Vec::new();
    for i in 0..open.len() {
        if used[i] {
            continue;
        }
        used[i] = true;
        let mut cur = open[i].clone();
        // first grow at the end, then (turned round) at the start
        for _ in 0..2 {
            loop {
                let end = cur[cur.len() - 1];
                if cur.len() > 2 && near(cur[0], end, JOIN_TOL) {
                    break;
                }
                let Some(j) = find(end, &used) else { break };
                used[j] = true;
                let mut next = open[j].clone();
                if !near(next[0], end, JOIN_TOL) {
                    next.reverse();
                }
                cur.extend(next.into_iter().skip(1));
            }
            cur.reverse();
        }
        let closed = cur.len() >= 4 && near(cur[0], cur[cur.len() - 1], JOIN_TOL);
        out.push((cur, closed));
    }
    out
}

fn shapes_from(prims: Vec<Prim>) -> Vec<Shape> {
    let mut shapes = Vec::new();
    let mut loose: Vec<Vec<Pt>> = Vec::new();
    for prim in prims {
        match prim {
            Prim::Circle { c, r } => {
                if r > 0.0 {
                    shapes.push(Circle::new(c, r).into());
                }
            }
            Prim::Path { pts, closed } => {
                let pts = clean(pts, closed);
                if closed && pts.len() >= 3 {
                    shapes.push(Path::new(pts, true).into());
                } else if !closed && pts.len() >= 2 {
                    loose.push(pts);
                }
            }
        }
    }
    for (pts, closed) in join(loose) {
        let pts = clean(pts, closed);
        if pts.len() >= if closed { 3 } else { 2 } {
            shapes.push(Path::new(pts, closed).into());
        }
    }
    shapes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(header: &str, entities: &str, blocks: &str) -> Vec<u8> {
        format!("  0\nSECTION\n  2\nHEADER\n{header}  0\nENDSEC\n  0\nSECTION\n  2\nBLOCKS\n{blocks}  0\nENDSEC\n  0\nSECTION\n  2\nENTITIES\n{entities}  0\nENDSEC\n  0\nEOF\n").into_bytes()
    }

    fn version(v: &str) -> String {
        format!("  9\n$ACADVER\n  1\n{v}\n")
    }

    fn line(a: Pt, b: Pt) -> String {
        format!("  0\nLINE\n  8\n0\n 10\n{}\n 20\n{}\n 30\n0.0\n 11\n{}\n 21\n{}\n 31\n0.0\n", a[0], a[1], b[0], b[1])
    }

    fn circle(c: Pt, r: f64) -> String {
        format!("  0\nCIRCLE\n  8\n0\n 10\n{}\n 20\n{}\n 40\n{r}\n", c[0], c[1])
    }

    fn path(s: &Shape) -> (&Vec<Pt>, bool) {
        let Shape::Path(p) = s else { panic!("a curve expected, got {}", s.label()) };
        (&p.points, p.closed)
    }

    #[test]
    fn r12_lines_that_meet_become_one_closed_curve() {
        // four lines of a rectangle, out of order and some turned round
        let lines = [line([0.0, 0.0], [10.0, 0.0]), line([10.0, 10.0], [10.0, 0.0]), line([0.0, 10.0], [0.0, 0.0]), line([10.0, 10.0], [0.0, 10.0])].concat();
        let imp = read(&file(&version("AC1009"), &lines, "")).unwrap();
        assert_eq!(imp.version, "R12");
        assert_eq!(imp.shapes.len(), 1);
        let (pts, closed) = path(&imp.shapes[0]);
        assert!(closed);
        assert_eq!(pts.len(), 4);
    }

    #[test]
    fn lines_that_do_not_meet_stay_apart() {
        let lines = [line([0.0, 0.0], [10.0, 0.0]), line([20.0, 0.0], [30.0, 0.0])].concat();
        let imp = read(&file("", &lines, "")).unwrap();
        assert_eq!(imp.shapes.len(), 2);
        assert!(imp.shapes.iter().all(|s| !path(s).1));
    }

    #[test]
    fn circles_stay_circles() {
        let imp = read(&file(&version("AC1014"), &circle([5.0, 5.0], 3.0), "")).unwrap();
        let Shape::Circle(c) = &imp.shapes[0] else { panic!("a circle") };
        assert_eq!((c.center, c.radius), ([5.0, 5.0], 3.0));
    }

    #[test]
    fn r12_polyline_with_a_bulge_is_a_half_circle() {
        let poly = "  0\nPOLYLINE\n  8\n0\n 66\n1\n 70\n0\n  0\nVERTEX\n  8\n0\n 10\n0.0\n 20\n0.0\n 42\n1.0\n  0\nVERTEX\n  8\n0\n 10\n2.0\n 20\n0.0\n  0\nSEQEND\n";
        let imp = read(&file(&version("AC1009"), poly, "")).unwrap();
        let (pts, closed) = path(&imp.shapes[0]);
        assert!(!closed);
        assert_eq!((pts[0], pts[pts.len() - 1]), ([0.0, 0.0], [2.0, 0.0]));
        assert!(pts.iter().all(|p| ((p[0] - 1.0).hypot(p[1]) - 1.0).abs() < 1e-6), "on the circle round (1, 0)");
        assert!(pts.iter().any(|p| p[1] < -0.9), "a positive bulge turns counter-clockwise, so below the chord");
    }

    #[test]
    fn r14_lwpolyline_arc_and_closed_flag() {
        let square = "  0\nLWPOLYLINE\n  8\n0\n 90\n4\n 70\n1\n 10\n0\n 20\n0\n 10\n4\n 20\n0\n 10\n4\n 20\n4\n 10\n0\n 20\n4\n";
        let arc = "  0\nARC\n  8\n0\n 10\n0\n 20\n0\n 40\n10\n 50\n0\n 51\n90\n";
        let imp = read(&file(&version("AC1027"), &format!("{square}{arc}"), "")).unwrap();
        assert_eq!(imp.version, "2013");
        assert_eq!(imp.shapes.len(), 2);
        let (pts, closed) = path(&imp.shapes[0]);
        assert!(closed && pts.len() == 4);
        let (arc, closed) = path(&imp.shapes[1]);
        assert!(!closed);
        assert!(near(arc[0], [10.0, 0.0], 1e-9) && near(arc[arc.len() - 1], [0.0, 10.0], 1e-9));
    }

    #[test]
    fn blocks_are_placed_scaled_and_turned() {
        let block = format!("  0\nBLOCK\n  8\n0\n  2\nB\n 10\n0\n 20\n0\n{}  0\nENDBLK\n", circle([1.0, 1.0], 1.0));
        let insert = "  0\nINSERT\n  8\n0\n  2\nB\n 10\n10\n 20\n20\n 41\n2\n 42\n2\n 50\n90\n";
        let imp = read(&file(&version("AC1014"), insert, &block)).unwrap();
        let Shape::Circle(c) = &imp.shapes[0] else { panic!("a circle") };
        // (1,1) scaled by 2 is (2,2), turned a quarter is (-2,2), moved to (10,20)
        assert!(near(c.center, [8.0, 22.0], 1e-9), "{:?}", c.center);
        assert!((c.radius - 2.0).abs() < 1e-9);
    }

    #[test]
    fn the_drawing_unit_is_converted_to_millimetres() {
        let header = format!("{}  9\n$INSUNITS\n 70\n     1\n", version("AC1021"));
        let imp = read(&file(&header, &line([0.0, 0.0], [2.0, 0.0]), "")).unwrap();
        assert_eq!(imp.unit, "Zoll");
        assert!(near(path(&imp.shapes[0]).0[1], [50.8, 0.0], 1e-9));
    }

    #[test]
    fn a_quadratic_spline_follows_its_control_points() {
        let spline = "  0\nSPLINE\n  8\n0\n 70\n8\n 71\n2\n 72\n6\n 73\n3\n 40\n0\n 40\n0\n 40\n0\n 40\n1\n 40\n1\n 40\n1\n 10\n0\n 20\n0\n 10\n5\n 20\n10\n 10\n10\n 20\n0\n";
        let imp = read(&file(&version("AC1015"), spline, "")).unwrap();
        let (pts, _) = path(&imp.shapes[0]);
        assert!(near(pts[0], [0.0, 0.0], 1e-9) && near(pts[pts.len() - 1], [10.0, 0.0], 1e-9));
        assert!(pts.iter().any(|p| near(*p, [5.0, 5.0], 1e-6)), "the middle of the curve is (5, 5)");
    }

    #[test]
    fn what_cannot_be_read_is_counted() {
        let text = "  0\nTEXT\n  8\n0\n  1\nHallo\n  0\nTEXT\n  8\n0\n  1\nWelt\n  0\nHATCH\n  8\n0\n";
        let imp = read(&file("", &format!("{text}{}", circle([0.0, 0.0], 1.0)), "")).unwrap();
        assert_eq!(imp.shapes.len(), 1);
        assert_eq!(imp.skipped, vec![("HATCH".to_string(), 1), ("TEXT".to_string(), 2)]);
    }

    #[test]
    fn not_a_dxf_and_binary_dxf_are_refused() {
        assert!(read(b"Hallo Welt").is_err());
        assert!(read(b"AutoCAD Binary DXF\r\n\x1a\0").unwrap_err().contains("Binäre"));
    }
}
