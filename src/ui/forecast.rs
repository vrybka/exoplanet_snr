use eframe::egui;

use crate::app::App;
use crate::snr::evaluate;
use crate::theme::{BAD, BG_PANEL, OK, STROKE, TEXT_DIM, TEXT_HEAD, WARN};

pub fn render_forecast_panel(ui: &mut egui::Ui, app: &mut App) {
    ui.add_space(8.0);

    ui.label(
        egui::RichText::new("Detection forecast")
            .strong()
            .size(20.0)
            .color(TEXT_HEAD),
    );
    ui.colored_label(TEXT_DIM, "Live · updates as you change inputs");
    ui.add_space(6.0);

    egui::Frame::group(ui.style())
        .fill(BG_PANEL)
        .stroke(egui::Stroke::new(1.0_f32, STROKE))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            match evaluate(&app.i, &app.t, app.band, app.i.aperture_cm, true) {
                Ok(result) => render_result(ui, &result, app.i.threshold),
                Err(error) => {
                    ui.colored_label(WARN, egui::RichText::new(format!("⚠ {error}")).strong());
                }
            }
        });

    ui.add_space(10.0);

    ui.add_space(16.0_f32);
    ui.colored_label(
        TEXT_DIM,
        "Planning estimate, not a guaranteed detection.",
    );

    egui::Frame::group(ui.style())
        .fill(BG_PANEL)
        .stroke(egui::Stroke::new(1.0_f32, STROKE))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(
                    "No unverified forecast or radiance-to-sky-brightness \
                     conversion is applied. Scintillation assumes sea-level \
                     altitude. Systematic blocks are independent between transits.",
                )
                .color(TEXT_DIM)
                .size(11.0),
            );
        });
}

fn render_result(ui: &mut egui::Ui, r: &crate::state::ResultData, threshold: f64) {
    use crate::theme::ACCENT;
    use crate::ui::stat_row;

    let detectable = r.snr >= threshold;
    let big_color = if detectable { OK } else { BAD };
    let big_label = if detectable {
        "DETECTABLE under assumptions"
    } else {
        "BELOW THRESHOLD"
    };

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.colored_label(TEXT_DIM, egui::RichText::new("Signal-to-Noise").size(11.0));
            ui.add(egui::Label::new(
                egui::RichText::new(format!("{:.2}", r.snr))
                    .size(36.0)
                    .strong()
                    .color(big_color),
            ));
            ui.colored_label(TEXT_DIM, format!("threshold {:.1}", threshold));
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::Frame::group(ui.style())
                .fill(if detectable {
                    egui::Color32::from_rgb(20, 50, 30)
                } else {
                    egui::Color32::from_rgb(60, 25, 25)
                })
                .stroke(egui::Stroke::new(1.0_f32, big_color))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.colored_label(big_color, egui::RichText::new(big_label).strong());
                });
        });
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(6.0);

    stat_row(
        ui,
        "Transit depth",
        format!("{:.0} ppm", r.depth_ppm),
        TEXT_HEAD,
    );
    stat_row(ui, "Airmass", format!("{:.2}", r.air), TEXT_HEAD);
    stat_row(
        ui,
        "In-transit samples",
        format!("{:.0}", r.n_in),
        TEXT_HEAD,
    );
    stat_row(
        ui,
        "Out-of-transit samples",
        format!("{:.0}", r.n_out),
        TEXT_HEAD,
    );
    stat_row(
        ui,
        "Photon noise (per exp.)",
        format!("{:.0} ppm", r.photon_ppm),
        ACCENT,
    );
    stat_row(ui, "Scintillation", format!("{:.0} ppm", r.scint_ppm), WARN);
    stat_row(
        ui,
        "Systematic floor",
        format!("{:.0} ppm", r.systematic_ppm),
        WARN,
    );

    ui.add_space(4.0);
    ui.colored_label(
        TEXT_DIM,
        format!(
            "Minimum aperture: {}",
            r.min_aperture_cm
                .map(|v| format!("{v:.1} cm"))
                .unwrap_or_else(|| "No solution up to 20 m under these assumptions".into())
        ),
    );
    ui.colored_label(TEXT_DIM, r.note.clone());
}
