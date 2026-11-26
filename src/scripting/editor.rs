//! The script editor: the text field of the script window (hosted by `src/app/script_ui.rs`), with syntax highlighting and an
//! autocomplete list that works like a code editor's —
//!
//! - it opens by itself while a command name, a parameter name or a value is typed, and on
//!   Ctrl+Space with nothing typed;
//! - ↑/↓ choose an entry, Enter or Tab take it, Escape closes the list; a click takes it too;
//! - after a command name it lists that command's parameters (required ones first), after `key=` the
//!   values the parameter takes, and after `$` the results of earlier lines (`$c.id`).
//!
//! What to offer is worked out by [`propose`], a plain function of the text and the caret, so it is
//! tested without a window. The list's keys are taken out of the input *before* the text field sees
//! them (otherwise ↑/↓ would move the caret and Enter would break the line), and the list stays open
//! while the pointer is on it, because pressing on it takes the focus away from the field.

use crate::mcp::tools::{catalog, ParamInfo};
use egui::text::{CCursor, CCursorRange};
use egui::{Event, Id, Key, Modifiers, Pos2, Rect};
use std::ops::Range;

/// The id of the script's text field (its caret is kept in egui's memory under it).
pub fn editor_id() -> Id {
    Id::new("script_editor")
}

/// One entry of the autocomplete list.
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    /// What replaces the typed fragment when the entry is taken.
    pub insert: String,
    /// What the list shows.
    pub label: String,
    /// A short note at the right (the parameter's type, the command a variable came from).
    pub hint: String,
    /// What it means, shown under the list for the chosen entry.
    pub detail: String,
    /// Whether the list opens again right after taking it (a command name → its parameters).
    pub reopen: bool,
}

/// What can be offered at the caret: the fragment to replace and the entries.
#[derive(Debug)]
pub struct Proposal {
    pub range: Range<usize>,
    pub items: Vec<Suggestion>,
    /// Something has been typed for it (a list for an empty fragment only opens on request).
    pub typed: bool,
}

/// The open list.
struct Popup {
    range: Range<usize>,
    items: Vec<Suggestion>,
    selected: usize,
    /// Where it opens (just below the fragment being completed).
    at: Pos2,
    /// Where it was drawn last frame, to keep it open while it is being clicked.
    rect: Option<Rect>,
    /// Where each entry was drawn last frame (every control reports its coordinates).
    rows: Vec<Rect>,
    /// The choice was moved with the keys: scroll it into view.
    follow: bool,
}

/// The editor's autocomplete state, kept on the app between frames.
#[derive(Default)]
pub struct EditorState {
    popup: Option<Popup>,
    /// Escape closed the list here (fragment start, fragment): it stays shut until that changes.
    dismissed: Option<(usize, String)>,
    /// The list was asked for on this line (Ctrl+Space, or a command name was just taken), so it
    /// shows even with nothing typed.
    forced: Option<usize>,
}

impl EditorState {
    /// Close the list (the text changed from outside the editor).
    pub fn close(&mut self) {
        self.popup = None;
        self.forced = None;
    }
}

/// The editor on `text`, filling the space it is given: the field (with highlighting), the list's
/// keys before it and the list after it. Free of the app, so it is tested with real egui input.
pub fn editor_ui(ui: &mut egui::Ui, text: &mut String, st: &mut EditorState) {
    let id = editor_id();
    let ctx = ui.ctx().clone();
    editor_keys(ui, id, text, st);
    let height = ui.available_height();
    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
        let mut job = highlight(text);
        job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(job))
    };
    let out = egui::ScrollArea::vertical()
        .id_salt("script_editor_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::TextEdit::multiline(text)
                .id(id)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .min_size(egui::vec2(0.0, height))
                .lock_focus(true)
                .layouter(&mut layouter)
                .show(ui)
        })
        .inner;
    update_completion(&ctx, &out, text, st);
    if let Some(index) = popup_ui(&ctx, st) {
        accept(&ctx, text, st, index);
    }
}

