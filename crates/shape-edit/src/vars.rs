//! Document variables: named values that other inputs can refer to (`breite = 100`,
//! `rand = dicke/2`). Each variable is a name and an expression; a variable may use other
//! variables, so the table is evaluated in dependency order and cycles are reported rather than
//! looping forever. This is the parametric layer described in app/docs/PARAMETRIK.md; the
//! original keeps the same idea as `utDocumentVariable` (expression stored as text, the number
//! derived from it — see MECHANICS.md Nachtrag 15).

use crate::calc::{self, CalcError, Lookup};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Something that happened to a variable. The app collects these and lets everything that
/// depends on variables react: users are renamed with the variable, shapes follow a new value,
/// bindings to a deleted variable are released.
#[derive(Clone, Debug, PartialEq)]
pub enum VarEvent {
    Added(String),
    Removed(String),
    Renamed { from: String, to: String },
    /// The worked-out value changed (or appeared, or became an error).
    ValueChanged(String),
}

/// One variable: a name and the expression that gives its value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub expr: String,
    /// A note for people (what the value is for). Old project files have none.
    #[serde(default)]
    pub comment: String,
}

/// The document's variables, in the order the person added them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variables {
    pub list: Vec<Variable>,
}

/// The variables with every value worked out (or the error for the ones that couldn't be).
#[derive(Clone, Debug, Default)]
pub struct Resolved {
    values: HashMap<String, f64>,
    errors: HashMap<String, CalcError>,
}

impl Resolved {
    /// The value of a name, or why it has none. A [`Lookup`], so it feeds [`calc::eval`] directly.
    pub fn value(&self, name: &str) -> Result<f64, CalcError> {
        if let Some(v) = self.values.get(name) {
            return Ok(*v);
        }
        Err(self.errors.get(name).cloned().unwrap_or_else(|| CalcError::Unknown(name.to_string())))
    }

    /// The names whose value differs from `old` (changed, new, or no longer there), sorted.
    pub fn changes_since(&self, old: &Resolved) -> Vec<String> {
        let mut names: Vec<String> = self.values.keys().chain(old.values.keys()).chain(self.errors.keys()).chain(old.errors.keys()).cloned().collect();
        names.sort();
        names.dedup();
        names.retain(|n| self.values.get(n) != old.values.get(n) || self.errors.contains_key(n) != old.errors.contains_key(n));
        names
    }

    /// The error of a variable, if it has one.
    pub fn error(&self, name: &str) -> Option<&CalcError> {
        self.errors.get(name)
    }

    /// Every name with a usable value, sorted — for autocomplete.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.values.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Lookup for Resolved {
    fn get(&self, name: &str) -> Result<f64, CalcError> {
        self.value(name)
    }
}

impl Variables {
    /// Add or change a variable by name; keeps the person's order.
    pub fn set(&mut self, name: &str, expr: &str) {
        match self.list.iter_mut().find(|v| v.name == name) {
            Some(v) => v.expr = expr.to_string(),
            None => self.list.push(Variable { name: name.to_string(), expr: expr.to_string(), comment: String::new() }),
        }
    }

