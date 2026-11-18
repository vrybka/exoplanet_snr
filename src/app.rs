use std::sync::mpsc;

use eframe::egui;

use crate::catalog::{cache_path, load_cache, now};
use crate::state::{Band, Camera, Cache, Input, Target};
use crate::theme;
use crate::theme::{TEXT_HEAD};
use crate::ui;

pub struct App {
    pub i: Input,
    pub t: Target,
    pub band: Band,
    pub camera: Camera,
    pub cache: Cache,
    pub query: String,
    pub status: String,
    pub rx: Option<mpsc::Receiver<Result<Target, String>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            i: Input::default(),
            t: Target::default(),
            band: Band::V,
            camera: Camera::Asi533,
            cache: load_cache(),
            query: String::new(),
            status: "Enter the values manually or look up in the catalog".into(),
            rx: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_theme(ctx);
        self.drain_catalog_channel();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("🔭").size(22.0_f32));
                ui.label(
                    egui::RichText::new("Exoplanet Transit SNR Estimator")
                        .strong()
                        .size(22.0_f32)
                        .color(TEXT_HEAD),
                );
            });
            ui.add_space(2.0);

            ui.columns(3, |columns| {
    
                egui::ScrollArea::both()
                    .id_salt("target_scroll")
                    .auto_shrink([false, false])
                    .show(&mut columns[1], |ui| {
                        ui::target::render_target_panel(ui, self);
                    });

                egui::ScrollArea::both()
                    .id_salt("observation_scroll")
                    .auto_shrink([false, false])
                    .show(&mut columns[2], |ui| {
                        ui::observation::render_observation_panel(ui, self);
                    });

                egui::ScrollArea::both()
                    .id_salt("forecast_scroll")
                    .auto_shrink([false, false])
                    .show(&mut columns[0], |ui| {
                        ui::forecast::render_forecast_panel(ui, self);
                    });
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}


impl App {
    /// Drain the background catalog-fetch channel, if any
    fn drain_catalog_channel(&mut self) {
        let received = match self.rx.as_ref() {
            Some(rx) => rx.try_recv(),
            None => return,
        };

        let result = match received {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.rx = None;
                self.status = "Catalog worker disconnected. Try the lookup again.".into();
                return;
            }
        };

        self.rx = None;

        match result {
            Ok(target) => {
                self.query = target.name.clone();
                self.t = target.clone();

                self.cache.targets.retain(|t| t.name != target.name);
                self.cache.targets.push(target);
                self.cache.fetched_unix = now();

                let saved = serde_json::to_vec_pretty(&self.cache)
                    .map_err(|error| error.to_string())
                    .and_then(|bytes| {
                        std::fs::write(cache_path(), bytes)
                            .map_err(|error| error.to_string())
                    });

                self.status = match saved {
                    Ok(()) => "Catalog refreshed and cached".into(),
                    Err(error) => {
                        format!("Catalog loaded; cache save failed: {error}")
                    }
                };
            }
            Err(error) => {
                self.status = format!(
                    "Refresh failed: {error}. Cached records remain available."
                );
            }
        }
    }
}