/// The list's keys, taken before the text field sees them: ↑/↓ choose, Enter/Tab take, Escape
/// closes; Ctrl+Space opens the list with nothing typed.
fn editor_keys(ui: &mut egui::Ui, id: Id, text: &mut String, st: &mut EditorState) {
    let ctx = ui.ctx().clone();
    // Escape: egui has already let go of the field's focus; close the list and take it back
    if st.popup.is_some() && ui.input(|i| i.key_pressed(Key::Escape)) {
        if let Some(p) = st.popup.take() {
            let fragment = text.get(p.range.clone()).unwrap_or_default().to_string();
            st.dismissed = Some((p.range.start, fragment));
        }
        st.forced = None;
        ctx.memory_mut(|m| m.request_focus(id));
        return;
    }
    if !ctx.memory(|m| m.has_focus(id)) {
        return;
    }
    if take_ctrl_space(ui) {
        if let Some(caret) = caret_byte(&ctx, id, text) {
            st.forced = Some(line_start(text, caret));
            st.dismissed = None;
        }
    }
    let Some(p) = st.popup.as_mut() else { return };
    let n = p.items.len().max(1);
    let (down, up, take) = ui.input_mut(|i| {
        let down = i.consume_key(Modifiers::NONE, Key::ArrowDown);
        let up = i.consume_key(Modifiers::NONE, Key::ArrowUp);
        let take = i.consume_key(Modifiers::NONE, Key::Enter) || i.consume_key(Modifiers::NONE, Key::Tab);
        (down, up, take)
    });
    if down {
        p.selected = (p.selected + 1) % n;
        p.follow = true;
    }
    if up {
        p.selected = (p.selected + n - 1) % n;
        p.follow = true;
    }
    if take {
        let index = p.selected;
        accept(&ctx, text, st, index);
    }
}

/// After the field is drawn: work out what to offer at the caret and open, update or close the
/// list.
fn update_completion(ctx: &egui::Context, out: &egui::text_edit::TextEditOutput, text: &str, st: &mut EditorState) {
    let cursor = out.cursor_range.filter(|_| out.response.has_focus());
    let Some(cursor) = cursor else {
        // the field lost its focus: keep the list only while the pointer is on it (a click on an
        // entry is under way), else close it
        let over = st.popup.as_ref().and_then(|p| p.rect).zip(ctx.pointer_latest_pos()).is_some_and(|(r, p)| r.contains(p));
        if !over {
            st.popup = None;
        }
        return;
    };
    if cursor.primary.ccursor != cursor.secondary.ccursor {
        st.popup = None; // text is selected: no list
        return;
    }
    let caret = byte_of_char(text, cursor.primary.ccursor.index);
    let line = line_start(text, caret);
    if st.forced.is_some_and(|l| l != line) {
        st.forced = None;
    }
    let Some(prop) = propose(text, caret) else {
        st.popup = None;
        return;
    };
    let fragment = text[prop.range.clone()].to_string();
    if st.dismissed.as_ref().is_some_and(|(start, f)| *start == prop.range.start && *f == fragment) {
        st.popup = None;
        return;
    }
    st.dismissed = None;
    if !prop.typed && st.forced != Some(line) {
        st.popup = None;
        return;
    }
    // the list opens just below the start of the fragment
    let start_char = text[..prop.range.start].chars().count();
    let r = out.galley.pos_from_cursor(&out.galley.from_ccursor(CCursor::new(start_char)));
    let at = out.galley_pos + r.left_bottom().to_vec2() + egui::vec2(-6.0, 3.0);
    let (selected, rect, rows, follow) = match st.popup.take() {
        Some(p) if p.items == prop.items => (p.selected.min(prop.items.len() - 1), p.rect, p.rows, p.follow),
        Some(p) => (0, p.rect, Vec::new(), false),
        None => (0, None, Vec::new(), false),
    };
    st.popup = Some(Popup { range: prop.range, items: prop.items, selected, at, rect, rows, follow });
}

