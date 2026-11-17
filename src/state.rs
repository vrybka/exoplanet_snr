use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Target {
    pub name: String,
    pub ra_deg: Option<f64>,
    pub dec_deg: Option<f64>,
    pub vmag: Option<f64>,
    pub rmag: Option<f64>,
    pub imag: Option<f64>,
    pub star_radius_solar: Option<f64>,
    pub planet_radius_jupiter: Option<f64>,
    pub duration_hours: Option<f64>,
    pub period_days: Option<f64>,
    pub epoch_bjd: Option<f64>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Cache {
    pub fetched_unix: u64,
    pub targets: Vec<Target>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Band { V, R, I, Clear }

impl Band {
    pub fn label(self) -> &'static str {
        match self {
            Self::V     => "V",
            Self::R     => "R",
            Self::I     => "I",
            Self::Clear => "Clear (approx.)",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Camera { Asi533, Asi294, Dslr, Custom }

impl Camera {
    pub fn label(self) -> &'static str {
        match self {
            Self::Asi533 => "ZWO ASI533",
            Self::Asi294 => "ZWO ASI294",
            Self::Dslr   => "Generic DSLR",
            Self::Custom => "Custom",
        }
    }
}

pub struct Input {
    pub aperture_cm: f64,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub jd_utc: f64,
    pub airmass_override: f64,

    // Camera
    pub exposure_s: f64,
    pub cadence_s: f64,
    pub qe: f64,
    pub read_e: f64,
    pub dark_e_s: f64,
    pub pixel_scale: f64,

    // Atmosphere
    pub seeing_arcsec: f64,
    pub sky_mag_arcsec2: f64,
    pub extinction_mag_airmass: f64,

    // Statistics
    pub systematic_ppm: f64,
    pub correlation_minutes: f64,
    pub comparison_hours: f64,
    pub transits: f64,
    pub threshold: f64,

    // Photometric calibration
    pub zero_point_photons_cm2_s: f64,
    pub throughput: f64,
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

pub struct ResultData {
    pub snr: f64,
    pub depth_ppm: f64,
    pub photon_ppm: f64,
    pub scint_ppm: f64,
    pub systematic_ppm: f64,
    pub air: f64,
    pub n_in: f64,
    pub n_out: f64,
    pub min_aperture_cm: Option<f64>,
    pub note: String,
}