    /// Make every variable's expression use the new name where it used `from`.
    pub fn rename_users(&mut self, from: &str, to: &str) {
        for v in &mut self.list {
            if calc::names_in(&v.expr).iter().any(|n| n == from) {
                v.expr = calc::rename_in(&v.expr, from, to);
            }
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.list.retain(|v| v.name != name);
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Work out every variable's value, following dependencies and marking cycles. `extra` are
    /// values the host supplies that aren't editable variables (e.g. the job's size), so an
    /// expression may use `platte_b` without it being in the list.
    pub fn resolve(&self, extra: &[(&str, f64)]) -> Resolved {
        let mut out = Resolved::default();
        for (name, value) in extra {
            out.values.insert((*name).to_string(), *value);
        }
        // depth-first, so a variable is computed after the ones it needs; `visiting` catches
        // cycles, `done` stops repeating work
        let mut visiting: Vec<String> = Vec::new();
        let mut done: Vec<String> = Vec::new();
        for v in &self.list {
            self.compute(&v.name, extra, &mut out, &mut visiting, &mut done);
        }
        out
    }

    fn compute(&self, name: &str, extra: &[(&str, f64)], out: &mut Resolved, visiting: &mut Vec<String>, done: &mut Vec<String>) {
        if done.iter().any(|d| d == name) || extra.iter().any(|(n, _)| *n == name) {
            return;
        }
        let Some(var) = self.list.iter().find(|v| v.name == name) else {
            return; // unknown name: left for the reader to report
        };
        if visiting.iter().any(|v| v == name) {
            // part of a cycle: mark every variable currently on the stack
            for v in visiting.iter() {
                out.errors.insert(v.clone(), CalcError::Cycle(v.clone()));
            }
            out.errors.insert(name.to_string(), CalcError::Cycle(name.to_string()));
            return;
        }
        if !calc::is_valid_name(name) {
            out.errors.insert(name.to_string(), CalcError::Syntax);
            done.push(name.to_string());
            return;
        }
        visiting.push(name.to_string());
        for dep in calc::names_in(&var.expr) {
            self.compute(&dep, extra, out, visiting, done);
        }
        visiting.retain(|v| v != name);
        done.push(name.to_string());
        // if a dependency is already marked a cycle, this one is too
        if out.errors.get(name).is_some_and(|e| matches!(e, CalcError::Cycle(_))) {
            return;
        }
        match calc::eval(&var.expr, out) {
            Ok(value) => {
                out.values.insert(name.to_string(), value);
            }
            Err(e) => {
                out.errors.insert(name.to_string(), e);
            }
        }
    }
}

/// The unfinished name at the end of `text`, if the cursor is in one: where it starts (a byte
/// index) and the text of it. `"breite/2 + ho"` gives `("ho")`; `"30"`, `"2*"` and `"a + 3x"`'s
/// number-led end give nothing, because a name can't start with a digit.
pub fn completion_token(text: &str) -> Option<(usize, &str)> {
    let end = text.len();
    let start = text.char_indices().rev().take_while(|(_, c)| calc::is_name_part(*c)).last().map(|(i, _)| i)?;
    let token = &text[start..end];
    let mut chars = token.chars();
    // a name starts with a letter or `_`; digits in front of it belong to a number (`2ab` is `2` then `ab`)
    let first = chars.next()?;
    if first.is_ascii_digit() {
        let skip = token.char_indices().find(|(_, c)| !c.is_ascii_digit()).map(|(i, _)| i)?;
        let t = &token[skip..];
        return (!t.is_empty() && calc::is_valid_name(t)).then(|| (start + skip, t));
    }
    calc::is_valid_name(token).then_some((start, token))
}

/// The names that continue `token` (case-insensitive), sorted, never the token itself. At most
/// `limit` are returned.
pub fn completions(names: &[String], token: &str, limit: usize) -> Vec<String> {
    let lower = token.to_lowercase();
    let mut out: Vec<String> = names.iter().filter(|n| n.to_lowercase().starts_with(&lower) && n.as_str() != token).cloned().collect();
    out.sort();
    out.dedup();
    out.truncate(limit);
    out
}

/// `text` with its unfinished name replaced by `name`.
pub fn apply_completion(text: &str, name: &str) -> String {
    match completion_token(text) {
        Some((start, _)) => format!("{}{}", &text[..start], name),
        None => format!("{text}{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_unfinished_name_at_the_end() {
        assert_eq!(completion_token("ho"), Some((0, "ho")));
        assert_eq!(completion_token("breite/2 + ho"), Some((11, "ho")));
        assert_eq!(completion_token("2*b"), Some((2, "b")));
        assert_eq!(completion_token("2b"), Some((1, "b")), "digits before belong to the number");
        assert_eq!(completion_token("30"), None);
        assert_eq!(completion_token("2*"), None);
        assert_eq!(completion_token(""), None);
        assert_eq!(completion_token("a_1"), Some((0, "a_1")));
    }

    #[test]
    fn suggests_matching_names_but_not_the_finished_one() {
        let names: Vec<String> = ["breite", "boden", "Rand", "hoehe"].iter().map(|s| s.to_string()).collect();
        assert_eq!(completions(&names, "b", 8), vec!["boden", "breite"]);
        assert_eq!(completions(&names, "r", 8), vec!["Rand"], "case does not matter");
        assert_eq!(completions(&names, "breite", 8), Vec::<String>::new(), "already complete");
        assert_eq!(completions(&names, "b", 1), vec!["boden"], "limit");
    }

    #[test]
    fn completion_replaces_only_the_unfinished_name() {
        assert_eq!(apply_completion("breite/2 + ho", "hoehe"), "breite/2 + hoehe");
        assert_eq!(apply_completion("b", "breite"), "breite");
        assert_eq!(apply_completion("2*", "rand"), "2*rand");
    }

    fn vars(pairs: &[(&str, &str)]) -> Variables {
        let mut v = Variables::default();
        for (n, e) in pairs {
            v.set(n, e);
        }
        v
    }

    #[test]
    fn resolves_in_dependency_order() {
        let v = vars(&[("dicke", "18"), ("rand", "dicke/2"), ("breite", "100 - 2*rand")]);
        let r = v.resolve(&[]);
        assert_eq!(r.value("dicke"), Ok(18.0));
        assert_eq!(r.value("rand"), Ok(9.0));
        assert_eq!(r.value("breite"), Ok(82.0));
    }

    #[test]
    fn order_in_the_list_does_not_matter() {
        // breite is defined before what it needs
        let v = vars(&[("breite", "rand + 1"), ("rand", "5")]);
        let r = v.resolve(&[]);
        assert_eq!(r.value("breite"), Ok(6.0));
    }

    #[test]
    fn host_values_are_available() {
        let v = vars(&[("breite", "platte_b - 20")]);
        let r = v.resolve(&[("platte_b", 300.0)]);
        assert_eq!(r.value("breite"), Ok(280.0));
    }

    #[test]
    fn cycles_are_flagged_not_looped() {
        let v = vars(&[("a", "b + 1"), ("b", "a")]);
        let r = v.resolve(&[]);
        assert!(matches!(r.error("a"), Some(CalcError::Cycle(_))));
        assert!(matches!(r.error("b"), Some(CalcError::Cycle(_))));
        // a self-reference is a cycle too
        let v2 = vars(&[("x", "x + 1")]);
        assert!(matches!(v2.resolve(&[]).error("x"), Some(CalcError::Cycle(_))));
    }

    #[test]
    fn unknown_names_report_but_dont_stop_others() {
        let v = vars(&[("good", "2*3"), ("bad", "nope + 1")]);
        let r = v.resolve(&[]);
        assert_eq!(r.value("good"), Ok(6.0));
        assert_eq!(r.error("bad"), Some(&CalcError::Unknown("nope".into())));
    }

    #[test]
    fn names_for_autocomplete_are_sorted() {
        let v = vars(&[("zeta", "1"), ("alpha", "2")]);
        assert_eq!(v.resolve(&[]).names(), vec!["alpha", "zeta"]);
    }
}
