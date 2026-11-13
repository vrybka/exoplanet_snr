// Exoplanet Transit SNR Estimator

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Serialize, Deserialize, Default)]
struct Target {
    name: String,
    ra_deg: Option<f64>,
    dec_deg: Option<f64>,
    vmag: Option<f64>,
    rmag: Option<f64>,
    imag: Option<f64>,
    star_radius_solar: Option<f64>,
    planet_radius_jupiter: Option<f64>,
    duration_hours: Option<f64>,
    period_days: Option<f64>,
    epoch_bjd: Option<f64>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct Cache {
    fetched_unix: u64,
    targets: Vec<Target>,
}

#[derive(Clone, Copy, PartialEq)]
enum Band { V, R, I, Clear }

impl Band {
    fn label(self) -> &'static str {
        match self {
            Self::V      => "V",
            Self::R      => "R",
            Self::I      => "I",
            Self::Clear  => "Clear (approx.)",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Camera { Asi533, Asi294, Dslr, Custom }

impl Camera {
    fn label(self) -> &'static str {
        match self {
            Self::Asi533 => "ZWO ASI533",
            Self::Asi294 => "ZWO ASI294",
            Self::Dslr   => "Generic DSLR",
            Self::Custom => "Custom",
        }
    }
}

struct Input {
    aperture_cm: f64,
    lat_deg: f64,
    lon_deg: f64,
    jd_utc: f64,
    airmass_override: f64,

    // Camera
    exposure_s: f64,
    cadence_s: f64,
    qe: f64,
    read_e: f64,
    dark_e_s: f64,
    pixel_scale: f64,

    // Amosphere
    seeing_arcsec: f64,
    sky_mag_arcsec2: f64,
    extinction_mag_airmass: f64,

    // Statistics
    systematic_ppm: f64,
    correlation_minutes: f64,
    comparison_hours: f64,
    transits: f64,
    threshold: f64,

    // Photometric calibration
    zero_point_photons_cm2_s: f64,
    throughput: f64,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            aperture_cm: 20.0,
            exposure_s: 60.0,
            cadence_s: 75.0,
            qe: 0.7,
            read_e: 2.0,
            dark_e_s: 0.005,
            pixel_scale: 1.0,
            seeing_arcsec: 2.5,
            sky_mag_arcsec2: 20.5,
            extinction_mag_airmass: 0.18,
            systematic_ppm: 1000.0,
            correlation_minutes: 30.0,
            zero_point_photons_cm2_s: 800_000.0,
            throughput: 0.4,
            lat_deg: 40.0,
            lon_deg: -105.0,
            jd_utc: 2_461_000.5,
            airmass_override: 0.0,
            comparison_hours: 2.0,
            transits: 1.0,
            threshold: 7.0,
        }
    }
}

struct ResultData {
    snr: f64,
    depth_ppm: f64,
    photon_ppm: f64,
    scint_ppm: f64,
    systematic_ppm: f64,
    air: f64,
    n_in: f64,
    n_out: f64,
    min_aperture_cm: Option<f64>,
    note: String,
}

struct App {
    i: Input,
    t: Target,
    band: Band,
    camera: Camera,
    cache: Cache,
    query: String,
    status: String,
    rx: Option<mpsc::Receiver<Result<Target, String>>>,
}

impl Default for App {
    fn default() -> Self {
        let cache = load_cache();
        Self {
            i: Input::default(),
            t: Target::default(),
            band: Band::V,
            camera: Camera::Asi533,
            cache,
            query: String::new(),
            status: "Enter the values manually or look up the catalog".into(),
            rx: None,
        }
    }
}

