//! The coordinate cross of the milling zero point in the 3D view: three arrows from the point the
//! machine counts from (X red, Y green, Z blue), each with its letter in a small round badge, like the
//! axis cross of the 2D sheet. The zero point is where the job says it is: the lower left corner or
//! the middle of the board, on the material surface or on the machine bed.

use super::{View3dContext, ViewDecoration};
use crate::toolpath::{Datum, Job, ZZero};
use egui::{Align2, Color32, FontId, Pos2, Stroke, Vec2};

const RED: Color32 = Color32::from_rgb(0xd0, 0x2a, 0x2a);
const GREEN: Color32 = Color32::from_rgb(0x1f, 0x9a, 0x3c);
const BLUE: Color32 = Color32::from_rgb(0x20, 0x50, 0xe0);

/// Where the machine's zero lies, as a job point (mm; Z up from the material top, so the machine bed
/// is at minus the thickness).
pub fn zero_point(job: &Job) -> [f32; 3] {
    let (x, y) = match job.datum {
        Datum::LowerLeft => (0.0, 0.0),
        Datum::Center => (job.width as f32 / 2.0, job.height as f32 / 2.0),
    };
    let z = match job.z_zero {
        ZZero::MaterialSurface => 0.0,
        ZZero::MachineBed => -(job.thickness as f32),
    };
    [x, y, z]
}

/// How long the arrows are (mm): a fifth of the board's longest side, at least 15 mm.
pub fn arrow_length(job: &Job) -> f32 {
    (job.width.max(job.height).max(job.thickness) as f32 * 0.2).max(15.0)
}

/// The zero cross: a decoration of the 3D view.
#[derive(Default)]
pub struct ZeroCross;

impl ViewDecoration for ZeroCross {
    fn name(&self) -> &'static str {
        "Nullpunkt-Achsenkreuz"
    }

    fn show(&mut self, ui: &mut egui::Ui, view: &mut View3dContext) -> Option<String> {
        let painter = ui.painter_at(view.area);
        let origin = view.proj.to_screen(view.zero);
        let len = view.arrow_length;
        // the arrows, far ones first so a near one is drawn over them: sort by how far they point
        // away from the viewer is overkill for three short lines, so they go in a fixed order
        for (dir, name, colour) in [([1.0, 0.0, 0.0], "X", RED), ([0.0, 1.0, 0.0], "Y", GREEN), ([0.0, 0.0, 1.0], "Z", BLUE)] {
            let tip = view.proj.to_screen([view.zero[0] + dir[0] * len, view.zero[1] + dir[1] * len, view.zero[2] + dir[2] * len]);
            let d = tip - origin;
            if d.length() < 4.0 {
                // seen end-on: only the badge, on the origin
                continue;
            }
            let unit = d / d.length();
            let side = Vec2::new(-unit.y, unit.x);
            painter.line_segment([origin, tip - unit * 9.0], Stroke::new(2.4_f32, colour));
            painter.add(egui::Shape::convex_polygon(vec![tip, tip - unit * 10.0 + side * 4.5, tip - unit * 10.0 - side * 4.5], colour, Stroke::new(1.0_f32, Color32::WHITE)));
            let badge: Pos2 = tip + unit * 11.0;
            painter.circle(badge, 8.0, colour, Stroke::new(1.4_f32, Color32::WHITE));
            painter.text(badge, Align2::CENTER_CENTER, name, FontId::proportional(11.0), Color32::WHITE);
        }
        painter.circle(origin, 3.5, Color32::WHITE, Stroke::new(1.8_f32, crate::theme::INK));
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_zero_follows_the_jobs_datum_and_z_zero() {
        let mut job = Job { width: 300.0, height: 200.0, thickness: 18.0, ..Job::default() };
        job.datum = Datum::LowerLeft;
        job.z_zero = ZZero::MaterialSurface;
        assert_eq!(zero_point(&job), [0.0, 0.0, 0.0]);
        job.datum = Datum::Center;
        job.z_zero = ZZero::MachineBed;
        assert_eq!(zero_point(&job), [150.0, 100.0, -18.0]);
    }

    #[test]
    fn the_arrows_have_a_sensible_length() {
        assert_eq!(arrow_length(&Job { width: 300.0, height: 200.0, thickness: 18.0, ..Job::default() }), 60.0);
        assert_eq!(arrow_length(&Job { width: 40.0, height: 30.0, thickness: 10.0, ..Job::default() }), 15.0);
    }

    #[test]
    fn x_points_right_and_z_up_in_the_top_view() {
        use crate::sim::{Camera, Projection};
        let job = Job { width: 300.0, height: 200.0, thickness: 18.0, ..Job::default() };
        let cam = Camera { yaw: 0.0, pitch: 0.0, ..Camera::default() };
        let rect = egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let proj = Projection::new(rect, &job, &cam);
        let o = proj.to_screen(zero_point(&job));
        let x = proj.to_screen([60.0, 0.0, 0.0]);
        let y = proj.to_screen([0.0, 60.0, 0.0]);
        assert!(x.x > o.x && (x.y - o.y).abs() < 1.0, "X goes right");
        assert!(y.y < o.y && (y.x - o.x).abs() < 1.0, "Y goes up the screen in the top view");
    }
}
