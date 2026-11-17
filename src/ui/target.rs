use std::sync::mpsc;
use std::thread;

use eframe::egui;

use crate::app::App;
use crate::catalog::fetch_target;
use crate::theme::{OK, TEXT_DIM, TEXT_HEAD};
use crate::ui::{optional, panel_card, section_header, signed_optional};


pub fn render_target_panel(ui: &mut egui::Ui, app: &mut App) {
    ui.add_space(4.0);

    section_header(ui, "Target");
    panel_card(ui, |ui| {
        // Catalog query row
        ui.horizontal(|ui| {
            let response = ui.add_sized(
                [200.0, 22.0],
                egui::TextEdit::singleline(&mut app.query)
                    .hint_text("Target name (e.g. WASP-12 b)"),
            );
            let enter = response.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let btn = egui::Button::new(
                egui::RichText::new("NASA lookup").color(TEXT_HEAD),
            )
            .fill(egui::Color32::from_rgb(40, 80, 130));
            let clicked = ui.add(btn).clicked();
            if (enter || clicked) && app.rx.is_none() {
                let (tx, rx) = mpsc::channel();
                let q = app.query.clone();
                thread::spawn(move || {
                    let _ = tx.send(fetch_target(&q));
                });
                app.rx = Some(rx);
            }
        });

        // Status row
        let fetching = app.rx.is_some();
        ui.horizontal(|ui| {
            ui.colored_label(
                if fetching { egui::Color32::from_rgb(240, 200, 80) } else { OK },
                egui::RichText::new("●").size(10.0),
            );
            ui.colored_label(
                if fetching { egui::Color32::from_rgb(240, 200, 80) } else { TEXT_DIM },
                if fetching { "Fetching catalog…".to_string() } else { app.status.clone() },
            );
        });

        if app.cache.fetched_unix > 0 {
            ui.colored_label(
                TEXT_DIM,
                format!("Catalog last refreshed: Unix {}", app.cache.fetched_unix),
            );
        }

        let names: Vec<String> = app.cache.targets.iter().map(|t| t.name.clone()).collect();
        ui.horizontal(|ui| {
            ui.colored_label(TEXT_DIM, "Cached target");
            egui::ComboBox::from_id_salt("cached_target")
                .selected_text(if app.t.name.is_empty() { "Select cached record" } else { &app.t.name })
                .show_ui(ui, |ui| {
                    for name in names {
                        if ui.selectable_label(app.t.name == name, &name).clicked() {
                            if let Some(t) = app.cache.targets.iter().find(|t| t.name == name) {
                                app.t = t.clone();
                                app.query = name;
                            }
                        }
                    }
                });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.colored_label(TEXT_DIM, "Target name");
            ui.add_sized(
                [160.0, 20.0],
                egui::TextEdit::singleline(&mut app.t.name),
            );
        });

        signed_optional(ui, "RA (deg)",  &mut app.t.ra_deg);
        signed_optional(ui, "Dec (deg)", &mut app.t.dec_deg);
        optional(ui, "V magnitude",             &mut app.t.vmag);
        optional(ui, "R magnitude",             &mut app.t.rmag);
        optional(ui, "I magnitude",             &mut app.t.imag);
        optional(ui, "Stellar radius (solar)",  &mut app.t.star_radius_solar);
        optional(ui, "Planet radius (Jupiter)", &mut app.t.planet_radius_jupiter);
        optional(ui, "Transit duration (hours)",&mut app.t.duration_hours);
    });

}