/// Draw the open list; returns the entry that was clicked.
fn popup_ui(ctx: &egui::Context, st: &mut EditorState) -> Option<usize> {
    let p = st.popup.as_mut()?;
    let mut clicked = None;
    let moved = ctx.input(|i| i.pointer.delta() != egui::Vec2::ZERO);
    let mut rows = Vec::with_capacity(p.items.len());
    let shown = egui::Area::new(Id::new("script_autocomplete"))
        .order(egui::Order::Foreground)
        .fixed_pos(p.at)
        .constrain(true)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                let width = 340.0;
                ui.set_width(width);
                egui::ScrollArea::vertical().id_salt("script_autocomplete_scroll").max_height(220.0).show(ui, |ui| {
                    for (i, s) in p.items.iter().enumerate() {
                        let chosen = i == p.selected;
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 22.0), egui::Sense::click());
                        rows.push(rect);
                        let visuals = ui.visuals();
                        if chosen {
                            ui.painter().rect_filled(rect, 3.0, visuals.selection.bg_fill);
                        } else if resp.hovered() {
                            ui.painter().rect_filled(rect, 3.0, visuals.widgets.hovered.weak_bg_fill);
                        }
                        let colour = if chosen { visuals.selection.stroke.color } else { crate::theme::ink() };
                        ui.painter().text(rect.left_center() + egui::vec2(6.0, 0.0), egui::Align2::LEFT_CENTER, &s.label, egui::FontId::monospace(13.0), colour);
                        if !s.hint.is_empty() {
                            ui.painter().text(rect.right_center() - egui::vec2(6.0, 0.0), egui::Align2::RIGHT_CENTER, &s.hint, egui::FontId::proportional(11.0), crate::theme::muted());
                        }
                        if resp.hovered() && moved {
                            p.selected = i;
                        }
                        if resp.clicked() {
                            clicked = Some(i);
                        }
                        if chosen && p.follow {
                            resp.scroll_to_me(None);
                        }
                    }
                });
                p.follow = false;
                // what the chosen entry means
                if let Some(s) = p.items.get(p.selected).filter(|s| !s.detail.is_empty()) {
                    ui.separator();
                    ui.add(egui::Label::new(egui::RichText::new(&s.detail).small().color(crate::theme::ink())).wrap());
                }
                ui.label(egui::RichText::new(t!("↑↓ wählen · Enter/Tab übernehmen · Esc schließen")).small().color(crate::theme::muted()));
            });
        });
    p.rect = Some(shown.response.rect);
    p.rows = rows;
    clicked
}

/// Take entry `index` of the open list: put it in place of the typed fragment, set the caret after
/// it and give the field its focus back.
fn accept(ctx: &egui::Context, text: &mut String, st: &mut EditorState, index: usize) {
    let Some(p) = st.popup.take() else { return };
    let Some(s) = p.items.get(index) else { return };
    if p.range.end > text.len() || !text.is_char_boundary(p.range.start) || !text.is_char_boundary(p.range.end) {
        return;
    }
    text.replace_range(p.range.clone(), &s.insert);
    let caret = p.range.start + s.insert.len();
    set_caret(ctx, editor_id(), text, caret);
    ctx.memory_mut(|m| m.request_focus(editor_id()));
    st.forced = s.reopen.then(|| line_start(text, caret));
    st.dismissed = None;
    ctx.request_repaint();
}

/// Put `line` into `text` below the caret's line (or at the end), the caret after its first `=`.
pub fn insert_line(ctx: &egui::Context, text: &mut String, st: &mut EditorState, line: &str) {
    let id = editor_id();
    let caret = caret_byte(ctx, id, text);
    // the end of the caret's line (before its line break), or the end of the text
    let at = match caret {
        Some(c) => text[c..].find('\n').map_or(text.len(), |i| c + i),
        None => text.len(),
    };
    // on an empty line the new line goes into it; otherwise it starts a new line
    let insert = if text[..at].is_empty() || text[..at].ends_with('\n') { line.to_string() } else { format!("\n{line}") };
    text.insert_str(at, &insert);
    let begin = at + insert.len() - line.len();
    let caret = line.find('=').map_or(begin + line.len(), |i| begin + i + 1);
    set_caret(ctx, id, text, caret);
    ctx.memory_mut(|m| m.request_focus(id));
    st.close();
}

