//! The view cube in the corner of the 3D view, after the one in Fusion 360: a small cube that
//! shows how the job is turned. Drag it to turn the view, click one of its sides to look at the job
//! from there (top, front, left, right, back, bottom), the house puts the view back, and the
//! little arrow under it opens the choice between the orthographic and the perspective view.
//! The coloured lines are the job's axes: X red, Y green, Z blue.

use super::{View3dContext, ViewDecoration};
use crate::theme::{INK, MUTED};
use egui::{Align2, Color32, FontId, Id, Pos2, Rect, Sense, Stroke, Vec2};
use std::f32::consts::{FRAC_PI_2, PI};

/// Half the cube's edge on screen (px) and the room it gets.
const CUBE: f32 = 23.0;
const BOX: f32 = 100.0;
/// Where each side of the cube looks from: (name, outward normal, turn, tilt). The view is
/// turned about the vertical (yaw) and tilted from straight above (pitch 0) to the horizon.
const SIDES: [(crate::lang::Tr, [f32; 3], f32, f32); 6] = [
    (tr!("OBEN"), [0.0, 0.0, 1.0], 0.0, 0.0),
    (tr!("VORNE"), [0.0, -1.0, 0.0], 0.0, FRAC_PI_2),
    (tr!("LINKS"), [-1.0, 0.0, 0.0], FRAC_PI_2, FRAC_PI_2),
    (tr!("RECHTS"), [1.0, 0.0, 0.0], -FRAC_PI_2, FRAC_PI_2),
    (tr!("HINTEN"), [0.0, 1.0, 0.0], PI, FRAC_PI_2),
    (tr!("UNTEN"), [0.0, 0.0, -1.0], 0.0, PI),
];

/// The view cube: a decoration of the 3D view.
#[derive(Default)]
pub struct ViewCube;

impl ViewDecoration for ViewCube {
    fn name(&self) -> &'static str {
        "Ansichtswürfel"
    }

    fn show(&mut self, ui: &mut egui::Ui, view: &mut View3dContext) -> Option<String> {
        ViewCube::draw(ui, view)
    }
}

impl ViewCube {
    /// Where a point of the cube (corners at ±1) lands: screen offset from the cube's middle and
    /// its depth (smaller is nearer to the viewer).
    fn cube_point(cam: &crate::sim::Camera, p: [f32; 3]) -> (Vec2, f32) {
        let (sy, cy) = cam.yaw.sin_cos();
        let (sp, cp) = cam.pitch.sin_cos();
        let rx = p[0] * cy - p[1] * sy;
        let ry = p[0] * sy + p[1] * cy;
        let up = ry * cp + p[2] * sp;
        (Vec2::new(rx * CUBE, -up * CUBE), ry * sp - p[2] * cp)
    }

