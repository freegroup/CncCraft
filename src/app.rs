//! The main window: design tools on the left, the 2D/3D view in the middle and the
//! toolpath tools on the right, like the original.

mod align;
mod cursor;
mod distance;
#[path = "dialogs/mod.rs"]
mod dialogs;
use dialogs::{dialog_buttons, dialog_group};
mod fields;
mod gcode_highlight;
mod gcode_overlay;
mod input;
mod grouping;
mod icons;
mod importing;
mod layers_ui;
mod layout;
mod menus;
mod selftest;
mod selftest_vector_ops;
mod shortcuts;
mod spline_deco;
mod tooltip;
mod variables;
mod vector_ops;

use icons::{icon_button, icon_button_tip};
use tooltip::{Tip, WithTip};
use layout::ToolForm;

use shortcuts::Action;

use crate::gcode;
use crate::geom::{dist, Item, Pt, Region};
use crate::sheet;
use crate::sim::{self, Camera, HeightMap};
use crate::tools::{material_label, Tool, ToolKind, ToolLibrary, MATERIALS};
use crate::toolpath::{default_toolpath, Clearing, Datum, Job, Move, Side, Strategy, Toolpath, ZZero};
use crate::theme::{ink, muted, canvas, separator, ACCENT, ACCENT_DEEP, CONSTRAINT, INK, MUTED, PRIMARY, SELECT};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Vec2};
use shape_edit::{
    handle_point, ops, selection_bounds, Circle, Draw, Editable, Editor as ShapeEditor, Ellipse, Flip, Gesture, Handle, Outcome, Path, Polygon,
    Rect as RectShape, Shape, SnapKind, Snapping, Text, Tool as EditTool, View,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A toolpath computation running in the background: either one new/edited toolpath or all of them
/// recomputed after a shape, layer or parameter change.
enum BgResult {
    One { tp: Toolpath, index: Option<usize> },
    All { tps: Vec<Toolpath>, gen: u64 },
}

struct BgCompute {
    rx: mpsc::Receiver<BgResult>,
    /// Which toolpath indices are being computed: `None` means all of them.
    computing: Option<usize>,
    /// Set to `true` to tell the background thread to stop early.
    cancel: Arc<AtomicBool>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DrawTool {
    Select,
    NodeEdit,
    Polyline,
    Spline,
    Rect,
    Circle,
    Ellipse,
    Polygon,
    Text,
    /// Vector operations (their own group in the tool column, see `vector_ops`).
    Trim,
    Erase,
    Connect,
    FilletOuter,
    FilletInner,
}

impl DrawTool {
    /// One of the vector operations (they work on vectors by clicking, see `vector_ops`).
    fn is_vector_op(self) -> bool {
        matches!(self, DrawTool::Trim | DrawTool::Erase | DrawTool::Connect | DrawTool::FilletOuter | DrawTool::FilletInner)
    }

    fn tip(&self) -> String {
        match self {
            DrawTool::Select => t!("Auswählen und verschieben"),
            DrawTool::NodeEdit => t!("Knoten bearbeiten (Punkte ziehen, Doppelklick fügt ein, Entf löscht)"),
            DrawTool::Polyline => t!("Linie / Polylinie zeichnen (Doppelklick oder Enter beendet)"),
            DrawTool::Spline => t!("Spline zeichnen (Doppelklick oder Enter beendet, Klick auf Anfang schließt)"),
            DrawTool::Rect => t!("Rechteck zeichnen"),
            DrawTool::Circle => t!("Kreis zeichnen"),
            DrawTool::Ellipse => t!("Ellipse zeichnen"),
            DrawTool::Polygon => t!("Polygon zeichnen"),
            DrawTool::Text => t!("Text einfügen"),
            DrawTool::Trim => t!("Trennen: einen Vektor anklicken, er wird an jeder Kreuzung mit anderen Vektoren geteilt"),
            DrawTool::Erase => t!("Löschen: einen Vektor anklicken, er wird gelöscht"),
            DrawTool::Connect => t!("Verbinden: zwei Punkte nacheinander anklicken, sie werden mit einer Linie verbunden"),
            DrawTool::FilletOuter => t!("Ohr: eine Ecke anklicken, sie bekommt einen Kreis um den Eckpunkt (Radius links, jederzeit änderbar)"),
            DrawTool::FilletInner => t!("Abrunden: eine Ecke anklicken, sie wird abgerundet (Radius links, jederzeit änderbar)"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    TwoD,
    ThreeD,
}

#[derive(Clone, Copy, PartialEq)]
enum Icon {
    Draw(DrawTool),
    Job,
    MirrorH,
    MirrorV,
    Center,
    Offset,
    Curves,
    Delete,
    Union,
    Subtract,
    Intersect,
    Profile,
    Pocket,
    Drill,
    VCarve,
    Cutout,
    Transform,
    Grid,
    Snap,
    ZoomFit,
    ZoomSelection,
    /// Opens the variables dialog (an f(x) button next to Job setup).
    Variables,
    /// Brings in a drawing from a file (a page marked DXF); next to the shape tools.
    ImportDxf,
    /// Make the selection one group, and take the outermost group apart (under the boolean operations).
    Group,
    Ungroup,
    /// Shows or hides the dimension lines (a switch beside grid and magnet).
    Dimensions,
    /// The dimension mode: constrain the distance between two points or a point and a line.
    Distance,
    /// Keep a line level or upright.
    Align,
    /// Show or hide the origin: the axis cross of the sheet and the zero cross of the 3D view.
    Origin,
}

/// What the right-click menu of the canvas works on: the shape under the pointer when it opened.
#[derive(Clone)]
struct ShapeMenu {
    id: u64,
    /// The file the shape was imported from, if it was.
    source: Option<shape_edit::Source>,
    grouped: bool,
}

/// What a copy carries: the shapes with their own sizes and level/upright constraints (never a
/// constraint that ties a shape to another shape or to the plate), and the toolpaths that cut
/// them.
#[derive(Clone, Default)]
struct Clip {
    items: Vec<Item>,
    distances: Vec<shape_edit::Distance>,
    aligns: Vec<shape_edit::Align>,
    toolpaths: Vec<Toolpath>,
}

impl Clip {
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// What the preview simulates, as the original's "Preview Type" (0x1000fa28c): all toolpaths,
/// the selected one, or the visible ones.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PreviewScope {
    All,
    Visible,
}

/// What the export writes, as the original's save form (0x100110f38): the selected toolpath or
/// the visible ones. When the visible ones need more than one tool, the export asks whether
/// everything goes into one file or each tool gets its own (see `toolgroups`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum OutputScope {
    All,
    Visible,
}

/// What waits for the answer to "save the changes first?": closing the program, or something
/// that replaces the document.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Pending {
    Close,
    NewJob,
    Open,
    Example,
}

#[derive(Serialize, Deserialize)]
struct Project {
    job: Job,
    shapes: Vec<Item>,
    toolpaths: Vec<Toolpath>,
    /// Named values the sizes can use. Old project files have none.
    #[serde(default)]
    vars: shape_edit::Variables,
    /// Distances that must hold between points and lines. Old project files have none.
    #[serde(default)]
    constraints: Vec<shape_edit::Distance>,
    /// Lines kept level or upright. Old project files have none.
    #[serde(default)]
    aligns: Vec<shape_edit::Align>,
    /// The layers and which one is active. Old project files have one layer with everything on it.
    #[serde(default)]
    layers: crate::layers::Layers,
    #[serde(default)]
    hooks: gcode::OldPostHooks,
    #[serde(default)]
    postprocs: Vec<gcode::PostProcessor>,
}

struct PpDialog {
    editing: gcode::PostProcessor,
    index: Option<usize>,
    hook_tab: usize,
}

struct ToolDialog {
    editing: Tool,
    /// The tool values when the dialog opened, for change-highlight in the list.
    original: Vec<crate::tools::ToolEntry>,
    material: String,
    /// Index into `entries`; `None` = creating a new tool.
    index: Option<usize>,
    /// Working copy of the library; propagated to lib only on save.
    entries: Vec<crate::tools::ToolEntry>,
}

/// A new shape being drawn by dragging. Selecting, moving, resizing and node editing are
/// gestures of the shape editor (`shape_edit::Editor`).
struct Drawing {
    start: Pt,
    end: Pt,
}

/// Arrow keys move the selection (or the picked node) by this much; with Shift by the larger step.
const NUDGE_STEP: f64 = 1.0; // estimated
const NUDGE_STEP_BIG: f64 = 10.0; // estimated
/// Snap radius in screen pixels; the original also keeps it in whole pixels (0x100785420).
const SNAP_RADIUS_PX: f64 = 10.0; // estimated
/// The snap-radius slider's limits; the original clamps the radius to its slider (0x10054e190).
const SNAP_RADIUS_MIN_PX: f64 = 2.0; // estimated
const SNAP_RADIUS_MAX_PX: f64 = 30.0; // estimated
/// Grid spacing for grid snapping, in mm.
/// The grid spacing the original stores in its settings (vcSnapGridSpacingMM, with grid
/// snapping off and the grid hidden: vcSnapGridDoSnap 0, vcSnapGridVisible 0).
const GRID_SPACING: f64 = 5.0;
/// Angle step for Shift while drawing lines; the original accepts 0°..88° (0x100785420).
const ANGLE_STEP: f64 = 15.0;

struct Editor {
    tp: Toolpath,
    index: Option<usize>,
    kind: &'static str,
    /// What is typed in the start and cut depth fields (a number, a formula or variables), and the
    /// behaviour they share with every number field (suggestions, cursor).
    depth_text: [String; 2],
    depth_input: input::ConstraintInput,
}

impl Editor {
    fn new(tp: Toolpath, index: Option<usize>, kind: &'static str) -> Self {
        let show = |expr: &Option<String>, value: f64| expr.clone().unwrap_or_else(|| fields::fmt(value));
        let depth_text = [show(&tp.start_expr, tp.start_depth), show(&tp.cut_expr, tp.cut_depth)];
        Editor { tp, index, kind, depth_text, depth_input: input::ConstraintInput::default() }
    }
}

/// The slowest and the fastest the simulation runs, as a multiple of real time (the slider's
/// middle is exactly real time).
pub(crate) const SIM_RATE_MIN: f64 = 1.0 / 60.0;
pub(crate) const SIM_RATE_MAX: f64 = 60.0;

struct Sim {
    hm: HeightMap,
    /// (toolpath index, moves done) while the animation runs
    anim: Option<(usize, usize)>,
    /// Machine time (seconds) that has come due but was not yet spent on moves.
    budget: f64,
    /// How far into the move being cut the cutter is (0 to 1), and where it is.
    frac: f64,
    pos: Option<[f64; 3]>,
}

impl Sim {
    fn new(hm: HeightMap, anim: Option<(usize, usize)>) -> Sim {
        Sim { hm, anim, budget: 0.0, frac: 0.0, pos: None }
    }
}

/// Inline editing of a fillet radius in the drawing.
pub(super) struct CornerEdit {
    pub id: u64,
    pub path: usize,
    pub node: usize,
    pub text: String,
    pub rect: Rect,
    pub error: Option<String>,
}

pub struct App {
    job: Job,
    shapes: Vec<Item>,
    next_id: u64,
    toolpaths: Vec<Toolpath>,
    editor: Option<Editor>,
    /// A toolpath being computed in a background thread.
    bg_compute: Option<BgCompute>,
    /// Incremented whenever toolpaths are added, removed or reordered; stale batches are dropped.
    tp_gen: u64,
    lib: ToolLibrary,
    tool: DrawTool,
    drawing: Option<Drawing>,
    /// Selection plus move / resize / node gestures.
    edit: ShapeEditor<Item>,
    /// Whether the gesture in progress saved an undo step (dropped again if nothing changed).
    gesture_undo: bool,
    poly: Vec<Pt>,
    zoom: f32,
    origin: Pos2,
    fitted: bool,
    mode: ViewMode,
    cam: Camera,
    sim: Option<Sim>,
    /// How fast the simulation runs: 1 is real time (the machine's own pace), the slider's
    /// middle; from a sixtieth of it to sixty times.
    sim_rate: f64,
    /// Simulation quality in grid points per mm (the original's toolpath.simulation_quality).
    sim_quality: f64,
    start_page: bool,
    job_dialog: Option<(Job, bool)>,
    postprocs: Vec<gcode::PostProcessor>,
    post_idx: usize,
    pp_dialog: Option<PpDialog>,
    tool_dialog: Option<ToolDialog>,
    undo: Vec<Vec<Item>>,
    redo: Vec<Vec<Item>>,
    path: Option<PathBuf>,
    recent_files: Vec<PathBuf>,
    thumb_pending: u8,
    /// When the thumbnail was started (seconds), so a screenshot that never comes does not hold the program.
    thumb_started: f64,
    /// What the save-changes question had chosen (closing, a new job, ...): it waits until the
    /// thumbnail of the saved file is taken, since the window or the document would be gone by then.
    after_thumb: Option<Pending>,
    thumb_restore_mode: Option<ViewMode>,
    /// The moves of the G-code shown over the 3D view (see `gcode_overlay`).
    gcode_overlay: Option<gcode_overlay::GcodeOverlay>,
    /// The selection the toolpath list last scrolled for (it scrolls to the toolpaths of a new selection).
    tp_scrolled_for: Vec<u64>,
    /// The file of an import that could not be found when it was to be read again (shows a notice).
    import_missing: Option<String>,
    /// The shape the canvas' right-click menu was opened on (only shapes with something to offer).
    shape_menu: Option<ShapeMenu>,
    status: String,
    mouse_mm: Option<Pt>,
    offset_dist: f64,
    offset_outside: bool,
    shot: Option<(PathBuf, u32)>,
    about: bool,
    /// The document as last saved, opened or created (as project text): what the changes are measured against.
    saved_text: String,
    /// "Don't save" was chosen: closing may go on.
    discard: bool,
    /// Changes would be lost by this: the question is open.
    unsaved: Option<Pending>,
    /// Whether there are unsaved changes, looked at a few times a second for the title bar.
    dirty_shown: bool,
    dirty_checked: f64,
    /// Take the saved text again after the next frame: the first frame may settle values (variables, constraints).
    rebase: bool,
    /// The export is asking how to save toolpaths that need more than one tool: their positions.
    /// With several tools: one file per tool (a ZIP) instead of everything in one file.
    export_per_tool: bool,
    /// The window for exporting the toolpaths (which ones, which post-processor) is open.
    export_dialog: bool,
    /// The window for exporting the drawing to another program's file is open, with the chosen
    /// format, its settings and which shapes.
    export_file: bool,
    export_format: usize,
    export_settings: crate::export::Settings,
    export_scope: dialogs::ExportScope,
    tools_open: bool,
    /// Snapping switches (View menu) and what a drawing tool's pointer snapped to.
    snap_geometry: bool,
    snap_grid: bool,
    snap_job: bool,
    grid_spacing: f64,
    snap_mark: Option<(Pt, SnapKind)>,
    /// Snap radius in whole pixels, kept between the slider's limits.
    snap_radius_px: f64,
    /// Dimension fields of a freshly drawn shape, and the text of the angle field.
    dims: Option<fields::DimEdit>,
    angle_text: String,
    /// Set by `--selftest`: synthetic mouse gestures and checks.
    selftest: Option<selftest::SelfTest>,
    /// The rectangle the self-test's variable stages work on.
    selftest_pair: Option<(u64, u64)>,
    selftest_rect: Option<u64>,
    /// GPU renderer for the 3D preview.
    gl3d: crate::view3d::Renderer,
    /// Draw the toolpath lines over the 3D preview.
    paths_in_3d: bool,
    /// The toolpath clicked in the list (what "selected" means for preview and export).
    tp_selected: Option<usize>,
    preview_scope: PreviewScope,
    output_scope: OutputScope,
    /// What was copied with ⌘C: shapes, their own constraints and the toolpaths that cut them.
    clipboard: Clip,
    /// Pastes whose toolpaths an undo takes back: (the new shapes, how many toolpaths there were before).
    paste_log: Vec<(Vec<u64>, usize)>,
    /// The form of the tool chosen in the tool column, beside it.
    tool_form: Option<ToolForm>,
    /// What the vector operations remember between clicks (see `vector_ops`).
    vops: vector_ops::VectorOps,
    /// Folded tool column: the tool last picked in each group, and the group whose tools are
    /// open beside it (with where).
    tool_choice: [usize; layout::GROUP_COUNT],
    /// The toolpath being dragged to a new place in the list (its index), and where each row's grip
    /// was drawn last frame (for the self-test).
    tp_drag: Option<usize>,
    tp_grips: Vec<Rect>,
    flyout: Option<(usize, Pos2)>,
    /// Named values the shape inputs can use (parametric sizes), their resolved values, and
    /// whether the variables dialog is open. `vars_dirty` triggers a re-resolve.
    vars: shape_edit::Variables,
    vars_resolved: shape_edit::Resolved,
    vars_open: bool,
    vars_dirty: bool,
    /// What happened to variables since the last frame (added, deleted, renamed), waiting for
    /// everything that depends on them to react; and what the last frame's reaction handled.
    var_events: Vec<shape_edit::VarEvent>,
    var_log: Vec<shape_edit::VarEvent>,
    /// The names the rest of the document uses for each variable (same order as `vars`): a
    /// name typed in the dialog only counts once it is valid, then users are renamed.
    vars_committed: Vec<String>,
    /// A variable just added: its name field takes the cursor (with the name selected).
    new_var: Option<variables::NewVariable>,
    /// The decorations of the 3D view (the view cube).
    view_decos: crate::viewdeco::ViewDecorations,
    /// Show the dimension lines of the selection and of constrained shapes ("Maße anzeigen").
    show_dims: bool,
    /// The dimension mode (D), the distance constraints of the document, the value being typed,
    /// where their values were drawn last frame, and whether toolpaths wait for a drag to end.
    dim_mode: Option<distance::DimMode>,
    distances: Vec<shape_edit::Distance>,
    /// The level/upright constraints, the mode that makes them (H) and where their symbols were
    /// drawn last frame.
    aligns: Vec<shape_edit::Align>,
    align_mode: Option<align::AlignMode>,
    align_labels: Vec<(usize, Rect)>,
    distance_edit: Option<distance::DistanceEdit>,
    distance_labels: Vec<(usize, Rect)>,
    /// The dimension lines of the distances shown without their value (the faint view): they can
    /// be grabbed on the line itself.
    distance_lines: Vec<(usize, [Pos2; 2])>,
    distances_pending: bool,
    /// Which tool cursor is drawn over the drawing this frame (for the self-test).
    custom_cursor_badge: Option<cursor::Badge>,
    distance_conflicts: usize,
    /// Tab was pressed while a distance value field was open (it suggests, it doesn't type).
    distance_tab: bool,
    /// The constraint chosen by a click on its value (Entf deletes it).
    selected_constraint: Option<distance::LabelTarget>,
    label_press: Option<distance::LabelPress>,
    /// Where the shown dimension values were drawn last frame (shape, which size, place): a
    /// click on one opens it for editing.
    dim_labels: Vec<(u64, shape_edit::Dim, Rect)>,
    /// The dimension lines of sizes shown without their value (the faint view), to grab them.
    dim_lines: Vec<(u64, shape_edit::Dim, [Pos2; 2])>,
    /// Fillet radius labels drawn in the canvas: (shape_id, path_index, node_index, screen rect).
    corner_labels: Vec<(u64, usize, usize, Rect)>,
    /// The fillet radius being edited inline: (shape_id, path_index, node_index, text, rect, error).
    corner_edit: Option<CornerEdit>,
    /// The job's size (X, Y, Z) the variables were last worked out for.
    vars_job: [f64; 3],
    /// The layer the toolpath editor window was drawn on last frame (for the self-test).
    editor_layer: Option<egui::LayerId>,
    /// Stored at the start of every frame so background work can wake the UI.
    egui_ctx: Option<egui::Context>,
    /// The open tab under the toolpath list, and whether that panel is unfolded.
    paths_open: bool,
    /// Where the drawing (or the 3D view) was laid out last frame.
    canvas_rect: Rect,
    /// A press that confirmed typed sizes is ignored by the drawing until it is let go.
    swallow_press: bool,
    /// What is drawn around and under the design (board, grid, rulers, axis cross…).
    sheet: sheet::Decorations,
    /// The "Tastenkürzel" window.
    shortcuts_open: bool,
    /// The layers (shown or hidden) every shape is on.
    layers: crate::layers::Layers,
    /// The small window for a new layer, a new name or a layer being deleted.
    layer_dialog: Option<layers_ui::LayerDialog>,
    /// Review only: keep the layer list open.
    layer_open: bool,
    /// The dark colours are in use right now.
    dark: bool,
    /// What the person chose: follow the system, light or dark (as in the original).
    theme_choice: crate::theme::Choice,
    lang: crate::lang::Lang,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, shot: Option<PathBuf>, selftest: bool) -> Self {
        // the theme is remembered between runs; the review runs (screenshot, self-test) use CNCCRAFT_DARK
        let theme_choice = if shot.is_some() || selftest {
            if std::env::var("CNCCRAFT_DARK").is_ok() { crate::theme::Choice::Dark } else { crate::theme::Choice::Light }
        } else {
            cc.storage.and_then(|s| s.get_string("cnccraft_theme").or_else(|| s.get_string("funke_theme"))).map_or(crate::theme::Choice::System, |v| crate::theme::Choice::from_key(&v))
        };
        // the review runs read German unless CNCCRAFT_LANG says otherwise; else the remembered
        // language, else the system's
        let lang = if shot.is_some() || selftest {
            std::env::var("CNCCRAFT_LANG").ok().and_then(|c| crate::lang::Lang::from_code(&c)).unwrap_or(crate::lang::Lang::De)
        } else {
            cc.storage.and_then(|s| s.get_string("cnccraft_lang")).and_then(|c| crate::lang::Lang::from_code(&c)).unwrap_or_else(crate::lang::Lang::system)
        };
        lang.apply();
        if shot.is_none() && !selftest {
            crate::quit::install(&cc.egui_ctx);
        }
        let dark = theme_choice.dark(cc.egui_ctx.system_theme());
        crate::theme::set_dark(dark);
        crate::theme::apply(&cc.egui_ctx);
        let mut app = App {
            job: Job::default(),
            shapes: vec![],
            next_id: 1,
            toolpaths: vec![],
            editor: None,
            bg_compute: None,
            tp_gen: 0,
            lib: cc.storage.and_then(|s| s.get_string("cnccraft_tools")).map_or_else(ToolLibrary::load, |json| ToolLibrary::load_from(&json)),
            tool: DrawTool::Select,
            drawing: None,
            edit: ShapeEditor::default(),
            gesture_undo: false,
            poly: vec![],
            zoom: 2.0,
            origin: Pos2::ZERO,
            fitted: false,
            mode: ViewMode::TwoD,
            cam: Camera::default(),
            sim: None,
            sim_rate: cc.storage.and_then(|s| s.get_string("cnccraft_sim_rate")).and_then(|v| v.parse::<f64>().ok()).map_or(1.0, |v| v.clamp(SIM_RATE_MIN, SIM_RATE_MAX)),
            sim_quality: cc.storage.and_then(|s| s.get_string("cnccraft_sim_quality")).and_then(|v| v.parse().ok()).unwrap_or(sim::SIMULATION_QUALITY),
            start_page: true,
            job_dialog: None,
            postprocs: gcode::builtins(),
            post_idx: cc.storage.and_then(|s| s.get_string("cnccraft_post_idx")).and_then(|v| v.parse::<usize>().ok()).unwrap_or(0),
            pp_dialog: None,
            tool_dialog: None,
            undo: vec![],
            redo: vec![],
            path: None,
            recent_files: cc.storage
                .and_then(|s| s.get_string("cnccraft_recent"))
                .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
                .map(|v| v.into_iter().map(PathBuf::from).collect())
                .unwrap_or_default(),
            thumb_pending: 0,
            thumb_started: 0.0,
            after_thumb: None,
            thumb_restore_mode: None,
            gcode_overlay: None,
            tp_scrolled_for: Vec::new(),
            import_missing: None,
            shape_menu: None,
            status: "Bereit".into(),
            mouse_mm: None,
            offset_dist: 5.0,
            offset_outside: true,
            shot: shot.map(|p| (p, 0)),
            about: false,
            saved_text: String::new(),
            discard: false,
            unsaved: None,
            dirty_shown: false,
            dirty_checked: 0.0,
            rebase: false,
            export_per_tool: cc.storage.and_then(|s| s.get_string("cnccraft_export_per_tool")).is_some_and(|v| v == "true"),
            export_dialog: false,
            export_file: false,
            export_format: 0,
            export_settings: crate::export::Settings::default(),
            export_scope: dialogs::ExportScope::All,
            tools_open: cc.storage.and_then(|s| s.get_string("cnccraft_tools_open")).map_or(true, |v| v != "false"),
            snap_geometry: cc.storage.and_then(|s| s.get_string("cnccraft_snap_geometry")).map_or(true, |v| v != "false"),
            snap_grid: cc.storage.and_then(|s| s.get_string("cnccraft_snap_grid")).is_some_and(|v| v == "true"),
            snap_job: cc.storage.and_then(|s| s.get_string("cnccraft_snap_job")).map_or(true, |v| v != "false"),
            grid_spacing: cc.storage.and_then(|s| s.get_string("cnccraft_grid_spacing")).and_then(|v| v.parse().ok()).unwrap_or(GRID_SPACING),
            snap_mark: None,
            snap_radius_px: cc.storage.and_then(|s| s.get_string("cnccraft_snap_radius")).and_then(|v| v.parse().ok()).unwrap_or(SNAP_RADIUS_PX),
            dims: None,
            angle_text: "0".into(),
            selftest: selftest.then(selftest::SelfTest::new),
            selftest_pair: None,
            selftest_rect: None,
            gl3d: Default::default(),
            paths_in_3d: cc.storage.and_then(|s| s.get_string("cnccraft_gcode_in_3d")).is_some_and(|v| v == "true"),
            clipboard: Clip::default(),
            paste_log: vec![],
            tool_form: None,
            vops: vector_ops::VectorOps::new(),
            tool_choice: [0; layout::GROUP_COUNT],
            tp_drag: None,
            tp_selected: None,
            preview_scope: cc.storage.and_then(|s| s.get_string("cnccraft_preview_scope")).map_or(PreviewScope::Visible, |v| match v.as_str() {
                "All" => PreviewScope::All,
                _ => PreviewScope::Visible,
            }),
            output_scope: cc.storage.and_then(|s| s.get_string("cnccraft_output_scope")).map_or(OutputScope::Visible, |v| match v.as_str() {
                "All" => OutputScope::All,
                _ => OutputScope::Visible,
            }),
            tp_grips: Vec::new(),
            flyout: None,
            vars: shape_edit::Variables::default(),
            vars_resolved: shape_edit::Resolved::default(),
            vars_open: false,
            vars_dirty: true,
            var_events: Vec::new(),
            var_log: Vec::new(),
            vars_committed: Vec::new(),
            new_var: None,
            view_decos: crate::viewdeco::ViewDecorations::default(),
            show_dims: cc.storage.and_then(|s| s.get_string("cnccraft_show_dims")).is_some_and(|v| v == "true"),
            dim_mode: None,
            distances: Vec::new(),
            aligns: Vec::new(),
            align_mode: None,
            align_labels: Vec::new(),
            distance_edit: None,
            distance_labels: Vec::new(),
            distance_lines: Vec::new(),
            distances_pending: false,
            custom_cursor_badge: None,
            distance_conflicts: 0,
            distance_tab: false,
            selected_constraint: None,
            label_press: None,
            dim_labels: Vec::new(),
            dim_lines: Vec::new(),
            corner_labels: Vec::new(),
            corner_edit: None,
            vars_job: [0.0; 3],
            editor_layer: None,
            egui_ctx: None,
            paths_open: cc.storage.and_then(|s| s.get_string("cnccraft_paths_open")).map_or(true, |v| v != "false"),
            canvas_rect: Rect::NOTHING,
            swallow_press: false,
            sheet: sheet::Decorations::default(),
            shortcuts_open: false,
            dark,
            theme_choice,
            lang,
            layers: crate::layers::Layers::default(),
            layer_dialog: None,
            layer_open: false,
        };
        app.post_idx = app.post_idx.min(app.postprocs.len().saturating_sub(1));
        if cc.storage.and_then(|s| s.get_string("cnccraft_cam_perspective")).is_some_and(|v| v == "true") {
            app.cam.perspective = true;
        }
        if app.shot.is_some() || app.selftest.is_some() {
            app.load_example();
            // for the review screenshot: show the job dialog if asked, so its look is visible
            if std::env::var("CNCCRAFT_SHOW_DIALOG").is_ok() {
                app.job_dialog = Some((app.job.clone(), true));
            }
            // the variables dialog with a few variables (one broken), to review it
            if std::env::var("CNCCRAFT_SHOW_VARS").is_ok() {
                app.vars.set("hoehe", "40");
                app.vars.set("rand", "hoehe/4");
                app.vars.set("breite", "platte_b - 2*rand");
                app.vars.set("kaputt", "gibtsnicht + 1");
                app.vars_open = true;
            }
            // three layers, the text's switched off, the list open, to review it
            if std::env::var("CNCCRAFT_SHOW_LAYERS").is_ok() {
                let text = app.shapes.iter().find(|s| matches!(s.shape, Shape::Text(_))).map(|s| s.id);
                let l2 = app.layers.add("Beschriftung");
                let l3 = app.layers.add("Bohrungen");
                app.layers.set_locked(l3, true);
                app.shapes.iter_mut().filter(|s| matches!(s.shape, Shape::Circle(_))).for_each(|s| s.layer = l3);
                app.layers.activate(1);
                if let Some(t) = text {
                    app.shapes.iter_mut().filter(|s| s.id == t).for_each(|s| s.layer = l2);
                }
                app.layers.set_visible(l2, false);
                app.layer_open = true;
            }
            // a rectangle whose width is tied to a variable, with the dimensions switched on
            if std::env::var("CNCCRAFT_SHOW_CONSTRAINT").is_ok() {
                app.vars.set("breite", "90");
                let id = app.add_shape(RectShape::new([100.0, 160.0], 10.0, 20.0));
                if let Some(it) = app.shapes.iter_mut().find(|s| s.id == id) {
                    it.bound = vec![(shape_edit::Dim::Width, "breite".into())];
                }
                app.vars_dirty = true;
                app.edit.selection = vec![id];
                app.show_dims = true;
            }
            // the toolpath form with a depth formula, to review it
            if std::env::var("CNCCRAFT_SHOW_DEPTH").is_ok() {
                let mut tp = default_toolpath("cutout", &app.lib, &app.job, vec![]);
                tp.cut_expr = Some("platte_d/2".into());
                tp.start_expr = Some("plat".into());
                app.editor = Some(Editor::new(tp, None, "cutout"));
                app.vars_dirty = true;
            }
            // a line or a circle selected, to review their selection feedback
            if let Ok(what) = std::env::var("CNCCRAFT_SHOW_STYLE") {
                let id = if what == "circle" { app.add_shape(Circle::new([200.0, 150.0], 20.0)) } else { app.add_shape(shape_edit::Path::new(vec![[170.0, 140.0], [260.0, 165.0]], false)) };
                app.edit.selection = vec![id];
            }
            // a vector kept level, and a rectangle's side, to review the symbols
            if std::env::var("CNCCRAFT_SHOW_ALIGN").is_ok() {
                let id = app.add_shape(shape_edit::Path::new(vec![[200.0, 150.0], [270.0, 158.0], [270.0, 178.0]], false));
                app.aligns.push(shape_edit::Align { a: shape_edit::Anchor::Point(id, 5), b: shape_edit::Anchor::Point(id, 6), orient: shape_edit::Orient::Horizontal });
                app.aligns.push(shape_edit::Align { a: shape_edit::Anchor::Point(id, 6), b: shape_edit::Anchor::Point(id, 7), orient: shape_edit::Orient::Vertical });
                app.show_dims = true;
            }
            // the "Neue Variable" dialog, to review it
            if std::env::var("CNCCRAFT_SHOW_NEWVAR").is_ok() {
                app.vars.set("breite", "300");
                app.open_new_variable();
                if let Some(nv) = &mut app.new_var {
                    nv.name = "hoehe".into();
                    nv.expr = "breite/6".into();
                    nv.input.select = None;
                }
            }
            // two rectangles with a horizontal distance and one from the plate's left edge
            if std::env::var("CNCCRAFT_SHOW_DISTANCE").is_ok() {
                // a rectangle with a set width: its top and bottom side show black
                let f = app.add_shape(RectShape::new([220.0, 150.0], 50.0, 30.0));
                app.distances.push(shape_edit::Distance { from: shape_edit::Anchor::Point(f, 0), to: shape_edit::Anchor::Point(f, 1), axis: shape_edit::Axis::Horizontal, expr: "50".into(), place: [220.0, 130.0], offset: None });
                let a = app.add_shape(RectShape::new([60.0, 70.0], 40.0, 30.0));
                let b = app.add_shape(RectShape::new([150.0, 90.0], 30.0, 30.0));
                app.distances.push(shape_edit::Distance { from: shape_edit::Anchor::Point(a, 1), to: shape_edit::Anchor::Point(b, 0), axis: shape_edit::Axis::Horizontal, expr: "breite/2".into(), place: [110.0, 140.0], offset: None });
                app.distances.push(shape_edit::Distance { from: shape_edit::Anchor::Edge(shape_edit::constraints::PLATE, 3), to: shape_edit::Anchor::Point(a, 0), axis: shape_edit::Axis::Aligned, expr: "25".into(), place: [20.0, 40.0], offset: None });
                app.vars.set("breite", "30");
                app.vars_dirty = true;
                app.show_dims = true;
                app.dim_mode = Some(distance::DimMode::new(app.tool));
            }
            // the folded tool column with the shapes group open, to review it
            if std::env::var("CNCCRAFT_FOLD_TOOLS").is_ok() {
                app.tools_open = false;
                app.flyout = Some((2, Pos2::ZERO));
            }
            // the export window (with the choice of files when the toolpaths need more than one tool)
            if std::env::var("CNCCRAFT_SHOW_EXPORT").is_ok() {
                app.export_dialog = true;
            }
            // the window exporting the drawing, with the given format ("dxf" or "svg")
            if let Ok(format) = std::env::var("CNCCRAFT_SHOW_EXPORT_FILE") {
                app.export_format = crate::export::FORMATS.iter().position(|f| f.extension == format).unwrap_or(0);
                app.open_export_file();
            }
            // and the finished carving in 3D, to review the preview
            if std::env::var("CNCCRAFT_SHOW_3D").is_ok() {
                app.start_preview(false);
            }
            // with the G-code overlay: the machine's moves as the post-processor writes them
            if std::env::var("CNCCRAFT_SHOW_GCODE").is_ok() {
                app.paths_in_3d = true;
            }
        }
        app.mark_clean();
        // a change that is not saved and the question about it, to review them
        if app.shot.is_some() && std::env::var("CNCCRAFT_SHOW_UNSAVED").is_ok() {
            app.add_shape(RectShape::new([60.0, 150.0], 30.0, 20.0));
            app.unsaved = Some(Pending::Close);
        }
        app
    }

    // ---------------- document ----------------

    fn new_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    fn push_undo(&mut self) {
        let snapshot = self.shapes.clone();
        self.push_undo_from(snapshot);
    }

    /// Save `before` as an undo step (taken before a change that already happened).
    fn push_undo_from(&mut self, before: Vec<Item>) {
        self.undo.push(before);
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn add_shape(&mut self, shape: impl Into<Shape>) -> u64 {
        let id = self.new_id();
        let mut item = Item::new(id, shape);
        item.layer = self.layers.active;
        // drawing on a switched-off layer switches it on, so the new shape does not vanish
        self.layers.set_visible(self.layers.active, true);
        self.shapes.push(item);
        id
    }

    fn load_example(&mut self) {
        self.job = Job { width: 300.0, height: 200.0, thickness: 18.0, ..Job::default() };
        self.shapes.clear();
        self.toolpaths.clear();
        self.tp_gen += 1;
        self.layers = crate::layers::Layers::default();
        let frame = self.add_shape(RectShape { corner_radius: 12.0, ..RectShape::new([150.0, 100.0], 270.0, 170.0) });
        let text = self.add_shape(Text::new([150.0, 118.0], "CncCraft", 42.0));
        let hexagon = self.add_shape(Polygon::new([62.0, 52.0], 22.0, 6));
        let pocket = self.add_shape(Ellipse::new([160.0, 52.0], 55.0, 20.0));
        let h1 = self.add_shape(Circle::new([255.0, 160.0], 3.0));
        let h2 = self.add_shape(Circle::new([255.0, 40.0], 3.0));
        let h3 = self.add_shape(Circle::new([245.0, 100.0], 3.0));
        let job = self.job.clone();
        let mut v = default_toolpath("vcarve", &self.lib, &job, vec![text, hexagon]);
        v.name = t!("V-Carving Schrift").into();
        let mut p = default_toolpath("pocket", &self.lib, &job, vec![pocket]);
        p.name = t!("Tasche Ellipse").into();
        let mut d = default_toolpath("drill", &self.lib, &job, vec![h1, h2, h3]);
        d.tool = self.lib.pick(&job.material, ToolKind::Drill, 3.0, None);
        d.cut_depth = 8.0;
        let c = default_toolpath("cutout", &self.lib, &job, vec![frame]);
        for tp in [v, p, d, c] {
            self.toolpaths.push(tp);
        }
        self.recompute_all();
        self.edit = ShapeEditor::default();
        self.edit.selection = vec![hexagon];
        self.start_page = false;
        self.fitted = false;
        self.sim = None;
        self.mark_clean();
    }

    fn new_job(&mut self, job: Job) {
        self.job = job;
        self.shapes.clear();
        self.toolpaths.clear();
        self.tp_gen += 1;
        self.layers = crate::layers::Layers::default();
        self.layer_dialog = None;
        self.vars = shape_edit::Variables::default();
        self.vars_committed.clear();
        self.var_events.clear();
        self.vars_dirty = true;
        self.distances.clear();
        self.aligns.clear();
        self.align_mode = None;
        self.dim_mode = None;
        self.distance_edit = None;
        self.edit = ShapeEditor::default();
        self.undo.clear();
        self.redo.clear();
        self.editor = None;
        self.sim = None;
        self.path = None;
        self.fitted = false;
        self.start_page = false;
        self.mode = ViewMode::TwoD;
        self.mark_clean();
    }

    /// The document as project text, to tell whether it changed since it was last saved.
    fn project_text(&self) -> String {
        let proj = Project { job: self.job.clone(), shapes: self.shapes.clone(), toolpaths: self.toolpaths.clone(), vars: self.vars.clone(), constraints: self.distances.clone(), aligns: self.aligns.clone(), layers: self.layers.clone(), hooks: gcode::OldPostHooks::default(), postprocs: self.postprocs.clone() };
        serde_json::to_string(&proj).unwrap_or_default()
    }

    /// The document as it is now counts as saved.
    pub(super) fn mark_clean(&mut self) {
        self.saved_text = self.project_text();
        self.discard = false;
        self.dirty_shown = false;
        self.rebase = true;
    }

    /// Whether the document changed since it was last saved, opened or created.
    pub(super) fn is_dirty(&self) -> bool {
        !self.discard && self.project_text() != self.saved_text
    }

    /// Do `what` now, or first ask whether to save, if that would lose changes.
    pub(super) fn guarded(&mut self, ctx: Option<&egui::Context>, what: Pending) {
        if self.is_dirty() {
            self.unsaved = Some(what);
        } else {
            self.run_pending(ctx, what);
        }
    }

    fn run_pending(&mut self, ctx: Option<&egui::Context>, what: Pending) {
        match what {
            Pending::Close => {
                self.discard = true;
                if let Some(ctx) = ctx {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            Pending::NewJob => self.job_dialog = Some((Job::default(), true)),
            Pending::Open => self.open_project(),
            Pending::Example => self.load_example(),
        }
        if what != Pending::Close {
            // leaving things as they are (a cancelled window) keeps the warning for later
            self.discard = false;
        }
    }

    /// A window that asks something and must be answered first is open.
    pub(super) fn modal_open(&self) -> bool {
        self.unsaved.is_some() || self.after_thumb.is_some() || self.export_dialog || self.export_file || self.job_dialog.is_some() || self.about
    }

    /// Drop everything that is being typed or drawn, and the pop-ups: the floating value fields and
    /// the lists sit above windows, so they would cover the question. What was typed but not yet
    /// taken over is dropped, as with Esc.
    pub(super) fn abort_inputs(&mut self) {
        let active = self.dims.is_some() || self.distance_edit.is_some() || self.drawing.is_some() || !self.poly.is_empty() || self.edit.is_busy() || self.layer_open || self.flyout.is_some();
        if !active {
            return;
        }
        self.dims = None;
        self.distance_edit = None;
        self.drawing = None;
        self.poly.clear();
        if self.edit.is_busy() {
            self.edit.cancel(&mut self.shapes);
            if self.gesture_undo {
                self.undo.pop();
                self.gesture_undo = false;
            }
        }
        self.layer_open = false;
        self.flyout = None;
        self.status = "Eingabe abgebrochen".into();
    }

    fn save_project(&mut self, ask: bool) {
        // the self-test must never open a native file window: nobody would close it
        if self.selftest.is_some() {
            return;
        }
        let path = if ask || self.path.is_none() {
            rfd::FileDialog::new().add_filter(t!("CncCraft-Projekt"), &["cnccraft"]).set_file_name("projekt.cnccraft").save_file()
        } else {
            self.path.clone()
        };
        let Some(path) = path else { return };
        let proj = Project { job: self.job.clone(), shapes: self.shapes.clone(), toolpaths: self.toolpaths.clone(), vars: self.vars.clone(), constraints: self.distances.clone(), aligns: self.aligns.clone(), layers: self.layers.clone(), hooks: gcode::OldPostHooks::default(), postprocs: self.postprocs.clone() };
        match serde_json::to_string_pretty(&proj).map_err(|e| e.to_string()).and_then(|s| std::fs::write(&path, s).map_err(|e| e.to_string())) {
            Ok(()) => {
                self.add_recent(&path);
                self.path = Some(path);
                self.thumb_pending = 1;
                self.mark_clean();
            }
            Err(e) => self.status = format!("Speichern fehlgeschlagen: {e}"),
        }
    }

    fn open_project(&mut self) {
        // the self-test must never open a native file window: nobody would close it
        if self.selftest.is_some() {
            return;
        }
        let Some(path) = rfd::FileDialog::new().add_filter(t!("CncCraft-Projekt"), &["cnccraft", "funke"]).pick_file() else { return };
        self.open_path(path);
    }

    fn open_path(&mut self, path: PathBuf) {
        let res = std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str::<Project>(&s).map_err(|e| e.to_string()));
        match res {
            Ok(p) => {
                self.new_job(p.job);
                self.next_id = p.shapes.iter().map(|s| s.id).max().unwrap_or(1) + 1;
                self.shapes = p.shapes;
                self.toolpaths = p.toolpaths;
                self.vars = p.vars;
                self.distances = p.constraints;
                self.aligns = p.aligns;
                self.layers = p.layers.repaired();
                self.layers.sync(&mut self.shapes);
                if !p.postprocs.is_empty() {
                    self.postprocs = p.postprocs;
                } else {
                    self.postprocs = gcode::migrate_old_format(&p.hooks);
                }
                self.post_idx = self.post_idx.min(self.postprocs.len().saturating_sub(1));
                self.vars_committed.clear();
                self.var_events.clear();
                self.vars_dirty = true;
                self.add_recent(&path);
                self.path = Some(path);
                self.mark_clean();
            }
            Err(e) => {
                self.recent_files.retain(|p| *p != path);
                self.status = format!("Öffnen fehlgeschlagen: {e}");
            }
        }
    }

    /// Whether the toolpaths need more than one tool, so the export has to say whether
    /// everything goes into one file or each tool gets its own.
    pub(super) fn export_choice(&self, indices: &[usize]) -> bool {
        self.tool_groups(indices).len() > 1
    }

    /// Save the toolpaths of the export scope, as the export window set it up: one machine file, or
    /// (several tools) one file for each tool in a ZIP. One file with several tools only works with
    /// a post-processor that can change tools.
    pub(super) fn export_gcode(&mut self) {
        let indices = self.output_indices();
        if indices.is_empty() {
            self.status = match self.output_scope {
                OutputScope::All => "Keine Werkzeugwege vorhanden".into(),
                _ => "Keine sichtbaren Werkzeugwege zum Speichern".into(),
            };
            return;
        }
        let per_tool = self.export_choice(&indices) && (self.export_per_tool || !self.pp().tool_change);
        self.write_export(&indices, per_tool);
    }

    /// The stretches of the same tool among the given toolpaths, as positions in `indices`.
    pub(super) fn tool_groups(&self, indices: &[usize]) -> Vec<Vec<usize>> {
        let tools: Vec<&Tool> = indices.iter().map(|&i| &self.toolpaths[i].tool).collect();
        crate::toolgroups::groups(&tools)
    }

    fn pp(&self) -> &gcode::PostProcessor {
        &self.postprocs[self.post_idx.min(self.postprocs.len().saturating_sub(1))]
    }

    fn add_recent(&mut self, path: &std::path::Path) {
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_path_buf());
        self.recent_files.truncate(10);
    }

    /// The name offered first in the save window: the project's own, else a general one.
    fn export_stem(&self) -> String {
        let stem = self.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let part = crate::toolgroups::file_part(&stem);
        if part.is_empty() { "werkzeugwege".into() } else { part }
    }

    /// Write the toolpaths at `indices`: to one machine file, or, with `per_tool`, one file for each
    /// stretch of the same tool, named `<name>_01_<tool>.<ext>`, together in one ZIP.
    fn write_export(&mut self, indices: &[usize], per_tool: bool) {
        // the self-test must never open a native file window: nobody would close it
        if self.selftest.is_some() {
            return;
        }
        let paths: Vec<&Toolpath> = indices.iter().map(|&i| &self.toolpaths[i]).collect();
        if per_tool {
            let Some(file) = rfd::FileDialog::new()
                .add_filter(t!("ZIP-Archiv"), &["zip"])
                .set_file_name(format!("{}.zip", self.export_stem()))
                .save_file()
            else {
                return;
            };
            let stem = file.file_stem().map(|s| crate::toolgroups::file_part(&s.to_string_lossy())).filter(|s| !s.is_empty()).unwrap_or_else(|| "werkzeugwege".into());
            let scope = self.export_scope_text(indices.len());
            let files: Vec<(String, Vec<u8>)> = crate::toolgroups::files(&self.job, &paths, self.pp(), &stem, Some(&scope)).into_iter().map(|(n, t)| (n, t.into_bytes())).collect();
            let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
            match std::fs::write(&file, crate::zipfile::stored(&files, seconds)) {
                Ok(()) => {}
                Err(e) => self.status = format!("Fehler: {e}"),
            }
            return;
        }
        let pp = self.pp().clone();
        let Some(file) = rfd::FileDialog::new()
            .add_filter(&pp.label(), &[&pp.ext])
            .set_file_name(format!("{}.{}", self.export_stem(), pp.ext))
            .save_file()
        else {
            return;
        };
        let meta = crate::transform::to_gcode::ExportMeta { scope: Some(self.export_scope_text(indices.len())), file: None, tool_numbers: None };
        let text = crate::transform::to_gcode::write_with_meta(&self.job, &paths, &pp, &meta);
        match std::fs::write(&file, text) {
            Ok(()) => {}
            Err(e) => self.status = format!("Fehler: {e}"),
        }
    }

    /// The question about more than one tool: everything in one file, or one file for each tool.
    fn delete_selection(&mut self) {
        if self.edit.selection.is_empty() {
            return;
        }
        self.push_undo();
        let sel = std::mem::take(&mut self.edit.selection);
        self.shapes.retain(|s| !sel.contains(&s.id));
        self.edit.retain_existing(&self.shapes);
        self.recompute_all();
    }

    /// The selected shapes, in the order they were selected.
    fn selected_shapes(&self) -> Vec<Shape> {
        self.edit.selection.iter().filter_map(|id| self.shapes.iter().find(|s| s.id == *id)).map(|s| s.shape.clone()).collect()
    }

    /// Replace every selected shape by `f(shape)`, keeping its id, as one undo step.
    fn map_selected(&mut self, f: impl Fn(&Shape) -> Shape) {
        if self.edit.selection.is_empty() {
            return;
        }
        self.push_undo();
        for it in self.shapes.iter_mut().filter(|s| self.edit.selection.contains(&s.id)) {
            it.shape = f(&it.shape);
        }
        self.edit.retain_existing(&self.shapes);
    }

    fn mirror(&mut self, flip: Flip) {
        let Some(axis) = ops::group_center(&self.selected_shapes()) else { return };
        self.map_selected(|s| s.mirrored(axis, flip));
    }

    fn center_on_job(&mut self) {
        let Some(c) = ops::group_center(&self.selected_shapes()) else { return };
        let (dx, dy) = (self.job.width / 2.0 - c[0], self.job.height / 2.0 - c[1]);
        self.map_selected(|s| s.translated(dx, dy));
    }

    fn to_curves(&mut self) {
        self.map_selected(Shape::to_curves);
        self.drop_bindings_of_selection();
    }

    fn duplicate(&mut self) {
        let clip = self.make_clip(&self.edit.selection.clone());
        self.paste_clip(&clip);
    }

    /// Expand (outside) or shrink (inside) the selection's outline into a new shape.
    fn offset_selection(&mut self) {
        let shapes = self.selected_shapes();
        if shapes.is_empty() {
            return;
        }
        let d = if self.offset_outside { self.offset_dist } else { -self.offset_dist };
        let Some(result) = Region::of_all(&shapes).offset(d).to_shape() else {
            self.status = "Versatz ergibt keine Form".into();
            return;
        };
        self.push_undo();
        let id = self.add_shape(result);
        self.edit.selection = vec![id];
        self.status = if d > 0.0 { "Vergrößerte Kontur erzeugt".into() } else { "Verkleinerte Kontur erzeugt".into() };
    }

    fn recompute_all(&mut self) {
        if self.toolpaths.is_empty() {
            return;
        }
        if let Some(bg) = &self.bg_compute {
            bg.cancel.store(true, Ordering::Relaxed);
        }
        if let Some(ctx) = &self.egui_ctx {
            let job = self.job.clone();
            let shapes: Vec<Item> = self.shapes.clone();
            let mut tps: Vec<Toolpath> = self.toolpaths.iter().map(|tp| tp.without_moves()).collect();
            let gen = self.tp_gen;
            let cancel = Arc::new(AtomicBool::new(false));
            let cancel2 = cancel.clone();
            let (tx, rx) = mpsc::channel();
            let ctx2 = ctx.clone();
            std::thread::spawn(move || {
                for tp in &mut tps {
                    if cancel2.load(Ordering::Relaxed) { return; }
                    tp.compute(&job, &shapes);
                }
                let _ = tx.send(BgResult::All { tps, gen });
                ctx2.request_repaint();
            });
            self.bg_compute = Some(BgCompute { rx, computing: None, cancel });
            self.sim = None;
        } else {
            for tp in &mut self.toolpaths {
                tp.compute(&self.job, &self.shapes);
            }
            self.sim = None;
        }
    }

    fn compute_in_background(&mut self, mut tp: Toolpath, index: Option<usize>, ctx: &egui::Context) {
        self.tp_gen += 1;
        if let Some(bg) = &self.bg_compute {
            bg.cancel.store(true, Ordering::Relaxed);
        }
        tp.moves.clear();
        let job = self.job.clone();
        let shapes: Vec<Item> = self.shapes.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            tp.compute(&job, &shapes);
            let _ = tx.send(BgResult::One { tp, index });
            ctx2.request_repaint();
        });
        self.bg_compute = Some(BgCompute { rx, computing: index, cancel });
        self.status = t!("Berechne Werkzeugweg…").into();
    }

    fn poll_bg_compute(&mut self) {
        let Some(bg) = &self.bg_compute else { return };
        match bg.rx.try_recv() {
            Ok(BgResult::One { tp, index }) => {
                let st = tp.stats();
                self.status = format!("{} berechnet: {} Bewegungen, ca. {}", tp.name, tp.moves.len(), fmt_time(st.minutes));
                self.bg_compute = None;
                self.tp_gen += 1;
                match index {
                    Some(i) if i < self.toolpaths.len() => self.toolpaths[i] = tp,
                    _ => self.toolpaths.push(tp),
                }
                self.sim = None;
            }
            Ok(BgResult::All { tps, gen }) => {
                self.bg_compute = None;
                if gen == self.tp_gen {
                    self.toolpaths = tps;
                    self.sim = None;
                } else {
                    self.recompute_all();
                }
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.status = t!("Berechnung fehlgeschlagen").into();
                self.bg_compute = None;
            }
        }
    }

    // ---------------- coordinates ----------------

    fn to_screen(&self, p: Pt) -> Pos2 {
        Pos2::new(self.origin.x + p[0] as f32 * self.zoom, self.origin.y - p[1] as f32 * self.zoom)
    }

    fn to_mm(&self, p: Pos2) -> Pt {
        [((p.x - self.origin.x) / self.zoom) as f64, ((self.origin.y - p.y) / self.zoom) as f64]
    }

    fn fit(&mut self, rect: Rect) {
        let m = 50.0;
        self.zoom = ((rect.width() - 2.0 * m) / self.job.width as f32).min((rect.height() - 2.0 * m) / self.job.height as f32).max(0.05);
        let c = rect.center();
        self.origin = Pos2::new(c.x - self.job.width as f32 * self.zoom / 2.0, c.y + self.job.height as f32 * self.zoom / 2.0);
        self.fitted = true;
    }

    /// Pick distances in millimetres for the current zoom: 8 px to grab, handles 4 px outside.
    /// Snapping for this frame: the pixel radius in mm at the current zoom, plus the job's
    /// corners and centre.
    fn update_snapping(&mut self) {
        let (w, h) = (self.job.width, self.job.height);
        let extra = if self.snap_job { vec![[0.0, 0.0], [w, 0.0], [w, h], [0.0, h], [w / 2.0, h / 2.0]] } else { vec![] };
        // whole pixels between the slider's limits, as the original keeps it (0x10054e190)
        self.snap_radius_px = self.snap_radius_px.round().clamp(SNAP_RADIUS_MIN_PX, SNAP_RADIUS_MAX_PX);
        self.edit.snapping = Snapping { geometry: self.snap_geometry, grid: self.snap_grid.then_some(self.grid_spacing), extra, radius: self.snap_radius_px / self.zoom as f64 };
        self.edit.angle_step = ANGLE_STEP;
    }

    /// Snap a drawing tool's point and remember what it caught, for the marker.
    fn snap(&mut self, p: Pt) -> Pt {
        let (q, kind) = self.edit.snap_free(&self.shapes, p);
        self.snap_mark = kind.map(|k| (q, k));
        q
    }

    fn view(&self) -> View {
        View { tolerance: 8.0 / self.zoom as f64, handle_pad: 4.0 / self.zoom as f64 }
    }

    fn edit_tool(&self) -> EditTool {
        if self.tool == DrawTool::NodeEdit {
            EditTool::Nodes
        } else {
            EditTool::Select
        }
    }

    // ---------------- UI ----------------

    fn do_undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.shapes, s));
            self.edit.retain_existing(&self.shapes);
            // the toolpaths a paste made go away with its shapes
            while let Some((ids, before)) = self.paste_log.last() {
                if ids.iter().all(|id| !self.shapes.iter().any(|s| s.id == *id)) {
                    self.toolpaths.truncate((*before).min(self.toolpaths.len()));
                    self.tp_gen += 1;
                    self.tp_selected = None;
                    self.sim = None;
                    self.paste_log.pop();
                } else {
                    break;
                }
            }
            // bound sizes follow their variables again (to change those, change the variable)
            self.vars_dirty = true;
            self.recompute_all();
        }
    }

    fn do_redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.shapes, s));
            self.vars_dirty = true;
            self.recompute_all();
        }
    }

    fn properties(&mut self, ui: &mut egui::Ui) {
        match self.edit.selection.len() {
            0 => {
                ui.label(egui::RichText::new(t!("Nichts ausgewählt. Klicke eine Form an oder ziehe einen Rahmen.")).color(muted()));
                return;
            }
            1 => {}
            n => {
                ui.label(t!("%{n} Formen ausgewählt", n = n));
                return;
            }
        }
        let id = self.edit.selection[0];
        let Some(idx) = self.shapes.iter().position(|s| s.id == id) else { return };
        let before = self.shapes.clone();
        if crate::props::edit(ui, &mut self.shapes[idx].shape) {
            // typed in the panel: a plain number now, a formula would only undo it
            self.shapes[idx].bound.clear();
            if !ui.ctx().input(|i| i.pointer.any_down()) {
                self.undo.push(before);
                self.redo.clear();
            }
        }
    }

    fn open_editor(&mut self, kind: &'static str) {
        let ids: Vec<u64> = self.edit.selection.clone();
        let tp = default_toolpath(kind, &self.lib, &self.job, ids);
        self.editor = Some(Editor::new(tp, None, kind));
        if self.edit.selection.is_empty() {
            self.status = "Tipp: zuerst Vektoren auswählen, dann den Werkzeugweg berechnen".into();
        }
    }

    fn start_preview(&mut self, animate: bool) {
        self.mode = ViewMode::ThreeD;
        self.start_page = false;
        let mut hm = HeightMap::for_preview(&self.job, self.sim_quality);
        if animate {
            self.sim = Some(Sim::new(hm, Some((0, 0))));
        } else {
            for &i in &self.preview_indices() {
                hm.carve(&self.toolpaths[i], None);
            }
            self.sim = Some(Sim::new(hm, None));
        }
    }

    /// The toolpaths the preview simulates, by the chosen preview type.
    fn preview_indices(&self) -> Vec<usize> {
        match self.preview_scope {
            PreviewScope::All => (0..self.toolpaths.len()).collect(),
            PreviewScope::Visible => (0..self.toolpaths.len()).filter(|&i| self.toolpaths[i].visible).collect(),
        }
    }

    /// The toolpaths the export writes (only those with something to cut: one whose shapes are all
    /// on hidden layers stays out).
    fn output_indices(&self) -> Vec<usize> {
        let cuts = |t: &Toolpath| t.moves.iter().any(|m| !m.rapid);
        match self.output_scope {
            OutputScope::All => (0..self.toolpaths.len()).filter(|&i| cuts(&self.toolpaths[i])).collect(),
            OutputScope::Visible => (0..self.toolpaths.len()).filter(|&i| self.toolpaths[i].visible && cuts(&self.toolpaths[i])).collect(),
        }
    }

    /// Which toolpaths were exported, for the header of the file ("Sichtbare Werkzeugwege (4)").
    fn export_scope_text(&self, count: usize) -> String {
        let what = match self.output_scope {
            OutputScope::All => t!("Alle Werkzeugwege"),
            OutputScope::Visible => t!("Sichtbare Werkzeugwege"),
        };
        format!("{what} ({count})")
    }

    /// Whether "all" and "visible" export different toolpaths: only when a hidden one has something
    /// to cut.
    pub(super) fn output_scope_matters(&self) -> bool {
        self.toolpaths.iter().any(|t| !t.visible && t.moves.iter().any(|m| !m.rapid))
    }

    /// Advance the simulation by `dt` seconds of the screen's time: the machine's own time passes
    /// `sim_rate` times as fast (1 is real time). Moves are cut as far as their time allows, so a
    /// long cut is carved gradually, not all at once.
    fn step_animation(&mut self, dt: f64) {
        let visible: Vec<usize> = self.preview_indices();
        let rate = self.sim_rate;
        let Some(sim) = &mut self.sim else { return };
        let Some((k, done)) = sim.anim else { return };
        let Some(&ti) = visible.get(k) else {
            sim.anim = None;
            return;
        };
        let tp = &self.toolpaths[ti];
        sim.budget += dt.min(0.25) * rate;
        let start = sim.pos.unwrap_or_else(|| tp.moves.first().map_or([0.0; 3], |m| m.to));
        let mut part = vec![Move { to: start, rapid: true }];
        let (mut at, mut frac, mut pos) = (done, sim.frac, start);
        while at < tp.moves.len() {
            let m = tp.moves[at];
            let secs = tp.move_seconds(at);
            let left = secs * (1.0 - frac);
            if left <= sim.budget {
                sim.budget -= left;
                part.push(m);
                pos = m.to;
                at += 1;
                frac = 0.0;
            } else {
                // the time runs out inside this move: cut a part of it
                frac = (frac + sim.budget / secs).min(1.0);
                let from = if at == 0 { m.to } else { tp.moves[at - 1].to };
                pos = [0, 1, 2].map(|i| from[i] + (m.to[i] - from[i]) * frac);
                part.push(Move { to: pos, rapid: m.rapid });
                sim.budget = 0.0;
                break;
            }
        }
        sim.hm.carve(&tp.with_moves(part), None);
        if at >= tp.moves.len() {
            sim.pos = None;
            sim.frac = 0.0;
            sim.anim = if k + 1 >= visible.len() { None } else { Some((k + 1, 0)) };
        } else {
            sim.pos = Some(pos);
            sim.frac = frac;
            sim.anim = Some((k, at));
        }
    }

    fn start_page_ui(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, canvas());
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect.shrink(40.0)), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.label(egui::RichText::new(t!("Willkommen bei CncCraft")).size(28.0).strong().color(ink()));
                ui.label(egui::RichText::new(t!("Das komplette Werkzeugset für CNC-Einsteiger")).color(muted()));
                ui.add_space(30.0);
            });
            ui.horizontal(|ui| {
                let total = 3.0 * 220.0 + 2.0 * 20.0;
                ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
                for (title, sub, which) in [
                    (t!("Neue Datei erstellen"), t!("Materialgröße und Dicke festlegen"), 0),
                    (t!("Datei öffnen"), t!("Ein gespeichertes Projekt laden"), 1),
                    (t!("Beispiel öffnen"), t!("Schild mit Schrift, Tasche und Bohrungen"), 2),
                ] {
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(220.0, 170.0), Sense::click());
                    let hover = resp.hovered();
                    ui.painter().rect(r, 10.0, crate::theme::card(), Stroke::new(if hover { 2.0_f32 } else { 1.0_f32 }, if hover { ACCENT } else { crate::theme::card_edge() }));
                    let ic = Rect::from_center_size(Pos2::new(r.center().x, r.top() + 55.0), Vec2::splat(56.0));
                    ui.painter().rect_filled(ic, 10.0, if hover { ACCENT } else { Color32::from_rgb(0xff, 0xf3, 0xcc) });
                    let p = ui.painter();
                    let c = ic.center();
                    // the tile is pale yellow in both themes, so its drawing is the dark ink in both
                    let st = Stroke::new(2.5_f32, INK);
                    match which {
                        0 => {
                            p.rect_stroke(Rect::from_center_size(c, Vec2::new(26.0, 32.0)), 2.0, st);
                            p.line_segment([c + Vec2::new(-6.0, 0.0), c + Vec2::new(6.0, 0.0)], st);
                            p.line_segment([c + Vec2::new(0.0, -6.0), c + Vec2::new(0.0, 6.0)], st);
                        }
                        1 => {
                            p.rect_stroke(Rect::from_center_size(c + Vec2::new(0.0, 3.0), Vec2::new(32.0, 22.0)), 2.0, st);
                            p.line_segment([c + Vec2::new(-16.0, -8.0), c + Vec2::new(-4.0, -8.0)], st);
                        }
                        _ => {
                            p.text(c, Align2::CENTER_CENTER, "Aa", FontId::proportional(24.0), INK);
                        }
                    }
                    p.text(Pos2::new(r.center().x, r.top() + 110.0), Align2::CENTER_CENTER, title, FontId::proportional(16.0), ink());
                    p.text(Pos2::new(r.center().x, r.top() + 135.0), Align2::CENTER_CENTER, sub, FontId::proportional(11.5), muted());
                    if resp.clicked() {
                        match which {
                            0 => self.guarded(None, Pending::NewJob),
                            1 => self.guarded(None, Pending::Open),
                            _ => self.guarded(None, Pending::Example),
                        }
                    }
                    ui.add_space(20.0);
                }
            });

            // Recent files list
            let mut open_recent: Option<PathBuf> = None;
            if !self.recent_files.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(t!("Zuletzt verwendet")).size(16.0).strong().color(ink()));
                });
                ui.add_space(8.0);
                let list_width = 500.0_f32.min(ui.available_width() - 40.0);
                ui.horizontal(|ui| {
                    ui.add_space(((ui.available_width() - list_width) / 2.0).max(0.0));
                    ui.vertical(|ui| {
                        ui.set_width(list_width);
                        for path in &self.recent_files {
                            let name = path.file_stem().map_or_else(|| "?".into(), |s| s.to_string_lossy().into_owned());
                            let dir = path.parent().map_or(String::new(), |p| p.to_string_lossy().into_owned());
                            let thumb = thumb_path(path);
                            let frame = egui::Frame::none()
                                .fill(crate::theme::card())
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 6.0));
                            let row = frame.show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    let tex_id = egui::Id::new(("thumb_tex", path.as_os_str()));
                                    let loaded = ui.data(|d| d.get_temp::<egui::TextureId>(tex_id));
                                    let tid = loaded.unwrap_or_else(|| {
                                        if let Ok(img) = image::open(&thumb) {
                                            let rgba = img.to_rgba8();
                                            let size = [rgba.width() as usize, rgba.height() as usize];
                                            let full_size_id = egui::Id::new(("thumb_size", path.as_os_str()));
                                            ui.data_mut(|d| d.insert_temp(full_size_id, [size[0] as f32, size[1] as f32]));
                                            let ci = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                                            let t = ui.ctx().load_texture(format!("thumb_{}", path.display()), ci, egui::TextureOptions::LINEAR);
                                            let id = t.id();
                                            ui.data_mut(|d| d.insert_temp(tex_id, id));
                                            ui.data_mut(|d| d.insert_temp(egui::Id::new(("thumb_handle", path.as_os_str())), t));
                                            id
                                        } else {
                                            egui::TextureId::default()
                                        }
                                    });
                                    if tid != egui::TextureId::default() {
                                        let th = 80.0_f32;
                                        let tw = th * 16.0 / 9.0;
                                        let img_resp = ui.add(egui::Image::new(egui::load::SizedTexture::new(tid, Vec2::new(tw, th))).rounding(6.0));
                                        let full_size_id = egui::Id::new(("thumb_size", path.as_os_str()));
                                        let full_size = ui.data(|d| d.get_temp::<[f32; 2]>(full_size_id)).unwrap_or([tw, th]);
                                        img_resp.on_hover_ui(|ui| {
                                            ui.add(egui::Image::new(egui::load::SizedTexture::new(tid, Vec2::new(full_size[0], full_size[1]))).rounding(6.0));
                                        });
                                    } else {
                                        ui.label(egui::RichText::new("📄").size(32.0));
                                    }
                                    ui.add_space(6.0);
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&name).strong().color(ink()));
                                        ui.label(egui::RichText::new(&dir).small().color(muted()));
                                    });
                                });
                            });
                            let resp = ui.interact(row.response.rect, egui::Id::new(("recent", path.as_os_str())), Sense::click());
                            if resp.hovered() {
                                ui.painter().rect_stroke(row.response.rect, 6.0, Stroke::new(1.5_f32, ACCENT));
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if resp.clicked() {
                                open_recent = Some(path.clone());
                            }
                            ui.add_space(2.0);
                        }
                    });
                });
            }
            if let Some(path) = open_recent {
                self.open_path(path);
            }
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = resp.rect;
        self.canvas_rect = rect;
        if !self.fitted {
            self.fit(rect);
        }
        // zoom and pan
        if resp.hovered() {
            let (scroll, zoom_delta, pos) = ui.input(|i| (i.smooth_scroll_delta, i.zoom_delta(), i.pointer.hover_pos()));
            let mut factor = zoom_delta;
            if scroll.y != 0.0 && !ui.input(|i| i.modifiers.shift) {
                factor *= (scroll.y * 0.003).exp();
            }
            if factor != 1.0 {
                if let Some(p) = pos {
                    let before = self.to_mm(p);
                    self.zoom = (self.zoom * factor).clamp(0.05, 200.0);
                    let after = self.to_screen(before);
                    self.origin += p - after;
                }
            }
        }
        if resp.dragged_by(egui::PointerButton::Middle) || resp.dragged_by(egui::PointerButton::Secondary) {
            self.origin += resp.drag_delta();
        }
        let mouse = resp.hover_pos().map(|p| self.to_mm(p));
        self.mouse_mm = mouse;

        // the worksheet under the design (board, grid…), each a decoration of its own
        self.sheet.paint(sheet::Layer::Under, &self.sheet_view(rect), &painter);

        // shapes
        let view = self.view();
        let hover_id = if self.tool == DrawTool::Select && !self.edit.is_busy() { mouse.and_then(|m| self.edit.shape_under(&self.shapes, m, view)) } else { None };
        // a shape that came from a file offers, on a right click, to be read from disk again, and one
        // in a group to be ungrouped; the shape under the pointer is looked at once, when the menu opens
        if resp.secondary_clicked() {
            self.shape_menu = mouse
                .and_then(|m| self.edit.shape_under(&self.shapes, m, view))
                .and_then(|id| self.shapes.iter().find(|s| s.id == id))
                .map(|s| ShapeMenu { id: s.id, source: s.source.clone(), grouped: !s.groups.is_empty() })
                .filter(|m| m.source.is_some() || m.grouped);
        }
        if let Some(menu) = self.shape_menu.clone() {
            let (mut reload, mut ungroup) = (false, false);
            resp.context_menu(|ui| {
                if let Some(source) = &menu.source {
                    let name = std::path::Path::new(&source.file).file_name().map_or_else(|| source.file.clone(), |n| n.to_string_lossy().into_owned());
                    ui.label(egui::RichText::new(t!("%{format}-Import: %{name}", format = source.format, name = name)).small().color(muted()));
                    if ui.button(t!("Neu von Platte laden")).on_hover_text(t!("Liest die Datei noch einmal ein und ersetzt die importierten Formen")).clicked() {
                        reload = true;
                        ui.close_menu();
                    }
                }
                if menu.source.is_some() && menu.grouped {
                    ui.separator();
                }
                if menu.grouped && ui.button(t!("Gruppe auflösen")).on_hover_text(t!("Löst die äußerste Gruppe auf; eine Gruppe darin bleibt")).clicked() {
                    ungroup = true;
                    ui.close_menu();
                }
            });
            if reload || ungroup {
                self.shape_menu = None;
            }
            if let Some(source) = menu.source.as_ref().filter(|_| reload) {
                self.reload_import(&source.file, None);
            }
            if ungroup {
                self.ungroup(&[menu.id]);
            }
        }
        // always, as in Fusion 360: edges that are still free are blue, edges whose length was set
        // (a size typed in or from a variable, a distance on the shape) black
        let mark_free = true;
        let free_edge = Color32::from_rgb(0x4f, 0x86, 0xd6);
        // a ring or a line shows its selection as a solid line, not in marching dashes
        let solid_selection = self.edit.style(&self.shapes) != shape_edit::SelectionStyle::Box;
        for s in self.shapes.iter().filter(|s| !s.hidden) {
            let sel = self.edit.selection.contains(&s.id);
            let base = if mark_free { free_edge } else { Color32::BLACK };
            let col = if sel { SELECT } else if Some(s.id) == hover_id { Color32::from_rgb(0xd0, 0x40, 0x40) } else { base };
            for c in s.shape.contours() {
                let mut pts: Vec<Pos2> = c.pts.iter().map(|p| self.to_screen(*p)).collect();
                if c.closed && !pts.is_empty() {
                    pts.push(pts[0]);
                }
                if sel && solid_selection {
                    painter.add(egui::Shape::line(pts, Stroke::new(2.0_f32, col)));
                } else if sel {
                    painter.extend(egui::Shape::dashed_line(&pts, Stroke::new(1.5_f32, col), 6.0, 4.0));
                } else {
                    painter.add(egui::Shape::line(pts, Stroke::new(1.3_f32, col)));
                }
            }
            if mark_free {
                for (p, q) in shape_edit::constraints::fixed_edges(s, &self.distances) {
                    painter.line_segment([self.to_screen(p), self.to_screen(q)], Stroke::new(2.0_f32, Color32::BLACK));
                }
            }
        }

        // the selection, as the selected shape's class shows it: a box with eight handles, a ring
        // with four, or the two ends of a line
        let style = self.edit.style(&self.shapes);
        let visible_handles = self.edit.visible_handles(&self.shapes);
        if style == shape_edit::SelectionStyle::Ends && self.tool == DrawTool::Select {
            let hover = mouse.and_then(|m| self.edit.end_under(&self.shapes, m, view));
            let dragging = matches!(self.edit.gesture(), Some(Gesture::Node { .. }));
            for (id, index, at) in self.edit.end_handles(&self.shapes) {
                let on = dragging && self.edit.active_node == Some((id, index)) || hover == Some((id, index));
                painter.circle(self.to_screen(at), if on { 6.5 } else { 5.0 }, if on { ACCENT } else { Color32::WHITE }, Stroke::new(1.5_f32, SELECT));
            }
            if hover.is_some() || dragging {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            } else if hover_id.is_some() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
            }
        }
        if let Some((lo, hi)) = selection_bounds(&self.shapes, &self.edit.selection).filter(|_| style != shape_edit::SelectionStyle::Ends) {
            let pad = view.handle_pad;
            let r = Rect::from_two_pos(self.to_screen([lo[0] - pad, lo[1] - pad]), self.to_screen([hi[0] + pad, hi[1] + pad]));
            if style == shape_edit::SelectionStyle::Box {
                painter.rect_stroke(r, 0.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0xe0, 0x20, 0xa0, 120)));
            }
            // sides whose size is tied to a variable are constraints: solid blue, no handle works there
            let (lock_x, lock_y) = self.edit.locked_axes(&self.shapes);
            let constraint = Stroke::new(2.0_f32, CONSTRAINT);
            let box_style = style == shape_edit::SelectionStyle::Box;
            if lock_x && box_style {
                painter.line_segment([r.left_top(), r.left_bottom()], constraint);
                painter.line_segment([r.right_top(), r.right_bottom()], constraint);
            }
            if lock_y && box_style {
                painter.line_segment([r.left_top(), r.right_top()], constraint);
                painter.line_segment([r.left_bottom(), r.right_bottom()], constraint);
            }
            if self.tool == DrawTool::Select {
                let active = match self.edit.gesture() {
                    Some(Gesture::Resize { handle, .. }) => Some(*handle),
                    _ => None,
                };
                let hover = active.or_else(|| mouse.and_then(|m| self.edit.handle_under(&self.shapes, m, view)));
                let rotating = self.edit.rotate_mode;
                for h in visible_handles.iter().copied() {
                    let p = self.to_screen(handle_point(lo, hi, pad, h));
                    let on = hover == Some(h);
                    let corner = h.axes() == (true, true);
                    if rotating && corner {
                        // rotate mode: open squares on the corners turn the selection
                        painter.rect(Rect::from_center_size(p, Vec2::splat(if on { 13.0 } else { 11.0 })), 0.0, if on { ACCENT } else { Color32::WHITE }, Stroke::new(2.5_f32, INK));
                    } else if !rotating {
                        let (ax, ay) = h.axes();
                        if (!ax || lock_x) && (!ay || lock_y) {
                            // nothing to drag here: a blue diamond instead of a square handle
                            let d = 5.5;
                            painter.add(egui::Shape::convex_polygon(vec![p + Vec2::new(0.0, -d), p + Vec2::new(d, 0.0), p + Vec2::new(0.0, d), p + Vec2::new(-d, 0.0)], CONSTRAINT, Stroke::new(1.0_f32, Color32::WHITE)));
                        } else {
                            painter.rect(Rect::from_center_size(p, Vec2::splat(if on { 9.0 } else { 7.0 })), 0.0, if on { ACCENT } else { Color32::WHITE }, Stroke::new(1.0_f32, SELECT));
                        }
                    }
                }
                if rotating {
                    painter.rect_filled(Rect::from_center_size(r.center(), Vec2::splat(7.0)), 0.0, INK);
                    if hover.is_some_and(|h| h.axes() == (true, true)) || self.edit.rotation_angle().is_some() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    }
                } else if let Some(h) = hover.filter(|h| {
                    let (ax, ay) = h.axes();
                    (!ax || lock_x) && (!ay || lock_y)
                }) {
                    // a locked side: say so instead of offering a resize
                    let _ = h;
                    ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
                } else if let Some(h) = hover {
                    ui.ctx().set_cursor_icon(match h {
                        Handle::TopLeft | Handle::BottomRight => egui::CursorIcon::ResizeNwSe,
                        Handle::TopRight | Handle::BottomLeft => egui::CursorIcon::ResizeNeSw,
                        Handle::Left | Handle::Right => egui::CursorIcon::ResizeHorizontal,
                        Handle::Top | Handle::Bottom => egui::CursorIcon::ResizeVertical,
                    });
                } else if hover_id.is_some() || matches!(self.edit.gesture(), Some(Gesture::Move { .. })) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
                }
                // the shape's own grips (its "selected decoration"): small round handles
                let grip_hover = mouse.and_then(|m| self.edit.grip_under(&self.shapes, m, view));
                let dragging = matches!(self.edit.gesture(), Some(Gesture::Grip { .. }));
                for g in self.edit.grips(&self.shapes) {
                    let on = dragging || grip_hover == Some(g);
                    painter.circle(self.to_screen(g.at), if on { 5.5 } else { 4.5 }, if on { ACCENT } else { Color32::WHITE }, Stroke::new(1.2_f32, SELECT));
                }
                if grip_hover.is_some() || dragging {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            }
            // the centre mark: a handle that moves the selection (the original's centre image is
            // the rotation centre, `Handles/handle_rotation_centre.png`; moving by it is ours)
            let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
            let on_centre = self.tool == DrawTool::Select && !self.edit.rotate_mode && mouse.is_some_and(|m| (m[0] - centre[0]).hypot(m[1] - centre[1]) <= view.tolerance);
            if on_centre {
                painter.circle(r.center(), 5.0, ACCENT, Stroke::new(1.2_f32, SELECT));
                ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
            } else {
                painter.circle_stroke(r.center(), 4.0, Stroke::new(1.0_f32, SELECT));
            }
        }

        // node handles on the selected curves (spline handles, path squares)
        self.draw_node_handles(&painter, mouse, view);
        self.draw_vector_ops(&painter, mouse);

        // a click in the drawing while sizes are being typed takes them, like Enter; that press
        // does nothing else until the button is let go
        if self.label_press_step(ui.ctx()) || self.accept_distance_on_press(ui.ctx(), rect) || self.accept_dims_on_press(ui.ctx(), rect) {
            self.swallow_press = true;
        }
        if self.swallow_press {
            if ui.input(|i| !i.pointer.any_down()) {
                self.swallow_press = false;
            }
        } else {
            self.canvas_input(&resp, mouse);
        }

        // a shape dragged just now stands where the pointer put it, not where its constraints want it:
        // hold them again before the dimensions are drawn (else they show the dragged state in red)
        self.apply_constraints();

        // what the pointer snapped to
        if let Some((q, kind)) = self.edit.last_snap.or(self.snap_mark) {
            let c = self.to_screen(q);
            let col = Color32::from_rgb(0x1e, 0x6f, 0xd9);
            let line = Stroke::new(1.5_f32, col);
            match kind {
                SnapKind::Centre => {
                    painter.circle_stroke(c, 5.0, line);
                }
                SnapKind::MidPoint => {
                    painter.add(egui::Shape::closed_line(vec![c + Vec2::new(0.0, -6.0), c + Vec2::new(6.0, 4.0), c + Vec2::new(-6.0, 4.0)], line));
                }
                SnapKind::Grid => {
                    painter.line_segment([c + Vec2::new(-6.0, 0.0), c + Vec2::new(6.0, 0.0)], line);
                    painter.line_segment([c + Vec2::new(0.0, -6.0), c + Vec2::new(0.0, 6.0)], line);
                }
                SnapKind::Node | SnapKind::Job => {
                    painter.rect_stroke(Rect::from_center_size(c, Vec2::splat(10.0)), 0.0, line);
                }
            }
            let label = match kind {
                SnapKind::Node => t!("Knoten"),
                SnapKind::Centre => t!("Mitte"),
                SnapKind::MidPoint => t!("Halbe Strecke"),
                SnapKind::Job => t!("Job-Punkt"),
                SnapKind::Grid => t!("Raster"),
            };
            painter.text(c + Vec2::new(9.0, -9.0), Align2::LEFT_BOTTOM, label, FontId::proportional(11.0), col);
        }

        // rubber band and the shape being drawn
        if let Some((a, b)) = self.edit.band() {
            let r = Rect::from_two_pos(self.to_screen(a), self.to_screen(b));
            painter.rect(r, 0.0, Color32::from_rgba_unmultiplied(0xff, 0xc0, 0x00, 30), Stroke::new(1.0_f32, ACCENT_DEEP));
        }
        if let Some(d) = &self.drawing {
            if let Some(shape) = self.drawn_shape(d.start, d.end) {
                for c in shape.contours() {
                    let mut pts: Vec<Pos2> = c.pts.iter().map(|p| self.to_screen(*p)).collect();
                    if c.closed && !pts.is_empty() {
                        pts.push(pts[0]);
                    }
                    painter.extend(egui::Shape::dashed_line(&pts, Stroke::new(1.2_f32, Color32::from_rgb(0x20, 0x60, 0xd0)), 5.0, 3.0));
                }
                // the shape's own dimension lines, with the sizes so far
                self.decoration_preview(&painter, &shape);
            }
        }
        if !self.poly.is_empty() {
            let draw_col = Color32::from_rgb(0x20, 0x60, 0xd0);
            if self.tool == DrawTool::Spline && self.poly.len() >= 2 {
                let mut all_pts = self.poly.clone();
                if let Some(m) = resp.hover_pos() {
                    all_pts.push(self.to_mm(m));
                }
                let preview_spline = shape_edit::Spline::from_points(&all_pts, false);
                let eval = preview_spline.evaluate();
                let screen: Vec<Pos2> = eval.iter().map(|p| self.to_screen(*p)).collect();
                painter.add(egui::Shape::line(screen, Stroke::new(1.3_f32, draw_col)));
            } else {
                let mut pts: Vec<Pos2> = self.poly.iter().map(|p| self.to_screen(*p)).collect();
                if let Some(m) = resp.hover_pos() {
                    pts.push(m);
                }
                painter.add(egui::Shape::line(pts, Stroke::new(1.3_f32, draw_col)));
            }
            for p in &self.poly {
                painter.circle_filled(self.to_screen(*p), 3.0, draw_col);
            }
        }
        // dimension lines, when switched on; then the typed values on the canvas: sizes of a new
        // shape, angle in rotate mode
        self.draw_dimensions(&painter);
        self.draw_distances(&painter, resp.hover_pos());
        self.draw_aligns(&painter);
        self.draw_corners(&painter);
        self.distance_edit_ui(ui.ctx(), &painter);
        self.dims_ui(ui.ctx(), &painter);
        self.corner_edit_ui(ui.ctx(), &painter);
        self.angle_ui(ui.ctx(), &painter);
        // a tool cursor of our own over the drawing (hiding the system pointer)
        self.custom_cursor(ui.ctx(), &painter, rect, resp.hovered());
        // the worksheet over the design (rulers, axis cross…), with the pointer as it is now
        self.sheet.paint(sheet::Layer::Over, &self.sheet_view(rect), &painter);
    }

    /// The worksheet as its decorations see it.
    fn sheet_view(&self, canvas: Rect) -> sheet::SheetView {
        let datum = match self.job.datum {
            Datum::LowerLeft => [0.0, 0.0],
            Datum::Center => [self.job.width / 2.0, self.job.height / 2.0],
        };
        sheet::SheetView { canvas, zoom: self.zoom, origin: self.origin, width: self.job.width, height: self.job.height, datum, pointer: self.mouse_mm }
    }

    /// The shape a drawing tool makes from a drag between `a` and `b`.
    fn drawn_shape(&self, a: Pt, b: Pt) -> Option<Shape> {
        if (b[0] - a[0]).abs().max((b[1] - a[1]).abs()) < 0.5 {
            return None;
        }
        // each shape class decides how a drag makes it
        Some(match self.tool {
            DrawTool::Rect => RectShape::from_drag(a, b).into(),
            DrawTool::Ellipse => Ellipse::from_drag(a, b).into(),
            DrawTool::Circle => Circle::from_drag(a, b).into(),
            DrawTool::Polygon => Polygon::from_drag(a, b).into(),
            _ => return None,
        })
    }

    fn canvas_input(&mut self, resp: &Response, mouse: Option<Pt>) {
        // the level/upright mode takes the clicks the same way (a drag still moves shapes)
        if self.align_mode.is_some() {
            let dragging_shape = resp.drag_started_by(egui::PointerButton::Primary) || resp.dragged_by(egui::PointerButton::Primary) || resp.drag_stopped() || self.edit.is_busy();
            if !dragging_shape {
                self.align_input(resp, mouse);
            }
            if self.align_mode.is_some() && !dragging_shape {
                return;
            }
        }
        // the dimension mode takes the clicks (the picks of a distance), nothing is selected or moved
        // (a drag still moves and resizes shapes, as in Fusion: only clicks pick points and lines)
        if self.dim_mode.is_some() {
            let dragging_shape = resp.drag_started_by(egui::PointerButton::Primary) || resp.dragged_by(egui::PointerButton::Primary) || resp.drag_stopped() || self.edit.is_busy();
            if !dragging_shape {
                self.dimension_input(resp, mouse);
            }
            if self.dim_mode.is_some() && !dragging_shape {
                return;
            }
        }
        let (additive, uniform, press_origin) = resp.ctx.input(|i| (i.modifiers.shift || i.modifiers.command, i.modifiers.shift, i.pointer.press_origin()));
        // where the button went down: egui only reports a drag once the pointer has moved a few
        // pixels, and by then it may already be off the handle
        let press = press_origin.map(|p| self.to_mm(p));
        let pointer = resp.interact_pointer_pos().or(resp.hover_pos()).map(|p| self.to_mm(p));
        let started = resp.drag_started_by(egui::PointerButton::Primary);
        let dragging = resp.dragged_by(egui::PointerButton::Primary);
        let view = self.view();
        let copy = resp.ctx.input(|i| i.modifiers.ctrl || i.modifiers.alt);
        self.update_snapping();
        // "Press SHIFT to disable snapping" while moving and drawing, as in the original
        let shapes_tool = !matches!(self.tool, DrawTool::Select | DrawTool::NodeEdit | DrawTool::Polyline | DrawTool::Spline);
        if uniform && (shapes_tool || matches!(self.edit.gesture(), Some(Gesture::Move { .. }))) {
            self.edit.snapping.radius = 0.0;
        }
        self.snap_mark = None;
        match self.tool {
            DrawTool::Select | DrawTool::NodeEdit => {
                let tool = self.edit_tool();
                if self.tool == DrawTool::NodeEdit && resp.double_clicked() {
                    if let Some(m) = mouse {
                        self.node_double_click(m, view);
                    }
                } else if started {
                    if let Some(p) = press.or(pointer) {
                        self.gesture_undo = self.edit.press(&self.shapes, p, view, tool, additive).edits();
                        if self.gesture_undo {
                            self.push_undo();
                        }
                    }
                } else if resp.clicked() {
                    if let Some(p) = mouse {
                        self.edit.press(&self.shapes, p, view, tool, additive);
                        self.edit.release(&self.shapes);
                    }
                }
                if dragging && self.edit.is_busy() {
                    if let Some(m) = pointer {
                        self.edit.drag(&mut self.shapes, m, uniform);
                    }
                }
                if resp.drag_stopped() && self.edit.is_busy() {
                    // "Hold CTRL to create copy": the originals stay, the moved shapes are new
                    let originals = match self.edit.gesture() {
                        Some(Gesture::Move { orig, .. }) if copy => Some(orig.clone()),
                        _ => None,
                    };
                    let outcome = self.edit.release(&self.shapes);
                    if let (Outcome::Moved, Some(orig)) = (outcome, originals) {
                        self.keep_originals(orig);
                        self.status = "Kopie erstellt".into();
                        self.gesture_undo = false;
                        return;
                    }
                    if self.gesture_undo && !outcome.changed_shapes() {
                        self.undo.pop();
                    }
                    self.gesture_undo = false;
                    match outcome {
                        Outcome::Moved => self.status = "Verschoben".into(),
                        // a size tied to a variable can't be dragged (the editor leaves it alone),
                        // so a resize or grip drag never touches a binding
                        Outcome::Resized => self.status = "Größe geändert (Umschalt: Seitenverhältnis halten)".into(),
                        Outcome::NodeMoved => {
                            self.drop_bindings_of_selection();
                            self.status = "Knoten verschoben".into();
                        }
                        Outcome::Rotated => self.status = "Gedreht (Umschalt: in Winkelschritten)".into(),
                        Outcome::Reshaped => self.status = "Form angepasst".into(),
                        Outcome::Selected if self.edit.rotate_mode => self.status = "Drehen: an einer Ecke ziehen oder Winkel eingeben · erneut klicken zum Skalieren".into(),
                        _ => {}
                    }
                    if outcome.changed_shapes() {
                        self.recompute_all();
                    }
                }
            }
            DrawTool::Polyline | DrawTool::Spline => {
                // Shift keeps the new segment on angle steps, otherwise the point snaps
                let mouse = mouse.map(|m| match self.poly.last() {
                    Some(&last) if uniform => shape_edit::snap_angle(last, m, ANGLE_STEP),
                    _ => self.snap(m),
                });
                if resp.double_clicked() {
                    self.finish_poly(false);
                } else if resp.clicked() {
                    if let Some(m) = mouse {
                        if self.poly.len() >= 3 && dist(m, self.poly[0]) * self.zoom as f64 <= 8.0 {
                            self.finish_poly(true);
                        } else {
                            self.poly.push(m);
                        }
                    }
                }
            }
            t if t.is_vector_op() => self.vector_op_input(resp, mouse),
            DrawTool::Text => {
                let mouse = mouse.map(|m| self.snap(m));
                if resp.clicked() {
                    if let Some(m) = mouse {
                        self.push_undo();
                        let id = self.add_shape(Text::new(m, t!("Text"), 20.0));
                        self.edit.selection = vec![id];
                        self.tool = DrawTool::Select;
                        self.status = "Text eingefügt – Inhalt unter „Eigenschaften“ ändern".into();
                    }
                }
            }
            _ => {
                if started {
                    if let Some(p) = press.or(pointer) {
                        let p = self.snap(p);
                        self.drawing = Some(Drawing { start: p, end: p });
                    }
                } else if !dragging {
                    // show what the next press would snap to
                    if let Some(m) = mouse {
                        self.snap(m);
                    }
                }
                if dragging {
                    if let Some(m) = pointer {
                        let m = self.snap(m);
                        if let Some(d) = &mut self.drawing {
                            d.end = m;
                        }
                    }
                }
                if resp.drag_stopped() {
                    if let Some(d) = self.drawing.take() {
                        if let Some(shape) = self.drawn_shape(d.start, d.end) {
                            self.push_undo();
                            let id = self.add_shape(shape);
                            self.edit.selection = vec![id];
                            // type the exact size straight away, as in the original
                            self.open_dims(id);
                        }
                    }
                }
            }
        }
    }

    fn finish_poly(&mut self, closed: bool) {
        let mut pts = std::mem::take(&mut self.poly);
        pts.dedup_by(|a, b| dist(*a, *b) < 1e-6);
        if pts.len() < 2 {
            return;
        }
        self.push_undo();
        let id = if self.tool == DrawTool::Spline {
            self.add_shape(shape_edit::Spline::from_points(&pts, closed))
        } else {
            self.add_shape(Path::new(pts, closed))
        };
        self.edit.selection = vec![id];
    }

    fn view3d(&mut self, ui: &mut egui::Ui) {
        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = resp.rect;
        self.canvas_rect = rect;
        // backdrop like the original's preview: soft vertical gradient
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(rect.left_top(), Color32::from_rgb(0xf7, 0xf8, 0xfa));
        mesh.colored_vertex(rect.right_top(), Color32::from_rgb(0xf7, 0xf8, 0xfa));
        mesh.colored_vertex(rect.right_bottom(), Color32::from_rgb(0xc9, 0xce, 0xd6));
        mesh.colored_vertex(rect.left_bottom(), Color32::from_rgb(0xc9, 0xce, 0xd6));
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        painter.add(mesh);
        if resp.dragged_by(egui::PointerButton::Primary) {
            let d = resp.drag_delta();
            self.cam.yaw -= d.x * 0.008;
            self.cam.pitch = (self.cam.pitch + d.y * 0.006).clamp(0.0, std::f32::consts::PI);
        }
        if resp.dragged_by(egui::PointerButton::Secondary) || resp.dragged_by(egui::PointerButton::Middle) {
            self.cam.pan += resp.drag_delta();
        }
        if resp.hovered() {
            let s = ui.input(|i| i.smooth_scroll_delta.y);
            self.cam.zoom = (self.cam.zoom * (s * 0.003).exp() * ui.input(|i| i.zoom_delta())).clamp(0.2, 20.0);
        }
        if self.sim.is_none() {
            self.sim = Some(Sim::new(HeightMap::for_preview(&self.job, self.sim_quality), None));
        }
        if self.sim.as_ref().is_some_and(|s| s.anim.is_some()) {
            let dt = ui.input(|i| i.unstable_dt) as f64;
            self.step_animation(dt);
            ui.ctx().request_repaint();
        }
        // the G-code overlay: the machine's moves as the post-processor writes them (not while the
        // cutter is animated, which shows the cutting itself)
        let overlay = self.paths_in_3d && !self.sim.as_ref().is_some_and(|s| s.anim.is_some());
        if overlay {
            self.refresh_gcode_overlay();
        }
        if let Some(sim) = &self.sim {
            let proj = sim::Projection::new(rect, &self.job, &self.cam);
            let layout = self.job.width.to_bits() ^ self.job.height.to_bits().rotate_left(21) ^ self.job.thickness.to_bits().rotate_left(42);
            self.gl3d.update(&sim.hm, &proj, layout);
            painter.add(self.gl3d.callback(rect, &proj));
            if overlay {
                self.paint_gcode_overlay(&painter, &proj);
            }
        }
        painter.text(rect.left_bottom() + Vec2::new(12.0, -12.0), Align2::LEFT_BOTTOM, t!("Ziehen: drehen · Rechtsklick ziehen: verschieben · Scrollen: zoomen · Würfel: Ansicht wählen"), FontId::proportional(12.0), MUTED);
        // the decorations of the 3D view (the view cube: turn the view, pick a side, home, perspective)
        let proj = sim::Projection::new(rect, &self.job, &self.cam);
        let visible = self.preview_indices();
        let cutter: Option<([f64; 3], &crate::tools::Tool)> = self.sim.as_ref().and_then(|s| {
            let pos = s.pos?;
            let (k, _) = s.anim?;
            let ti = *visible.get(k)?;
            Some((pos, &self.toolpaths[ti].tool))
        });
        let mut view = crate::viewdeco::View3dContext { area: rect, cam: &mut self.cam, proj: &proj, zero: crate::viewdeco::zero_point(&self.job), arrow_length: crate::viewdeco::arrow_length(&self.job), cutter, thickness: self.job.thickness as f32, renderer: &self.gl3d };
        if let Some(message) = self.view_decos.show(ui, &mut view).pop() {
            self.status = message;
        }
        if let Some((k, _)) = self.sim.as_ref().and_then(|s| s.anim) {
            painter.text(rect.left_top() + Vec2::new(12.0, 12.0), Align2::LEFT_TOP, t!("Fräse Werkzeugweg %{n} …", n = k + 1), FontId::proportional(14.0), INK);
        }
    }

    fn keys(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        let nudge = ctx.input(|i| {
            // ⌘⇧ + arrows: the fixed nudge distance, as the original's tip says; ⇧ alone: ten times
            let step = if i.modifiers.shift && !i.modifiers.command { NUDGE_STEP_BIG } else { NUDGE_STEP };
            let mut d = [0.0, 0.0];
            for (key, dx, dy) in [(egui::Key::ArrowLeft, -1.0, 0.0), (egui::Key::ArrowRight, 1.0, 0.0), (egui::Key::ArrowUp, 0.0, 1.0), (egui::Key::ArrowDown, 0.0, -1.0)] {
                if i.key_pressed(key) {
                    d = [d[0] + dx * step, d[1] + dy * step];
                }
            }
            d
        });
        if nudge != [0.0, 0.0] {
            let before = self.shapes.clone();
            let tool = self.edit_tool();
            if self.edit.nudge(&mut self.shapes, tool, nudge[0], nudge[1]) {
                self.push_undo_from(before);
            }
        }
        for action in ctx.input(shortcuts::pressed) {
            self.run(action);
        }
    }

    /// Do what a shortcut (or its menu entry) stands for.
    fn run(&mut self, action: Action) {
        match action {
            Action::NodeEdit => self.toggle_node_edit(),
            Action::Cancel => self.cancel(),
            Action::Apply if !self.poly.is_empty() => self.finish_poly(false),
            Action::ClosePolyline if self.poly.len() >= 3 => self.finish_poly(true),
            Action::NextLine if !self.poly.is_empty() => {
                // the tool stays on, so the next click starts a new line
                self.finish_poly(false);
                self.status = "Linie beendet – nächster Klick beginnt eine neue".into();
            }
            // Enter with one shape chosen: its sizes can be typed (with variables)
            Action::Apply if self.dims.is_none() && self.edit.selection.len() == 1 && self.tool == DrawTool::Select => self.open_selected_dims(),
            Action::Apply | Action::ClosePolyline | Action::NextLine => {}
            Action::Delete if self.delete_selected_constraint() => {}
            Action::Delete => {
                let before = self.shapes.clone();
                if self.tool == DrawTool::NodeEdit && self.edit.active_node.is_some() {
                    if self.edit.delete_active_node(&mut self.shapes) {
                        self.push_undo_from(before);
                        self.status = "Knoten gelöscht".into();
                        self.recompute_all();
                    }
                } else {
                    self.delete_selection();
                }
            }
            Action::Undo => self.do_undo(),
            Action::Redo => self.do_redo(),
            Action::SelectAll => self.edit.selection = self.shapes.iter().filter(|s| !s.hidden && !s.frozen).map(|s| s.id).collect(),
            Action::Group => self.group_selection(),
            Action::Ungroup => self.ungroup_selection(),
            Action::Duplicate => self.duplicate(),
            Action::Copy => self.copy_selection(),
            Action::Cut => {
                self.copy_selection();
                self.delete_selection();
            }
            Action::Paste => self.paste(),
            Action::New => self.guarded(None, Pending::NewJob),
            Action::Open => self.guarded(None, Pending::Open),
            Action::Save => self.save_project(false),
            Action::SaveAs => self.save_project(true),
            Action::View2D => self.mode = ViewMode::TwoD,
            Action::View3D => self.mode = ViewMode::ThreeD,
            Action::FitView => {
                self.fitted = false;
                self.cam = Camera { perspective: self.cam.perspective, ..Camera::default() };
            }
            Action::Dimension => self.toggle_dim_mode(),
            Action::Align => self.toggle_align_mode(),
            Action::Select => self.pick_tool(DrawTool::Select),
            Action::Line => self.pick_tool(DrawTool::Polyline),
            Action::SplineTool => self.pick_tool(DrawTool::Spline),
            Action::Rectangle => self.pick_tool(DrawTool::Rect),
            Action::Circle => self.pick_tool(DrawTool::Circle),
            Action::Offset => self.column_click(Icon::Offset),
            Action::Shortcuts => self.shortcuts_open = !self.shortcuts_open,
        }
    }

    /// A tool chosen by its key: the same as clicking it in the tool column.
    fn pick_tool(&mut self, tool: DrawTool) {
        self.dim_mode = None;
        self.align_mode = None;
        self.distance_edit = None;
        self.column_click(Icon::Draw(tool));
        self.status = tool.tip().into();
    }

    /// N: the selected vector into node editing (a drawn shape becomes a plain vector, as in
    /// the original), and back to selecting.
    fn toggle_node_edit(&mut self) {
        if self.tool == DrawTool::NodeEdit {
            self.tool = DrawTool::Select;
            self.status = DrawTool::Select.tip().into();
            return;
        }
        if self.edit.selection.is_empty() {
            self.status = "Erst einen Vektor auswählen, dann „N“ drücken".into();
            return;
        }
        let shapes_only = self.shapes.iter().any(|s| self.edit.selection.contains(&s.id) && s.nodes().is_none());
        if shapes_only {
            self.to_curves();
        }
        self.dims = None;
        self.edit.rotate_mode = false;
        self.tool = DrawTool::NodeEdit;
        self.status = DrawTool::NodeEdit.tip().into();
    }

    /// Esc: stop a drag (putting the shapes back), else leave the rotate mode, else drop a line
    /// being drawn, else go back to selecting, else clear the selection.
    fn cancel(&mut self) {
        if self.selected_constraint.take().is_some() {
            return;
        }
        if self.cancel_align_mode() {
            return;
        }
        if self.cancel_dim_mode() {
            return;
        }
        if self.edit.is_busy() {
            self.edit.cancel(&mut self.shapes);
            if self.gesture_undo {
                self.undo.pop();
                self.gesture_undo = false;
            }
        } else if self.edit.rotate_mode {
            self.edit.rotate_mode = false;
        } else if !self.poly.is_empty() || self.drawing.is_some() {
            self.poly.clear();
            self.drawing = None;
        } else if self.tool != DrawTool::Select {
            self.tool = DrawTool::Select;
        } else {
            self.edit.selection.clear();
        }
    }

    /// Gather what copying `ids` takes along: the shapes, the constraints that belong to a single
    /// one of them (width, height, diameter, point positions, level/upright) and the toolpaths
    /// that cut them.
    fn make_clip(&self, ids: &[u64]) -> Clip {
        let items: Vec<Item> = ids.iter().filter_map(|id| self.shapes.iter().find(|s| s.id == *id)).cloned().collect();
        let own = |a: u64, b: u64| a == b && ids.contains(&a);
        let distances = self.distances.iter().filter(|c| own(c.from.shape(), c.to.shape())).cloned().collect();
        let aligns = self.aligns.iter().filter(|c| own(c.a.shape(), c.b.shape())).cloned().collect();
        let toolpaths = self
            .toolpaths
            .iter()
            .filter(|tp| tp.shape_ids.iter().any(|id| ids.contains(id)))
            .map(|tp| Toolpath { shape_ids: tp.shape_ids.iter().copied().filter(|id| ids.contains(id)).collect(), ..tp.clone() })
            .collect();
        Clip { items, distances, aligns, toolpaths }
    }

    /// Give the copies (old id → new id) the constraints and toolpaths of what they were copied from;
    /// `shift` is how far the copies lie from the originals.
    fn add_extras(&mut self, clip: &Clip, map: &[(u64, u64)], shift: [f64; 2]) {
        let new = |id: u64| map.iter().find(|(old, _)| *old == id).map(|(_, n)| *n);
        let anchor = |a: shape_edit::Anchor| match a {
            shape_edit::Anchor::Point(id, k) => new(id).map(|n| shape_edit::Anchor::Point(n, k)),
            shape_edit::Anchor::Edge(id, k) => new(id).map(|n| shape_edit::Anchor::Edge(n, k)),
        };
        for c in &clip.distances {
            if let (Some(from), Some(to)) = (anchor(c.from), anchor(c.to)) {
                self.distances.push(shape_edit::Distance { from, to, place: [c.place[0] + shift[0], c.place[1] + shift[1]], ..c.clone() });
            }
        }
        for c in &clip.aligns {
            if let (Some(a), Some(b)) = (anchor(c.a), anchor(c.b)) {
                self.aligns.push(shape_edit::Align { a, b, ..c.clone() });
            }
        }
        let before = self.toolpaths.len();
        for tp in &clip.toolpaths {
            let ids: Vec<u64> = tp.shape_ids.iter().filter_map(|id| new(*id)).collect();
            if ids.is_empty() {
                continue;
            }
            let copy = Toolpath { shape_ids: ids, name: t!("%{name} (Kopie)", name = tp.name), ..tp.clone() };
            self.toolpaths.push(copy);
        }
        if self.toolpaths.len() != before {
            self.paste_log.push((map.iter().map(|(_, n)| *n).collect(), before));
            self.recompute_all();
        }
        self.vars_dirty = true;
    }

    fn copy_selection(&mut self) {
        let clip = self.make_clip(&self.edit.selection.clone());
        if !clip.is_empty() {
            self.status = format!("{} Objekt(e) kopiert, mit {} Werkzeugweg(en) und {} eigenen Constraint(s)", clip.items.len(), clip.toolpaths.len(), clip.distances.len() + clip.aligns.len());
            self.clipboard = clip;
        }
    }

    /// Paste the copied shapes a little offset, like a duplicate, and select them.
    fn paste(&mut self) {
        let clip = self.clipboard.clone();
        self.paste_clip(&clip);
    }

    /// Put the shapes of `clip` in, a little offset, with their own constraints and toolpaths.
    fn paste_clip(&mut self, clip: &Clip) {
        if clip.is_empty() {
            return;
        }
        self.push_undo();
        self.edit.selection.clear();
        let mut map = vec![];
        let first_copy = self.shapes.len();
        for it in &clip.items {
            let id = self.new_id();
            let mut copy = it.translated(10.0, -10.0);
            copy.id = id;
            // a copy stays on its layer (the original's paste data carries the layer,
            // `vcCadLayerPasteData`); a layer that is gone, hidden or locked would swallow it, so
            // it goes on the layer in use then
            let usable = self.layers.is_visible(it.layer) && !self.layers.is_locked(it.layer);
            copy.layer = if usable { it.layer } else { self.layers.active };
            self.layers.set_visible(copy.layer, true);
            self.shapes.push(copy);
            map.push((it.id, id));
            self.edit.selection.push(id);
        }
        // the copies of a group form a group of their own
        let mut pasted = self.shapes.split_off(first_copy);
        self.regroup_copies(&mut pasted);
        self.shapes.extend(pasted);
        self.add_extras(clip, &map, [10.0, -10.0]);
        self.status = format!("{} Objekt(e) eingefügt", map.len());
    }

    /// After a ⌃-drag: the moved shapes become copies and the originals go back where they were.
    fn keep_originals(&mut self, orig: Vec<Item>) {
        let mut copies = vec![];
        let mut map = vec![];
        for o in orig {
            let id = self.new_id();
            if let Some(moved) = self.shapes.iter_mut().find(|s| s.id == o.id) {
                moved.id = id;
                copies.push(id);
                map.push((o.id, id));
            }
            self.shapes.push(o);
        }
        // the copies of a group form a group of their own
        let mut moved: Vec<Item> = self.shapes.iter().filter(|s| copies.contains(&s.id)).cloned().collect();
        self.regroup_copies(&mut moved);
        for m in moved {
            if let Some(s) = self.shapes.iter_mut().find(|s| s.id == m.id) {
                s.groups = m.groups;
            }
        }
        // the copies bring their own constraints and the toolpaths of their originals
        let old: Vec<u64> = map.iter().map(|(o, _)| *o).collect();
        let clip = self.make_clip(&old);
        self.add_extras(&clip, &map, [0.0, 0.0]);
        self.edit.selection = copies;
    }

    fn save_thumbnail(&mut self, ctx: &egui::Context) {
        if self.thumb_pending == 0 || self.shot.is_some() {
            return;
        }
        let now = ctx.input(|i| i.time);
        if self.thumb_pending != 1 && now - self.thumb_started > 3.0 {
            // no picture came: give up, and let whatever waited for it go on
            self.thumb_pending = 0;
            if let Some(mode) = self.thumb_restore_mode.take() {
                self.mode = mode;
            }
            return;
        }
        if self.thumb_pending == 1 {
            self.thumb_started = now;
            // Switch to 3D and ensure a simulation exists
            self.thumb_restore_mode = Some(self.mode);
            self.mode = ViewMode::ThreeD;
            if self.sim.is_none() {
                self.start_preview(false);
            }
            self.thumb_pending = 2;
            ctx.request_repaint();
            return;
        }
        if self.thumb_pending == 2 {
            // Wait one frame for the 3D view to render
            self.thumb_pending = 3;
            ctx.request_repaint();
            return;
        }
        if self.thumb_pending == 3 {
            self.thumb_pending = 4;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
            ctx.request_repaint();
            return;
        }
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            self.thumb_pending = 0;
            // Restore the original view mode
            if let Some(mode) = self.thumb_restore_mode.take() {
                self.mode = mode;
            }
            let Some(path) = &self.path else { return };
            let thumb = thumb_path(path);
            let [w, h] = img.size;
            let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
            if let Some(full) = image::RgbaImage::from_raw(w as u32, h as u32, bytes) {
                let ppp = ctx.pixels_per_point();
                let r = self.canvas_rect;
                let x = (r.left() * ppp).round() as u32;
                let y = (r.top() * ppp).round() as u32;
                let cw = ((r.width() * ppp).round() as u32).min(w as u32 - x);
                let ch = ((r.height() * ppp).round() as u32).min(h as u32 - y);
                let cropped = image::imageops::crop_imm(&full, x, y, cw, ch).to_image();
                let tw = 320u32;
                let th = (tw as f32 * ch as f32 / cw as f32).round().max(1.0) as u32;
                let small = image::imageops::resize(&cropped, tw, th, image::imageops::FilterType::Triangle);
                let _ = small.save(&thumb);
            }
        }
    }

    fn screenshot(&mut self, ctx: &egui::Context) {
        let Some((path, frames)) = &mut self.shot else { return };
        *frames += 1;
        ctx.request_repaint();
        if *frames == 110 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
        }
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            let [w, h] = img.size;
            let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            match image::save_buffer(&*path, &bytes, w as u32, h as u32, image::ExtendedColorType::Rgba8) {
                Ok(()) => eprintln!("Bild gespeichert: {}", path.display()),
                Err(e) => eprintln!("Bild konnte nicht gespeichert werden: {e}"),
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if *frames > 400 {
            eprintln!("Kein Bild erhalten");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("cnccraft_theme", self.theme_choice.key().to_string());
        storage.set_string("cnccraft_lang", self.lang.code().to_string());
        storage.set_string("cnccraft_post_idx", self.post_idx.to_string());
        storage.set_string("cnccraft_gcode_in_3d", self.paths_in_3d.to_string());
        let paths: Vec<String> = self.recent_files.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        if let Ok(json) = serde_json::to_string(&paths) {
            storage.set_string("cnccraft_recent", json);
        }
        storage.set_string("cnccraft_tools_open", self.tools_open.to_string());
        storage.set_string("cnccraft_snap_geometry", self.snap_geometry.to_string());
        storage.set_string("cnccraft_snap_grid", self.snap_grid.to_string());
        storage.set_string("cnccraft_snap_job", self.snap_job.to_string());
        storage.set_string("cnccraft_grid_spacing", self.grid_spacing.to_string());
        storage.set_string("cnccraft_snap_radius", self.snap_radius_px.to_string());
        storage.set_string("cnccraft_show_dims", self.show_dims.to_string());
        storage.set_string("cnccraft_paths_open", self.paths_open.to_string());
        storage.set_string("cnccraft_sim_rate", self.sim_rate.to_string());
        storage.set_string("cnccraft_sim_quality", self.sim_quality.to_string());
        storage.set_string("cnccraft_preview_scope", format!("{:?}", self.preview_scope));
        storage.set_string("cnccraft_output_scope", format!("{:?}", self.output_scope));
        storage.set_string("cnccraft_export_per_tool", self.export_per_tool.to_string());
        storage.set_string("cnccraft_cam_perspective", self.cam.perspective.to_string());
        storage.set_string("cnccraft_tools", self.lib.to_json());
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.selftest_input(raw_input);
        self.take_tabs(raw_input);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.egui_ctx = Some(ctx.clone());
        self.sync_theme(ctx);
        // a layer was switched on or off: what is on it joins or leaves the picking and the cutting
        if self.layers.sync(&mut self.shapes) {
            self.edit.retain_existing(&self.shapes);
            self.recompute_all();
        }
        if self.selftest.is_some() {
            // the synthetic events come one per frame, so keep the frames coming
            ctx.request_repaint();
        }
        // closing the window (or quitting) with unsaved changes asks first
        if ctx.input(|i| i.viewport().close_requested()) && self.shot.is_none() && self.selftest.is_none() && self.is_dirty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.unsaved = Some(Pending::Close);
        }
        // the title bar shows whether there are unsaved changes (looked at twice a second)
        let now = ctx.input(|i| i.time);
        if now - self.dirty_checked > 0.5 {
            self.dirty_checked = now;
            self.dirty_shown = self.is_dirty();
        }
        crate::quit::set_dirty(self.dirty_shown);
        // "Quit" from the Dock (or ⌘Q) was held back by the system because of unsaved changes
        if crate::quit::take_request() && self.shot.is_none() && self.selftest.is_none() {
            if self.is_dirty() {
                self.unsaved = Some(Pending::Close);
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        self.keys(ctx);
        // a window that wants an answer first: nothing is left half typed above it
        if self.modal_open() {
            self.abort_inputs();
        }
        self.refresh_vars();
        self.apply_constraints();
        self.poll_bg_compute();
        if self.bg_compute.is_some() {
            ctx.request_repaint();
        }
        if self.rebase {
            self.rebase = false;
            self.saved_text = self.project_text();
        }
        // rotating shapes is not possible for now: it comes back later as an angle constraint
        self.edit.rotate_mode = false;
        // the arrangement of the original: see the layout module
        self.title_bar(ctx);
        self.layer_window(ctx);
        if !self.start_page {
            self.status_bar(ctx);
            self.tool_column(ctx);
            self.tool_form_panel(ctx);
            self.toolpath_panel(ctx);
        }
        // a tooltip with an icon and one without, held on screen to review their look
        if self.shot.is_some() && std::env::var("CNCCRAFT_SHOW_TIP").is_ok() {
            let tips = [
                (Pos2::new(320.0, 110.0), Tip::new(t!("Tasche"), t!("Räumt die Fläche innerhalb geschlossener Vektoren bis zur Zieltiefe aus: als Ringe von innen nach außen, hin und her im Raster oder als eine durchgehende Spirale. Flächen innerhalb der Tasche bleiben als Inseln stehen.")).icon(Icon::Pocket).big(), false),
                (Pos2::new(320.0, 260.0), layout::tool_tip(Icon::Union), true),
                (Pos2::new(320.0, 380.0), Tip::new(t!("Zustellung"), t!("So tief schneidet der Fräser je Durchgang. Die Zieltiefe wird in gleich große Durchgänge geteilt.")), false),
            ];
            for (i, (at, tip, disabled)) in tips.iter().enumerate() {
                egui::Area::new(egui::Id::new(("review_tip", i))).order(egui::Order::Tooltip).fixed_pos(*at).show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| if *disabled { tip.show_disabled(ui) } else { tip.show(ui) });
                });
            }
        }
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            if self.start_page {
                self.start_page_ui(ui);
                return;
            }
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            self.canvas_toolbar(ui);
            if self.mode == ViewMode::TwoD {
                self.canvas(ui);
            } else {
                self.view3d(ui);
            }
        });
        if !self.start_page {
            self.sheet_interact(ctx);
        }
        self.job_dialog_ui(ctx);
        self.variables_dialog(ctx);
        self.new_variable_dialog(ctx);
        self.export_dialog_ui(ctx);
        self.export_file_ui(ctx);
        self.unsaved_ui(ctx);
        self.about_ui(ctx);
        if self.shortcuts_open {
            dialogs::shortcuts::window(ctx, &mut self.shortcuts_open);
        }
        self.import_missing_ui(ctx);
        self.pp_dialog_ui(ctx);
        self.tool_dialog_ui(ctx);
        self.save_thumbnail(ctx);
        if self.thumb_pending == 0 {
            if let Some(what) = self.after_thumb.take() {
                self.run_pending(Some(ctx), what);
            }
        }
        self.screenshot(ctx);
    }
}

