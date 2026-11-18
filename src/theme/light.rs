//! The light theme.

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

        canvas:        Color32::from_rgb(0xf2, 0xf2, 0xf2),
        ink:           Color32::from_rgb(0x1f, 0x29, 0x37),
        muted:         Color32::from_rgb(0x63, 0x6a, 0x77),
        disabled:      Color32::from_rgb(0xb9, 0xb9, 0xb9),
        panel:         Color32::from_rgb(0xf7, 0xf7, 0xf9),
        surface:       Color32::from_rgb(0xff, 0xff, 0xff),
        separator:     Color32::from_rgb(0xe4, 0xe5, 0xe9),
        hairline:      Color32::from_rgb(0xe6, 0xe8, 0xeb),
        outline:       Color32::from_rgb(0xd5, 0xd7, 0xdd),
        hover:         Color32::from_rgb(0xec, 0xed, 0xf1),
        row:           Color32::from_rgb(0xf2, 0xf2, 0xf4),
        row_active:    Color32::from_rgb(0xe1, 0xe1, 0xe4),
        selected_fill: Color32::from_rgb(0xff, 0xec, 0xb0),
        icon_line:     Color32::from_rgb(0x2a, 0x78, 0xd6),
        icon_top:      Color32::from_rgb(0x8d, 0xe6, 0xff),
        icon_bottom:   Color32::from_rgb(0x3e, 0xaf, 0xf6),
        icon_back:     Color32::from_rgb(0xff, 0xff, 0xff),
        field_bg:      Color32::from_rgb(0xfa, 0xfa, 0xfb),
        field_stroke:  Color32::from_rgb(0xd1, 0xd5, 0xdb),
        chip:          Color32::from_rgb(0xf3, 0xf4, 0xf6),
        chip_hover:    Color32::from_rgb(0xe5, 0xe7, 0xeb),
        card:          Color32::from_rgb(0xff, 0xff, 0xff),
        card_edge:     Color32::from_rgb(0xe6, 0xe8, 0xeb),
        error:         Color32::from_rgb(0xc0, 0x30, 0x30),
        grip:          Color32::from_rgb(0xaa, 0xaa, 0xaa),
        warning:       Color32::from_rgb(0xc0, 0x50, 0x00),
        key_bg:        Color32::from_rgb(0x58, 0x55, 0x52),
        layer_active:  Color32::from_rgb(0x22, 0x49, 0x85),
        on_accent:     Color32::WHITE,
        spline_arm:    Color32::from_rgb(0x40, 0x80, 0xd0),
        halo:          Color32::from_white_alpha(200),

        gcode_cmd:         Color32::from_rgb(0x00, 0x64, 0xb4),
        gcode_num:         Color32::from_rgb(0xa0, 0x50, 0x00),
        gcode_comment:     Color32::from_rgb(0x64, 0x78, 0x64),
        gcode_placeholder: Color32::from_rgb(0x8c, 0x32, 0xa0),
        gcode_text:        Color32::from_rgb(0x1e, 0x1e, 0x1e),

        sim_rapid:  Color32::from_rgba_premultiplied(0x00, 0xaa, 0xbb, 0xa0),
        sim_plunge: Color32::from_rgb(0xe0, 0xb0, 0x20),

        extreme_bg:      Color32::WHITE,
        selection_bg:    Color32::from_rgb(0xff, 0xe0, 0x80),
        active_bg:       Color32::from_rgb(0xff, 0xc0, 0x00),
        inactive_weak_bg: Color32::from_rgb(0xff, 0xff, 0xff),
        shadow_alpha:    34,

        board_desk: Color32::from_rgb(0xd9, 0xdc, 0xe0),

        is_dark: false,
    }
}
