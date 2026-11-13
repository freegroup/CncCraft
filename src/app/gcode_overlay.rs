//! The G-code overlay of the 3D view: the machine's moves as the post-processor writes them (the
//! way from home to the first toolpath, the travel at safe_z, the hooks' moves), read back from
//! the G-code itself, so the view shows what the machine will really do.

use super::{tp_color, App};
use crate::sim;
use crate::toolpath::Toolpath;
use crate::transform::from_gcode::{self, Step};
use crate::transform::to_gcode;
use egui::Color32;
use std::hash::{Hash, Hasher};

/// The moves of the current G-code, kept until something they depend on changes.
pub(super) struct GcodeOverlay {
    key: u64,
    steps: Vec<Step>,
    /// The first G-code line of each toolpath's part, and the colour of its cuts.
    sections: Vec<(usize, Color32)>,
}

impl GcodeOverlay {
    /// The colour of the cuts on a G-code line: that of the toolpath whose part it is in.
    fn colour_at(&self, line: usize) -> Color32 {
        let k = self.sections.partition_point(|(start, _)| *start <= line);
        k.checked_sub(1).map_or(Color32::GRAY, |k| self.sections[k].1)
    }
}

impl App {
    /// A fingerprint of everything the G-code depends on, cheap enough to take every frame.
    fn gcode_key(&self, indices: &[usize]) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        let j = &self.job;
        for v in [j.width, j.height, j.thickness, j.safe_z, j.home_z] {
            v.to_bits().hash(&mut h);
        }
        (j.inches, format!("{:?}{:?}", j.z_zero, j.datum)).hash(&mut h);
        let pp = self.pp();
        (&pp.job_start, &pp.job_end, &pp.on_tool_change, &pp.before_path, &pp.after_path, pp.tool_change).hash(&mut h);
        for &i in indices {
            let tp = &self.toolpaths[i];
            (&tp.name, &tp.tool.name, tp.tool.number, tp.tool.rpm, tp.tool.feed.to_bits(), tp.tool.plunge.to_bits(), tp.strategy.label()).hash(&mut h);
            for m in &tp.moves {
                m.rapid.hash(&mut h);
                for c in m.to {
                    c.to_bits().hash(&mut h);
                }
            }
        }
        h.finish()
    }

    /// Write the G-code of the shown toolpaths again and read its moves back, if anything changed.
    pub(super) fn refresh_gcode_overlay(&mut self) {
        let indices: Vec<usize> = (0..self.toolpaths.len()).filter(|&i| self.toolpaths[i].visible && !self.toolpaths[i].moves.is_empty()).collect();
        let key = self.gcode_key(&indices);
        if self.gcode_overlay.as_ref().is_some_and(|o| o.key == key) {
            return;
        }
        let paths: Vec<&Toolpath> = indices.iter().map(|&i| &self.toolpaths[i]).collect();
        let (code, starts) = to_gcode::write_with_sections(&self.job, &paths, self.pp());
        let steps = from_gcode::trace_job(&code, &self.job);
        let sections = starts.into_iter().zip(&paths).map(|(line, tp)| (line, tp_color(&tp.strategy).gamma_multiply(0.75))).collect();
        self.gcode_overlay = Some(GcodeOverlay { key, steps, sections });
    }

    pub(super) fn paint_gcode_overlay(&self, painter: &egui::Painter, proj: &sim::Projection) {
        if let Some(overlay) = &self.gcode_overlay {
            sim::paint_steps(painter, proj, &overlay.steps, &|line| overlay.colour_at(line));
        }
    }
}