// ---------------- helpers ----------------

fn thumb_path(project: &std::path::Path) -> PathBuf {
    let mut p = project.as_os_str().to_owned();
    p.push(".thumb.png");
    PathBuf::from(p)
}

/// One labelled dimension text field (value shown in the dialog's units).
fn dim_row(ui: &mut egui::Ui, label: &str, v: &mut f64, f: f64, unit: &str, max: f64) {
    ui.horizontal(|ui| {
        ui.add_sized(Vec2::new(78.0, 20.0), egui::Label::new(egui::RichText::new(label).color(ink())));
        let mut text = format!("{:.3}", *v * f);
        // trim trailing zeros for a tidy field
        while text.contains('.') && (text.ends_with('0') || text.ends_with('.')) {
            text.pop();
        }
        let resp = ui.add_sized(Vec2::new(96.0, 22.0), egui::TextEdit::singleline(&mut text).horizontal_align(egui::Align::Min));
        if resp.changed() {
            if let Ok(parsed) = text.trim().replace(',', ".").parse::<f64>() {
                *v = (parsed / f).clamp(0.05, max);
            }
        }
        ui.label(egui::RichText::new(unit).color(muted()));
    });
}

/// A two-option segmented toggle (like the in/mm switch), primary side filled.
fn segmented<T: PartialEq + Copy>(ui: &mut egui::Ui, value: &mut T, options: [(T, String); 2]) -> bool {
    let mut changed = false;
    egui::Frame::none().fill(separator()).rounding(4.0).inner_margin(2.0).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (opt, label) in options {
                let on = *value == opt;
                let b = egui::Button::new(egui::RichText::new(label).color(if on { Color32::WHITE } else { ink() }))
                    .fill(if on { PRIMARY } else { Color32::TRANSPARENT })
                    .rounding(4.0)
                    .min_size(Vec2::new(58.0, 22.0));
                if ui.add(b).clicked() && !on {
                    *value = opt;
                    changed = true;
                }
            }
        });
    });
    changed
}