    /// Draw the view cube with its buttons in the upper right corner of the 3D view `area`, and
    /// do what is clicked or dragged.
    fn draw(ui: &mut egui::Ui, view: &mut View3dContext) -> Option<String> {
        let mut message = None;
        let area = view.area;
        let boxr = Rect::from_min_size(area.right_top() + Vec2::new(-BOX - 12.0, 14.0), Vec2::splat(BOX));
        let middle = boxr.center();
        let painter = ui.painter_at(boxr.expand(10.0));
        let cube_rect = Rect::from_center_size(middle, Vec2::splat(CUBE * 2.9));
        let resp = ui.interact(cube_rect, Id::new("view_cube"), Sense::click_and_drag());
        let pointer = ui.input(|i| i.pointer.hover_pos());

        // the cube turns the view when dragged (the same way as dragging the picture):
        // part follows the mouse, like Fusion 360
        if resp.dragged_by(egui::PointerButton::Primary) {
            let d = resp.drag_delta();
            view.cam.yaw += d.x * 0.012;
            view.cam.pitch = (view.cam.pitch - d.y * 0.01).clamp(0.0, PI);
        }

        // the sides facing the viewer, nearest last
        struct Face {
            name: String,
            poly: Vec<Pos2>,
            facing: f32,
            shade: f32,
            index: usize,
        }
        let mut faces: Vec<Face> = Vec::new();
        for (index, (name, n, _, _)) in SIDES.iter().enumerate() {
            let (_, facing) = Self::cube_point(&view.cam, *n);
            if facing >= 0.0 {
                continue;
            }
            // the four corners of this side
            let (u, v) = match index {
                0 | 5 => ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
                1 | 4 => ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
                _ => ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
            };
            let corner = |a: f32, b: f32| [n[0] + u[0] * a + v[0] * b, n[1] + u[1] * a + v[1] * b, n[2] + u[2] * a + v[2] * b];
            let poly: Vec<Pos2> = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().map(|(a, b)| middle + Self::cube_point(&view.cam, corner(*a, *b)).0).collect();
            // light from above and the left: the top is brightest, the sides a little darker
            let shade = 0.78 + 0.2 * (-facing).min(1.0) + if index == 0 { 0.06 } else { 0.0 };
            faces.push(Face { name: name.get(), poly, facing, shade, index });
        }
        faces.sort_by(|a, b| b.facing.total_cmp(&a.facing));

        // the side under the pointer
        let hovered = pointer.filter(|p| cube_rect.contains(*p)).and_then(|p| faces.iter().find(|f| point_in_poly(p, &f.poly)).map(|f| f.index));

        // soft shadow under the cube
        painter.circle_filled(middle + Vec2::new(8.0, CUBE * 1.25), CUBE * 0.95, Color32::from_black_alpha(18));

        for f in &faces {
            let base = (255.0 * f.shade.min(1.0)) as u8;
            let mut fill = Color32::from_rgb(base.saturating_sub(18), base.saturating_sub(14), base);
            if hovered == Some(f.index) {
                fill = Color32::from_rgb(0xff, 0xd9, 0x66);
            }
            painter.add(egui::Shape::convex_polygon(f.poly.clone(), fill, Stroke::new(1.0_f32, Color32::from_gray(150))));
            // the name, when the side faces the viewer well enough to read it
            if -f.facing > 0.5 {
                let c = f.poly.iter().fold(Vec2::ZERO, |s, p| s + p.to_vec2()) / 4.0;
                painter.text(c.to_pos2(), Align2::CENTER_CENTER, &f.name, FontId::proportional(5.5 + 4.0 * (-f.facing)), INK);
            }
        }

        // the axes from the corner at the origin of the job (−1, −1, −1): X red, Y green, Z blue
        let o = Self::cube_point(&view.cam, [-1.0, -1.0, -1.0]).0;
        for (dir, colour, label) in [([1.0, 0.0, 0.0], Color32::from_rgb(0xe0, 0x20, 0x20), "X"), ([0.0, 1.0, 0.0], Color32::from_rgb(0x20, 0xb0, 0x30), "Y"), ([0.0, 0.0, 1.0], Color32::from_rgb(0x20, 0x40, 0xe0), "Z")] {
            let tip = Self::cube_point(&view.cam, [-1.0 + dir[0] * 2.5, -1.0 + dir[1] * 2.5, -1.0 + dir[2] * 2.5]).0;
            painter.line_segment([middle + o, middle + tip], Stroke::new(2.0_f32, colour));
            painter.text(middle + tip + (tip - o).normalized() * 8.0, Align2::CENTER_CENTER, label, FontId::proportional(11.0), colour);
        }

        // a click on a side looks from there
        if resp.clicked() {
            if let Some(i) = hovered {
                let (_, _, yaw, pitch) = SIDES[i];
                view.cam.yaw = yaw;
                view.cam.pitch = pitch;
                message = Some(t!("Ansicht: %{side}", side = SIDES[i].0.get().to_lowercase()));
            }
        }
        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(if resp.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
        }

        // the house: back to the starting view (the perspective choice stays)
        let home = Rect::from_center_size(boxr.left_top() + Vec2::new(4.0, 2.0), Vec2::splat(26.0));
        let home_resp = ui.interact(home, Id::new("view_cube_home"), Sense::click()).on_hover_text(t!("Ausgangsansicht"));
        let tone = if home_resp.hovered() { INK } else { MUTED };
        let c = home.center();
        let stroke = Stroke::new(1.5_f32, tone);
        painter.add(egui::Shape::line(vec![c + Vec2::new(-10.0, 0.0), c + Vec2::new(0.0, -9.0), c + Vec2::new(10.0, 0.0)], stroke));
        painter.add(egui::Shape::line(vec![c + Vec2::new(-7.0, -2.0), c + Vec2::new(-7.0, 9.0), c + Vec2::new(7.0, 9.0), c + Vec2::new(7.0, -2.0)], stroke));
        painter.rect_stroke(Rect::from_center_size(c + Vec2::new(0.0, 5.5), Vec2::new(5.0, 7.0)), 1.0, stroke);
        if home_resp.clicked() {
            *view.cam = crate::sim::Camera { perspective: view.cam.perspective, ..Default::default() };
        }

        // the arrow: orthographic or perspective
        let arrow = Rect::from_center_size(boxr.right_bottom() + Vec2::new(2.0, 4.0), Vec2::splat(26.0));
        let arrow_resp = ui.interact(arrow, Id::new("view_cube_menu"), Sense::click()).on_hover_text(t!("Orthogonal oder perspektivisch"));
        let tone = if arrow_resp.hovered() { INK } else { MUTED };
        let c = arrow.center();
        painter.add(egui::Shape::line(vec![c + Vec2::new(-8.0, -3.0), c + Vec2::new(8.0, -3.0)], Stroke::new(1.5_f32, tone)));
        painter.add(egui::Shape::convex_polygon(vec![c + Vec2::new(-7.0, 0.0), c + Vec2::new(7.0, 0.0), c + Vec2::new(0.0, 7.0)], Color32::TRANSPARENT, Stroke::new(1.5_f32, tone)));
        let popup = Id::new("view_cube_popup");
        if arrow_resp.clicked() {
            ui.memory_mut(|m| m.toggle_popup(popup));
        }
        egui::popup_below_widget(ui, popup, &arrow_resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
            ui.set_min_width(170.0);
            ui.selectable_value(&mut view.cam.perspective, false, t!("Orthogonal"));
            ui.selectable_value(&mut view.cam.perspective, true, t!("Perspektivisch"));
        });
        message
    }
}

/// Whether `p` is inside the convex polygon (any winding).
fn point_in_poly(p: Pos2, poly: &[Pos2]) -> bool {
    let mut sign = 0.0f32;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        if cross.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}