/// What can be offered at byte `caret` of `text`, if anything.
pub fn propose(text: &str, caret: usize) -> Option<Proposal> {
    if caret > text.len() || !text.is_char_boundary(caret) {
        return None;
    }
    let start = line_start(text, caret);
    let before = &text[start..caret];
    if in_comment_or_quotes(before) {
        return None;
    }
    let word_start = start + before.char_indices().rev().find(|(_, c)| c.is_whitespace()).map_or(0, |(i, c)| i + c.len_utf8());
    let word = &text[word_start..caret];
    let mut head: Vec<&str> = text[start..word_start].split_whitespace().collect();
    // "name = command …": the binding in front is not part of the command
    if head.len() >= 2 && head[1] == "=" && is_identifier(head[0]) {
        head.drain(..2);
    }
    let tools = catalog();
    if head.is_empty() {
        // the command name
        if word.contains('=') || word.starts_with('$') {
            return None;
        }
        let items = rank(word, tools.iter().map(|t| (t.name.as_str(), t)))
            .into_iter()
            .map(|t| Suggestion { insert: format!("{} ", t.name), label: t.name.clone(), hint: String::new(), detail: t.description.clone(), reopen: true })
            .collect();
        return finish(Proposal { range: word_start..caret, items, typed: !word.is_empty() }, word);
    }
    let tool = tools.iter().find(|t| t.name == head[0])?;
    let vars = variables(&text[..start]);
    // a value: after "key="
    if let Some(eq) = word.find('=') {
        let param = tool.param(&word[..eq])?;
        let value = &word[eq + 1..];
        // inside a list, the current element
        let cut = value.rfind(['[', ',']).map_or(0, |i| i + 1);
        let part = &value[cut..];
        let items = value_items(param, part, &vars, cut > 0, value.is_empty());
        let range = word_start + eq + 1 + cut..caret;
        return finish(Proposal { range, items, typed: true }, part);
    }
    // a variable inside a list written with spaces ("[1, $c.id]")
    let cut = word.rfind(['[', ',']).map_or(0, |i| i + 1);
    if word[cut..].starts_with('$') {
        let items = variable_items(&word[cut..], &vars, false);
        return finish(Proposal { range: word_start + cut..caret, items, typed: true }, &word[cut..]);
    }
    // a parameter name; those already on the line are not offered again
    let used: Vec<&str> = head[1..].iter().filter_map(|t| t.split('=').next()).collect();
    let free = tool.params.iter().filter(|p| !used.contains(&p.name.as_str()));
    let items = rank(word, free.map(|p| (p.name.as_str(), p)))
        .into_iter()
        .map(|p| Suggestion {
            insert: format!("{}=", p.name),
            label: p.name.clone(),
            hint: param_hint(p),
            detail: p.description.clone(),
            reopen: !p.values.is_empty() || (takes_ids(p) && !vars.is_empty()),
        })
        .collect();
    finish(Proposal { range: word_start..caret, items, typed: !word.is_empty() }, word)
}

/// Drop entries that would change nothing (the value is already typed in full); `None` if none is
/// left.
fn finish(mut prop: Proposal, typed: &str) -> Option<Proposal> {
    prop.items.retain(|s| s.insert != typed);
    (!prop.items.is_empty()).then_some(prop)
}

/// The values to offer for `param` given the typed `part`: earlier results after `$` (or for an
/// id parameter with nothing typed yet), else the parameter's fixed values.
fn value_items(param: &ParamInfo, part: &str, vars: &[(String, String, usize)], in_list: bool, empty: bool) -> Vec<Suggestion> {
    if part.starts_with('$') || (empty && takes_ids(param) && !vars.is_empty()) {
        // a list parameter with nothing typed: offer the id already wrapped in [ ]
        return variable_items(part, vars, param.kind == "array" && !in_list);
    }
    rank(part, param.values.iter().map(|v| (v.as_str(), v)))
        .into_iter()
        .map(|v| Suggestion { insert: v.clone(), label: v.clone(), hint: String::new(), detail: param.description.clone(), reopen: false })
        .collect()
}

/// `$name.id` for each earlier result matching the typed `part` (wrapped in `[ ]` for a list).
fn variable_items(part: &str, vars: &[(String, String, usize)], wrap: bool) -> Vec<Suggestion> {
    let refs: Vec<(String, &(String, String, usize))> = vars.iter().map(|v| (format!("${}.id", v.0), v)).collect();
    rank(part, refs.iter().map(|(r, v)| (r.as_str(), (r, *v))))
        .into_iter()
        .map(|(r, (_, tool, line))| Suggestion {
            insert: if wrap { format!("[{r}]") } else { r.clone() },
            label: r.clone(),
            hint: tool.clone(),
            detail: t!("Ergebnis von „%{tool}“ in Zeile %{line}", tool = tool, line = line).to_string(),
            reopen: false,
        })
        .collect()
}

/// The results kept earlier in the script: (name, command, line number), a later one with the same
/// name replacing an earlier one.
fn variables(text: &str) -> Vec<(String, String, usize)> {
    let mut out: Vec<(String, String, usize)> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let Some((lhs, rhs)) = line.split_once(" = ") else { continue };
        let name = lhs.trim();
        if !is_identifier(name) || in_comment_or_quotes(lhs) {
            continue;
        }
        let tool = rhs.split_whitespace().next().unwrap_or_default().to_string();
        out.retain(|v| v.0 != name);
        out.push((name.to_string(), tool, i + 1));
    }
    out
}

