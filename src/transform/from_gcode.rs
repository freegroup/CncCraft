//! Reading G-code back into machine moves, to draw what the post-processor really wrote: the way
//! from home to the first toolpath, the travel at safe_z, and whatever its hooks add. Plain logic
//! without a window, so it can be tested.

use super::frame;
use crate::toolpath::Job;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// G0: the machine travels.
    Rapid,
    /// G1 (and arcs, which are drawn straight): the machine cuts or feeds.
    Feed,
}

/// One move of the machine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    pub from: [f64; 3],
    pub to: [f64; 3],
    pub kind: Kind,
    /// The line of the G-code the move comes from (counted from 0).
    pub line: usize,
}

/// The letter-and-number words of a line, without its comments: `(...)` and everything after `;`.
fn words(line: &str) -> Vec<(char, f64)> {
    let mut clean = String::new();
    let mut depth = 0;
    for c in line.chars() {
        match c {
            ';' if depth == 0 => break,
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            _ if depth == 0 => clean.push(c),
            _ => {}
        }
    }
    let chars: Vec<char> = clean.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_alphabetic() {
            let letter = chars[i].to_ascii_uppercase();
            i += 1;
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | '-' | '+')) {
                i += 1;
            }
            if let Ok(v) = chars[start..i].iter().collect::<String>().parse::<f64>() {
                out.push((letter, v));
            }
        } else {
            i += 1;
        }
    }
    out
}

/// The machine's moves in millimetres, in the machine's own coordinates. The machine starts at the
/// work origin (X0 Y0), as the post-processor's "home" assumes; its height is whatever the first
/// line that names one says.
pub fn trace(code: &str) -> Vec<Step> {
    let mut steps = Vec::new();
    let mut unit = 1.0;
    let mut absolute = true;
    let mut motion: Option<Kind> = None;
    let mut pos = [0.0, 0.0, f64::NAN];
    for (line, text) in code.lines().enumerate() {
        let words = words(text);
        for &(letter, v) in &words {
            if letter == 'G' {
                match v {
                    v if v == 0.0 => motion = Some(Kind::Rapid),
                    v if v == 1.0 || v == 2.0 || v == 3.0 => motion = Some(Kind::Feed),
                    v if v == 20.0 => unit = 25.4,
                    v if v == 21.0 => unit = 1.0,
                    v if v == 90.0 => absolute = true,
                    v if v == 91.0 => absolute = false,
                    _ => {}
                }
            }
        }
        let mut axes: [Option<f64>; 3] = [None; 3];
        for &(letter, v) in &words {
            if let Some(i) = "XYZ".find(letter) {
                axes[i] = Some(v);
            }
        }
        let Some(kind) = motion else { continue };
        if axes.iter().all(Option::is_none) {
            continue;
        }
        let mut to = pos;
        for i in 0..3 {
            if let Some(v) = axes[i] {
                let base = if pos[i].is_nan() { 0.0 } else { pos[i] };
                to[i] = if absolute { v * unit } else { base + v * unit };
            }
        }
        // a height that was never named: the move starts at the height it goes to
        let mut from = pos;
        for i in 0..3 {
            if to[i].is_nan() {
                to[i] = 0.0;
            }
            if from[i].is_nan() {
                from[i] = to[i];
            }
        }
        pos = to;
        if from != to {
            steps.push(Step { from, to, kind, line });
        }
    }
    steps
}

/// The moves of a job's G-code in the job's own coordinates (millimetres, height zero at the
/// material surface), where they can be drawn over the job.
pub fn trace_job(code: &str, job: &Job) -> Vec<Step> {
    let (_, offset) = frame(job);
    let mut steps = trace(code);
    for step in &mut steps {
        for p in [&mut step.from, &mut step.to] {
            for i in 0..3 {
                p[i] -= offset[i];
            }
        }
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_follow_modal_words_and_skip_comments() {
        let code = "( Job )\nG21\nG90\nG0 Z10 ; hoch\nG0 X5 Y5 Z5\nG1 Z-1 F100\nX10 (bleibt G1) \nM5\nG0 X0 Y0 Z10 ( Y99 )\n";
        let steps = trace(code);
        let moves: Vec<(Kind, [f64; 3], [f64; 3])> = steps.iter().map(|s| (s.kind, s.from, s.to)).collect();
        assert_eq!(
            moves,
            vec![
                (Kind::Rapid, [0.0, 0.0, 10.0], [5.0, 5.0, 5.0]),
                (Kind::Feed, [5.0, 5.0, 5.0], [5.0, 5.0, -1.0]),
                (Kind::Feed, [5.0, 5.0, -1.0], [10.0, 5.0, -1.0]),
                (Kind::Rapid, [10.0, 5.0, -1.0], [0.0, 0.0, 10.0]),
            ],
            "the lone Z10 (machine height not known before) is no move; the rest follows"
        );
        assert_eq!(steps[0].line, 4);
        assert_eq!(steps[2].line, 6, "a line with only coordinates keeps the motion mode");
    }

    #[test]
    fn inches_and_relative_moves_are_read_in_millimetres() {
        let steps = trace("G20\nG0 X1 Y2 Z0.5\nG91\nG1 X1\nG90\nG1 X0\n");
        assert_eq!(steps[0].to, [25.4, 50.8, 12.7]);
        assert_eq!(steps[1].to, [50.8, 50.8, 12.7], "relative: one more inch");
        assert_eq!(steps[2].to, [0.0, 50.8, 12.7]);
    }

    #[test]
    fn words_may_stand_without_spaces() {
        let steps = trace("G0X3Y4Z5\n");
        assert_eq!(steps[0].to, [3.0, 4.0, 5.0]);
    }
}
