use super::*;

/// How many pages the start-page carousel has: welcome, PatternMaster, HoruCNC.
const PAGES: usize = 3;

impl App {
    pub(super) fn start_page_ui(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, canvas());
        self.start_page_idx = self.start_page_idx.min(PAGES - 1);

        // the carousel: a wide arrow on each side, the page between them, dots underneath
        let area = rect.shrink2(Vec2::new(24.0, 24.0));
        let arrow_w = 56.0;
        let page_rect = Rect::from_min_max(Pos2::new(area.left() + arrow_w, area.top()), Pos2::new(area.right() - arrow_w, area.bottom() - 40.0));

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(page_rect), |ui| match self.start_page_idx {
            0 => self.start_welcome_page(ui),
            1 => self.start_sister_page(ui, 0),
            _ => self.start_sister_page(ui, 1),
        });

        // the left/right arrows (disabled at the ends)
        let idx = self.start_page_idx;
        if self.carousel_arrow(ui, Rect::from_center_size(Pos2::new(area.left() + arrow_w / 2.0, area.center().y), Vec2::new(40.0, 56.0)), false, idx > 0) {
            self.start_page_idx = idx.saturating_sub(1);
        }
        if self.carousel_arrow(ui, Rect::from_center_size(Pos2::new(area.right() - arrow_w / 2.0, area.center().y), Vec2::new(40.0, 56.0)), true, idx + 1 < PAGES) {
            self.start_page_idx = (idx + 1).min(PAGES - 1);
        }

        // the dots: one per page, the current one filled; a click jumps to it
        let dot_gap = 22.0;
        let dots_w = (PAGES - 1) as f32 * dot_gap;
        let dy = area.bottom() - 16.0;
        for i in 0..PAGES {
            let c = Pos2::new(area.center().x - dots_w / 2.0 + i as f32 * dot_gap, dy);
            let dot = ui.interact(Rect::from_center_size(c, Vec2::splat(20.0)), egui::Id::new(("start_dot", i)), Sense::click());
            let on = i == self.start_page_idx;
            let col = if on { ACCENT } else if dot.hovered() { ink() } else { crate::theme::card_edge() };
            ui.painter().circle_filled(c, if on { 6.0 } else { 5.0 }, col);
            if dot.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if dot.clicked() {
                self.start_page_idx = i;
            }
        }
    }

    /// One carousel arrow (‹ or ›); returns whether it was clicked. Faint and non-clickable at an end.
    fn carousel_arrow(&self, ui: &mut egui::Ui, r: Rect, right: bool, enabled: bool) -> bool {
        let resp = ui.interact(r, egui::Id::new(("start_arrow", right)), if enabled { Sense::click() } else { Sense::hover() });
        let hover = enabled && resp.hovered();
        let col = if !enabled { crate::theme::card_edge().gamma_multiply(0.5) } else if hover { ACCENT } else { muted() };
        let c = r.center();
        let st = Stroke::new(3.0_f32, col);
        let dx = if right { 7.0 } else { -7.0 };
        ui.painter().line_segment([c + Vec2::new(-dx, -11.0), c + Vec2::new(dx, 0.0)], st);
        ui.painter().line_segment([c + Vec2::new(dx, 0.0), c + Vec2::new(-dx, 11.0)], st);
        if hover {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        resp.clicked()
    }

    /// Page 1 of the carousel: welcome, the action cards and the recent files.
    fn start_welcome_page(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let avail = ui.available_width();
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                // the app name with its version beside it at the same size, the version in the accent
                // colour; one laid-out label so it stays centred as a single block
                let mut job = egui::text::LayoutJob::default();
                let font = egui::FontId::proportional(28.0);
                job.append(&t!("Willkommen bei CncCraft"), 0.0, egui::text::TextFormat { font_id: font.clone(), color: ink(), ..Default::default() });
                job.append(&format!("  v{}", env!("CARGO_PKG_VERSION")), 0.0, egui::text::TextFormat { font_id: font, color: ACCENT, ..Default::default() });
                ui.label(job);
                ui.label(egui::RichText::new(t!("Das komplette Werkzeugset für CNC-Einsteiger")).color(muted()));
                ui.add_space(26.0);
            });
            ui.horizontal(|ui| {
                let total = 3.0 * 220.0 + 2.0 * 20.0;
                ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
                for (title, sub, which) in [
                    (t!("Neue Datei erstellen"), t!("Materialgröße und Dicke festlegen"), 0),
                    (t!("Datei öffnen"), t!("Ein gespeichertes Projekt laden"), 1),
                    (t!("Beispiel öffnen"), t!("Schild mit Schrift, Tasche und Bohrungen"), 2),
                ] {
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(220.0, 170.0), Sense::click());
                    let hover = resp.hovered();
                    ui.painter().rect(r, 10.0, crate::theme::card(), Stroke::new(if hover { 2.0_f32 } else { 1.0_f32 }, if hover { ACCENT } else { crate::theme::card_edge() }));
                    let ic = Rect::from_center_size(Pos2::new(r.center().x, r.top() + 55.0), Vec2::splat(56.0));
                    ui.painter().rect_filled(ic, 10.0, if hover { ACCENT } else { Color32::from_rgb(0xff, 0xf3, 0xcc) });
                    let p = ui.painter();
                    let c = ic.center();
                    let st = Stroke::new(2.5_f32, INK);
                    match which {
                        0 => {
                            p.rect_stroke(Rect::from_center_size(c, Vec2::new(26.0, 32.0)), 2.0, st);
                            p.line_segment([c + Vec2::new(-6.0, 0.0), c + Vec2::new(6.0, 0.0)], st);
                            p.line_segment([c + Vec2::new(0.0, -6.0), c + Vec2::new(0.0, 6.0)], st);
                        }
                        1 => {
                            p.rect_stroke(Rect::from_center_size(c + Vec2::new(0.0, 3.0), Vec2::new(32.0, 22.0)), 2.0, st);
                            p.line_segment([c + Vec2::new(-16.0, -8.0), c + Vec2::new(-4.0, -8.0)], st);
                        }
                        _ => {
                            p.text(c, Align2::CENTER_CENTER, "Aa", FontId::proportional(24.0), INK);
                        }
                    }
                    p.text(Pos2::new(r.center().x, r.top() + 110.0), Align2::CENTER_CENTER, title, FontId::proportional(16.0), ink());
                    p.text(Pos2::new(r.center().x, r.top() + 135.0), Align2::CENTER_CENTER, sub, FontId::proportional(11.5), muted());
                    if resp.clicked() {
                        match which {
                            0 => self.guarded(None, Pending::NewJob),
                            1 => self.guarded(None, Pending::Open),
                            _ => self.guarded(None, Pending::Example),
                        }
                    }
                    ui.add_space(20.0);
                }
            });

            // the recent files below, aligned to the block of action cards above (same left edge
            // and width), left-aligned, wrapping across that width
            let mut open_recent: Option<PathBuf> = None;
            if !self.recent_files.is_empty() {
                let block_w = 3.0 * 220.0 + 2.0 * 20.0;
                let left_pad = ((avail - block_w) / 2.0).max(0.0);
                ui.add_space(26.0);
                ui.horizontal(|ui| {
                    ui.add_space(left_pad);
                    ui.label(egui::RichText::new(t!("Zuletzt verwendet")).size(16.0).strong().color(ink()));
                });
                ui.add_space(12.0);
                let (thumb_card_w, thumb_card_h, thumb_gap) = (160.0_f32, 140.0_f32, 14.0_f32);
                // how many fit across the cards' block, so the row never runs wider than them
                let per_row = (((block_w + thumb_gap) / (thumb_card_w + thumb_gap)).floor() as usize).max(1);
                let shown = self.recent_files.len().min(per_row);
                ui.horizontal(|ui| {
                    ui.add_space(left_pad);
                    for (i, path) in self.recent_files.iter().enumerate() {
                        if i >= shown { break; }
                        let name = path.file_stem().map_or_else(|| "?".into(), |s| s.to_string_lossy().into_owned());
                        let thumb = thumb_path(path);
                        let (r, resp) = ui.allocate_exact_size(Vec2::new(thumb_card_w, thumb_card_h), Sense::click());
                        let hover = resp.hovered();
                        let p = ui.painter();
                        p.rect(r, 8.0, crate::theme::card(), Stroke::new(if hover { 2.0_f32 } else { 1.0_f32 }, if hover { ACCENT } else { crate::theme::card_edge() }));
                        let img_rect = Rect::from_min_size(r.min + Vec2::new(6.0, 6.0), Vec2::new(thumb_card_w - 12.0, thumb_card_h - 40.0));
                        let tex_id = egui::Id::new(("thumb_tex", path.as_os_str()));
                        let loaded = ui.data(|d| d.get_temp::<egui::TextureId>(tex_id));
                        let tid = loaded.unwrap_or_else(|| {
                            if let Ok(img) = image::open(&thumb) {
                                let rgba = img.to_rgba8();
                                let size = [rgba.width() as usize, rgba.height() as usize];
                                let ci = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                                let t = ui.ctx().load_texture(format!("thumb_{}", path.display()), ci, egui::TextureOptions::LINEAR);
                                let id = t.id();
                                ui.data_mut(|d| d.insert_temp(tex_id, id));
                                ui.data_mut(|d| d.insert_temp(egui::Id::new(("thumb_handle", path.as_os_str())), t));
                                id
                            } else {
                                egui::TextureId::default()
                            }
                        });
                        if tid != egui::TextureId::default() {
                            p.image(tid, img_rect, Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)), Color32::WHITE);
                            p.rect_stroke(img_rect, 4.0, Stroke::new(1.0_f32, crate::theme::card_edge()));
                        } else {
                            p.rect_filled(img_rect, 4.0, crate::theme::surface());
                        }
                        let name_trunc: String = if name.chars().count() > 18 { format!("{}…", name.chars().take(17).collect::<String>()) } else { name };
                        p.text(Pos2::new(r.center().x, r.bottom() - 16.0), Align2::CENTER_CENTER, name_trunc, FontId::proportional(12.0), ink());
                        if hover {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if resp.clicked() {
                            open_recent = Some(path.clone());
                        }
                        resp.on_hover_text(path.display().to_string());
                        if i + 1 < shown {
                            ui.add_space(thumb_gap);
                        }
                    }
                });
            }
            if let Some(path) = open_recent {
                self.open_path(path);
            }
            ui.add_space(20.0);
        });
    }

    /// Page 2/3 of the carousel: one sister app (0 = PatternMaster, 1 = HoruCNC) as a full page —
    /// a wide header band, a large preview and the text filling the width, not a small card.
    fn start_sister_page(&mut self, ui: &mut egui::Ui, which: usize) {
        let page = ui.available_rect_before_wrap();
        if let Some(url) = sister_app_page(ui, which, page) {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
        }
    }
}