/// A small 3D block sketch with X/Y/Z marks, like the job illustration.
fn block_sketch(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(72.0, 60.0), Sense::hover());
    let p = ui.painter();
    let o = rect.left_top() + Vec2::new(8.0, 20.0);
    let w = 38.0;
    let h = 24.0;
    let d = Vec2::new(16.0, -11.0); // depth offset
    let fl = [o, o + Vec2::new(w, 0.0), o + Vec2::new(w, h), o + Vec2::new(0.0, h)];
    let top = [fl[0] + d, fl[1] + d, o + Vec2::new(w, 0.0), o];
    let wood = Color32::from_rgb(0xd8, 0xb8, 0x88);
    let line = Stroke::new(1.3_f32, Color32::from_gray(90));
    // top face
    p.add(egui::Shape::convex_polygon(vec![o, o + Vec2::new(w, 0.0), o + Vec2::new(w, 0.0) + d, o + d], wood.gamma_multiply(1.1), line));
    // right face
    p.add(egui::Shape::convex_polygon(vec![o + Vec2::new(w, 0.0), o + Vec2::new(w, h), o + Vec2::new(w, h) + d, o + Vec2::new(w, 0.0) + d], wood.gamma_multiply(0.82), line));
    // front face
    p.add(egui::Shape::convex_polygon(fl.to_vec(), wood, line));
    let _ = top;
    // axis hints
    p.text(o + Vec2::new(w / 2.0, h + 7.0), Align2::CENTER_CENTER, "X", FontId::proportional(9.0), muted());
    p.text(o + Vec2::new(w + d.x / 2.0 + 6.0, h / 2.0 + d.y / 2.0), Align2::CENTER_CENTER, "Y", FontId::proportional(9.0), muted());
    p.text(o + Vec2::new(-5.0, h / 2.0), Align2::CENTER_CENTER, "Z", FontId::proportional(9.0), muted());
}

