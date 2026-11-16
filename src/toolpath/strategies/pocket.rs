//! Pocket tool: clear the inside of a region. How it is cleared is its own file: concentric
//! offset rings (`pocket_offset`), a raster fill at a chosen angle (`pocket_raster`) or one
//! continuous spiral (`pocket_spiral`). This file holds what they share.

use super::{pocket_offset, pocket_raster, pocket_spiral};
use crate::geom::{dist, Pt, Region, Ring};
use crate::toolpath::{stays_inside, CutContext, Out, ToolpathKind};

/// How a pocket is cleared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clearing {
    /// Concentric rings, from the middle outwards, joined without lifting where they neighbour.
    Offset,
    /// Back-and-forth lines, then a round along the wall.
    Raster,
    /// One continuous spiral from the middle out to the wall: never a new start, each turn goes
    /// over into the next one smoothly.
    Spiral,
}

impl Clearing {
    /// From the strategy's stored switches (raster wins over spiral).
    pub fn of(raster: bool, spiral: bool) -> Clearing {
        if raster {
            Clearing::Raster
        } else if spiral {
            Clearing::Spiral
        } else {
            Clearing::Offset
        }
    }

    /// The stored switches for this way of clearing: (raster, spiral).
    pub fn switches(self) -> (bool, bool) {
        (self == Clearing::Raster, self == Clearing::Spiral)
    }
}

pub struct PocketTool {
    pub clearing: Clearing,
    pub climb: bool,
    /// Raster fill direction in degrees (0 = along X).
    pub raster_angle: f64,
}

/// What every way of clearing works from.
pub(super) struct Pocket {
    /// Where the cutter's centre may go: the pocket shrunk by the tool radius.
    pub inset: Region,
    /// The same a hair bigger, so a way along its very edge still counts as inside.
    reach: Region,
    pub step: f64,
    pub climb: bool,
    /// The longest way the cutter may feed over to the next pass without lifting.
    max_link: f64,
}

/// How far `reach` lies outside `inset` (mm): far below what a machine resolves.
const EDGE_TOLERANCE: f64 = 0.005;

impl Pocket {
    /// Whether the cutter may stay down and feed straight from `a` to `b`: the neighbouring pass,
    /// close by, and the way across stays where the cutter's centre may go.
    pub fn stay_down(&self, a: Pt, b: Pt) -> bool {
        dist(a, b) <= self.max_link && stays_inside(&self.reach, a, b)
    }

    /// Whether `p` lies where the cutter's centre may go.
    pub fn may_go(&self, p: Pt) -> bool {
        self.reach.contains(p)
    }

    /// The concentric passes, outermost first: each level is the inset shrunk once more by the
    /// stepover (shrunk from the level before, so it stays quick).
    pub fn offset_levels(&self) -> Vec<Vec<Ring>> {
        let mut levels = vec![];
        let mut shrunk = self.inset.clone();
        if shrunk.is_empty() {
            return levels;
        }
        levels.push(shrunk.rings());
        for _ in 0..2000 {
            shrunk = shrunk.offset(-self.step);
            if shrunk.is_empty() {
                break;
            }
            levels.push(shrunk.rings());
        }
        levels
    }
}

impl ToolpathKind for PocketTool {
    fn label(&self) -> String {
        t!("Tasche")
    }

    fn cut(&self, out: &mut Out, cx: &CutContext) -> Option<String> {
        let region = Region::from_contours(cx.contours);
        let inset = region.offset(-cx.radius());
        let warning = (inset.is_empty() && !cx.contours.is_empty()).then(|| t!("Werkzeug zu groß für diese Tasche").into());
        let pocket = Pocket {
            reach: inset.offset(EDGE_TOLERANCE),
            inset,
            step: cx.tool.stepover.max(0.05),
            climb: self.climb,
            max_link: cx.tool.diameter.max(cx.tool.stepover),
        };
        let depths = cx.depths();
        match self.clearing {
            Clearing::Offset => pocket_offset::cut(out, cx, &pocket, &depths),
            Clearing::Raster => pocket_raster::cut(out, cx, &pocket, &depths, self.raster_angle),
            Clearing::Spiral => pocket_spiral::cut(out, cx, &pocket, &depths),
        }
        warning
    }
}
