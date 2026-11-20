use super::*;

impl App {
    pub(super) fn start_preview(&mut self, animate: bool) {
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

    pub(super) fn preview_indices(&self) -> Vec<usize> {
        match self.preview_scope {
            PreviewScope::All => (0..self.toolpaths.len()).collect(),
            PreviewScope::Visible => (0..self.toolpaths.len()).filter(|&i| self.toolpaths[i].visible).collect(),
        }
    }

    pub(super) fn output_indices(&self) -> Vec<usize> {
        let cuts = |t: &Toolpath| t.moves.iter().any(|m| !m.rapid);
        match self.output_scope {
            OutputScope::All => (0..self.toolpaths.len()).filter(|&i| cuts(&self.toolpaths[i])).collect(),
            OutputScope::Visible => (0..self.toolpaths.len()).filter(|&i| self.toolpaths[i].visible && cuts(&self.toolpaths[i])).collect(),
        }
    }

    pub(super) fn export_scope_text(&self, count: usize) -> String {
        let what = match self.output_scope {
            OutputScope::All => t!("Alle Werkzeugwege"),
            OutputScope::Visible => t!("Sichtbare Werkzeugwege"),
        };
        format!("{what} ({count})")
    }

    pub(super) fn output_scope_matters(&self) -> bool {
        self.toolpaths.iter().any(|t| !t.visible && t.moves.iter().any(|m| !m.rapid))
    }

    pub(super) fn step_animation(&mut self, dt: f64) {
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
}