/// Candidates whose key starts with `typed` (any case), then those that contain it, each group in
/// the given order. Everything when nothing is typed.
fn rank<'a, T>(typed: &str, candidates: impl Iterator<Item = (&'a str, T)>) -> Vec<T> {
    let typed = typed.to_lowercase();
    let (mut first, mut rest) = (Vec::new(), Vec::new());
    for (key, item) in candidates {
        let key = key.to_lowercase();
        if key.starts_with(&typed) {
            first.push(item);
        } else if key.contains(&typed) {
            rest.push(item);
        }
    }
    first.extend(rest);
    first
}

/// The note next to a parameter in the list: its type, and whether it must be given.
fn param_hint(p: &ParamInfo) -> String {
    if p.required {
        format!("{} · {}", p.kind, t!("Pflicht"))
    } else {
        p.kind.clone()
    }
}

/// Whether a parameter takes shape or line ids (so earlier results are offered for it).
fn takes_ids(p: &ParamInfo) -> bool {
    p.kind == "array" || p.name == "id" || p.name.ends_with("_id") || p.name.ends_with("_ids")
}

/// Where a comment starts in `line` (its `#`, or the first `/` of `//`), outside quotes.
fn comment_start(line: &str) -> Option<usize> {
    let mut quoted = false;
    let mut prev: Option<(usize, char)> = None;
    for (i, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '#' if !quoted => return Some(i),
            '/' if !quoted && prev.is_some_and(|(_, p)| p == '/') => return prev.map(|(j, _)| j),
            _ => {}
        }
        prev = Some((i, c));
    }
    None
}

/// Whether the caret, at the end of `before` (the line up to it), is in a comment or a quoted text.
fn in_comment_or_quotes(before: &str) -> bool {
    comment_start(before).is_some() || before.chars().filter(|&c| c == '"').count() % 2 == 1
}

/// A bare variable name: letters, digits and underscores, starting with a letter.
fn is_identifier(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The byte where the line holding byte `caret` starts.
fn line_start(text: &str, caret: usize) -> usize {
    text[..caret].rfind('\n').map_or(0, |i| i + 1)
}

/// The byte offset of the `n`-th character (egui keeps the caret in characters).
fn byte_of_char(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map_or(s.len(), |(i, _)| i)
}

/// The caret of the field `id` as a byte offset into `text`, if it has one.
fn caret_byte(ctx: &egui::Context, id: Id, text: &str) -> Option<usize> {
    let state = egui::text_edit::TextEditState::load(ctx, id)?;
    Some(byte_of_char(text, state.cursor.char_range()?.primary.index))
}

/// Put the caret of the field `id` at byte `caret` of `text`.
fn set_caret(ctx: &egui::Context, id: Id, text: &str, caret: usize) {
    let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    let index = text[..caret.min(text.len())].chars().count();
    state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(index))));
    state.store(ctx, id);
}

/// Take a Ctrl+Space out of the input (and the space it would type); returns whether there was one.
fn take_ctrl_space(ui: &mut egui::Ui) -> bool {
    ui.input_mut(|i| {
        let found = i.events.iter().any(|e| matches!(e, Event::Key { key: Key::Space, pressed: true, modifiers, .. } if modifiers.ctrl));
        if found {
            i.events.retain(|e| !matches!(e, Event::Key { key: Key::Space, modifiers, .. } if modifiers.ctrl) && !matches!(e, Event::Text(t) if t == " "));
        }
        found
    })
}

