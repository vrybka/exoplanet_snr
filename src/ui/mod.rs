pub mod forecast;
pub mod observation;
pub mod target;

use eframe::egui;

use crate::theme::{ACCENT, BG_DEEP, BG_FIELD, STROKE, TEXT_DIM};

pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(6.0);
    egui::Frame::group(ui.style())
        .fill(BG_FIELD)
        .stroke(egui::Stroke::new(1.0_f32, STROKE))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.colored_label(
                    ACCENT,
                    egui::RichText::new(title.to_uppercase())
                        .strong()
                        .size(16.0),
                );
            });
        });
    ui.add_space(4.0);
}

pub fn panel_card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(BG_DEEP)
        .stroke(egui::Stroke::new(1.0_f32, STROKE))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(10))
        .show(ui, body);
}

pub fn number(ui: &mut egui::Ui, label: &str, v: &mut f64) {
    egui::Grid::new(format!("grid_num_{}", label))
        .num_columns(2)
        .spacing([12.0, 4.0])
        .min_col_width(180.0)
        .show(ui, |ui| {
            ui.colored_label(TEXT_DIM, label);
            ui.add(egui::DragValue::new(v).speed(0.1));
            ui.end_row();
        });
}

pub fn signed_optional(ui: &mut egui::Ui, label: &str, v: &mut Option<f64>) {
    egui::Grid::new(format!("grid_sopt_{}", label))
        .num_columns(3)
        .spacing([12.0, 4.0])
        .min_col_width(180.0)
        .show(ui, |ui| {
            let mut known = v.is_some();
            ui.checkbox(&mut known, egui::RichText::new(label).color(TEXT_DIM));
            if known {
                let mut x = v.unwrap_or(0.0);
                ui.add(egui::DragValue::new(&mut x).speed(0.1));
                *v = Some(x);
            } else {
                *v = None;
                ui.colored_label(TEXT_DIM, "—");
            }
            ui.end_row();
        });
}

pub fn optional(ui: &mut egui::Ui, label: &str, v: &mut Option<f64>) {
    egui::Grid::new(format!("grid_opt_{}", label))
        .num_columns(2)
        .spacing([12.0, 4.0])
        .min_col_width(180.0)
        .show(ui, |ui| {
            ui.colored_label(TEXT_DIM, label);
            let mut x = v.unwrap_or(0.0);
            ui.add(egui::DragValue::new(&mut x).speed(0.1));
            *v = if x > 0.0 { Some(x) } else { None };
            ui.end_row();
        });
}

pub fn stat_row(ui: &mut egui::Ui, label: &str, value: String, accent: egui::Color32) {
    egui::Grid::new(format!("stat_{}", label))
        .num_columns(2)
        .spacing([12.0, 4.0])
        .min_col_width(180.0)
        .show(ui, |ui| {
            ui.colored_label(TEXT_DIM, label);
            ui.colored_label(accent, egui::RichText::new(value).strong());
            ui.end_row();
        });
}
