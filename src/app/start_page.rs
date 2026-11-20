use super::*;

impl App {
    pub(super) fn start_page_ui(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, canvas());
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect.shrink(40.0)), |ui| {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {

            let avail = ui.available_width();

            // ── hero: welcome + three action cards ──
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.label(egui::RichText::new(t!("Willkommen bei CncCraft")).size(28.0).strong().color(ink()));
                ui.label(egui::RichText::new(t!("Das komplette Werkzeugset für CNC-Einsteiger")).color(muted()));
                ui.add_space(30.0);
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

            // ── recent files: horizontal thumbnail cards ──
            let mut open_recent: Option<PathBuf> = None;
            if !self.recent_files.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(t!("Zuletzt verwendet")).size(16.0).strong().color(ink()));
                });
                ui.add_space(12.0);
                let thumb_card_w = 160.0_f32;
                let thumb_card_h = 140.0_f32;
                let thumb_gap = 14.0_f32;
                let max_shown = ((avail + thumb_gap) / (thumb_card_w + thumb_gap)).floor() as usize;
                let shown = self.recent_files.len().min(max_shown).max(1);
                let row_w = shown as f32 * thumb_card_w + (shown.saturating_sub(1)) as f32 * thumb_gap;
                ui.horizontal(|ui| {
                    ui.add_space(((avail - row_w) / 2.0).max(0.0));
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

            // ── sister apps: distinct panel zone ──
            ui.add_space(40.0);
            let panel_w = 800.0_f32.min(avail);
            ui.horizontal(|ui| {
                ui.add_space(((avail - panel_w) / 2.0).max(0.0));
                ui.vertical(|ui| {
                    ui.set_width(panel_w);
                    egui::Frame::none()
                        .fill(crate::theme::panel())
                        .rounding(14.0)
                        .inner_margin(egui::Margin::symmetric(24.0, 24.0))
                        .show(ui, |ui| {
                            other_apps_ui(ui);
                        });
                });
            });
            ui.add_space(20.0);

            });
        });
    }
}

