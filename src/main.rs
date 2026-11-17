mod app;
mod astronomy;
mod snr;
mod state;
mod theme;
mod ui;
mod catalog;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1500.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Transit SNR Estimator",
        options,
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}
