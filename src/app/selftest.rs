//! `--selftest`: drives the real window with synthetic mouse events, exactly as a person would
//! (move, press, drag in small steps, release), and checks what happened to the shapes. Prints a
//! line per check and exits with status 1 if any failed.

use super::{App, DrawTool};
use crate::geom::{Item, Pt};
use egui::{Event, Modifiers, PointerButton, Pos2, Vec2};
use shape_edit::{handle_point, selection_bounds, Dim, Editable, Handle, Shape};
use std::collections::VecDeque;

type Check = Box<dyn Fn(&App) -> Result<String, String>>;

struct Stage {
    name: &'static str,
    /// Sets up the stage and returns the gesture plus the check to run after it.
    run: fn(&mut App) -> (Vec<Event>, Check),
    /// Keys held down during the gesture.
    held: Modifiers,
}

const fn stage(name: &'static str, run: fn(&mut App) -> (Vec<Event>, Check)) -> Stage {
    Stage { name, run, held: Modifiers::NONE }
}

pub(super) struct SelfTest {
    frame: u32,
    /// Frames the view has stayed the same, and how it looked (before the first stage).
    stable: u32,
    last_view: Option<(u32, u32, u32, Option<(u32, u32)>)>,
    stage: usize,
    queue: VecDeque<Event>,
    check: Option<Check>,
    settle: u32,
    failures: usize,
}

const STAGES: &[Stage] = &[
    stage("Größe über den linken Anfasser ändern", resize_left),
    stage("Kreis: der rechte Anfasser vergrößert", resize_corner),
    stage("Form verschieben", move_shape),
    stage("Rahmen aufziehen und auswählen", band_select),
    stage("Knoten ziehen", drag_node),
    stage("Rechteck an der Job-Ecke einrasten", draw_snapped_rect),
    stage("Breite als Formel eintippen", type_width),
    stage("Tab ins Höhenfeld, Klick in die Arbeitsfläche übernimmt", tab_then_click),
    stage("Erneut klicken dreht nicht mehr (Drehen kommt später als Winkel-Constraint)", no_rotation),
    stage("Eckenradius am eigenen Griff ziehen", corner_radius_grip),
    stage("Taste N: Knotenmodus", node_key),
    Stage { name: "Mit ⌃ ziehen: Kopie", run: ctrl_copy, held: Modifiers::CTRL },
    stage("Variablen mit Vervollständigung in Breite und Höhe", vars_typed),
    stage("Variable ändern: die Form folgt, untere linke Ecke bleibt", vars_changed),
    stage("Unbekannte Variable: das Feld bleibt offen", vars_invalid_stays_open),
    stage("Esc schließt das Feld, nichts wurde übernommen", vars_escape),
    stage("Gebundene Breite und Höhe lassen sich nicht ziehen", vars_locked_drag),
    stage("Variable umbenennen: Verwender ziehen nach, neuer Wert wirkt", vars_renamed),
    stage("Variable löschen: gebundene Größe wird zur Zahl", vars_deleted),
    stage("Maße anzeigen: die Werte der Auswahl werden gezeichnet", dims_shown),
    stage("Kurzer Klick auf einen Wert ohne Constraint wählt nichts", dims_label_quick_click),
    stage("Gedrückt halten auf dem Wert öffnet ihn zum Ändern", dims_label_click),
    stage("Eingetippte Größe ist ein Abstands-Constraint wie mit D", typed_size_is_a_constraint),
    stage("Constraint anklicken und Entf: der Wert bleibt, die Bindung fällt weg", delete_length_constraint),
    stage("Werte unverändert übernehmen ist kein Constraint", accept_untouched_values),
    stage("Bemaßen (D): zwei Ecken wählen, platzieren, Abstand eintippen", distance_by_mouse),
    stage("Die bemaßte Form und die Maßlinie folgen, wenn die erste sich bewegt", distance_follows),
    stage("Ziehen am Wert verschiebt das Maß", distance_label_drag),
    stage("Bemaßen mit der Kante der Platte als Bezug", distance_to_plate),
    stage("Zwei Punkte derselben Form bemaßen: die Form wird skaliert", distance_same_shape),
    stage("Eine Linie allein bemaßen: ihre Länge", distance_line_length),
    stage("D beendet das Bemaßen", distance_mode_off),
    stage("H: eine schräge Vektorlinie anklicken, sie wird waagrecht", align_line),
    stage("Symbol anklicken und Entf: die Linie bleibt, das Constraint ist weg", align_delete),
    stage("Zwei Punkte zweier Formen: waagrecht, die zweite Form folgt", align_two_points),
    stage("H beendet das Werkzeug", align_mode_off),
    stage("Werkzeugweg-Liste: per Ziehen am Griff an eine andere Stelle", toolpath_list_drag),
    stage("Tasten L, C, R wählen Linie, Kreis, Rechteck", tool_keys),
    stage("Taste S wählt den Selektionsmodus", tool_keys_back),
    stage("Abstand zur Plattenkante: senkrecht verschieben bleibt frei", plate_constraint_drag_vertical),
    stage("Abstand zur Plattenkante: die Höhe lässt sich ziehen", plate_constraint_resize_height),
    Stage { name: "Großes S (mit Umschalt) im Bemaßen-Modus wählt ebenfalls Auswählen", run: capital_s_leaves_dimension_mode, held: Modifiers::SHIFT },
    stage("Neue Variable: Dialog mit markiertem Namen, OK legt sie an", new_variable_focus),
    stage("Neue Variable: der Ausdruck vervollständigt vorhandene Variablen", new_variable_autocomplete),
    stage("Abstands-Constraint: der Wert vervollständigt Variablen (Enter, Tab)", distance_value_autocomplete),
    stage("Ebene ausblenden: Form nicht wählbar, nicht gefräst, nicht exportiert", layer_hidden),
    stage("Ebene wieder einblenden: Form wählbar und im Werkzeugweg", layer_shown),
    stage("Ebene sperren: Form bleibt sichtbar, ist aber nicht anklickbar", layer_locked),
    stage("Ebene entsperren: Form lässt sich wieder anklicken", layer_unlocked),
    stage("Auswählen (S): am Mittelpunkt der gewählten Form ziehen verschiebt sie", move_by_centre),
    stage("Auswählen (S): am Mittelpunkt einer nicht gewählten Form ziehen verschiebt sie", move_unselected_by_centre),
    stage("Vorschau und Export: ausgewählter, sichtbare und alle Werkzeugwege", scopes),
    stage("Werkzeugwechsel: Eintrag in der Liste und Frage beim Speichern", tool_changes),
    stage("Ungespeicherte Änderungen: Warnung vor dem Ersetzen und Schließen", unsaved_warning),
    stage("Kopieren und Einfügen: Werkzeugweg und eigene Constraints kommen mit, Bezüge zu anderem nicht", copy_paste),
    stage("⌘C und ⌘V (am Mac als Kopieren-/Einfügen-Ereignis) kopieren und fügen ein", cmd_c_cmd_v),
    stage("Nullpunkt-Schalter: Achsenkreuz im Blatt und in 3D zugleich aus und wieder an", origin_toggle),
    stage("DXF-Datei importieren: eine Gruppe, ein Klick wählt alle, Auflösen ist möglich", dxf_import_is_a_group),
    stage("Gruppe auflösen: die Formen sind wieder einzeln, Gruppieren ist wieder möglich", dxf_ungroup),
    stage("Nach dem Auflösen: ein Klick auf den Kreis wählt nur den Kreis", dxf_click_one_after_ungroup),
    stage("DXF mit nur einer Form: eine Gruppe aus einem Mitglied, Auflösen ist möglich", dxf_single_shape),
    stage("Form gewählt: genau die Werkzeugwege, die sie fräsen, leuchten in der Liste", toolpaths_of_selected_shape),
    stage("Ziehen an der Linie (ohne Wert, Form nicht gewählt) verschiebt das Maß", distance_line_drag),
    stage("Gewählte Form: ihre Maßlinie lässt sich an der Linie und am Wert herausziehen", shape_dim_line_drag),
    stage("Simulation: in der Mitte des Reglers läuft sie in Echtzeit", simulation_in_real_time),
];

