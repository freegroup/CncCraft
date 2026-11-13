//! Profile tool: cut along a contour, offset to one side by the tool radius, with optional
//! holding tabs. Open vectors are cut as a zig-zag.

use super::{ring_dir, tab_spans, CutContext, Out, Side, ToolpathKind, TAB_LENGTH, TAB_THICKNESS};
use crate::geom::{Pt, Region};

pub struct ProfileTool {
    pub side: Side,
    pub climb: bool,
    pub tabs: bool,
    pub tab_count: u32,
}

impl ToolpathKind for ProfileTool {
    fn label(&self) -> String {
        t!("Profil")
    }

    fn cut(&self, out: &mut Out, cx: &CutContext) -> Option<String> {
        let mut warning = None;
        let depths = cx.depths();
        let closed = cx.closed();
        let region = Region::from_contours(&closed);
        let off = match self.side {
            Side::Outside => cx.radius(),
            Side::Inside => -cx.radius(),
            Side::On => 0.0,
        };
        let rings = region.offset(off).rings();
        if rings.is_empty() && !closed.is_empty() {
            warning = Some(t!("Werkzeug zu groß für diese Form").into());
        }
        let tab_top = -(cx.cut_depth - TAB_THICKNESS.min(cx.cut_depth));
        for ring in &rings {
            let pts = ring_dir(ring, self.climb);
            let spans = if self.tabs { tab_spans(&pts, self.tab_count, TAB_LENGTH + cx.tool.diameter) } else { vec![] };
            for &z in &depths {
                let t = (!spans.is_empty() && z < tab_top).then_some((tab_top, spans.as_slice()));
                out.cut_ring(&pts, z, t);
            }
            out.retract();
        }
        // open vectors: cut as a back-and-forth so the tool doesn't fly back each pass
        for c in cx.contours.iter().filter(|c| !c.closed) {
            for (i, &z) in depths.iter().enumerate() {
                if i % 2 == 0 {
                    out.cut_open(&c.pts, z);
                } else {
                    let rev: Vec<Pt> = c.pts.iter().rev().copied().collect();
                    out.cut_open(&rev, z);
                }
            }
            out.retract();
        }
        warning
    }
}