/// Colour the script for the editor: comments muted, the command in ink, parameter names in the
/// accent, `$references` in the constraint blue, numbers in green.
pub fn highlight(text: &str) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};
    let font = egui::FontId::monospace(13.0);
    let ink = crate::theme::ink();
    let muted = crate::theme::muted();
    let mut job = LayoutJob::default();
    let put = |job: &mut LayoutJob, s: &str, color: egui::Color32| job.append(s, 0.0, TextFormat { font_id: font.clone(), color, ..Default::default() });
    for line in text.split_inclusive('\n') {
        // a comment runs to the end of the line
        let (code, comment) = line.split_at(comment_start(line).unwrap_or(line.len()));
        let mut index = 0;
        let mut rest = code;
        while !rest.is_empty() {
            let space = rest.find(|c: char| !c.is_whitespace()).unwrap_or(rest.len());
            if space > 0 {
                put(&mut job, &rest[..space], ink);
                rest = &rest[space..];
                continue;
            }
            let len = word_len(rest);
            let word = &rest[..len];
            if index == 0 || word == "=" {
                put(&mut job, word, ink);
            } else if let Some(eq) = word.find('=') {
                put(&mut job, &word[..eq], crate::theme::ACCENT_DEEP);
                put(&mut job, "=", muted);
                put(&mut job, &word[eq + 1..], value_colour(&word[eq + 1..]));
            } else {
                put(&mut job, word, value_colour(word));
            }
            index += 1;
            rest = &rest[len..];
        }
        if !comment.is_empty() {
            put(&mut job, comment, muted);
        }
    }
    job
}

/// The length of the word at the start of `s`: up to the next space outside quotes.
fn word_len(s: &str) -> usize {
    let mut quoted = false;
    for (i, c) in s.char_indices() {
        if c == '"' {
            quoted = !quoted;
        } else if c.is_whitespace() && !quoted {
            return i;
        }
    }
    s.len()
}

