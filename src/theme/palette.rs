//! The colour slots every theme must fill.

use egui::Color32;

/// Every colour the interface uses. A theme is one instance of this struct.
#[derive(Clone)]
#[allow(dead_code)]
pub struct Palette {
    // --- brand / sheet constants (same in every theme) -----------------------
    pub accent: Color32,
    pub accent_deep: Color32,
    /// Drawing ink on the (always-white) sheet.
    pub sheet_ink: Color32,
    /// Muted annotations on the sheet.
    pub sheet_muted: Color32,
    pub select: Color32,
    pub sheet_label: Color32,
    pub sheet_label_edge: Color32,
    pub constraint: Color32,
    pub primary: Color32,

    // --- interface -----------------------------------------------------------
    pub canvas: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub disabled: Color32,
    pub panel: Color32,
    pub surface: Color32,
    pub separator: Color32,
    pub hairline: Color32,
    pub outline: Color32,
    pub hover: Color32,
    pub row: Color32,
    pub row_active: Color32,
    pub selected_fill: Color32,
    pub icon_line: Color32,
    pub icon_top: Color32,
    pub icon_bottom: Color32,
    pub icon_back: Color32,
    pub field_bg: Color32,
    pub field_stroke: Color32,
    pub chip: Color32,
    pub chip_hover: Color32,
    pub card: Color32,
    pub card_edge: Color32,
    pub error: Color32,
    pub grip: Color32,
    pub warning: Color32,
    pub key_bg: Color32,
    pub layer_active: Color32,
    pub on_accent: Color32,
    pub spline_arm: Color32,
    pub halo: Color32,

    // --- G-code syntax -------------------------------------------------------
    pub gcode_cmd: Color32,
    pub gcode_num: Color32,
    pub gcode_comment: Color32,
    pub gcode_placeholder: Color32,
    pub gcode_text: Color32,

    // --- simulation ----------------------------------------------------------
    pub sim_rapid: Color32,
    pub sim_plunge: Color32,

    // --- egui visuals (theme-specific overrides for `apply`) ------------------
    pub extreme_bg: Color32,
    pub selection_bg: Color32,
    pub active_bg: Color32,
    pub inactive_weak_bg: Color32,
    pub shadow_alpha: u8,

    // --- sheet board desk colour ---------------------------------------------
    pub board_desk: Color32,

    /// Whether this palette counts as dark (for egui's base visuals).
    pub is_dark: bool,
}
