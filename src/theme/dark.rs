//! The dark theme.

use egui::Color32;
use super::palette::Palette;

pub fn palette() -> Palette {
    Palette {
        accent:           Color32::from_rgb(0xff, 0xc0, 0x00),
        accent_deep:      Color32::from_rgb(0xf8, 0xb0, 0x02),
        sheet_ink:        Color32::from_rgb(0x1f, 0x29, 0x37),
        sheet_muted:      Color32::from_rgb(0x6b, 0x72, 0x80),
        select:           Color32::from_rgb(0xe0, 0x20, 0xa0),
        sheet_label:      Color32::WHITE,
        sheet_label_edge: Color32::from_rgb(0xb4, 0xb8, 0xc0),
        constraint:       Color32::from_rgb(0x1e, 0x6f, 0xd9),
        primary:          Color32::from_rgb(0x2a, 0x6c, 0xf0),

        canvas:        Color32::from_rgb(0x17, 0x18, 0x1b),
        ink:           Color32::from_rgb(0xe9, 0xeb, 0xef),
        muted:         Color32::from_rgb(0xa9, 0xaf, 0xbb),
        disabled:      Color32::from_rgb(0x5b, 0x5e, 0x68),
        panel:         Color32::from_rgb(0x20, 0x22, 0x26),
        surface:       Color32::from_rgb(0x2a, 0x2c, 0x32),
        separator:     Color32::from_rgb(0x37, 0x3a, 0x41),
        hairline:      Color32::from_rgb(0x37, 0x3a, 0x41),
        outline:       Color32::from_rgb(0x62, 0x66, 0x72),
        hover:         Color32::from_rgb(0x37, 0x3a, 0x42),
        row:           Color32::from_rgb(0x2a, 0x2c, 0x32),
        row_active:    Color32::from_rgb(0x3a, 0x3d, 0x45),
        selected_fill: Color32::from_rgb(0x5a, 0x47, 0x12),
        icon_line:     Color32::from_rgb(0x6c, 0xb8, 0xff),
        icon_top:      Color32::from_rgb(0x3e, 0x8f, 0xd4),
        icon_bottom:   Color32::from_rgb(0x24, 0x62, 0xa2),
        icon_back:     Color32::from_rgb(0x1d, 0x24, 0x30),
        field_bg:      Color32::from_rgb(0x18, 0x19, 0x1d),
        field_stroke:  Color32::from_rgb(0x4a, 0x4d, 0x56),
        chip:          Color32::from_rgb(0x3b, 0x3e, 0x47),
        chip_hover:    Color32::from_rgb(0x4c, 0x50, 0x5a),
        card:          Color32::from_rgb(0x30, 0x32, 0x39),
        card_edge:     Color32::from_rgb(0x64, 0x68, 0x74),
        error:         Color32::from_rgb(0xff, 0x8f, 0x8f),
        grip:          Color32::from_rgb(0x70, 0x74, 0x7e),
        warning:       Color32::from_rgb(0xf0, 0xa0, 0x40),
        key_bg:        Color32::from_rgb(0x6a, 0x68, 0x64),
        layer_active:  Color32::from_rgb(0x1e, 0x42, 0x7a),
        on_accent:     Color32::WHITE,
        spline_arm:    Color32::from_rgb(0x60, 0x98, 0xe0),
        halo:          Color32::from_white_alpha(200),

        gcode_cmd:         Color32::from_rgb(0x56, 0xb4, 0xe9),
        gcode_num:         Color32::from_rgb(0xe6, 0x9f, 0x00),
        gcode_comment:     Color32::from_rgb(0x78, 0x8c, 0x78),
        gcode_placeholder: Color32::from_rgb(0xc8, 0x82, 0xdc),
        gcode_text:        Color32::from_rgb(0xd2, 0xd2, 0xd2),

        sim_rapid:  Color32::from_rgba_premultiplied(0x00, 0xbb, 0xcc, 0xa0),
        sim_plunge: Color32::from_rgb(0xe8, 0xc0, 0x30),

        extreme_bg:      Color32::from_rgb(0x1a, 0x1b, 0x1f),
        selection_bg:    Color32::from_rgb(0x6b, 0x55, 0x10),
        active_bg:       Color32::from_rgb(0x4a, 0x4d, 0x56),
        inactive_weak_bg: Color32::from_rgb(0x3b, 0x3e, 0x47),
        shadow_alpha:    90,

        board_desk: Color32::from_rgb(0x2d, 0x2f, 0x35),

        is_dark: true,
    }
}
