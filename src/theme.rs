use eframe::egui;

pub const BG_DEEP:   egui::Color32 = egui::Color32::from_rgb(14, 18, 26);
pub const BG_PANEL:  egui::Color32 = egui::Color32::from_rgb(22, 28, 38);
pub const BG_FIELD:  egui::Color32 = egui::Color32::from_rgb(30, 38, 50);
pub const STROKE:    egui::Color32 = egui::Color32::from_rgb(60, 72, 90);
pub const TEXT_DIM:  egui::Color32 = egui::Color32::from_rgb(170, 182, 196);
pub const TEXT_HEAD: egui::Color32 = egui::Color32::from_rgb(220, 232, 248);
pub const ACCENT:    egui::Color32 = egui::Color32::from_rgb(90, 170, 240);
pub const OK:        egui::Color32 = egui::Color32::from_rgb(110, 220, 150);
pub const WARN:      egui::Color32 = egui::Color32::from_rgb(240, 150, 100);
pub const BAD:       egui::Color32 = egui::Color32::from_rgb(240, 110, 110);

pub fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(TEXT_HEAD);
    v.window_fill = BG_DEEP;
    v.panel_fill  = BG_PANEL;
    v.faint_bg_color    = BG_FIELD;
    v.extreme_bg_color  = BG_FIELD;
    v.hyperlink_color   = ACCENT;

    v.widgets = egui::style::Widgets {
        noninteractive: widget_visuals(BG_PANEL, STROKE),
        inactive:       widget_visuals(BG_FIELD,  STROKE),
        hovered:        widget_visuals(BG_FIELD,  ACCENT),
        active:         widget_visuals(egui::Color32::from_rgb(40, 80, 130), ACCENT),
        open:           widget_visuals(BG_FIELD,  ACCENT),
    };
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.spacing.item_spacing   = egui::vec2(8.0, 6.0);
        s.spacing.window_margin  = egui::Margin::same(14);
        s.spacing.button_padding = egui::vec2(10.0, 6.0);
        s.spacing.interact_size  = egui::vec2(44.0, 22.0);
    });

    ctx.set_pixels_per_point(1.25);
}


pub fn widget_visuals(bg: egui::Color32, stroke: egui::Color32) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: bg,
        bg_stroke: egui::Stroke::new(1.0_f32, stroke),
        fg_stroke: egui::Stroke::new(1.0_f32, TEXT_HEAD),
        expansion: 0.0,
        corner_radius: egui::CornerRadius::same(4),
    }
}