/// Draws the "more for your CNC" section on the start page.
fn other_apps_ui(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(t!("Noch mehr für deine Fräse")).size(16.0).strong().color(ink()));
        ui.label(egui::RichText::new(t!("Kostenlose Apps aus derselben Werkstatt – laufen direkt im Browser, ohne Installation")).color(muted()));
    });
    ui.add_space(12.0);
    let apps = [
        (
            0,
            "PatternMaster",
            t!("Muster für deine CNC-Fräse"),
            t!("Muster wählen, an den Reglern drehen und sofort in 3D sehen, wie dein Werkstück aussieht. Die Fräsbahnen rechnet PatternMaster für dich."),
            [t!("Rauten, Wellen, Waben"), t!("3D-Vorschau"), t!("Kein Konto")],
            [Color32::from_rgb(0xf8, 0xb8, 0x62), Color32::from_rgb(0xe0, 0x74, 0x3a), Color32::from_rgb(0x1c, 0x11, 0x07)],
            "https://freegroup.github.io/patternmaster/",
        ),
        (
            1,
            "HoruCNC",
            t!("Vom Bild zum CNC-Programm"),
            t!("Zeichnung oder Foto vor die Kamera halten, ein paar Filter wählen und den G-Code für GRBL oder Marlin herunterladen."),
            [t!("Kamera oder Bild"), t!("3D-Relief"), t!("GRBL & Marlin")],
            [Color32::from_rgb(0x64, 0xd2, 0xff), Color32::from_rgb(0x08, 0xb7, 0xff), Color32::from_rgb(0x06, 0x19, 0x21)],
            "https://freegroup.github.io/HoruCNC/",
        ),
    ];
    let (w, h, gap, band_h, shot_h) = (360.0_f32, 340.0_f32, 20.0_f32, 72.0_f32, 126.0_f32);
    let shots = sister_previews(ui.ctx());
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - (2.0 * w + gap)) / 2.0).max(0.0));
        for (which, name, sub, body, chips, [color, deep, on], url) in apps {
            let (slot, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
            let hover = resp.hovered();
            let r = if hover { slot.translate(Vec2::new(0.0, -2.0)) } else { slot };
            let p = ui.painter();
            if hover {
                p.rect_filled(r.translate(Vec2::new(0.0, 5.0)), 12.0, Color32::from_black_alpha(35));
            }
            p.rect(r, 12.0, crate::theme::card(), Stroke::new(if hover { 2.0_f32 } else { 1.0_f32 }, if hover { deep } else { crate::theme::card_edge() }));
            let band = Rect::from_min_size(r.min, Vec2::new(w, band_h));
            let mut mesh = egui::Mesh::default();
            let (n, rad) = (90, 12.0_f32);
            for i in 0..=n {
                let f = i as f32 / n as f32;
                let x = band.left() + f * w;
                let d = (x - band.left()).min(band.right() - x);
                let inset = if d < rad { rad - (rad * rad - (rad - d) * (rad - d)).max(0.0).sqrt() } else { 0.0 };
                let c = Color32::from_rgb(
                    (color.r() as f32 + (deep.r() as f32 - color.r() as f32) * f) as u8,
                    (color.g() as f32 + (deep.g() as f32 - color.g() as f32) * f) as u8,
                    (color.b() as f32 + (deep.b() as f32 - color.b() as f32) * f) as u8,
                );
                mesh.colored_vertex(Pos2::new(x, band.top() + inset), c);
                mesh.colored_vertex(Pos2::new(x, band.bottom()), c);
                if i > 0 {
                    let k = 2 * i as u32;
                    mesh.add_triangle(k - 2, k - 1, k);
                    mesh.add_triangle(k - 1, k, k + 1);
                }
            }
            p.add(egui::Shape::mesh(mesh));
            let deco = p.with_clip_rect(band);
            let soft = Stroke::new(2.0_f32, on.gamma_multiply(0.14));
            if which == 0 {
                for k in 0..5 {
                    let y0 = band.top() + 10.0 + k as f32 * 14.0;
                    let pts: Vec<Pos2> = (0..=40)
                        .map(|i| {
                            let x = band.left() + 150.0 + i as f32 * 6.0;
                            Pos2::new(x, y0 + (x * 0.06 + k as f32).sin() * 5.0)
                        })
                        .collect();
                    deco.add(egui::Shape::line(pts, soft));
                }
            } else {
                let c = Pos2::new(band.right() - 50.0, band.center().y);
                for rad in [14.0, 28.0, 42.0, 56.0, 70.0] {
                    deco.circle_stroke(c, rad, soft);
                }
            }
            let ic = Pos2::new(band.left() + 40.0, band.center().y);
            p.circle_filled(ic, 24.0, Color32::WHITE);
            if which == 0 {
                for i in -1..=1 {
                    for j in -1..=1 {
                        let m = ic + Vec2::new(i as f32 * 9.0, j as f32 * 9.0);
                        let d = 4.2;
                        p.add(egui::Shape::convex_polygon(
                            vec![m + Vec2::new(0.0, -d), m + Vec2::new(d, 0.0), m + Vec2::new(0.0, d), m + Vec2::new(-d, 0.0)],
                            deep,
                            Stroke::NONE,
                        ));
                    }
                }
            } else {
                let st = Stroke::new(2.4_f32, deep);
                p.rect_stroke(Rect::from_center_size(ic + Vec2::new(0.0, 2.0), Vec2::new(28.0, 19.0)), 4.0, st);
                p.rect_filled(Rect::from_center_size(ic + Vec2::new(-5.0, -9.5), Vec2::new(9.0, 4.0)), 1.5, deep);
                p.circle_stroke(ic + Vec2::new(0.0, 2.0), 5.5, st);
            }
            let tx = ic.x + 38.0;
            p.text(Pos2::new(tx, band.center().y - 10.0), Align2::LEFT_CENTER, name, FontId::proportional(21.0), on);
            p.text(Pos2::new(tx, band.center().y + 13.0), Align2::LEFT_CENTER, sub.to_string(), FontId::proportional(12.5), on.gamma_multiply(0.8));

            let shot = Rect::from_min_size(Pos2::new(r.left() + 1.0, band.bottom()), Vec2::new(w - 2.0, shot_h));
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
            p.line_segment([shot.left_bottom(), shot.right_bottom()], Stroke::new(1.0_f32, crate::theme::card_edge()));

            let pad = 16.0;
            let text = p.layout(body.to_string(), FontId::proportional(12.5), ink(), w - 2.0 * pad);
            let ty = shot.bottom() + 12.0;
            let text_h = text.size().y;
            p.galley(Pos2::new(r.left() + pad, ty), text, ink());

            let mut cx = r.left() + pad;
            let cy = ty + text_h + 10.0;
            for chip in chips {
                let g = p.layout_no_wrap(chip.to_string(), FontId::proportional(11.0), ink());
                let cr = Rect::from_min_size(Pos2::new(cx, cy), g.size() + Vec2::new(16.0, 8.0));
                if cr.right() > r.right() - pad {
                    break;
                }
                p.rect_filled(cr, 11.0, color.gamma_multiply(0.25));
                p.galley(cr.min + Vec2::new(8.0, 4.0), g, ink());
                cx = cr.right() + 6.0;
            }

            let g = p.layout_no_wrap(t!("Im Browser öffnen").to_string(), FontId::proportional(13.0), on);
            let size = g.size() + Vec2::new(44.0, 14.0);
            let br = Rect::from_min_size(r.right_bottom() - size - Vec2::new(pad, 14.0), size);
            p.rect_filled(br, br.height() / 2.0, if hover { deep } else { color });
            p.galley(br.min + Vec2::new(14.0, 7.0), g, on);
            let a = Pos2::new(br.right() - 18.0, br.center().y);
            let ast = Stroke::new(2.0_f32, on);
            p.line_segment([a - Vec2::new(5.0, 0.0), a + Vec2::new(5.0, 0.0)], ast);
            p.line_segment([a + Vec2::new(1.0, -4.0), a + Vec2::new(5.0, 0.0)], ast);
            p.line_segment([a + Vec2::new(1.0, 4.0), a + Vec2::new(5.0, 0.0)], ast);

            if hover {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
            resp.on_hover_text(url);
            ui.add_space(gap);
        }
    });
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