/// Selecting a shape lights up the toolpaths that cut it in the list, and only those; with nothing
/// selected nothing lights up.
fn toolpaths_of_selected_shape(app: &mut App) -> (Vec<Event>, Check) {
    let (k, id) = app.toolpaths.iter().enumerate().find_map(|(k, t)| t.shape_ids.first().map(|id| (k, *id))).expect("the example has toolpaths");
    app.edit.selection = vec![id];
    let lit = app.toolpaths_of_selection();
    app.edit.selection.clear();
    let none = app.toolpaths_of_selection();
    let check: Check = Box::new(move |app| {
        let all_cut_it = lit.iter().all(|&i| app.toolpaths[i].shape_ids.contains(&id));
        let others_do_not = (0..app.toolpaths.len()).filter(|i| !lit.contains(i)).all(|i| !app.toolpaths[i].shape_ids.contains(&id));
        let msg = format!("Form {id}: Werkzeugwege {lit:?} leuchten (erwartet auch {k}), alle fräsen sie: {all_cut_it}, die anderen nicht: {others_do_not}, ohne Auswahl {none:?}");
        if lit.contains(&k) && all_cut_it && others_do_not && none.is_empty() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// A DXF line entity.
fn dxf_line(a: Pt, b: Pt) -> String {
    format!("  0\nLINE\n  8\n0\n 10\n{}\n 20\n{}\n 30\n0.0\n 11\n{}\n 21\n{}\n 31\n0.0\n", a[0], a[1], b[0], b[1])
}

/// Four loose lines that make a rectangle with the corners `lo` and `hi`.
fn dxf_rectangle(lo: Pt, hi: Pt) -> String {
    let corners = [lo, [hi[0], lo[1]], hi, [lo[0], hi[1]]];
    (0..4).map(|i| dxf_line(corners[i], corners[(i + 1) % 4])).collect()
}

/// Write a DXF file (R14) with these entities to the temporary folder.
fn write_dxf(name: &str, entities: &str) -> std::path::PathBuf {
    let text = format!("  0\nSECTION\n  2\nHEADER\n  9\n$ACADVER\n  1\nAC1014\n  0\nENDSEC\n  0\nSECTION\n  2\nENTITIES\n{entities}  0\nENDSEC\n  0\nEOF\n");
    let path = std::env::temp_dir().join(format!("cnccraft-selftest-{}-{name}.dxf", std::process::id()));
    std::fs::write(&path, text).expect("a temporary file can be written");
    path
}

/// A DXF file (four loose lines that make a rectangle, a circle, one more line) is written to disk
/// and imported. It comes in as one group: a click on the circle picks all three shapes, "Gruppe
/// auflösen" is offered and "Gruppieren" is not (it is one group already).
fn dxf_import_is_a_group(app: &mut App) -> (Vec<Event>, Check) {
    let entities = format!("{}  0\nCIRCLE\n  8\n0\n 10\n40\n 20\n100\n 30\n0.0\n 40\n10\n{}", dxf_rectangle([20.0, 150.0], [60.0, 170.0]), dxf_line([100.0, 160.0], [140.0, 160.0]));
    let path = write_dxf("gruppe", &entities);

    app.tool = DrawTool::Select;
    app.mode = super::ViewMode::TwoD;
    let before = app.shapes.len();
    app.import_file(&path);
    let _ = std::fs::remove_file(&path);
    let ids: Vec<u64> = app.shapes.iter().skip(before).map(|s| s.id).collect();
    app.edit.selection.clear();
    // on the circle's left edge
    let at = app.to_screen([30.0, 100.0]);
    let click = vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    let check: Check = Box::new(move |app| {
        let (mut picked, mut want) = (app.edit.selection.clone(), ids.clone());
        picked.sort();
        want.sort();
        let grouped = ids.iter().all(|id| app.shapes.iter().find(|s| s.id == *id).is_some_and(|s| s.groups.len() == 1));
        let (group_on, ungroup_on) = (app.icon_state(super::Icon::Group).1, app.icon_state(super::Icon::Ungroup).1);
        let msg = format!("{} Formen importiert, alle in einer Gruppe: {grouped}, der Klick wählte {} davon, Gruppieren aktiv: {group_on}, Gruppe auflösen aktiv: {ungroup_on}", ids.len(), picked.len());
        if ids.len() == 3 && grouped && picked == want && !group_on && ungroup_on {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (click, check)
}

/// After ungrouping the import its three shapes are all still selected; a click on the circle (one
/// of the selected shapes) picks the circle alone, so each shape can be worked on by itself.
fn dxf_click_one_after_ungroup(app: &mut App) -> (Vec<Event>, Check) {
    let circle = app.edit.selection.iter().copied().find(|id| app.shapes.iter().any(|s| s.id == *id && matches!(s.shape, Shape::Circle(_)))).expect("the import has a circle");
    let at = app.to_screen([30.0, 100.0]);
    let click = vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    let check: Check = Box::new(move |app| {
        let msg = format!("gewählt: {:?} (der Kreis ist {circle})", app.edit.selection);
        if app.edit.selection == vec![circle] {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (click, check)
}

/// A file that comes in as one shape only (a rectangle of four lines) is a group of one: "Gruppe
/// auflösen" is offered, "Gruppieren" is not.
fn dxf_single_shape(app: &mut App) -> (Vec<Event>, Check) {
    let path = write_dxf("einzeln", &dxf_rectangle([200.0, 150.0], [240.0, 170.0]));
    app.tool = DrawTool::Select;
    let before = app.shapes.len();
    app.import_file(&path);
    let _ = std::fs::remove_file(&path);
    let ids: Vec<u64> = app.shapes.iter().skip(before).map(|s| s.id).collect();
    let check: Check = Box::new(move |app| {
        let grouped = ids.iter().all(|id| app.shapes.iter().find(|s| s.id == *id).is_some_and(|s| s.groups.len() == 1));
        let (group_on, ungroup_on) = (app.icon_state(super::Icon::Group).1, app.icon_state(super::Icon::Ungroup).1);
        let msg = format!("{} Form importiert, in einer Gruppe: {grouped}, gewählt: {}, Gruppieren aktiv: {group_on}, Gruppe auflösen aktiv: {ungroup_on}", ids.len(), app.edit.selection.len());
        if ids.len() == 1 && grouped && app.edit.selection == ids && !group_on && ungroup_on {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// Ungrouping the selected import: the three shapes lie loose again and can be grouped again.
fn dxf_ungroup(app: &mut App) -> (Vec<Event>, Check) {
    let ids = app.edit.selection.clone();
    app.ungroup_selection();
    let check: Check = Box::new(move |app| {
        let loose = ids.iter().all(|id| app.shapes.iter().find(|s| s.id == *id).is_some_and(|s| s.groups.is_empty()));
        let (group_on, ungroup_on) = (app.icon_state(super::Icon::Group).1, app.icon_state(super::Icon::Ungroup).1);
        let msg = format!("{} Formen gewählt, alle ohne Gruppe: {loose}, Gruppieren aktiv: {group_on}, Gruppe auflösen aktiv: {ungroup_on}", app.edit.selection.len());
        if ids.len() == 3 && loose && app.edit.selection.len() == 3 && group_on && !ungroup_on {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// The rectangle with both sizes tied to variables: dragging its right and top handles does
/// nothing (a constraint), while the shape stays selected.
fn vars_locked_drag(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    app.tool = DrawTool::Select;
    app.edit.selection = vec![id];
    app.edit.rotate_mode = false;
    let right = handle_screen(app, id, Handle::Right);
    let mut events = drag(right, Vec2::new(40.0, 0.0));
    let top = handle_screen(app, id, Handle::Top);
    events.extend(drag(top, Vec2::new(0.0, -40.0)));
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
        let msg = format!("Größe {:.1}×{:.1}, {} Bindungen, gesperrt: {:?}", hi[0] - lo[0], hi[1] - lo[1], own_distances(app, id).len(), app.edit.locked_axes(&app.shapes));
        if near(hi[0] - lo[0], 100.0, 1e-6) && near(hi[1] - lo[1], 50.0, 1e-6) && own_distances(app, id).len() == 2 && app.edit.locked_axes(&app.shapes) == (true, true) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

/// A fresh rectangle (centre 100/100, 20 × 10: lower-left corner 90/95) with the variables
/// `breite = 60` and `hoehe = 25`, its size fields open.
fn vars_typed(app: &mut App) -> (Vec<Event>, Check) {
    app.tool = DrawTool::Select;
    app.mode = super::ViewMode::TwoD;
    app.vars.set("breite", "60");
    app.vars.set("hoehe", "25");
    app.vars_dirty = true;
    app.refresh_vars();
    let id = app.add_shape(shape_edit::Rect::new([100.0, 100.0], 20.0, 10.0));
    app.edit.selection = vec![id];
    app.open_dims(id);
    app.selftest_rect = Some(id);
    // empty canvas below the material
    let click = app.to_screen([app.job.width / 2.0, -20.0]);
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
        let msg = format!("{} Constraints, Ecke {:.1}/{:.1}, Größe {:.1}×{:.1}, Felder offen: {} {:?} ({})", own_distances(app, id).len(), lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1], app.dims.is_some(), app.dims_texts(), app.status);
        if own_distances(app, id).len() == 2 && near(lo[0], 90.0, 1e-6) && near(lo[1], 95.0, 1e-6) && near(hi[0] - lo[0], 60.0, 1e-6) && near(hi[1] - lo[1], 25.0, 1e-6) && app.dims.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    // "br" + Tab takes "breite"; Tab goes on to the height; "ho" + Tab takes "hoehe"; a click
    // beside the shape takes both
    let events = vec![
        // two idle frames first, so the first field has the cursor before anything is typed
        Event::PointerMoved(click),
        Event::PointerMoved(click),
        Event::Text("br".into()),
        key(egui::Key::Tab),
        key(egui::Key::Tab),
        Event::Text("ho".into()),
        key(egui::Key::Tab),
        Event::PointerMoved(click),
        Event::PointerButton { pos: click, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: click, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    (events, check)
}

/// `hoehe` becomes 50 and `breite` a formula of it: the bound rectangle follows both, and its
/// lower-left corner does not move.
fn vars_changed(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("the previous stage made it");
    app.vars.set("hoehe", "50");
    app.vars.set("breite", "hoehe*2");
    app.vars_dirty = true;
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
        let msg = format!("Ecke {:.1}/{:.1}, Größe {:.1}×{:.1}", lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]);
        if near(lo[0], 90.0, 1e-6) && near(lo[1], 95.0, 1e-6) && near(hi[0] - lo[0], 100.0, 1e-6) && near(hi[1] - lo[1], 50.0, 1e-6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// Open the fields again and type a name that doesn't exist, then Enter: nothing is taken and the
/// fields stay open.
fn vars_invalid_stays_open(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    app.edit.selection = vec![id];
    app.open_dims(id);
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
        let msg = format!("Felder offen: {}, Breite {:.1}, Meldung: {}", app.dims.is_some(), hi[0] - lo[0], app.status);
        if app.dims.is_some() && near(hi[0] - lo[0], 100.0, 1e-6) && app.status.contains("nicht übernommen") {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let idle = app.to_screen([app.job.width / 2.0, -20.0]);
    (vec![Event::PointerMoved(idle), Event::PointerMoved(idle), Event::Text("nope".into()), key(egui::Key::Enter)], check)
}

/// `hoehe` is renamed to `h` the way the dialog does it (name changed, event queued): the shape's
/// binding and the variable `breite` that used it now say `h`, and a new value for `h` still
/// reaches the shape.
fn vars_renamed(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let i = app.vars.list.iter().position(|v| v.name == "hoehe").expect("made earlier");
    app.vars.list[i].name = "h".into();
    app.vars_committed = app.vars.list.iter().map(|v| v.name.clone()).collect();
    app.var_events.push(shape_edit::VarEvent::Renamed { from: "hoehe".into(), to: "h".into() });
    app.vars.set("h", "40");
    app.vars_dirty = true;
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
        let breite = app.vars.list.iter().find(|v| v.name == "breite").map(|v| v.expr.clone()).unwrap_or_default();
let exprs: Vec<String> = own_distances(app, id).iter().map(|c| c.expr.clone()).collect();
        let msg = format!("Constraints {exprs:?}, breite = {breite}, Größe {:.1}×{:.1}", hi[0] - lo[0], hi[1] - lo[1]);
        let uses_new = exprs.iter().any(|e| e == "h") && !exprs.iter().any(|e| e.contains("hoehe"));
        if uses_new && breite == "h*2" && near(hi[0] - lo[0], 80.0, 1e-6) && near(hi[1] - lo[1], 40.0, 1e-6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// Deleting `h` releases the sizes bound to it: they keep their value but are plain numbers.
fn vars_deleted(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let i = app.vars.list.iter().position(|v| v.name == "h").expect("made earlier");
    app.vars.list.remove(i);
    app.var_events.push(shape_edit::VarEvent::Removed(app.vars_committed.remove(i)));
    app.vars_dirty = true;
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
let heights = own_distances(app, id).iter().filter(|c| shape_edit::constraints::size_dim(it, c) == Some(Dim::Height)).count();
        let msg = format!("Höhen-Constraints {heights}, Größe {:.1}×{:.1}", hi[0] - lo[0], hi[1] - lo[1]);
        // height was held by h only: free now; width by breite (= h*2, now an error): still there
        if heights == 0 && near(hi[1] - lo[1], 40.0, 1e-6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// "Maße anzeigen" on, a fresh rectangle selected: its two values are drawn as clickable labels.
fn dims_shown(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.add_shape(shape_edit::Rect::new([150.0, 60.0], 30.0, 20.0));
    app.selftest_rect = Some(id);
    app.tool = DrawTool::Select;
    app.edit.selection = vec![id];
    app.show_dims = true;
    let check: Check = Box::new(move |app| {
        let n = app.dim_labels.iter().filter(|(i, _, _)| *i == id).count();
        let msg = format!("{n} Werte gezeichnet");
        if n == 2 {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// Hold on the drawn width: the fields open with the cursor in it, the typed value is taken by Enter.
fn dims_label_click(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let Some((_, _, r)) = app.dim_labels.iter().find(|(i, d, _)| *i == id && *d == Dim::Width).cloned() else { return (vec![], Box::new(|_| Err("kein Wertfeld gezeichnet".into()))) };
    let at = r.center();
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
        let msg = format!("Breite {:.2}, Höhe {:.2}, Felder offen: {}", hi[0] - lo[0], hi[1] - lo[1], app.dims.is_some());
        if near(hi[0] - lo[0], 45.0, 1e-6) && near(hi[1] - lo[1], 20.0, 1e-6) && app.dims.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let mut events = vec![Event::PointerMoved(at), Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() }];
    // held still for about two thirds of a second (the clock runs 60 frames a second here)
    events.extend((0..40).map(|_| Event::PointerMoved(at)));
    events.extend([Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() }, Event::Text("45".into()), key(egui::Key::Enter)]);
    (events, check)
}

/// A quick click on the drawn width opens nothing: a value is only changed by holding on it.
fn dims_label_quick_click(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let Some((_, _, r)) = app.dim_labels.iter().find(|(i, d, _)| *i == id && *d == Dim::Width).cloned() else { return (vec![], Box::new(|_| Err("kein Wertfeld gezeichnet".into()))) };
    let at = r.center();
    let check: Check = Box::new(|app| {
        let msg = format!("Felder offen: {}", app.dims.is_some());
        if app.dims.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let events = vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    (events, check)
}

/// The width typed in the previous stage is now a constraint of that edge (bound as text "45").
fn typed_size_is_a_constraint(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let own: Vec<(Option<Dim>, String)> = own_distances(app, id).iter().map(|c| (shape_edit::constraints::size_dim(it, c), c.expr.clone())).collect();
        let msg = format!("Constraints {own:?}, Bindungen {:?}", it.bound);
        let width = own.iter().any(|(d, e)| *d == Some(Dim::Width) && e == "45");
        let height = own.iter().any(|(d, _)| *d == Some(Dim::Height));
        if width && !height {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// Click the shown width (it is a distance constraint now) and press Delete: the constraint goes, the
/// width stays.
fn delete_length_constraint(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return (vec![], Box::new(|_| Err("Rechteck fehlt".into()))) };
    let index = app.distances.iter().position(|c| shape_edit::constraints::size_dim(it, c) == Some(Dim::Width));
    let Some((_, r)) = index.and_then(|i| app.distance_labels.iter().find(|(n, _)| *n == i).cloned()) else { return (vec![], Box::new(|_| Err("kein Wertfeld gezeichnet".into()))) };
    let at = r.center();
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
        let widths = own_distances(app, id).iter().filter(|c| shape_edit::constraints::size_dim(it, c) == Some(Dim::Width)).count();
        let msg = format!("Breiten-Constraints {widths}, Breite {:.2}, Auswahl {}", hi[0] - lo[0], app.selected_constraint.is_some());
        if widths == 0 && near(hi[0] - lo[0], 45.0, 1e-6) && app.selected_constraint.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let events = vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
        Event::PointerMoved(Pos2::new(5.0, 5.0)),
        key(egui::Key::Delete),
    ];
    (events, check)
}

/// A new rectangle's fields taken over with Enter and nothing typed: plain values, no constraint.
fn accept_untouched_values(app: &mut App) -> (Vec<Event>, Check) {
    app.dim_mode = None;
    app.tool = DrawTool::Select;
    let id = app.add_shape(shape_edit::Rect::new([60.0, 170.0], 30.0, 15.0));
    app.edit.selection = vec![id];
    app.open_dims(id);
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("Bindungen {:?}, Felder offen: {}", it.bound, app.dims.is_some());
        if it.bound.is_empty() && app.dims.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![Event::PointerMoved(Pos2::new(5.0, 5.0)), Event::PointerMoved(Pos2::new(5.0, 5.0)), key(egui::Key::Enter)], check)
}

/// Press on the drawn distance and drag it 40 px down: the dimension line moves with it and the
/// value does not open.
fn distance_label_drag(app: &mut App) -> (Vec<Event>, Check) {
    let Some((_, r)) = app.distance_labels.first().cloned() else { return (vec![], Box::new(|_| Err("kein Abstand gezeichnet".into()))) };
    let before = app.distances[0].place;
    let at = r.center();
    let check: Check = Box::new(move |app| {
        let now = app.distances[0].place;
        let msg = format!("Platz vorher {:.1}/{:.1}, jetzt {:.1}/{:.1}, Feld offen: {}", before[0], before[1], now[0], now[1], app.distance_edit.is_some());
        if (now[1] - before[1]).abs() > 5.0 && app.distance_edit.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(at, Vec2::new(0.0, 40.0)), check)
}

/// With the switch on and no shape chosen the dimension shows as a faint line without its value;
/// dragging the line moves the dimension just the same.
fn distance_line_drag(app: &mut App) -> (Vec<Event>, Check) {
    // the middle of the first dimension line on the screen
    let mut items = app.shapes.clone();
    items.push(shape_edit::constraints::plate_item(app.job.width, app.job.height));
    let Some(c) = app.distances.first().cloned() else { return (vec![], Box::new(|_| Err("kein Abstand vorhanden".into()))) };
    let Some(m) = c.measure(&items) else { return (vec![], Box::new(|_| Err("Abstand nicht messbar".into()))) };
    let n = [-m.dir[1], m.dir[0]];
    let place = c.place_world(&m);
    let off = |p: shape_edit::Pt| {
        let t = (place[0] - p[0]) * n[0] + (place[1] - p[1]) * n[1];
        [p[0] + n[0] * t, p[1] + n[1] * t]
    };
    let (ta, tb) = (app.to_screen(off(m.a)), app.to_screen(off(m.b)));
    let middle = ta + (tb - ta) / 2.0;
    app.cancel_dim_mode();
    app.cancel_align_mode();
    app.tool = DrawTool::Select;
    app.show_dims = true;
    app.edit.selection.clear();
    let places_before: Vec<[f64; 2]> = app.distances.iter().map(|c| c.place).collect();
    let shapes_before: Vec<_> = app.shapes.iter().map(|s| s.bounds()).collect();
    let check: Check = Box::new(move |app| {
        let moved = app.distances.iter().zip(&places_before).filter(|(c, b)| (c.place[1] - b[1]).abs() > 5.0).count();
        let shapes_now: Vec<_> = app.shapes.iter().map(|s| s.bounds()).collect();
        let msg = format!("{moved} Maß(e) verschoben, Formen blieben: {}, Linien ohne Wert: {}", shapes_now == shapes_before, app.distance_lines.len());
        // only a dimension moved, not a shape it hangs on
        if moved >= 1 && shapes_now == shapes_before && app.distance_edit.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    // two idle frames first, so the dimension is drawn without its value when the drag starts
    let idle = middle + Vec2::new(0.0, 160.0);
    let mut events = vec![Event::PointerMoved(idle), Event::PointerMoved(idle + Vec2::new(1.0, 0.0))];
    events.extend(drag(middle, Vec2::new(0.0, 40.0)));
    (events, check)
}

/// A chosen shape shows its size as a dimension line; pulling the line (or its value) moves the
/// line away from the shape, the shape stays.
fn shape_dim_line_drag(app: &mut App) -> (Vec<Event>, Check) {
    app.cancel_dim_mode();
    app.cancel_align_mode();
    app.tool = DrawTool::Select;
    app.show_dims = true;
    let id = app.add_shape(shape_edit::Circle::new([60.0, 40.0], 12.0));
    app.edit.selection = vec![id];
    let Some(line) = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.shape.decoration().into_iter().next()) else { return (vec![], Box::new(|_| Err("keine Maßlinie".into()))) };
    let out = app.dim_out(&line);
    let (a, b) = (app.to_screen(line.from), app.to_screen(line.to));
    // the line itself, a little beside its middle (the value sits on the middle)
    let on_line = a + (b - a) * 0.2 + out * 16.0;
    let bounds_before = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds());
    let check: Check = Box::new(move |app| {
        let it = app.shapes.iter().find(|s| s.id == id);
        let pulled = it.map_or(0.0, |s| s.dim_offsets.iter().map(|(_, e)| *e).sum::<f64>());
        let same = it.and_then(|s| s.bounds()) == bounds_before;
        let msg = format!("Maßlinie um {pulled:.1} mm herausgezogen, Form blieb: {same}");
        if pulled > 3.0 && same {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let idle = on_line + Vec2::new(0.0, 160.0);
    let mut events = vec![Event::PointerMoved(idle), Event::PointerMoved(idle + Vec2::new(1.0, 0.0))];
    events.extend(drag(on_line, out * 40.0));
    (events, check)
}

/// With the speed slider in its middle (1×) the simulation takes as long as the milling does: the
/// seconds the screen gives it add up to the machine time of the toolpaths, shown in the list.
fn simulation_in_real_time(app: &mut App) -> (Vec<Event>, Check) {
    app.sim_rate = 1.0;
    app.preview_scope = super::PreviewScope::Visible;
    app.start_preview(true);
    let expected: f64 = app.preview_indices().iter().map(|&i| app.toolpaths[i].stats().minutes * 60.0).sum();
    let (mut screen_seconds, dt) = (0.0, 0.2);
    while app.sim.as_ref().is_some_and(|s| s.anim.is_some()) && screen_seconds < 1.0e5 {
        app.step_animation(dt);
        screen_seconds += dt;
    }
    app.sim = None;
    let check: Check = Box::new(move |_| {
        let msg = format!("Bildschirmzeit {screen_seconds:.1} s, Fräszeit {expected:.1} s");
        if expected > 10.0 && (screen_seconds - expected).abs() <= 2.0 * dt {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// Two fresh rectangles; D, a click on the first one's upper-right corner and on the second one's
/// lower-left corner, a click above them (horizontal), 25 and Enter.
fn distance_by_mouse(app: &mut App) -> (Vec<Event>, Check) {
    // D from another tool goes back to selecting
    app.tool = DrawTool::Rect;
    app.show_dims = true;
    let a = app.add_shape(shape_edit::Rect::new([230.0, 150.0], 20.0, 20.0));
    let b = app.add_shape(shape_edit::Rect::new([270.0, 150.0], 20.0, 20.0));
    app.selftest_pair = Some((a, b));
    app.edit.selection.clear();
    let click = |p: Pos2| {
        [Event::PointerMoved(p), Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() }, Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() }]
    };
    let mut events = vec![key(egui::Key::D)];
    events.extend(click(app.to_screen([240.0, 160.0])));
    events.extend(click(app.to_screen([260.0, 140.0])));
    // above, between the two x: horizontal
    events.push(Event::PointerMoved(app.to_screen([250.0, 185.0])));
    events.extend(click(app.to_screen([250.0, 185.0])));
    events.push(Event::Text("25".into()));
    events.push(key(egui::Key::Enter));
    let check: Check = Box::new(move |app| {
        let ll = |id: u64| app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()).map(|(lo, _)| lo);
        let ur = |id: u64| app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()).map(|(_, hi)| hi);
        let (Some(la), Some(ua), Some(lb)) = (ll(a), ur(a), ll(b)) else { return Err("Rechtecke fehlen".into()) };
        let msg = format!("{} Abstand(e), Lücke {:.2}, Höhe unten gleich: {}, Modus an: {}, Werkzeug Auswählen: {}", app.distances.len(), lb[0] - ua[0], near(lb[1], la[1], 1e-9), app.dim_mode.is_some(), app.tool == DrawTool::Select);
        if app.tool == DrawTool::Select && app.distances.iter().filter(|c| c.from.shape() != c.to.shape()).count() == 1 && near(lb[0] - ua[0], 25.0, 1e-9) && near(la[0], 220.0, 1e-9) && near(lb[1], 140.0, 1e-9) && app.distance_edit.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

/// The first rectangle is moved by 10: the second keeps its 25 to it.
fn distance_follows(app: &mut App) -> (Vec<Event>, Check) {
    let (a, b) = app.selftest_pair.expect("made earlier");
    // where the dimension line of the A-to-B distance is now, in the world
    let world_place = move |app: &App| {
        let mut items = app.shapes.clone();
        items.push(shape_edit::constraints::plate_item(app.job.width, app.job.height));
        let c = app.distances.iter().find(|c| c.from.shape() == a && c.to.shape() == b)?;
        let m = c.measure(&items)?;
        Some(c.place_world(&m))
    };
    let before = world_place(app);
    if let Some(it) = app.shapes.iter_mut().find(|s| s.id == a) {
        *it = it.translated(10.0, 0.0);
    }
    let check: Check = Box::new(move |app| {
        let ll = |id: u64| app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()).map(|(lo, _)| lo);
        let ur = |id: u64| app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()).map(|(_, hi)| hi);
        let (Some(ua), Some(lb)) = (ur(a), ll(b)) else { return Err("Rechtecke fehlen".into()) };
        // the dimension line went along with the shapes (10 mm right), it didn't stay where it was
        let after = world_place(app);
        let along = matches!((before, after), (Some(p), Some(q)) if near(q[0] - p[0], 10.0, 1e-6) && near(q[1], p[1], 1e-6));
        let msg = format!("Lücke {:.2}, zweite Form links unten bei {:.1}, Maßlinie {before:?} → {after:?}", lb[0] - ua[0], lb[0]);
        if near(lb[0] - ua[0], 25.0, 1e-9) && near(lb[0], 275.0, 1e-9) && along {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// The first rectangle's lower-left corner and the plate's left edge: 200 from the plate (the plate
/// is the reference however the two are picked).
fn distance_to_plate(app: &mut App) -> (Vec<Event>, Check) {
    let (a, _) = app.selftest_pair.expect("made earlier");
    let click = |p: Pos2| {
        [Event::PointerMoved(p), Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() }, Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() }]
    };
    // a's lower-left corner (it was moved by 10 before); the plate's left edge: x 0, y 100
    let mut events = Vec::new();
    events.extend(click(app.to_screen([230.0, 140.0])));
    events.extend(click(app.to_screen([0.0, 100.0])));
    events.push(Event::PointerMoved(app.to_screen([100.0, 160.0])));
    events.extend(click(app.to_screen([100.0, 160.0])));
    events.push(Event::Text("200".into()));
    events.push(key(egui::Key::Enter));
    let check: Check = Box::new(move |app| {
        let Some((lo, _)) = app.shapes.iter().find(|s| s.id == a).and_then(|s| s.bounds()) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("{} Abstände, linke Seite bei x = {:.2}", app.distances.len(), lo[0]);
        if app.distances.iter().filter(|c| c.from.shape() != c.to.shape()).count() == 2 && near(lo[0], 200.0, 1e-9) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

/// L, then C, then R: the last one chosen is the rectangle tool.
fn tool_keys(app: &mut App) -> (Vec<Event>, Check) {
    let check: Check = Box::new(|app| {
        // the rectangle tool draws its own cursor over the drawing
        let drawn = app.custom_cursor_badge.is_some();
        let msg = format!("Werkzeug: {}, eigener Cursor: {drawn}", app.tool.tip());
        if app.tool == DrawTool::Rect && drawn {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    // over the middle of the material, so the cursor is drawn
    let at = app.to_screen([app.job.width / 2.0, app.job.height / 2.0]);
    (vec![key(egui::Key::L), key(egui::Key::C), key(egui::Key::R), Event::PointerMoved(at), Event::PointerMoved(at)], check)
}

fn tool_keys_back(_app: &mut App) -> (Vec<Event>, Check) {
    let check: Check = Box::new(|app| {
        let msg = format!("Werkzeug: {}", app.tool.tip());
        if app.tool == DrawTool::Select {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![key(egui::Key::S)], check)
}

/// A fresh rectangle: its lower-left and lower-right corner, placed above, 50 typed: the width
/// grows from the first corner.
fn distance_same_shape(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.add_shape(shape_edit::Rect::new([150.0, 30.0], 20.0, 10.0));
    app.selftest_rect = Some(id);
    let click = |p: Pos2| {
        [Event::PointerMoved(p), Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() }, Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() }]
    };
    let mut events = Vec::new();
    events.extend(click(app.to_screen([140.0, 25.0])));
    events.extend(click(app.to_screen([160.0, 25.0])));
    events.push(Event::PointerMoved(app.to_screen([150.0, 50.0])));
    events.extend(click(app.to_screen([150.0, 50.0])));
    events.push(Event::Text("50".into()));
    events.push(key(egui::Key::Enter));
    let check: Check = Box::new(move |app| {
        let Some((lo, hi)) = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("{} Abstände, Breite {:.2}, Höhe {:.2}, links bei {:.2}", app.distances.len(), hi[0] - lo[0], hi[1] - lo[1], lo[0]);
        if near(hi[0] - lo[0], 50.0, 1e-9) && near(hi[1] - lo[1], 10.0, 1e-9) && near(lo[0], 140.0, 1e-9) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

/// The same rectangle's bottom side, then an empty place: that side's length, 60 typed.
fn distance_line_length(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let click = |p: Pos2| {
        [Event::PointerMoved(p), Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() }, Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() }]
    };
    let mut events = Vec::new();
    events.extend(click(app.to_screen([150.0, 25.0])));
    // an empty place above the prompt bar at the bottom edge (which would catch the click)
    let mut items = app.shapes.clone();
    items.push(shape_edit::constraints::plate_item(app.job.width, app.job.height));
    let tol = 11.0 / app.zoom as f64;
    let empty = (0..40).flat_map(|r| (0..24).map(move |c| [20.0 + c as f64 * 11.0, 15.0 + r as f64 * 4.5])).find(|p| shape_edit::constraints::pick(&items, *p, tol).is_none() && app.to_screen(*p).y < 678.0);
    let Some(empty) = empty else {
        let why: Vec<String> = [(150.0, 15.0), (120.0, 15.0)].iter().map(|(x, y)| format!("{:?} -> {:?} y {:.0}", [*x, *y], shape_edit::constraints::pick(&items, [*x, *y], tol), app.to_screen([*x, *y]).y)).collect();
        return (vec![], Box::new(move |_| Err(format!("kein freier Platz: {why:?}, zoom {}", 1))));
    };
    events.extend(click(app.to_screen(empty)));
    events.push(Event::Text("60".into()));
    events.push(key(egui::Key::Enter));
    let check: Check = Box::new(move |app| {
        let Some((lo, hi)) = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("{} Abstände {:?}, Breite {:.2}, Status: {}, Feld: {}", app.distances.len(), app.distances.iter().map(|c| (c.from, c.to, c.expr.clone())).collect::<Vec<_>>(), hi[0] - lo[0], app.status, app.distance_edit.is_some());
        if near(hi[0] - lo[0], 60.0, 1e-9) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

/// A rectangle 260 from the plate's left edge (the edge and its lower-left corner): dragging it up
/// by 20 mm must work, only the distance sideways is held.
fn plate_constraint_drag_vertical(app: &mut App) -> (Vec<Event>, Check) {
    // the dimension mode is on, as it stays after a distance was set: a drag must still move shapes
    app.dim_mode = None;
    app.distance_edit = None;
    app.toggle_dim_mode();
    app.show_dims = false;
    let id = app.add_shape(shape_edit::Rect::new([270.0, 60.0], 20.0, 20.0));
    app.selftest_rect = Some(id);
    app.edit.selection.clear();
    app.distances.push(shape_edit::Distance { from: shape_edit::Anchor::Edge(shape_edit::constraints::PLATE, 3), to: shape_edit::Anchor::Point(id, 0), axis: shape_edit::Axis::Aligned, expr: "260".into(), place: [130.0, 30.0], offset: None });
    let top = app.to_screen([270.0, 70.0]);
    let check: Check = Box::new(move |app| {
        let Some((lo, hi)) = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("unten links {:.2}/{:.2}, Größe {:.2}×{:.2}", lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]);
        if near(lo[0], 260.0, 1e-6) && near(lo[1], 70.0, 0.5) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(top, Vec2::new(0.0, -20.0 * app.zoom)), check)
}

/// The same rectangle: its top handle pulled up by 10 mm makes it taller.
fn plate_constraint_resize_height(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    app.edit.selection = vec![id];
    let top = handle_screen(app, id, Handle::Top);
    let check: Check = Box::new(move |app| {
        let Some((lo, hi)) = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("unten links {:.2}/{:.2}, Größe {:.2}×{:.2}", lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]);
        if near(lo[0], 260.0, 1e-6) && near(hi[1] - lo[1], 30.0, 0.5) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(top, Vec2::new(0.0, -10.0 * app.zoom)), check)
}

/// In the dimension mode, with Shift held (a capital S): back to the select tool, mode over.
fn capital_s_leaves_dimension_mode(app: &mut App) -> (Vec<Event>, Check) {
    app.dim_mode = None;
    app.toggle_dim_mode();
    app.distance_edit = None;
    let check: Check = Box::new(|app| {
        let msg = format!("Modus an: {}, Werkzeug: {}", app.dim_mode.is_some(), app.tool.tip());
        if app.dim_mode.is_none() && app.tool == DrawTool::Select {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![Event::Key { key: egui::Key::S, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::SHIFT }], check)
}

/// "Neue Variable": the dialog opens with the name selected, so typing replaces it; the expression
/// is changed, OK (Enter) puts the variable into the document.
fn new_variable_focus(app: &mut App) -> (Vec<Event>, Check) {
    app.vars_open = true;
    app.open_new_variable();
    let check: Check = Box::new(|app| {
        let last = app.vars.list.last().map(|v| (v.name.clone(), v.expr.clone()));
        let msg = format!("letzte Variable {last:?}, Dialog offen: {}", app.new_var.is_some());
        if last == Some(("neuername".to_string(), "10".to_string())) && app.new_var.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    // a few quiet frames for the dialog to be drawn and take the cursor, then the name is typed
    let mut events: Vec<Event> = (0..6).map(|_| Event::PointerMoved(Pos2::new(5.0, 5.0))).collect();
    events.push(Event::Text("neuername".into()));
    events.push(key(egui::Key::Enter));
    (events, check)
}

/// The expression of a new variable completes the names that exist: "pl" offers platte_b, platte_l
/// and platte_d; Enter takes the first, the next Enter is OK.
fn new_variable_autocomplete(app: &mut App) -> (Vec<Event>, Check) {
    app.vars_open = true;
    app.open_new_variable();
    let quiet = |n: usize| (0..n).map(|_| Event::PointerMoved(Pos2::new(5.0, 5.0)));
    let mut events: Vec<Event> = quiet(6).collect();
    events.push(Event::Text("kurz".into()));
    events.push(key(egui::Key::Tab));
    events.extend(quiet(4));
    events.push(Event::Text("pl".into()));
    events.extend(quiet(4));
    events.push(key(egui::Key::Enter));
    events.extend(quiet(4));
    events.push(key(egui::Key::Enter));
    let check: Check = Box::new(|app| {
        let last = app.vars.list.last().map(|v| (v.name.clone(), v.expr.clone()));
        let nv = app.new_var.as_ref().map(|n| (n.name.clone(), n.expr.clone(), n.input.popup.len(), n.input.sel));
        let msg = format!("letzte Variable {last:?}, Dialog {nv:?}");
        if last == Some(("kurz".to_string(), "platte_b".to_string())) && app.new_var.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

/// The value field of a distance constraint completes variables, as the size fields do: "plat" and
/// Enter take `platte_b`, the next Enter sets the constraint; then the same with Tab.
fn distance_value_autocomplete(app: &mut App) -> (Vec<Event>, Check) {
    let (a, b) = app.selftest_pair.expect("made earlier");
    let before = app.distances.len();
    app.dim_mode = None;
    let draft = shape_edit::Distance { from: shape_edit::Anchor::Point(a, 1), to: shape_edit::Anchor::Point(b, 0), axis: shape_edit::Axis::Horizontal, expr: String::new(), place: [250.0, 190.0], offset: None };
    app.open_distance_edit(draft, "");
    let quiet = |n: usize| (0..n).map(|_| Event::PointerMoved(Pos2::new(5.0, 5.0)));
    let mut events: Vec<Event> = quiet(6).collect();
    events.push(Event::Text("plat".into()));
    events.extend(quiet(4));
    events.push(key(egui::Key::Enter));
    events.extend(quiet(4));
    events.push(key(egui::Key::Enter));
    let check: Check = Box::new(move |app| {
        let exprs: Vec<String> = app.distances.iter().map(|c| c.expr.clone()).collect();
        let msg = format!("{} Abstände (vorher {before}), Ausdrücke {exprs:?}, Feld offen: {}", app.distances.len(), app.distance_edit.is_some());
        if app.distances.iter().any(|c| c.expr == "platte_b") && app.distance_edit.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

/// A vector with a slightly leaning first segment: H, a click on that segment: its second node moves
/// until the segment lies level (the first node stays).
fn align_line(app: &mut App) -> (Vec<Event>, Check) {
    app.dim_mode = None;
    app.tool = DrawTool::Select;
    app.edit.selection.clear();
    let id = app.add_shape(shape_edit::Path::new(vec![[100.0, 160.0], [160.0, 168.0], [160.0, 176.0]], false));
    app.selftest_rect = Some(id);
    let on_segment = app.to_screen([115.0, 162.0]);
    let check: Check = Box::new(move |app| {
        let nodes = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.nodes()).unwrap_or_default();
        let msg = format!("{} Waagrecht/Senkrecht, Knoten {nodes:?}, Status: {}, Modus: {:?}, pick {:?}, s13 {:?}", app.aligns.len(), app.status, app.align_mode.is_some(), { let mut items = app.shapes.clone(); items.push(shape_edit::constraints::plate_item(app.job.width, app.job.height)); shape_edit::constraints::pick(&items, [45.0, 162.0], 4.0) }, app.shapes.iter().find(|s| s.id == 13).map(|s| (s.shape.label(), s.bounds())));
        if app.aligns.len() == 1 && nodes.len() == 3 && near(nodes[1][1], nodes[0][1], 1e-6) && near(nodes[0][1], 160.0, 1e-6) && near(nodes[1][0], 160.0, 1e-6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let events = vec![
        key(egui::Key::H),
        Event::PointerMoved(on_segment),
        Event::PointerMoved(on_segment),
        Event::PointerButton { pos: on_segment, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: on_segment, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    (events, check)
}

/// Click the symbol (it is chosen) and press Delete: the constraint goes, the segment stays level.
fn align_delete(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let Some((_, r)) = app.align_labels.first().cloned() else { return (vec![], Box::new(|_| Err("kein Symbol gezeichnet".into()))) };
    let at = r.center();
    let check: Check = Box::new(move |app| {
        let nodes = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.nodes()).unwrap_or_default();
        let msg = format!("{} Waagrecht/Senkrecht, Knoten {nodes:?}", app.aligns.len());
        if app.aligns.is_empty() && nodes.len() == 3 && near(nodes[1][1], nodes[0][1], 1e-6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let events = vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
        Event::PointerMoved(Pos2::new(5.0, 5.0)),
        key(egui::Key::Delete),
    ];
    (events, check)
}

/// Two points of two shapes (the corners of two new rectangles): level, the second shape moves up
/// or down, not sideways.
fn align_two_points(app: &mut App) -> (Vec<Event>, Check) {
    let a = app.add_shape(shape_edit::Rect::new([40.0, 130.0], 20.0, 20.0));
    let b = app.add_shape(shape_edit::Rect::new([90.0, 140.0], 20.0, 20.0));
    // A's lower-left corner (30/120) and B's lower-left corner (80/130)
    let click = |p: Pos2| {
        [Event::PointerMoved(p), Event::PointerMoved(p), Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() }, Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() }]
    };
    let mut events = Vec::new();
    events.extend(click(app.to_screen([30.0, 120.0])));
    events.extend(click(app.to_screen([80.0, 130.0])));
    let check: Check = Box::new(move |app| {
        let ll = |id: u64| app.shapes.iter().find(|s| s.id == id).and_then(|s| s.bounds()).map(|(lo, _)| lo);
        let (Some(la), Some(lb)) = (ll(a), ll(b)) else { return Err("Rechtecke fehlen".into()) };
        let msg = format!("{} Waagrecht/Senkrecht, A unten links {:.1}/{:.1}, B {:.1}/{:.1}", app.aligns.len(), la[0], la[1], lb[0], lb[1]);
        if app.aligns.len() == 1 && near(la[1], lb[1], 1e-6) && near(lb[0], 80.0, 1e-6) && near(la[1], 120.0, 1e-6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

/// Drag the first toolpath's grip below the last row: it becomes the last one, the others move up.
fn toolpath_list_drag(app: &mut App) -> (Vec<Event>, Check) {
    app.vars_open = false;
    let before: Vec<String> = app.toolpaths.iter().map(|t| t.name.clone()).collect();
    let Some(first) = app.tp_grips.first().cloned() else { return (vec![], Box::new(|_| Err("keine Liste gezeichnet".into()))) };
    let last = app.tp_grips.last().cloned().unwrap_or(first);
    let check: Check = Box::new(move |app| {
        let now: Vec<String> = app.toolpaths.iter().map(|t| t.name.clone()).collect();
        let mut want = before.clone();
        let moved = want.remove(0);
        want.push(moved);
        let msg = format!("Reihenfolge {now:?}");
        if now == want {
            Ok(msg)
        } else {
            Err(format!("{msg}, erwartet {want:?}"))
        }
    });
    let by = Vec2::new(0.0, last.center().y - first.center().y + 30.0);
    (drag(first.center(), by), check)
}

/// H again: the mode is over.
fn align_mode_off(_app: &mut App) -> (Vec<Event>, Check) {
    let check: Check = Box::new(|app| {
        let msg = format!("Modus an: {}", app.align_mode.is_some());
        if app.align_mode.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![key(egui::Key::H)], check)
}

/// D again: the mode is over and clicks select once more.
fn distance_mode_off(app: &mut App) -> (Vec<Event>, Check) {
    let check: Check = Box::new(|app| {
        let msg = format!("Modus an: {}", app.dim_mode.is_some());
        if app.dim_mode.is_none() {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let _ = app;
    (vec![key(egui::Key::D)], check)
}

fn vars_escape(app: &mut App) -> (Vec<Event>, Check) {
    let id = app.selftest_rect.expect("made earlier");
    let check: Check = Box::new(move |app| {
        let Some(it) = app.shapes.iter().find(|s| s.id == id) else { return Err("Rechteck fehlt".into()) };
        let (lo, hi) = it.bounds().unwrap_or_default();
        let msg = format!("Felder offen: {}, Breite {:.1}", app.dims.is_some(), hi[0] - lo[0]);
        if app.dims.is_none() && near(hi[0] - lo[0], 100.0, 1e-6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![key(egui::Key::Escape)], check)
}

/// Drag the rectangle's corner-radius grip (its "selected decoration") towards its middle.
fn corner_radius_grip(app: &mut App) -> (Vec<Event>, Check) {
    let Some((id, r)) = new_rect(app) else { return (vec![], Box::new(|_| Err("kein Rechteck".into()))) };
    app.tool = DrawTool::Select;
    app.edit.selection = vec![id];
    app.edit.rotate_mode = false;
    let Some(g) = app.edit.grips(&app.shapes).first().copied() else { return (vec![], Box::new(|_| Err("kein Griff".into()))) };
    let from = app.to_screen(g.at);
    let by = (app.to_screen(r.center) - from).normalized() * 20.0;
    let want = match Shape::Rect(r.clone()).with_grip(g.kind, app.to_mm(from + by)) {
        Shape::Rect(w) => w.corner_radius,
        _ => 0.0,
    };
    let check: Check = Box::new(move |app| {
        let Some(Shape::Rect(r2)) = app.shapes.iter().find(|s| s.id == id).map(|s| &s.shape) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("Eckenradius {:.2} mm (erwartet {:.2}), Größe {:.1}×{:.1}", r2.corner_radius, want, r2.width, r2.height);
        if want > 1.0 && near(r2.corner_radius, want, 0.6) && r2.width == r.width && r2.height == r.height {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(from, by), check)
}

/// With the rectangle selected, press N: node editing, and the rectangle becomes a plain vector.
fn node_key(app: &mut App) -> (Vec<Event>, Check) {
    let Some(id) = app.shapes.iter().rev().find(|s| matches!(s.shape, Shape::Rect(_))).map(|s| s.id) else {
        return (vec![], Box::new(|_| Err("kein Rechteck".into())));
    };
    app.tool = DrawTool::Select;
    app.edit.selection = vec![id];
    let check: Check = Box::new(move |app| {
        let nodes = app.shapes.iter().find(|s| s.id == id).and_then(|s| s.nodes()).map_or(0, |n| n.len());
        let msg = format!("Werkzeug {:?}, {} Knoten", app.tool, nodes);
        if app.tool == DrawTool::NodeEdit && nodes >= 4 {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![key(egui::Key::N)], check)
}

/// Drag the ellipse with ⌃ held: the original stays, a moved copy is made and selected.
fn ctrl_copy(app: &mut App) -> (Vec<Event>, Check) {
    let ell = find(app, |s| matches!(s, Shape::Ellipse(_))).expect("example has an ellipse");
    app.tool = DrawTool::Select;
    app.edit.selection.clear();
    let c = ell.shape.center();
    let px = 50.0;
    let mm = px as f64 / app.zoom as f64;
    let before = app.shapes.iter().filter(|s| matches!(s.shape, Shape::Ellipse(_))).count();
    let check: Check = Box::new(move |app| {
        let centres: Vec<Pt> = app.shapes.iter().filter(|s| matches!(s.shape, Shape::Ellipse(_))).map(|s| s.shape.center()).collect();
        let kept = centres.iter().any(|p| near(p[0], c[0], 1e-6) && near(p[1], c[1], 1e-6));
        let copied = centres.iter().any(|p| near(p[0], c[0] + mm, 0.6) && near(p[1], c[1], 0.01));
        let sel_new = app.edit.selection.len() == 1 && app.edit.selection[0] != ell.id;
        let msg = format!("{} → {} Ellipsen, Original geblieben: {kept}, Kopie verschoben: {copied}, Kopie ausgewählt: {sel_new}", before, centres.len());
        if centres.len() == before + 1 && kept && copied && sel_new {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let grab = app.to_screen([c[0] - 55.0, c[1]]);
    (drag(grab, Vec2::new(px, 0.0)), check)
}

/// The distances that sit on one shape (its own sizes), for the tests.
fn own_distances(app: &App, id: u64) -> Vec<&shape_edit::Distance> {
    app.distances.iter().filter(|c| c.from.shape() == id && c.to.shape() == id).collect()
}

fn key(k: egui::Key) -> Event {
    Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::default() }
}

fn new_rect(app: &App) -> Option<(u64, shape_edit::Rect)> {
    app.shapes.iter().rev().find_map(|s| match &s.shape {
        Shape::Rect(r) if r.corner_radius == 0.0 => Some((s.id, r.clone())),
        _ => None,
    })
}

/// The width field of the rectangle just drawn has the cursor: type a formula and press Enter.
fn type_width(_app: &mut App) -> (Vec<Event>, Check) {
    let check: Check = Box::new(|app| {
        let Some((_, r)) = new_rect(app) else { return Err("kein Rechteck".into()) };
        let msg = format!("Breite {:.2} mm, Höhe {:.2} mm", r.width, r.height);
        if (r.width - 75.0).abs() < 1e-9 {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![Event::Text("150/2".into()), key(egui::Key::Enter)], check)
}

/// Open the rectangle's size fields again: typing replaces the selected width, Tab goes to the
/// height with its value selected, and a click beside the shape takes both, like Enter, without
/// changing the selection.
fn tab_then_click(app: &mut App) -> (Vec<Event>, Check) {
    let Some((id, _)) = new_rect(app) else { return (vec![], Box::new(|_| Err("kein Rechteck".into()))) };
    app.tool = DrawTool::Select;
    app.edit.selection = vec![id];
    app.open_dims(id);
    // empty canvas below the material
    let click = app.to_screen([app.job.width / 2.0, -20.0]);
    let check: Check = Box::new(move |app| {
        let Some(Shape::Rect(r)) = app.shapes.iter().find(|s| s.id == id).map(|s| &s.shape) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("Breite {:.2}, Höhe {:.2}, Felder offen: {}, ausgewählt: {}", r.width, r.height, app.dims.is_some(), app.edit.selection == vec![id]);
        if r.width == 50.0 && r.height == 20.0 && app.dims.is_none() && app.edit.selection == vec![id] {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    let events = vec![
        Event::Text("50".into()),
        key(egui::Key::Tab),
        Event::Text("20".into()),
        Event::PointerMoved(click),
        Event::PointerButton { pos: click, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: click, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    (events, check)
}

/// Clicking the selected rectangle once more used to start the rotate mode; rotating shapes is not
/// possible for now (it comes back later as an angle constraint), so nothing turns.
fn no_rotation(app: &mut App) -> (Vec<Event>, Check) {
    let Some((id, _)) = new_rect(app) else { return (vec![], Box::new(|_| Err("kein Rechteck".into()))) };
    app.tool = DrawTool::Select;
    app.edit.selection = vec![id];
    app.edit.rotate_mode = false;
    let (lo, hi) = bounds_of(app, id);
    // a click on the bottom edge, away from its handle: twice
    let on_edge = app.to_screen([lo[0] + (hi[0] - lo[0]) * 0.3, lo[1]]);
    let click = |p: Pos2| [Event::PointerMoved(p), Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() }, Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() }];
    let mut events = Vec::new();
    events.extend(click(on_edge));
    events.extend((0..10).map(|_| Event::PointerMoved(on_edge)));
    events.extend(click(on_edge));
    let check: Check = Box::new(move |app| {
        let Some(Shape::Rect(r)) = app.shapes.iter().find(|s| s.id == id).map(|s| &s.shape) else { return Err("Rechteck fehlt".into()) };
        let msg = format!("Drehmodus: {}, Drehung {:.1}°", app.edit.rotate_mode, r.rotation);
        if !app.edit.rotate_mode && r.rotation == 0.0 {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (events, check)
}

fn draw_snapped_rect(app: &mut App) -> (Vec<Event>, Check) {
    app.tool = DrawTool::Rect;
    app.edit.selection.clear();
    let before: Vec<u64> = app.shapes.iter().map(|s| s.id).collect();
    // start 4 px off the job's lower-left corner: inside the snap radius
    let from = app.to_screen([0.0, 0.0]) + Vec2::new(4.0, -3.0);
    let to = app.to_screen([40.0, 30.0]);
    let check: Check = Box::new(move |app| {
        let new: Vec<&Item> = app.shapes.iter().filter(|s| !before.contains(&s.id)).collect();
        let Some(Shape::Rect(r)) = new.first().map(|s| &s.shape) else {
            return Err(format!("{} neue Formen, kein Rechteck", new.len()));
        };
        let lo = [r.center[0] - r.width / 2.0, r.center[1] - r.height / 2.0];
        let msg = format!("Ecke {:.2}/{:.2} mm (Ziel 0/0)", lo[0], lo[1]);
        if lo[0].abs() < 1e-9 && lo[1].abs() < 1e-9 {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(from, to - from), check)
}

impl SelfTest {
    pub(super) fn new() -> Self {
        SelfTest { frame: 0, stable: 0, last_view: None, stage: 0, queue: VecDeque::new(), check: None, settle: 0, failures: 0 }
    }
}

/// A press-drag-release gesture from `from` by `by` screen points, in small steps.
fn drag(from: Pos2, by: Vec2) -> Vec<Event> {
    let mut ev = vec![Event::PointerMoved(from), Event::PointerButton { pos: from, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() }];
    for i in 1..=8 {
        ev.push(Event::PointerMoved(from + by * (i as f32 / 8.0)));
    }
    ev.push(Event::PointerButton { pos: from + by, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() });
    ev
}

fn find(app: &App, f: impl Fn(&Shape) -> bool) -> Option<Item> {
    app.shapes.iter().find(|s| f(&s.shape)).cloned()
}

fn bounds_of(app: &App, id: u64) -> (Pt, Pt) {
    app.shapes.iter().find(|s| s.id == id).and_then(Editable::bounds).unwrap_or_default()
}

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

fn handle_screen(app: &App, id: u64, h: Handle) -> Pos2 {
    let (lo, hi) = selection_bounds(&app.shapes, &[id]).unwrap_or_default();
    app.to_screen(handle_point(lo, hi, app.view().handle_pad, h))
}

fn resize_left(app: &mut App) -> (Vec<Event>, Check) {
    let star = find(app, |s| matches!(s, Shape::Rect(_))).expect("example has the cut-out frame");
    app.tool = DrawTool::Select;
    app.edit.selection = vec![star.id];
    let (lo, hi) = bounds_of(app, star.id);
    let px = 40.0;
    let mm = px as f64 / app.zoom as f64;
    let check: Check = Box::new(move |app| {
        let (l2, h2) = bounds_of(app, star.id);
        let msg = format!("links {:.1}→{:.1} mm, rechts {:.1}→{:.1} mm", lo[0], l2[0], hi[0], h2[0]);
        if near(l2[0], lo[0] - mm, 0.6) && near(h2[0], hi[0], 0.01) && near(l2[1], lo[1], 0.01) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(handle_screen(app, star.id, Handle::Left), Vec2::new(-px, 0.0)), check)
}

fn resize_corner(app: &mut App) -> (Vec<Event>, Check) {
    // a circle shows a ring with four side handles (no corners): its right handle makes it larger,
    // the left side stays
    let circle = find(app, |s| matches!(s, Shape::Circle(_))).expect("example has holes");
    app.edit.selection = vec![circle.id];
    let (lo, hi) = bounds_of(app, circle.id);
    let px = 30.0;
    let dx = px as f64 / app.zoom as f64;
    let corner = handle_screen(app, circle.id, Handle::BottomRight);
    let check: Check = Box::new(move |app| {
        let (l2, h2) = bounds_of(app, circle.id);
        let msg = format!("Breite {:.1}→{:.1} mm, links {:.1}→{:.1} mm", hi[0] - lo[0], h2[0] - l2[0], lo[0], l2[0]);
        if near(l2[0], lo[0], 0.01) && near(h2[0], hi[0] + dx, 0.6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    // (the corner of a circle's box does nothing, see the library test; the side handle works)
    let _ = corner;
    (drag(handle_screen(app, circle.id, Handle::Right), Vec2::new(px, 0.0)), check)
}

fn move_shape(app: &mut App) -> (Vec<Event>, Check) {
    let ell = find(app, |s| matches!(s, Shape::Ellipse(_))).expect("example has an ellipse");
    app.edit.selection.clear();
    let c = ell.shape.center();
    let px = 60.0;
    let mm = px as f64 / app.zoom as f64;
    let check: Check = Box::new(move |app| {
        let c2 = app.shapes.iter().find(|s| s.id == ell.id).map(|s| s.shape.center()).unwrap_or_default();
        let msg = format!("Mitte {:.1}/{:.1} → {:.1}/{:.1} mm, ausgewählt: {}", c[0], c[1], c2[0], c2[1], app.edit.selection == vec![ell.id]);
        if near(c2[0], c[0] + mm, 0.6) && near(c2[1], c[1], 0.01) && app.edit.selection == vec![ell.id] {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    // grab it on its line, at the left end (vectors are picked on their line)
    let grab = app.to_screen([c[0] - 55.0, c[1]]);
    (drag(grab, Vec2::new(px, 0.0)), check)
}

fn band_select(app: &mut App) -> (Vec<Event>, Check) {
    app.edit.selection.clear();
    let holes: Vec<u64> = app.shapes.iter().filter(|s| matches!(s.shape, Shape::Circle(_))).map(|s| s.id).collect();
    // a box around the three drill holes on the right, starting on empty material
    let from = app.to_screen([235.0, 175.0]);
    // far enough right for the hole an earlier stage enlarged (by pixels, so it depends on zoom)
    let to = app.to_screen([285.0, 25.0]);
    let check: Check = Box::new(move |app| {
        let mut sel = app.edit.selection.clone();
        sel.sort();
        let msg = format!("{} von {} Bohrungen ausgewählt", sel.iter().filter(|id| holes.contains(id)).count(), holes.len());
        if sel == holes {
            Ok(msg)
        } else {
            Err(format!("{msg}, Auswahl {sel:?}"))
        }
    });
    (drag(from, to - from), check)
}

fn drag_node(app: &mut App) -> (Vec<Event>, Check) {
    // the hexagon of the example
    let star = find(app, |s| matches!(s, Shape::Polygon(_) | Shape::Path(_))).expect("example has a hexagon");
    app.edit.selection = vec![star.id];
    app.to_curves();
    app.tool = DrawTool::NodeEdit;
    let item = app.shapes.iter().find(|s| s.id == star.id).cloned().expect("star is still there");
    let n0 = item.nodes().expect("curves have nodes")[0];
    let (px, py) = (25.0, -15.0);
    let (dx, dy) = (px as f64 / app.zoom as f64, -py as f64 / app.zoom as f64);
    let check: Check = Box::new(move |app| {
        let n = app.shapes.iter().find(|s| s.id == star.id).and_then(|s| s.nodes()).map(|n| n[0]).unwrap_or_default();
        let msg = format!("Knoten {:.1}/{:.1} → {:.1}/{:.1} mm", n0[0], n0[1], n[0], n[1]);
        if near(n[0], n0[0] + dx, 0.6) && near(n[1], n0[1] + dy, 0.6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(app.to_screen(n0), Vec2::new(px, py)), check)
}

impl App {
    /// Feed the next synthetic event, run checks, finish when every stage has run.
    pub(super) fn selftest_input(&mut self, raw: &mut egui::RawInput) {
        let Some(mut t) = self.selftest.take() else { return };
        t.frame += 1;
        // a steady clock (60 frames a second), so a slow frame rate when the window isn't in
        // front can't stretch a click past egui's click time
        raw.time = Some(t.frame as f64 / 60.0);
        // wait until the window has its final size and the view has stopped changing: gestures
        // are computed from the view, and macOS may resize a new window a few frames late
        let view = (self.zoom.to_bits(), self.origin.x.to_bits(), self.origin.y.to_bits(), raw.screen_rect.map(|r| (r.width().to_bits(), r.height().to_bits())));
        if t.stage == 0 && t.queue.is_empty() && t.last_view != Some(view) {
            t.last_view = Some(view);
            t.stable = 0;
        } else {
            t.stable += 1;
        }
        if t.frame > 15 && t.stable >= 20 {
            // the keys a stage holds stay down until its gesture is through
            if t.stage > 0 && !t.queue.is_empty() {
                raw.modifiers = STAGES[t.stage - 1].held;
            }
            if let Some(e) = t.queue.pop_front() {
                raw.events.push(e);
            } else if let Some(check) = t.check.take() {
                if t.settle < 3 {
                    t.settle += 1;
                    t.check = Some(check);
                } else {
                    t.settle = 0;
                    let name = STAGES[t.stage - 1].name;
                    match check(self) {
                        Ok(msg) => println!("OK      {name}: {msg}"),
                        Err(msg) => {
                            t.failures += 1;
                            println!("FEHLER  {name}: {msg}");
                        }
                    }
                }
            } else if t.stage < STAGES.len() {
                let (events, check) = (STAGES[t.stage].run)(self);
                t.queue.extend(events);
                t.check = Some(check);
                t.stage += 1;
            } else {
                println!("{} von {} Prüfungen bestanden", STAGES.len() - t.failures, STAGES.len());
                std::process::exit(if t.failures == 0 { 0 } else { 1 });
            }
        }
        self.selftest = Some(t);
    }
}

/// The text goes to a second layer that is switched off: it is not picked by a click, and the
/// toolpath that cuts it has nothing left to cut (so the export carries nothing of it either).
fn layer_hidden(app: &mut App) -> (Vec<Event>, Check) {
    // the variables window of the stages before is closed: it would sit over the middle of the sheet
    app.vars_open = false;
    app.new_var = None;
    let text = find(app, |s| matches!(s, Shape::Text(_))).expect("example has the text");
    let layer = app.layers.add("Beschriftung");
    app.layers.activate(1);
    for it in app.shapes.iter_mut().filter(|s| s.id == text.id) {
        it.layer = layer;
    }
    app.layers.set_visible(layer, false);
    app.tool = DrawTool::Select;
    app.edit.selection.clear();
    let (lo, hi) = bounds_of(app, text.id);
    let centre = app.to_screen([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0]);
    let click = vec![
        Event::PointerMoved(centre),
        Event::PointerButton { pos: centre, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: centre, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    let check: Check = Box::new(move |app| {
        let hidden = app.shapes.iter().find(|s| s.id == text.id).is_some_and(|s| s.hidden);
        let picked = app.edit.selection.contains(&text.id);
        let empty = app.toolpaths.iter().filter(|t| t.shape_ids.contains(&text.id) && t.shape_ids.len() == 1).all(|t| t.warning.is_some());
        let msg = format!("ausgeblendet: {hidden}, angeklickt gewählt: {picked}, Werkzeugweg leer: {empty}");
        if hidden && !picked && empty {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (click, check)
}

fn layer_shown(app: &mut App) -> (Vec<Event>, Check) {
    let text = find(app, |s| matches!(s, Shape::Text(_))).expect("example has the text");
    let layer = app.shapes.iter().find(|s| s.id == text.id).map_or(1, |s| s.layer);
    app.layers.set_visible(layer, true);
    let (lo, hi) = bounds_of(app, text.id);
    let centre = app.to_screen([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0]);
    let click = vec![
        Event::PointerMoved(centre),
        Event::PointerButton { pos: centre, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: centre, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    let check: Check = Box::new(move |app| {
        let shown = app.shapes.iter().find(|s| s.id == text.id).is_some_and(|s| !s.hidden);
        let picked = app.edit.selection.contains(&text.id);
        let cut = app.toolpaths.iter().filter(|t| t.shape_ids.contains(&text.id)).all(|t| t.moves.len() > 4);
        let msg = format!("sichtbar: {shown}, angeklickt gewählt: {picked}, Werkzeugweg gerechnet: {cut}");
        if shown && picked && cut {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (click, check)
}

/// A circle goes to a locked layer: still drawn and still cut, but a click does not pick it.
fn layer_locked(app: &mut App) -> (Vec<Event>, Check) {
    let circle = find(app, |s| matches!(s, Shape::Circle(_))).expect("example has holes");
    let layer = app.layers.add("Gesperrt");
    app.layers.activate(1);
    for it in app.shapes.iter_mut().filter(|s| s.id == circle.id) {
        it.layer = layer;
    }
    app.layers.set_locked(layer, true);
    app.tool = DrawTool::Select;
    app.edit.selection.clear();
    let (lo, hi) = bounds_of(app, circle.id);
    // on the circle's line (right side), where a click picks it
    let at = app.to_screen([hi[0], (lo[1] + hi[1]) / 2.0]);
    let click = vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    let check: Check = Box::new(move |app| {
        let it = app.shapes.iter().find(|s| s.id == circle.id);
        let (frozen, hidden) = it.map_or((false, true), |s| (s.frozen, s.hidden));
        let picked = app.edit.selection.contains(&circle.id);
        let msg = format!("gesperrt: {frozen}, ausgeblendet: {hidden}, angeklickt gewählt: {picked}");
        if frozen && !hidden && !picked {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (click, check)
}

fn layer_unlocked(app: &mut App) -> (Vec<Event>, Check) {
    let circle = find(app, |s| matches!(s, Shape::Circle(_))).expect("example has holes");
    let layer = app.shapes.iter().find(|s| s.id == circle.id).map_or(1, |s| s.layer);
    app.layers.set_locked(layer, false);
    let (lo, hi) = bounds_of(app, circle.id);
    let at = app.to_screen([hi[0], (lo[1] + hi[1]) / 2.0]);
    let click = vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::default() },
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::default() },
    ];
    let check: Check = Box::new(move |app| {
        let picked = app.edit.selection.contains(&circle.id);
        let msg = format!("angeklickt gewählt: {picked}");
        if picked {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (click, check)
}

/// The centre mark of a selected shape is a handle for moving it, like its outline.
fn move_by_centre(app: &mut App) -> (Vec<Event>, Check) {
    let circle = find(app, |s| matches!(s, Shape::Circle(_))).expect("example has holes");
    app.tool = DrawTool::Select;
    app.edit.selection = vec![circle.id];
    app.edit.rotate_mode = false;
    let (lo, hi) = bounds_of(app, circle.id);
    let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let px = 40.0;
    let mm = px as f64 / app.zoom as f64;
    let check: Check = Box::new(move |app| {
        let (l2, h2) = bounds_of(app, circle.id);
        let moved = (l2[0] + h2[0]) / 2.0 - centre[0];
        let msg = format!("Mittelpunkt x {:.1} → {:.1} mm (erwartet etwa +{:.1})", centre[0], (l2[0] + h2[0]) / 2.0, mm);
        if moved > mm * 0.5 && near(h2[0] - l2[0], hi[0] - lo[0], 1e-6) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(app.to_screen(centre), Vec2::new(px, 0.0)), check)
}

/// The same for a shape that is not selected yet: its centre mark picks it and moves it.
fn move_unselected_by_centre(app: &mut App) -> (Vec<Event>, Check) {
    let rect = find(app, |s| matches!(s, Shape::Ellipse(_))).expect("example has the pocket ellipse");
    app.tool = DrawTool::Select;
    app.edit.selection.clear();
    let (lo, hi) = bounds_of(app, rect.id);
    let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let px = 30.0;
    let mm = px as f64 / app.zoom as f64;
    let check: Check = Box::new(move |app| {
        let (l2, h2) = bounds_of(app, rect.id);
        let moved = (l2[0] + h2[0]) / 2.0 - centre[0];
        let picked = app.edit.selection.contains(&rect.id);
        let msg = format!("gewählt: {picked}, verschoben um {moved:.1} mm (erwartet etwa +{mm:.1})");
        if picked && moved > mm * 0.5 {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (drag(app.to_screen(centre), Vec2::new(px, 0.0)), check)
}

/// The preview type and the export choice pick the toolpaths as the original's forms do: the
/// visible ones or all of them.
fn scopes(app: &mut App) -> (Vec<Event>, Check) {
    use super::{OutputScope, PreviewScope};
    app.tp_selected = Some(1);
    app.output_scope = OutputScope::All;
    let export_all = app.output_indices();
    // hide the first toolpath: "visible" leaves it out, "all" keeps it
    if let Some(t) = app.toolpaths.first_mut() {
        t.visible = false;
    }
    app.preview_scope = PreviewScope::Visible;
    app.output_scope = OutputScope::Visible;
    let visible = (app.preview_indices(), app.output_indices());
    app.preview_scope = PreviewScope::All;
    let all = app.preview_indices();
    let n = app.toolpaths.len();
    if let Some(t) = app.toolpaths.first_mut() {
        t.visible = true;
    }
    app.preview_scope = PreviewScope::Visible;
    app.output_scope = OutputScope::Visible;
    app.tp_selected = None;
    let check: Check = Box::new(move |_| {
        let msg = format!("export-alle {:?}, sichtbar {:?}/{:?}, alle {:?}", export_all, visible.0, visible.1, all);
        if export_all.len() == n && !visible.0.contains(&0) && !visible.1.contains(&0) && visible.0.len() + 1 == n && all.len() == n {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// Copy a shape that has a toolpath, an own size constraint and a distance to the plate: the copy
/// gets its own toolpath and its own size constraint, but not the distance to the plate.
fn copy_paste(app: &mut App) -> (Vec<Event>, Check) {
    use super::shortcuts::Action;
    use shape_edit::{Anchor, Axis, Distance};
    let e = find(app, |s| matches!(s, Shape::Ellipse(_))).expect("example has the pocket ellipse");
    let plate = shape_edit::constraints::PLATE;
    let base = app.distances.len();
    let own = Distance { from: Anchor::Point(e.id, 0), to: Anchor::Point(e.id, 1), axis: Axis::Horizontal, expr: "60".into(), place: [0.0, 0.0], offset: None };
    let to_plate = Distance { from: Anchor::Point(e.id, 4), to: Anchor::Edge(plate, 0), axis: Axis::Horizontal, expr: "100".into(), place: [0.0, 0.0], offset: None };
    app.distances.push(own);
    app.distances.push(to_plate);
    let (shapes, paths) = (app.shapes.len(), app.toolpaths.len());
    app.tool = DrawTool::Select;
    app.edit.selection = vec![e.id];
    app.run(Action::Copy);
    app.run(Action::Paste);
    let pasted = app.edit.selection.clone();
    let check: Check = Box::new(move |app| {
        let new_id = pasted.first().copied().unwrap_or(0);
        let on_copy: Vec<&Distance> = app.distances.iter().filter(|c| c.from.shape() == new_id || c.to.shape() == new_id).collect();
        let own_only = on_copy.len() == 1 && on_copy[0].from.shape() == new_id && on_copy[0].to.shape() == new_id;
        let copy_path = app.toolpaths.iter().filter(|t| t.shape_ids == vec![new_id]).count();
        let msg = format!(
            "Formen {}→{}, Werkzeugwege {}→{} (davon an der Kopie {}), Constraints an der Kopie {} (nur eigene: {}), insgesamt {}→{}",
            shapes, app.shapes.len(), paths, app.toolpaths.len(), copy_path, on_copy.len(), own_only, base, app.distances.len()
        );
        if pasted.len() == 1 && app.shapes.len() == shapes + 1 && app.toolpaths.len() == paths + 1 && copy_path == 1 && own_only {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// On a Mac, ⌘C and ⌘V arrive as copy and paste events, not as keys: they must still work.
fn cmd_c_cmd_v(app: &mut App) -> (Vec<Event>, Check) {
    let circle = find(app, |s| matches!(s, Shape::Circle(_))).expect("example has holes");
    app.tool = DrawTool::Select;
    app.edit.selection = vec![circle.id];
    let before = app.shapes.len();
    let check: Check = Box::new(move |app| {
        let msg = format!("Formen {}→{}", before, app.shapes.len());
        if app.shapes.len() == before + 1 {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![Event::Copy, Event::Paste(String::new())], check)
}

/// The origin switch hides and shows both the sheet's axis cross and the 3D zero cross.
fn origin_toggle(app: &mut App) -> (Vec<Event>, Check) {
    let switch = |app: &mut App| {
        let sheet = app.sheet.switches().find(|(n, _)| *n == "Koordinatenkreuz").map(|(_, on)| *on);
        let zero = app.view_decos.switches().find(|(n, _)| *n == "Nullpunkt-Achsenkreuz").map(|(_, on)| *on);
        (sheet, zero)
    };
    let start = switch(app);
    app.set_origin_shown(false);
    let off = switch(app);
    app.set_origin_shown(true);
    let on = switch(app);
    let check: Check = Box::new(move |_| {
        let msg = format!("vorher {start:?}, aus {off:?}, an {on:?}");
        if start == (Some(true), Some(true)) && off == (Some(false), Some(false)) && on == (Some(true), Some(true)) {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// A tool entry stands above the first toolpath and wherever the tool changes; exporting the
/// visible toolpaths asks about one file or one file for each tool as soon as there is more than
/// one tool, exporting a single toolpath does not.
fn tool_changes(app: &mut App) -> (Vec<Event>, Check) {
    use super::OutputScope;
    app.output_scope = OutputScope::Visible;
    let tools: Vec<&crate::tools::Tool> = app.toolpaths.iter().map(|t| &t.tool).collect();
    let entries = crate::toolgroups::headers(&tools);
    let entry_count = entries.iter().filter(|&&e| e).count();
    let indices = app.output_indices();
    let groups = app.tool_groups(&indices).len();
    // the export window asks for one file or one per tool only with more than one tool
    let asked = app.export_choice(&indices).then(|| indices.clone());
    // one toolpath is one tool: no question
    let asked_for_one = app.export_choice(&indices[..1]);
    let per_tool = {
        let paths: Vec<&crate::toolpath::Toolpath> = indices.iter().map(|&i| &app.toolpaths[i]).collect();
        crate::toolgroups::files(&app.job, &paths, app.pp(), "schild", None)
    };
    let check: Check = Box::new(move |_| {
        let names: Vec<&str> = per_tool.iter().map(|f| f.0.as_str()).collect();
        let msg = format!("{entry_count} Werkzeugeinträge in der Liste, {groups} Werkzeuggruppen beim Export, gefragt: {}, bei einem Werkzeugweg gefragt: {asked_for_one}, Dateien {names:?}", asked.is_some());
        let numbered = names.iter().enumerate().all(|(n, f)| f.starts_with(&format!("schild_{:02}_", n + 1)));
        if entries.first() == Some(&true) && entry_count >= groups && groups > 1 && asked == Some(indices.clone()) && !asked_for_one && names.len() == groups && numbered {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}

/// Changes since the last save are noticed; replacing the document asks first, and an unchanged
/// document is replaced without a question.
fn unsaved_warning(app: &mut App) -> (Vec<Event>, Check) {
    use super::Pending;
    app.unsaved = None;
    app.mark_clean();
    app.saved_text = app.project_text();
    let clean = !app.is_dirty();
    let before = app.shapes.len();
    let id = app.add_shape(shape_edit::Rect::new([50.0, 50.0], 10.0, 10.0));
    let dirty = app.is_dirty();
    // a value field is open above the drawing: the question must not come up under it, so it is dropped
    app.edit.selection = vec![id];
    app.open_dims(id);
    let field_open = app.dims.is_some();
    app.unsaved = Some(Pending::Close);
    if app.modal_open() {
        app.abort_inputs();
    }
    let field_gone = field_open && app.dims.is_none();
    app.unsaved = None;
    // with changes: the question comes up and nothing is replaced yet
    app.guarded(None, Pending::Example);
    let asked = app.unsaved == Some(Pending::Example) && app.shapes.len() == before + 1;
    app.unsaved = None;
    app.guarded(None, Pending::NewJob);
    let asked_new = app.unsaved == Some(Pending::NewJob) && app.job_dialog.is_none();
    app.unsaved = None;
    // "Nicht speichern" lets closing go on
    app.discard = true;
    let discard_ok = !app.is_dirty();
    app.discard = false;
    // without changes: no question
    app.shapes.retain(|s| s.id != id);
    app.mark_clean();
    app.saved_text = app.project_text();
    app.guarded(None, Pending::NewJob);
    let silent = app.unsaved.is_none() && app.job_dialog.take().is_some();
    let back = app.shapes.len() == before && !app.is_dirty();
    let check: Check = Box::new(move |_| {
        let msg = format!("sauber {clean}, geändert {dirty}, Frage beim Beispiel {asked}, beim neuen Job {asked_new}, „Nicht speichern“ {discard_ok}, ohne Änderung still {silent}, wieder sauber {back}, Eingabefeld vor der Frage weg {field_gone}");
        if clean && dirty && asked && asked_new && discard_ok && silent && back && field_gone {
            Ok(msg)
        } else {
            Err(msg)
        }
    });
    (vec![], check)
}
