use eframe::egui;

use crate::app::App;
use crate::state::{Band, Camera};
use crate::theme::TEXT_DIM;
use crate::ui::{number, panel_card, section_header};

pub fn render_observation_panel(ui: &mut egui::Ui, app: &mut App) {
    section_header(ui, "Observation");
    panel_card(ui, |ui| {
        // Band
        ui.horizontal(|ui| {
            ui.colored_label(TEXT_DIM, "Band");
            egui::ComboBox::from_id_salt("band")
                .selected_text(app.band.label())
                .show_ui(ui, |ui| {
                    for b in [Band::V, Band::R, Band::I, Band::Clear] {
                        ui.selectable_value(&mut app.band, b, b.label());
                    }
                });
        });

        ui.colored_label(
            TEXT_DIM,
            "Camera presets are illustrative planning values; \
             verify your camera's gain, QE and noise.",
        );

        ui.horizontal(|ui| {
            ui.colored_label(TEXT_DIM, "Camera");
            egui::ComboBox::from_id_salt("camera")
                .selected_text(app.camera.label())
                .show_ui(ui, |ui| {
                    for c in [Camera::Asi533, Camera::Asi294, Camera::Dslr, Camera::Custom] {
                        if ui.selectable_value(&mut app.camera, c, c.label()).clicked() {
                            match c {
                                Camera::Asi533 => { app.i.qe = 0.7;  app.i.read_e = 2.0; app.i.dark_e_s = 0.005; }
                                Camera::Asi294 => { app.i.qe = 0.7;  app.i.read_e = 2.3; app.i.dark_e_s = 0.02;  }
                                Camera::Dslr   => { app.i.qe = 0.45; app.i.read_e = 4.0; app.i.dark_e_s = 0.05;  }
                                Camera::Custom => {}
                            }
                        }
                    }
                });
        });

        number(ui, "Aperture (cm)",                              &mut app.i.aperture_cm);
        number(ui, "Exposure (s)",                              &mut app.i.exposure_s);
        number(ui, "Cadence incl. overhead (s)",                &mut app.i.cadence_s);
        number(ui, "QE (0–1)",                                  &mut app.i.qe);
        number(ui, "Read noise (e-/pixel)",                     &mut app.i.read_e);
        number(ui, "Dark current (e-/pixel/s)",                 &mut app.i.dark_e_s);
        number(ui, "Plate scale (arcsec/pixel)",                &mut app.i.pixel_scale);
        number(ui, "Seeing FWHM (arcsec; manual)",              &mut app.i.seeing_arcsec);
        number(ui, "Sky brightness (mag/arcsec²; manual)",      &mut app.i.sky_mag_arcsec2);
        number(ui, "Latitude (deg)",                            &mut app.i.lat_deg);
        number(ui, "Longitude east-positive (deg)",             &mut app.i.lon_deg);
        number(ui, "Julian date UTC at transit midpoint",       &mut app.i.jd_utc);
        number(ui, "Airmass override (0 = compute)",            &mut app.i.airmass_override);
        number(ui, "Extinction (mag/airmass)",                 &mut app.i.extinction_mag_airmass);
        number(ui, "Systematic floor (ppm / correlation block)",&mut app.i.systematic_ppm);
        number(ui, "Correlation timescale (minutes)",           &mut app.i.correlation_minutes);
        number(ui, "Comparison baseline (hours)",               &mut app.i.comparison_hours);
        number(ui, "Number of complete transits",               &mut app.i.transits);
        number(ui, "Detection threshold (SNR)",                 &mut app.i.threshold);

        ui.collapsing(
            egui::RichText::new("Photometric calibration assumptions").color(TEXT_DIM),
            |ui| {
                number(ui, "Zero-mag photons / cm² / s",     &mut app.i.zero_point_photons_cm2_s);
                number(ui, "Optics + band throughput (0–1)", &mut app.i.throughput);
                ui.colored_label(
                    TEXT_DIM,
                    "Defaults are approximate; \
                     use measured band-specific rates for reliable estimates.",
                );
            },
        );
    });
}