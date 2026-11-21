use super::*;

impl App {
    pub(super) fn save_project(&mut self, ask: bool) {
        if self.selftest.is_some() {
            return;
        }
        let path = if ask || self.path.is_none() {
            rfd::FileDialog::new().add_filter(t!("CncCraft-Projekt"), &["cnccraft"]).set_file_name("projekt.cnccraft").save_file()
        } else {
            self.path.clone()
        };
        let Some(path) = path else { return };
        let proj = Project { job: self.job.clone(), shapes: self.shapes.clone(), toolpaths: self.toolpaths.clone(), vars: self.vars.clone(), constraints: self.distances.clone(), aligns: self.aligns.clone(), layers: self.layers.clone(), hooks: gcode::OldPostHooks::default(), postprocs: self.postprocs.clone() };
        match serde_json::to_string_pretty(&proj).map_err(|e| e.to_string()).and_then(|s| std::fs::write(&path, s).map_err(|e| e.to_string())) {
            Ok(()) => {
                self.add_recent(&path);
                self.path = Some(path);
                self.thumb_pending = 1;
                self.mark_clean();
            }
            Err(e) => self.status = format!("Speichern fehlgeschlagen: {e}"),
        }
    }

    pub(super) fn open_project(&mut self) {
        if self.selftest.is_some() {
            return;
        }
        let Some(path) = rfd::FileDialog::new().add_filter(t!("CncCraft-Projekt"), &["cnccraft", "funke"]).pick_file() else { return };
        self.open_path(path);
    }

    pub(super) fn open_path(&mut self, path: PathBuf) {
        let res = std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str::<Project>(&s).map_err(|e| e.to_string()));
        match res {
            Ok(p) => {
                self.new_job(p.job);
                self.next_id = p.shapes.iter().map(|s| s.id).max().unwrap_or(1) + 1;
                self.shapes = p.shapes;
                self.toolpaths = p.toolpaths;
                self.vars = p.vars;
                self.distances = p.constraints;
                self.aligns = p.aligns;
                self.layers = p.layers.repaired();
                self.layers.sync(&mut self.shapes);
                self.image_textures.clear();
                self.calibrate_mode = None;
                if !p.postprocs.is_empty() {
                    self.postprocs = p.postprocs;
                } else {
                    self.postprocs = gcode::migrate_old_format(&p.hooks);
                }
                self.post_idx = self.post_idx.min(self.postprocs.len().saturating_sub(1));
                self.vars_committed.clear();
                self.var_events.clear();
                self.vars_dirty = true;
                self.add_recent(&path);
                self.path = Some(path);
                self.mark_clean();
            }
            Err(e) => {
                self.recent_files.retain(|p| *p != path);
                self.status = format!("Öffnen fehlgeschlagen: {e}");
            }
        }
    }

    pub(super) fn export_choice(&self, indices: &[usize]) -> bool {
        self.tool_groups(indices).len() > 1
    }

    pub(super) fn export_gcode(&mut self) {
        let indices = self.output_indices();
        if indices.is_empty() {
            self.status = match self.output_scope {
                OutputScope::All => "Keine Werkzeugwege vorhanden".into(),
                _ => "Keine sichtbaren Werkzeugwege zum Speichern".into(),
            };
            return;
        }
        let per_tool = self.export_choice(&indices) && (self.export_per_tool || !self.pp().tool_change);
        self.write_export(&indices, per_tool);
    }

    pub(super) fn tool_groups(&self, indices: &[usize]) -> Vec<Vec<usize>> {
        let tools: Vec<&Tool> = indices.iter().map(|&i| &self.toolpaths[i].tool).collect();
        crate::toolgroups::groups(&tools)
    }

    pub(super) fn pp(&self) -> &gcode::PostProcessor {
        &self.postprocs[self.post_idx.min(self.postprocs.len().saturating_sub(1))]
    }

    pub(super) fn add_recent(&mut self, path: &std::path::Path) {
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_path_buf());
        self.recent_files.truncate(10);
    }

    pub(super) fn export_stem(&self) -> String {
        let stem = self.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let part = crate::toolgroups::file_part(&stem);
        if part.is_empty() { "werkzeugwege".into() } else { part }
    }

    /// What every exported file's header says besides the job: which toolpaths and the material.
    fn export_meta(&self, count: usize) -> crate::transform::to_gcode::ExportMeta {
        crate::transform::to_gcode::ExportMeta { scope: Some(self.export_scope_text(count)), material: self.materials.get(&self.job.material).cloned(), ..Default::default() }
    }

    fn write_export(&mut self, indices: &[usize], per_tool: bool) {
        if self.selftest.is_some() {
            return;
        }
        let paths: Vec<&Toolpath> = indices.iter().map(|&i| &self.toolpaths[i]).collect();
        if per_tool {
            let Some(file) = rfd::FileDialog::new()
                .add_filter(t!("ZIP-Archiv"), &["zip"])
                .set_file_name(format!("{}.zip", self.export_stem()))
                .save_file()
            else {
                return;
            };
            let stem = file.file_stem().map(|s| crate::toolgroups::file_part(&s.to_string_lossy())).filter(|s| !s.is_empty()).unwrap_or_else(|| "werkzeugwege".into());
            let base = self.export_meta(indices.len());
            let files: Vec<(String, Vec<u8>)> = crate::toolgroups::files(&self.job, &paths, self.pp(), &stem, &base).into_iter().map(|(n, t)| (n, t.into_bytes())).collect();
            let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
            match std::fs::write(&file, crate::zipfile::stored(&files, seconds)) {
                Ok(()) => {}
                Err(e) => self.status = format!("Fehler: {e}"),
            }
            return;
        }
        let pp = self.pp().clone();
        let Some(file) = rfd::FileDialog::new()
            .add_filter(&pp.label(), &[&pp.ext])
            .set_file_name(format!("{}.{}", self.export_stem(), pp.ext))
            .save_file()
        else {
            return;
        };
        let meta = self.export_meta(indices.len());
        let text = crate::transform::to_gcode::write_with_meta(&self.job, &paths, &pp, &meta);
        match std::fs::write(&file, text) {
            Ok(()) => {}
            Err(e) => self.status = format!("Fehler: {e}"),
        }
    }
}
