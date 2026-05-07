use egui::{Color32, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals};

pub struct Palette {
    pub app_bg: Color32,
    pub chat_bg: Color32,
    pub sidebar_bg: Color32,
    pub panel: Color32,
    pub panel_alt: Color32,
    pub panel_soft: Color32,
    pub panel_lifted: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub text_subtle: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub accent_strong: Color32,
    pub accent_warm: Color32,
    pub user_bubble: Color32,
    pub assistant_bubble: Color32,
    pub local_bubble: Color32,
    pub error_bubble: Color32,
    pub success: Color32,
    pub warning: Color32,
}

pub fn palette() -> Palette {
    Palette {
        app_bg: Color32::from_rgb(9, 11, 15),
        chat_bg: Color32::from_rgb(13, 16, 22),
        sidebar_bg: Color32::from_rgb(15, 18, 24),
        panel: Color32::from_rgb(19, 23, 31),
        panel_alt: Color32::from_rgb(25, 30, 40),
        panel_soft: Color32::from_rgb(32, 38, 50),
        panel_lifted: Color32::from_rgb(37, 43, 56),
        border: Color32::from_rgb(47, 56, 72),
        border_strong: Color32::from_rgb(82, 96, 121),
        text: Color32::from_rgb(239, 243, 248),
        text_muted: Color32::from_rgb(177, 187, 205),
        text_subtle: Color32::from_rgb(123, 136, 160),
        accent: Color32::from_rgb(118, 205, 224),
        accent_soft: Color32::from_rgb(29, 70, 82),
        accent_strong: Color32::from_rgb(84, 171, 198),
        accent_warm: Color32::from_rgb(225, 179, 104),
        user_bubble: Color32::from_rgb(37, 90, 105),
        assistant_bubble: Color32::from_rgb(24, 29, 39),
        local_bubble: Color32::from_rgb(31, 38, 52),
        error_bubble: Color32::from_rgb(94, 38, 46),
        success: Color32::from_rgb(104, 211, 156),
        warning: Color32::from_rgb(229, 184, 103),
    }
}

pub fn apply(ctx: &egui::Context, text_scale: f32) {
    let colors = palette();
    let mut style: Style = (*ctx.style()).clone();
    style.visuals = Visuals::dark();
    style.visuals.window_fill = colors.panel;
    style.visuals.panel_fill = colors.app_bg;
    style.visuals.extreme_bg_color = colors.app_bg;
    style.visuals.faint_bg_color = colors.panel_alt;
    style.visuals.code_bg_color = colors.panel_soft;
    style.visuals.widgets.noninteractive.bg_fill = colors.panel;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text);
    style.visuals.widgets.inactive.bg_fill = colors.panel_soft;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text_muted);
    style.visuals.widgets.hovered.bg_fill = colors.panel_lifted;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors.text);
    style.visuals.widgets.active.bg_fill = colors.accent_soft;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors.text);
    style.visuals.selection.bg_fill = colors.accent_soft;
    style.visuals.selection.stroke = Stroke::new(1.0, colors.accent);
    style.visuals.window_rounding = Rounding::same(8.0);

    style.spacing.item_spacing = egui::vec2(9.0, 9.0);
    style.spacing.button_padding = egui::vec2(13.0, 8.0);
    style.spacing.menu_margin = egui::Margin::same(12.0);
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(scaled_size(24.0, text_scale), FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(scaled_size(16.0, text_scale), FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(scaled_size(15.0, text_scale), FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(scaled_size(13.5, text_scale), FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(scaled_size(15.0, text_scale), FontFamily::Monospace),
        ),
    ]
    .into();

    ctx.set_style(style);
}

pub fn scaled_size(size: f32, text_scale: f32) -> f32 {
    size * text_scale
}