fn cache_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_default()
        .join("transit_catalog_cache.json")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn load_cache() -> Cache {
    fs::read(cache_path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

// NASA Exoplanet Archive lookup

fn fetch_target(name: &str) -> Result<Target, String> {
    // Sanitize the user-provided name
    let term = name.trim().replace('+', " ").to_lowercase();
    let safe = term.replace(['%', '_'], "").replace('\'', "''");
    if safe.trim().is_empty() {
        return Err("Enter a target name".into());
    }

    let sql = format!(
        "select top 1 pl_name, ra, dec, sy_vmag, sy_rmag, sy_imag, st_rad, \
         pl_radj, pl_trandur, pl_orbper, pl_tranmid \
         from pscomppars where lower(pl_name) like '%{}%'",
        safe
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = client
        .get("https://exoplanetarchive.ipac.caltech.edu/TAP/sync")
        .query(&[("query", sql.as_str()), ("format", "json")])
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let row = json
        .as_array()
        .and_then(|a| a.first())
        .ok_or("Target not found in NASA archive")?;

    let n = |k: &str| row.get(k).and_then(|v| v.as_f64());

    Ok(Target {
        name: row["pl_name"].as_str().unwrap_or(name).to_owned(),
        ra_deg:                 n("ra"),
        dec_deg:                n("dec"),
        vmag:                   n("sy_vmag"),
        rmag:                   n("sy_rmag"),
        imag:                   n("sy_imag"),
        star_radius_solar:      n("st_rad"),
        planet_radius_jupiter:  n("pl_radj"),
        duration_hours:         n("pl_trandur"),
        period_days:            n("pl_orbper"),
        epoch_bjd:              n("pl_tranmid"),
    })
}

// Astronomy helpers

/// Local sidereal time at the given Julian date, in degrees.
fn julian_sidereal(jd: f64) -> f64 {
    (280.46061837 + 360.98564736629 * (jd - 2_451_545.0))
        .rem_euclid(360.0)
}

/// Airmass at the observation time. Kasten–Young approximation is used.
fn airmass(i: &Input, t: &Target) -> Result<f64, String> {
    if i.airmass_override > 0.0 {
        return Ok(i.airmass_override);
    }

    let (ra, dec) = (
        t.ra_deg.ok_or("Missing RA; enter an airmass override")?,
        t.dec_deg.ok_or("Missing Dec; enter an airmass override")?,
    );

    let ha = (julian_sidereal(i.jd_utc) + i.lon_deg - ra).to_radians();
    let lat = i.lat_deg.to_radians();
    let dec = dec.to_radians();
    let alt = (lat.sin() * dec.sin() + lat.cos() * dec.cos() * ha.cos())
        .asin()
        .to_degrees();

    if alt <= 0.0 {
        return Err("Target below horizon at selected time".into());
    }

    Ok(1.0 / ((alt + 244.0 / (165.0 + 47.0 * alt.powf(1.1))).to_radians().sin()))
}

// SNR evaluation

fn evaluate(
    i: &Input,
    t: &Target,
    band: Band,
    ap: f64,
    solve: bool,
) -> Result<ResultData, String> {
    let rstar = t.star_radius_solar
        .ok_or("Missing stellar radius: enter it manually")?;
    let rp = t.planet_radius_jupiter
        .ok_or("Missing planet radius: enter it manually")?;
    let dur = t.duration_hours
        .ok_or("Missing transit duration: enter it manually")?;

    if rstar <= 0.0 || rp <= 0.0 || dur <= 0.0 {
        return Err("Radii and duration must be positive".into());
    }
    if ap <= 0.0
        || i.exposure_s <= 0.0
        || i.cadence_s < i.exposure_s
        || i.comparison_hours <= 0.0
        || i.transits < 1.0
        || i.threshold <= 0.0
        || i.pixel_scale <= 0.0
        || i.seeing_arcsec <= 0.0
        || i.correlation_minutes <= 0.0
        || i.qe <= 0.0
        || i.throughput <= 0.0
        || i.zero_point_photons_cm2_s <= 0.0
    {
        return Err("Check positive settings; cadence must be at least exposure".into());
    }

    // Pick the magnitude that matches the selected band.
    let (mag, note) = match band {
        Band::V     => (t.vmag, "V-band magnitude"),
        Band::R     => (t.rmag, "R-band magnitude"),
        Band::I     => (t.imag, "I-band magnitude"),
        Band::Clear => (
            t.vmag,
            "Clear uses V magnitude as a proxy: adjust zero-point and \
             throughput for your optics, QE and stellar spectrum",
        ),
    };
    let mag = mag.ok_or_else(|| format!("Missing {} magnitude; enter it manually", band.label()))?;

    let air = airmass(i, t)?;

    // Geometric transit depth: (R_p / R_star)^2. Jupiter radius / solar radius ≈ 0.10045.
    let depth = (rp * 0.10045 / rstar).powi(2);

    // Star counts (electrons) per exposure.
    let area  = std::f64::consts::PI * (ap / 2.0).powi(2);
    let star  = i.zero_point_photons_cm2_s
        * area * i.throughput * i.qe * i.exposure_s
        * 10f64.powf(-0.4 * (mag + i.extinction_mag_airmass * (air - 1.0)));

    // Sky and detector noise contributions.
    let extraction_arcsec2 = std::f64::consts::PI * (1.5 * i.seeing_arcsec).powi(2);
    let pixels             = extraction_arcsec2 / i.pixel_scale.powi(2);
    let sky                = i.zero_point_photons_cm2_s
        * area * i.throughput * i.qe * i.exposure_s
        * 10f64.powf(-0.4 * i.sky_mag_arcsec2)
        * extraction_arcsec2;
    let variance = star + sky + pixels * (i.read_e.powi(2) + i.dark_e_s * i.exposure_s);
    let photon   = variance.sqrt() / star;

    // Young scintillation approximation (D in cm, sea-level assumed).
    let scint = 0.09
        * ap.powf(-2.0 / 3.0)
        * air.powf(1.75)
        * (2.0 * i.exposure_s).powf(-0.5);

    let nin  = (dur * 3600.0 / i.cadence_s).floor().max(1.0);
    let nout = (i.comparison_hours * 3600.0 / i.cadence_s).floor().max(1.0);

    let white = (photon.powi(2) + scint.powi(2))
        * (1.0 / nin + 1.0 / nout)
        / i.transits;

    // Systematic noise is independent per correlation block, so shared within a block.
    let blocks_in  = (dur * 60.0 / i.correlation_minutes).max(1.0);
    let blocks_out = (i.comparison_hours * 60.0 / i.correlation_minutes).max(1.0);
    let sys = (i.systematic_ppm * 1e-6).powi(2)
        * (1.0 / blocks_in + 1.0 / blocks_out)
        / i.transits;

    let snr = depth / (white + sys).sqrt();

    // Binary-search the minimum aperture that achieves the threshold SNR.
    let min_aperture_cm = if solve {
        let mut lo = 1.0;
        let mut hi = 2000.0;
        if evaluate(i, t, band, hi, false)?.snr < i.threshold {
            None
        } else {
            for _ in 0..45 {
                let mid = (lo + hi) / 2.0;
                if evaluate(i, t, band, mid, false)?.snr >= i.threshold {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            Some(hi)
        }
    } else {
        None
    };

    Ok(ResultData {
        snr,
        depth_ppm:    depth * 1e6,
        photon_ppm:   photon * 1e6,
        scint_ppm:    scint * 1e6,
        systematic_ppm: i.systematic_ppm,
        air,
        n_in:         nin,
        n_out:        nout,
        min_aperture_cm,
        note:         note.into(),
    })
}

// egui helpers

fn number(ui: &mut egui::Ui, label: &str, v: &mut f64) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(v).speed(0.1));
    });
}

fn signed_optional(ui: &mut egui::Ui, label: &str, v: &mut Option<f64>) {
    ui.horizontal(|ui| {
        let mut known = v.is_some();
        ui.checkbox(&mut known, label);
        if known {
            let mut x = v.unwrap_or(0.0);
            ui.add(egui::DragValue::new(&mut x).speed(0.1));
            *v = Some(x);
        } else {
            *v = None;
            ui.label("missing");
        }
    });
}

fn optional(ui: &mut egui::Ui, label: &str, v: &mut Option<f64>) {
    let mut x = v.unwrap_or(0.0);
    number(ui, label, &mut x);
    *v = if x > 0.0 { Some(x) } else { None };
}

// egui App 

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // Drain the background catalog-fetch channel, if any
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.rx = None;
                match result {
                    Ok(t) => {
                        self.query = t.name.clone();
                        self.t = t.clone();
                        self.cache.targets.retain(|x| x.name != t.name);
                        self.cache.targets.push(t);
                        self.cache.fetched_unix = now();
                        match fs::write(
                            cache_path(),
                            serde_json::to_vec_pretty(&self.cache).unwrap_or_default(),
                        ) {
                            Ok(_)  => self.status = "Catalog refreshed and cached".into(),
                            Err(e) => self.status =
                                format!("Catalog loaded; cache write failed: {e}"),
                        }
                    }
                    Err(e) => self.status = format!(
                        "Refresh failed: {e}. Cached records remain available."
                    ),
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {

                ui.heading("Exoplanet Transit SNR Estimator");
                ui.label("Planning estimate, not a guaranteed detection. \
                          All calculations work offline with manual inputs.");

                ui.separator();
                ui.heading("Target");

                ui.horizontal(|ui| {
                    let response = ui.text_edit_singleline(&mut self.query);
                    let enter = response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let clicked = ui.button("NASA lookup").clicked();
                    if (enter || clicked) && self.rx.is_none() {
                        let (tx, rx) = mpsc::channel();
                        let q = self.query.clone();
                        thread::spawn(move || {
                            let _ = tx.send(fetch_target(&q));
                        });
                        self.rx = Some(rx);
                    }
                });

                if self.rx.is_some() {
                    ui.label("Fetching catalog…");
                }
                ui.label(&self.status);

                if self.cache.fetched_unix > 0 {
                    ui.label(format!(
                        "Catalog last refreshed: Unix {}",
                        self.cache.fetched_unix
                    ));
                }

                let names: Vec<String> =
                    self.cache.targets.iter().map(|t| t.name.clone()).collect();
                egui::ComboBox::from_label("Cached target")
                    .selected_text(if self.t.name.is_empty() {
                        "Select cached record"
                    } else {
                        &self.t.name
                    })
                    .show_ui(ui, |ui| {
                        for name in names {
                            if ui
                                .selectable_label(self.t.name == name, &name)
                                .clicked()
                            {
                                if let Some(t) =
                                    self.cache.targets.iter().find(|t| t.name == name)
                                {
                                    self.t = t.clone();
                                    self.query = name;
                                }
                            }
                        }
                    });

                ui.horizontal(|ui| {
                    ui.label("Target name");
                    ui.text_edit_singleline(&mut self.t.name);
                });

                signed_optional(ui, "RA (deg)", &mut self.t.ra_deg);
                signed_optional(ui, "Dec (deg)", &mut self.t.dec_deg);
                optional(ui, "V magnitude",                &mut self.t.vmag);
                optional(ui, "R magnitude",                &mut self.t.rmag);
                optional(ui, "I magnitude",                &mut self.t.imag);
                optional(ui, "Stellar radius (solar)",     &mut self.t.star_radius_solar);
                optional(ui, "Planet radius (Jupiter)",    &mut self.t.planet_radius_jupiter);
                optional(ui, "Transit duration (hours)",   &mut self.t.duration_hours);

                // Observation

                ui.separator();
                ui.heading("Observation");

                egui::ComboBox::from_label("Band")
                    .selected_text(self.band.label())
                    .show_ui(ui, |ui| {
                        for b in [Band::V, Band::R, Band::I, Band::Clear] {
                            ui.selectable_value(&mut self.band, b, b.label());
                        }
                    });

                ui.label("Camera presets are illustrative planning values; \
                          verify your camera's gain, QE and read noise.");

                egui::ComboBox::from_label("Camera")
                    .selected_text(self.camera.label())
                    .show_ui(ui, |ui| {
                        for c in [Camera::Asi533, Camera::Asi294, Camera::Dslr, Camera::Custom] {
                            if ui
                                .selectable_value(&mut self.camera, c, c.label())
                                .clicked()
                            {
                                match c {
                                    Camera::Asi533 => {
                                        self.i.qe        = 0.7;
                                        self.i.read_e    = 2.0;
                                        self.i.dark_e_s  = 0.005;
                                    }
                                    Camera::Asi294 => {
                                        self.i.qe        = 0.7;
                                        self.i.read_e    = 2.3;
                                        self.i.dark_e_s  = 0.02;
                                    }
                                    Camera::Dslr => {
                                        self.i.qe        = 0.45;
                                        self.i.read_e    = 4.0;
                                        self.i.dark_e_s  = 0.05;
                                    }
                                    Camera::Custom => {}
                                }
                            }
                        }
                    });

                number(ui, "Aperture (cm)",                              &mut self.i.aperture_cm);
                number(ui, "Exposure (s)",                              &mut self.i.exposure_s);
                number(ui, "Cadence incl. overhead (s)",                &mut self.i.cadence_s);
                number(ui, "QE (0–1)",                                  &mut self.i.qe);
                number(ui, "Read noise (e-/pixel)",                     &mut self.i.read_e);
                number(ui, "Dark current (e-/pixel/s)",                 &mut self.i.dark_e_s);
                number(ui, "Plate scale (arcsec/pixel)",                &mut self.i.pixel_scale);
                number(ui, "Seeing FWHM (arcsec; manual)",              &mut self.i.seeing_arcsec);
                number(ui, "Sky brightness (mag/arcsec²; manual)",      &mut self.i.sky_mag_arcsec2);
                number(ui, "Latitude (deg)",                            &mut self.i.lat_deg);
                number(ui, "Longitude east-positive (deg)",             &mut self.i.lon_deg);
                number(ui, "Julian date UTC at transit midpoint",       &mut self.i.jd_utc);
                number(ui, "Airmass override (0 = compute)",            &mut self.i.airmass_override);
                number(ui, "Extinction (mag/airmass)",                 &mut self.i.extinction_mag_airmass);
                number(ui, "Systematic floor (ppm / correlation block)", &mut self.i.systematic_ppm);
                number(ui, "Correlation timescale (minutes)",           &mut self.i.correlation_minutes);
                number(ui, "Comparison baseline (hours)",               &mut self.i.comparison_hours);
                number(ui, "Number of complete transits",               &mut self.i.transits);
                number(ui, "Detection threshold (SNR)",                 &mut self.i.threshold);

                ui.collapsing("Photometric calibration assumptions", |ui| {
                    number(ui, "Zero-mag photons / cm² / s",           &mut self.i.zero_point_photons_cm2_s);
                    number(ui, "Optics + band throughput (0–1)",       &mut self.i.throughput);
                    ui.label("Defaults are approximate; \
                              use measured band-specific rates for reliable estimates.");
                });

                // Detection forecast

                ui.separator();
                ui.heading("Detection forecast");

                match evaluate(&self.i, &self.t, self.band, self.i.aperture_cm, true) {
                    Ok(r) => {
                        ui.strong(format!(
                            "SNR {:.2} · {} (threshold {:.1})",
                            r.snr,
                            if r.snr >= self.i.threshold {
                                "DETECTABLE under assumptions"
                            } else {
                                "BELOW THRESHOLD"
                            },
                            self.i.threshold
                        ));
                        ui.label(format!(
                            "Depth {:.0} ppm · airmass {:.2} · in/out samples {:.0}/{:.0}",
                            r.depth_ppm, r.air, r.n_in, r.n_out
                        ));
                        ui.label(format!(
                            "Per-exposure photon {:.0} ppm · scintillation {:.0} ppm · \
                             systematic floor {:.0} ppm",
                            r.photon_ppm, r.scint_ppm, r.systematic_ppm
                        ));
                        ui.label(format!(
                            "Minimum aperture: {}",
                            r.min_aperture_cm
                                .map(|v| format!("{v:.1} cm"))
                                .unwrap_or_else(|| "No solution up to 20 m under these assumptions".into())
                        ));
                        ui.label(r.note);
                    }
                    Err(e) => {
                        ui.colored_label(egui::Color32::ORANGE, e);
                    }
                }

                ui.separator();
                ui.small(
                    "No unverified forecast or radiance-to-sky-brightness conversion is applied. \
                     Scintillation assumes sea-level altitude. Systematic blocks are independent between transits. ",
                );
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Transit SNR Estimator",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}