//! The post-processors: a dialect's settings and the G-code hooks that go around and between the
//! toolpaths. The transformers that use them are in [`crate::transform`].

use serde::{Deserialize, Serialize};

/// A complete post-processor definition: dialect settings plus injectable G-code hooks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostProcessor {
    pub name: String,
    #[serde(default)]
    pub inch: bool,
    pub ext: String,
    pub tool_change: bool,
    pub job_start: String,
    pub job_end: String,
    pub on_tool_change: String,
    pub before_path: String,
    pub after_path: String,
    pub builtin: bool,
}

impl PostProcessor {
    pub fn label(&self) -> String {
        format!("{} (*.{})", self.name, self.ext)
    }
}

pub fn builtins() -> Vec<PostProcessor> {
    vec![
        PostProcessor {
            name: "G-Code".into(),
            inch: false,
            ext: "nc".into(),
            tool_change: true,
            job_start: "G54              ; Werkstück-Koordinatensystem 1\nG0 Z{home_z}     ; Eilgang zur Referenzhöhe".into(),
            job_end: "G0 Z{safe_z}     ; Eilgang zur Sicherheitshöhe\nM5               ; Spindel AUS\nM9               ; Kühlmittel AUS".into(),
            on_tool_change: "G0 Z{safe_z}       ; Eilgang zur Sicherheitshöhe\nM5                 ; Spindel AUS\nT{tool_number} M6  ; Werkzeugwechsel\nM0                 ; Halt – weiter mit Cycle Start".into(),
            before_path: String::new(),
            after_path: "G0 Z{safe_z}     ; Eilgang zur Sicherheitshöhe".into(),
            builtin: true,
        },
        PostProcessor {
            name: "GRBL".into(),
            inch: false,
            ext: "gcode".into(),
            tool_change: false,
            job_start: "G54              ; Werkstück-Koordinatensystem 1\nG0 Z{home_z}     ; Eilgang zur Referenzhöhe".into(),
            job_end: "G0 Z{safe_z}     ; Eilgang zur Sicherheitshöhe\nM5               ; Spindel AUS".into(),
            on_tool_change: String::new(),
            before_path: String::new(),
            after_path: "G0 Z{safe_z}     ; Eilgang zur Sicherheitshöhe".into(),
            builtin: true,
        },
    ]
}

/// Backwards compatibility: deserialize the old Project format that stored `Post` enum + `PostHooks`.
pub fn migrate_old_format(hooks: &OldPostHooks) -> Vec<PostProcessor> {
    let mut list = builtins();
    let non_default = OldPostHooks::default();
    if hooks != &non_default {
        for pp in &mut list {
            pp.job_start = hooks.job_start.clone();
            pp.job_end = hooks.job_end.clone();
            pp.on_tool_change = hooks.tool_change.clone();
            pp.before_path = hooks.before_toolpath.clone();
            pp.after_path = hooks.after_toolpath.clone();
        }
    }
    list
}

/// The old PostHooks struct for backwards-compatible deserialization.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct OldPostHooks {
    #[serde(default)]
    pub job_start: String,
    #[serde(default)]
    pub job_end: String,
    #[serde(default)]
    pub tool_change: String,
    #[serde(default)]
    pub before_toolpath: String,
    #[serde(default)]
    pub after_toolpath: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_grbl_has_no_tool_change() {
        let b = builtins();
        assert!(b[0].tool_change);
        assert!(!b[1].tool_change);
    }
}