fn tp_color(s: &Strategy) -> Color32 {
    match s {
        Strategy::Profile { tabs: true, .. } => Color32::from_rgb(0x1e, 0x6f, 0xd9),
        Strategy::Profile { .. } => Color32::from_rgb(0x2a, 0x8a, 0xe0),
        Strategy::Pocket { .. } => Color32::from_rgb(0x1f, 0x9d, 0x55),
        Strategy::Drill => Color32::from_rgb(0xd0, 0x30, 0x30),
        Strategy::VCarve { .. } => Color32::from_rgb(0xe0, 0x7a, 0x10),
    }
}

fn fmt_time(min: f64) -> String {
    let s = (min * 60.0).round() as i64;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

impl App {
    /// Pick the theme: follow the system, light or dark.
    /// Switch the interface language; it takes effect with the next frame.
    pub(super) fn set_lang(&mut self, ctx: &egui::Context, lang: crate::lang::Lang) {
        self.lang = lang;
        lang.apply();
        ctx.request_repaint();
    }

    pub(super) fn set_theme_choice(&mut self, ctx: &egui::Context, choice: crate::theme::Choice) {
        self.theme_choice = choice;
        self.sync_theme(ctx);
        self.status = match choice {
            crate::theme::Choice::System => t!("Design folgt dem System"),
            crate::theme::Choice::Light => t!("Helles Design"),
            crate::theme::Choice::Dark => t!("Dunkles Design"),
        }
        .into();
    }

    /// Bring the colours in line with the choice (and with the system when it follows it).
    pub(super) fn sync_theme(&mut self, ctx: &egui::Context) {
        let dark = self.theme_choice.dark(ctx.system_theme());
        if dark != self.dark || dark != crate::theme::is_dark() {
            self.dark = dark;
            crate::theme::set_dark(dark);
            crate::theme::apply(ctx);
        }
    }
}
