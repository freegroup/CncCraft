//! Self-test stages for the vector operations: each clicks in the drawing as a person would and
//! checks what became of the vectors. They run last and start from an empty drawing.

use super::selftest::Check;
use super::{App, DrawTool};
use egui::{Event, Modifiers, PointerButton};
use shape_edit::{CornerKind, Path, Pt, Shape};

/// A click at drawing point `p`.
fn click(app: &App, p: Pt) -> Vec<Event> {
    let at = app.to_screen(p);
    vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ]
}

/// The paths of the drawing, in order.
fn paths(app: &App) -> Vec<Path> {
    app.shapes.iter().filter_map(|s| match &s.shape {
        Shape::Path(p) => Some(p.clone()),
        _ => None,
    }).collect()
}

fn result(ok: bool, msg: String) -> Result<String, String> {
    if ok {
        Ok(msg)
    } else {
        Err(msg)
    }
}

/// Trennen: a line crossed by another falls apart into two pieces at the crossing.
pub(super) fn trim_splits_at_crossing(app: &mut App) -> (Vec<Event>, Check) {
    // the stage before leaves the 3D preview open: the vector tools work in the 2D drawing
    app.mode = super::ViewMode::TwoD;
    app.sim = None;
    app.shapes.clear();
    app.toolpaths.clear();
    app.edit.selection.clear();
    app.add_shape(Path::new(vec![[100.0, 100.0], [200.0, 100.0]], false));
    app.add_shape(Path::new(vec![[150.0, 50.0], [150.0, 150.0]], false));
    app.tool = DrawTool::Trim;
    let events = click(app, [120.0, 100.0]);
    let check: Check = Box::new(|app| {
        let p = paths(app);
        let ends: Vec<(Pt, Pt)> = p.iter().map(|p| (p.points[0], *p.points.last().unwrap())).collect();
        let left = ends.contains(&([100.0, 100.0], [150.0, 100.0]));
        let right = ends.contains(&([150.0, 100.0], [200.0, 100.0]));
        result(p.len() == 3 && left && right, format!("{} Vektoren, linkes Stück: {left}, rechtes Stück: {right}", p.len()))
    });
    (events, check)
}

/// Löschen: the piece clicked is gone, the others stay.
pub(super) fn erase_deletes_the_vector_clicked(app: &mut App) -> (Vec<Event>, Check) {
    app.tool = DrawTool::Erase;
    let events = click(app, [180.0, 100.0]);
    let check: Check = Box::new(|app| {
        let p = paths(app);
        let gone = !p.iter().any(|p| p.points.contains(&[200.0, 100.0]));
        result(p.len() == 2 && gone, format!("{} Vektoren, rechtes Stück gelöscht: {gone}", p.len()))
    });
    (events, check)
}

/// Verbinden: a line from the end of a polyline to a free point grows the polyline.
pub(super) fn connect_grows_the_polyline(app: &mut App) -> (Vec<Event>, Check) {
    app.tool = DrawTool::Connect;
    let mut events = click(app, [100.0, 100.0]);
    events.extend(click(app, [100.0, 160.0]));
    let check: Check = Box::new(|app| {
        let p = paths(app);
        // a free point comes from the screen and is only as exact as a pixel's position
        let near = |a: &[Pt], b: &[Pt]| a.len() == b.len() && a.iter().zip(b).all(|(p, q)| shape_edit::geometry::dist(*p, *q) < 1e-3);
        let grown = p.iter().any(|p| near(&p.points, &[[100.0, 160.0], [100.0, 100.0], [150.0, 100.0]]));
        result(p.len() == 2 && grown, format!("{} Vektoren, Linienzug verlängert: {grown}", p.len()))
    });
    (events, check)
}

/// Verbinden: a line between the two ends of one polyline closes it.
pub(super) fn connect_closes_the_polyline(app: &mut App) -> (Vec<Event>, Check) {
    app.tool = DrawTool::Connect;
    let mut events = click(app, [100.0, 160.0]);
    events.extend(click(app, [150.0, 100.0]));
    let check: Check = Box::new(|app| {
        let p = paths(app);
        let closed = p.iter().any(|p| p.closed && p.points.len() == 3);
        result(p.len() == 2 && closed, format!("{} Vektoren, Dreieck geschlossen: {closed}", p.len()))
    });
    (events, check)
}

/// Abrunden: the corner clicked gets a rounded corner with the form's radius; its point stays.
pub(super) fn fillet_inner_rounds_the_corner(app: &mut App) -> (Vec<Event>, Check) {
    app.tool = DrawTool::FilletInner;
    app.vops.radius_text = "5".into();
    let events = click(app, [100.0, 100.0]);
    let check: Check = Box::new(|app| {
        let p = paths(app);
        let tri = p.iter().find(|p| p.closed);
        let corner = tri.and_then(|t| t.corner(1)).map(|c| (c.kind, c.radius));
        let kept = tri.is_some_and(|t| t.points.contains(&[100.0, 100.0]));
        result(corner == Some((CornerKind::Round, 5.0)) && kept, format!("Ecke: {corner:?}, Eckpunkt erhalten: {kept}"))
    });
    (events, check)
}

/// Ohr: another corner gets an ear, the rounded one stays.
pub(super) fn fillet_outer_gives_an_ear(app: &mut App) -> (Vec<Event>, Check) {
    app.tool = DrawTool::FilletOuter;
    app.vops.radius_text = "3".into();
    let events = click(app, [150.0, 100.0]);
    let check: Check = Box::new(|app| {
        let p = paths(app);
        let tri = p.iter().find(|p| p.closed);
        let ear = tri.and_then(|t| t.corner(2)).map(|c| (c.kind, c.radius));
        let round_kept = tri.and_then(|t| t.corner(1)).map(|c| c.kind);
        result(ear == Some((CornerKind::Ear, 3.0)) && round_kept == Some(CornerKind::Round), format!("Ohr: {ear:?}, abgerundete Ecke bleibt: {round_kept:?}"))
    });
    (events, check)
}

/// Abrunden: a click on the rounded corner picks it, so the form shows and changes its radius.
pub(super) fn fillet_click_picks_the_corner(app: &mut App) -> (Vec<Event>, Check) {
    app.tool = DrawTool::FilletInner;
    app.vops.radius_text = "1".into();
    let events = click(app, [100.0, 100.0]);
    let check: Check = Box::new(|app| {
        let picked = app.vops.picked.map(|(_, path, node)| (path, node));
        let p = paths(app);
        let radius = p.iter().find(|p| p.closed).and_then(|t| t.corner(1)).map(|c| c.radius);
        result(picked == Some((0, 1)) && radius == Some(5.0) && app.vops.radius_text == "5", format!("gewählt: {picked:?}, Radius bleibt {radius:?}, Formular zeigt {}", app.vops.radius_text))
    });
    (events, check)
}
