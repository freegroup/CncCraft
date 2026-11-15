//! Connect: a straight line between two points. Where a point is the end of an open path, the
//! line becomes part of that path; between the two ends of one path it closes it.

use crate::forms::Path;
use crate::geometry::dist;
use crate::Pt;

/// An end of an open path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Start,
    Finish,
}

/// Which end of the open path `path` lies at `p` (within `tol`), if one does.
pub fn end_at(path: &Path, p: Pt, tol: f64) -> Option<End> {
    if path.closed || path.points.len() < 2 {
        return None;
    }
    if dist(path.points[0], p) <= tol {
        Some(End::Start)
    } else if dist(*path.points.last().unwrap(), p) <= tol {
        Some(End::Finish)
    } else {
        None
    }
}

/// `path` grown at `end` by a straight line to `q`. When `q` is its other end, the path closes
/// instead (it needs three points for that).
pub fn extend(path: &Path, end: End, q: Pt, tol: f64) -> Path {
    let mut out = path.clone();
    let other = match end {
        End::Start => *path.points.last().unwrap(),
        End::Finish => path.points[0],
    };
    if dist(other, q) <= tol && path.points.len() >= 3 {
        out.closed = true;
        return out;
    }
    match end {
        End::Start => {
            out.points.insert(0, q);
            for c in &mut out.corners {
                c.node += 1;
            }
        }
        End::Finish => out.points.push(q),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::{Corner, CornerKind};

    #[test]
    fn a_line_from_an_end_grows_the_path() {
        let p = Path::new(vec![[0.0, 0.0], [10.0, 0.0]], false);
        assert_eq!(end_at(&p, [10.0, 0.0], 0.01), Some(End::Finish));
        assert_eq!(end_at(&p, [5.0, 0.0], 0.01), None);
        assert_eq!(extend(&p, End::Finish, [10.0, 10.0], 0.01).points, vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]);
        assert_eq!(extend(&p, End::Start, [0.0, 10.0], 0.01).points, vec![[0.0, 10.0], [0.0, 0.0], [10.0, 0.0]]);
    }

    #[test]
    fn joining_both_ends_closes_the_path_and_keeps_its_corners() {
        let mut p = Path::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], false);
        p.corners.push(Corner { node: 1, kind: CornerKind::Round, radius: 2.0, expr: None, label_offset: None });
        let closed = extend(&p, End::Finish, [0.0, 0.0], 0.01);
        assert!(closed.closed && closed.points.len() == 3);
        let grown = extend(&p, End::Start, [-5.0, 0.0], 0.01);
        assert_eq!(grown.corners[0].node, 2, "the corner stays on its point");
    }
}