/// The colour of a value: a reference blue, a number green, anything else ink.
fn value_colour(value: &str) -> egui::Color32 {
    let core = value.trim_start_matches('[').trim_end_matches(']');
    if core.contains('$') {
        crate::theme::CONSTRAINT
    } else if core.split(',').all(|v| v.trim().parse::<f64>().is_ok()) && !core.is_empty() {
        egui::Color32::from_rgb(0x2e, 0x7d, 0x32)
    } else {
        crate::theme::ink()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The editor in a real egui context, fed with real input events frame by frame.
    struct Harness {
        ctx: egui::Context,
        text: String,
        st: EditorState,
    }

    impl Harness {
        /// An empty editor with the keyboard focus in it.
        fn new() -> Harness {
            let ctx = egui::Context::default();
            crate::theme::apply(&ctx, false);
            let mut h = Harness { ctx, text: String::new(), st: EditorState::default() };
            h.frame(vec![]);
            h.ctx.memory_mut(|m| m.request_focus(editor_id()));
            h.frame(vec![]);
            h
        }

        fn frame(&mut self, events: Vec<Event>) {
            let raw = egui::RawInput { events, screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 700.0))), ..Default::default() };
            let (text, st) = (&mut self.text, &mut self.st);
            let _ = self.ctx.run(raw, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| editor_ui(ui, text, st));
            });
        }

        fn type_text(&mut self, s: &str) {
            self.frame(vec![Event::Text(s.into())]);
            self.frame(vec![]);
        }

        fn press(&mut self, key: Key, modifiers: Modifiers) {
            self.frame(vec![Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers }]);
            self.frame(vec![]);
        }

        fn click(&mut self, at: Pos2) {
            let button = |pressed| Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Modifiers::NONE };
            self.frame(vec![Event::PointerMoved(at)]);
            self.frame(vec![button(true)]);
            self.frame(vec![button(false)]);
            self.frame(vec![]);
        }

        fn shown(&self) -> Vec<String> {
            self.st.popup.as_ref().map(|p| p.items.iter().map(|s| s.label.clone()).collect()).unwrap_or_default()
        }
    }

    #[test]
    fn arrow_down_and_enter_take_an_entry_and_its_parameters_follow() {
        let mut h = Harness::new();
        h.type_text("draw_");
        assert_eq!(h.shown()[..2], ["draw_rectangle".to_string(), "draw_circle".to_string()]);
        h.press(Key::ArrowDown, Modifiers::NONE);
        h.press(Key::Enter, Modifiers::NONE);
        assert_eq!(h.text, "draw_circle ", "Enter took the second entry instead of breaking the line");
        assert_eq!(h.shown().first().map(String::as_str), Some("cx"), "the command's parameters open at once");
        h.press(Key::Enter, Modifiers::NONE);
        assert_eq!(h.text, "draw_circle cx=");
    }

    #[test]
    fn a_click_on_an_entry_takes_it() {
        let mut h = Harness::new();
        h.type_text("draw_");
        let rows = h.st.popup.as_ref().map(|p| p.rows.clone()).unwrap_or_default();
        assert!(rows.len() >= 3, "the list reports where its entries are: {rows:?}");
        let third = h.shown()[2].clone();
        h.click(rows[2].center());
        assert_eq!(h.text, format!("{third} "));
        h.type_text("cx=1");
        assert_eq!(h.text, format!("{third} cx=1"), "the keyboard focus came back to the field");
    }

    #[test]
    fn escape_closes_the_list_and_typing_goes_on() {
        let mut h = Harness::new();
        h.type_text("draw_");
        assert!(!h.shown().is_empty());
        h.press(Key::Escape, Modifiers::NONE);
        assert!(h.shown().is_empty(), "Escape closed the list");
        h.type_text("c");
        assert_eq!(h.text, "draw_c", "the focus stayed in the field");
        assert_eq!(h.shown(), vec!["draw_circle".to_string()], "typing on opens it again");
    }

    #[test]
    fn ctrl_space_opens_the_list_without_typing_a_space() {
        let mut h = Harness::new();
        let ctrl = Modifiers { ctrl: true, ..Default::default() };
        h.frame(vec![Event::Key { key: Key::Space, physical_key: None, pressed: true, repeat: false, modifiers: ctrl }, Event::Text(" ".into())]);
        h.frame(vec![]);
        assert_eq!(h.text, "");
        assert_eq!(h.shown().len(), catalog().len(), "every command is offered");
    }

    fn labels(text: &str) -> Vec<String> {
        propose(text, text.len()).map(|p| p.items.into_iter().map(|s| s.label).collect()).unwrap_or_default()
    }

    #[test]
    fn a_command_name_is_completed_from_its_start() {
        let l = labels("draw_c");
        assert_eq!(l.first().map(String::as_str), Some("draw_circle"));
    }

    #[test]
    fn a_command_is_found_by_part_of_its_name_too() {
        let l = labels("delete");
        assert!(l.contains(&"guide_delete".to_string()), "{l:?}");
    }

    #[test]
    fn after_a_command_its_parameters_are_offered_required_first() {
        let p = propose("draw_rectangle ", 15).unwrap();
        assert!(!p.typed, "nothing typed yet: it only opens on request");
        let l: Vec<_> = p.items.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(l.last(), Some(&"corner_radius"), "the optional one comes last: {l:?}");
        assert_eq!(p.items[0].insert, format!("{}=", p.items[0].label));
    }

    #[test]
    fn a_parameter_already_given_is_not_offered_again() {
        let l = labels("draw_circle cx=10 c");
        assert!(l.contains(&"cy".to_string()) && !l.contains(&"cx".to_string()), "{l:?}");
    }

    #[test]
    fn a_binding_in_front_does_not_hide_the_command() {
        let l = labels("c = draw_circle r");
        assert_eq!(l, vec!["radius".to_string()]);
    }

    #[test]
    fn fixed_values_are_offered_after_key_equals() {
        assert_eq!(labels("add_toolpath kind=p"), vec!["profile".to_string(), "pocket".to_string()]);
    }

    #[test]
    fn a_value_typed_in_full_closes_the_list() {
        assert!(propose("view mode=3d", 12).is_none(), "Enter must break the line, not take 3d again");
    }

    #[test]
    fn earlier_results_are_offered_after_a_dollar() {
        let text = "c = draw_circle cx=1 cy=2 radius=3\nadd_toolpath kind=profile shape_ids=[$";
        let p = propose(text, text.len()).unwrap();
        assert_eq!(p.items[0].insert, "$c.id");
        assert_eq!(p.items[0].hint, "draw_circle");
    }

    #[test]
    fn an_id_list_with_nothing_typed_gets_the_result_wrapped() {
        let text = "c = draw_circle cx=1 cy=2 radius=3\nadd_toolpath shape_ids=";
        let p = propose(text, text.len()).unwrap();
        assert_eq!(p.items[0].insert, "[$c.id]");
    }

    #[test]
    fn nothing_is_offered_inside_a_comment_or_quotes() {
        assert!(propose("# draw_c", 8).is_none());
        assert!(propose("add_text text=\"draw_c", 21).is_none());
    }

    #[test]
    fn highlighting_keeps_every_character() {
        crate::theme::apply(&egui::Context::default(), false); // the colours come from the theme
        let text = "c = draw_circle cx=1 shape_ids=[$c.id] # Größe\nadd_text text=\"a # b\" // über\nview mode=3d\n";
        assert_eq!(highlight(text).text, text);
    }
}