/// The two sister apps' data (colours, text, URL).
fn sister_apps() -> [SisterApp; 2] {
    [
        SisterApp {
            name: "PatternMaster",
            sub: t!("Muster für deine CNC-Fräse"),
            body: t!("Muster wählen, an den Reglern drehen und sofort in 3D sehen, wie dein Werkstück aussieht. Die Fräsbahnen rechnet PatternMaster für dich."),
            chips: [t!("Rauten, Wellen, Waben"), t!("3D-Vorschau"), t!("Kein Konto")],
            colors: [Color32::from_rgb(0xf8, 0xb8, 0x62), Color32::from_rgb(0xe0, 0x74, 0x3a), Color32::from_rgb(0x1c, 0x11, 0x07)],
            url: "https://freegroup.github.io/patternmaster/",
        },
        SisterApp {
            name: "HoruCNC",
            sub: t!("Vom Bild zum CNC-Programm"),
            body: t!("Zeichnung oder Foto vor die Kamera halten, ein paar Filter wählen und den G-Code für GRBL oder Marlin herunterladen."),
            chips: [t!("Kamera oder Bild"), t!("3D-Relief"), t!("GRBL & Marlin")],
            colors: [Color32::from_rgb(0x64, 0xd2, 0xff), Color32::from_rgb(0x08, 0xb7, 0xff), Color32::from_rgb(0x06, 0x19, 0x21)],
            url: "https://freegroup.github.io/HoruCNC/",
        },
    ]
}

struct SisterApp {
    name: &'static str,
    sub: String,
    body: String,
    chips: [String; 3],
    colors: [Color32; 3],
    url: &'static str,
}

/// Draw one sister app as a full page filling `page`: a wide header band at the top, a large
/// centred preview, the description, chips and a call-to-action. Returns its URL when clicked.
fn sister_app_page(ui: &mut egui::Ui, which: usize, page: Rect) -> Option<&'static str> {
    let app = &sister_apps()[which];
    let SisterApp { name, sub, body, chips, colors: [color, deep, on], url, .. } = app;
    let (color, deep, on) = (*color, *deep, *on);
    let p = ui.painter();

    // the header band spans the whole page width, with a left-to-right gradient
    let band_h = 120.0_f32;
    let band = Rect::from_min_size(page.min, Vec2::new(page.width(), band_h));
    let mut mesh = egui::Mesh::default();
    let n = 120;
    for i in 0..=n {
        let f = i as f32 / n as f32;
        let x = band.left() + f * band.width();
        let c = Color32::from_rgb(
            (color.r() as f32 + (deep.r() as f32 - color.r() as f32) * f) as u8,
            (color.g() as f32 + (deep.g() as f32 - color.g() as f32) * f) as u8,
            (color.b() as f32 + (deep.b() as f32 - color.b() as f32) * f) as u8,
        );
        mesh.colored_vertex(Pos2::new(x, band.top()), c);
        mesh.colored_vertex(Pos2::new(x, band.bottom()), c);
        if i > 0 {
            let k = 2 * i as u32;
            mesh.add_triangle(k - 2, k - 1, k);
            mesh.add_triangle(k - 1, k, k + 1);
        }
    }
    p.add(egui::Shape::mesh(mesh));
    // faint decoration in the band (waves for PatternMaster, rings for HoruCNC)
    let deco = p.with_clip_rect(band);
    let soft = Stroke::new(2.0_f32, on.gamma_multiply(0.12));
    if which == 0 {
        for k in 0..6 {
            let y0 = band.top() + 14.0 + k as f32 * 18.0;
            let pts: Vec<Pos2> = (0..=80).map(|i| {
                let x = band.right() - 560.0 + i as f32 * 7.0;
                Pos2::new(x, y0 + (x * 0.05 + k as f32).sin() * 6.0)
            }).collect();
            deco.add(egui::Shape::line(pts, soft));
        }
    } else {
        let cc = Pos2::new(band.right() - 90.0, band.center().y);
        for rad in [18.0, 36.0, 54.0, 72.0, 92.0] {
            deco.circle_stroke(cc, rad, soft);
        }
    }
    // the icon and the app's name
    let ic = Pos2::new(band.left() + 70.0, band.center().y);
    p.circle_filled(ic, 34.0, Color32::WHITE);
    if which == 0 {
        for i in -1..=1 {
            for j in -1..=1 {
                let m = ic + Vec2::new(i as f32 * 13.0, j as f32 * 13.0);
                let d = 6.0;
                p.add(egui::Shape::convex_polygon(vec![m + Vec2::new(0.0, -d), m + Vec2::new(d, 0.0), m + Vec2::new(0.0, d), m + Vec2::new(-d, 0.0)], deep, Stroke::NONE));
            }
        }
    } else {
        let st = Stroke::new(3.0_f32, deep);
        p.rect_stroke(Rect::from_center_size(ic + Vec2::new(0.0, 3.0), Vec2::new(40.0, 28.0)), 5.0, st);
        p.rect_filled(Rect::from_center_size(ic + Vec2::new(-7.0, -14.0), Vec2::new(13.0, 6.0)), 2.0, deep);
        p.circle_stroke(ic + Vec2::new(0.0, 3.0), 8.0, st);
    }
    let tx = ic.x + 54.0;
    p.text(Pos2::new(tx, band.center().y - 14.0), Align2::LEFT_CENTER, *name, FontId::proportional(32.0), on);
    p.text(Pos2::new(tx, band.center().y + 18.0), Align2::LEFT_CENTER, sub.clone(), FontId::proportional(16.0), on.gamma_multiply(0.85));

    // the preview, large and centred below the band
    let pad = 48.0_f32.min(page.width() * 0.08);
    let content_w = page.width() - 2.0 * pad;
    let shot_w = content_w.min(720.0);
    let shot_h = (page.height() - band_h - 220.0).clamp(160.0, 420.0);
    let shot = Rect::from_min_size(Pos2::new(page.center().x - shot_w / 2.0, band.bottom() + 28.0), Vec2::new(shot_w, shot_h));
    let shots = sister_previews(ui.ctx());
    let tex = &shots[which];
    let [tw, th] = tex.size().map(|v| v as f32);
    let (sa, ta) = (shot.width() / shot.height(), tw / th);
    let uv = if ta > sa {
        let k = sa / ta;
        Rect::from_min_max(Pos2::new((1.0 - k) / 2.0, 0.0), Pos2::new((1.0 + k) / 2.0, 1.0))
    } else {
        let k = ta / sa;
        Rect::from_min_max(Pos2::new(0.0, (1.0 - k) / 2.0), Pos2::new(1.0, (1.0 + k) / 2.0))
    };
    p.image(tex.id(), shot, uv, Color32::WHITE);
    p.rect_stroke(shot, 6.0, Stroke::new(1.0_f32, crate::theme::card_edge()));

    // the description, centred under the preview
    let text = p.layout(body.clone(), FontId::proportional(15.0), ink(), shot_w);
    let text_h = text.size().y;
    let ty = shot.bottom() + 22.0;
    p.galley(Pos2::new(page.center().x - text.size().x / 2.0, ty), text, ink());

    // the chips, centred in one row
    let chip_galleys: Vec<_> = chips.iter().map(|c| p.layout_no_wrap(c.clone(), FontId::proportional(12.5), ink())).collect();
    let chip_gap = 8.0;
    let chips_w: f32 = chip_galleys.iter().map(|g| g.size().x + 20.0 + chip_gap).sum::<f32>() - chip_gap;
    let mut cx = page.center().x - chips_w / 2.0;
    let cy = ty + text_h + 16.0;
    for g in chip_galleys {
        let cr = Rect::from_min_size(Pos2::new(cx, cy), g.size() + Vec2::new(20.0, 10.0));
        p.rect_filled(cr, 12.0, color.gamma_multiply(0.25));
        p.galley(cr.min + Vec2::new(10.0, 5.0), g, ink());
        cx = cr.right() + chip_gap;
    }

    // the call-to-action button, centred under the chips; the whole button is clickable
    let g = p.layout_no_wrap(t!("Im Browser öffnen").to_string(), FontId::proportional(15.0), on);
    let size = g.size() + Vec2::new(56.0, 20.0);
    let br = Rect::from_min_size(Pos2::new(page.center().x - size.x / 2.0, cy + 48.0), size);
    let resp = ui.interact(br, egui::Id::new(("sister_cta", which)), Sense::click());
    let hover = resp.hovered();
    let p = ui.painter();
    p.rect_filled(br, br.height() / 2.0, if hover { deep } else { color });
    p.galley(br.min + Vec2::new(18.0, 10.0), g, on);
    let a = Pos2::new(br.right() - 22.0, br.center().y);
    let ast = Stroke::new(2.2_f32, on);
    p.line_segment([a - Vec2::new(6.0, 0.0), a + Vec2::new(6.0, 0.0)], ast);
    p.line_segment([a + Vec2::new(1.0, -5.0), a + Vec2::new(6.0, 0.0)], ast);
    p.line_segment([a + Vec2::new(1.0, 5.0), a + Vec2::new(6.0, 0.0)], ast);
    if hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let clicked = resp.clicked();
    resp.on_hover_text(*url);
    clicked.then_some(*url)
}

fn sister_previews(ctx: &egui::Context) -> [egui::TextureHandle; 2] {
    let id = egui::Id::new("sister_previews");
    if let Some(t) = ctx.data(|d| d.get_temp::<[egui::TextureHandle; 2]>(id)) {
        return t;
    }
    let load = |name: &str, bytes: &[u8]| {
        let img = image::load_from_memory(bytes).expect("bundled preview image").to_rgba8();
        let size = [img.width() as usize, img.height() as usize];
        ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied(size, &img), egui::TextureOptions::LINEAR)
    };
    let t = [
        load("sister_patternmaster", include_bytes!("../../assets/sister/patternmaster.png")),
        load("sister_horucnc", include_bytes!("../../assets/sister/horucnc.png")),
    ];
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    t
}
