use crate::astronomy::airmass;
use crate::state::{Band, Input, ResultData, Target};

pub fn evaluate(
    i: &Input,
    t: &Target,
    band: Band,
    ap: f64,
    solve: bool,
) -> Result<ResultData, String> {
    let rstar = t
        .star_radius_solar
        .ok_or("Missing stellar radius: enter it manually")?;
    let rp = t
        .planet_radius_jupiter
        .ok_or("Missing planet radius: enter it manually")?;
    let dur = t
        .duration_hours
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

    let (mag, note) = match band {
        Band::V => (t.vmag, "V-band magnitude"),
        Band::R => (t.rmag, "R-band magnitude"),
        Band::I => (t.imag, "I-band magnitude"),
        Band::Clear => (
            t.vmag,
            "Clear uses V magnitude as a proxy: adjust zero-point and \
             throughput for your optics, QE and stellar spectrum",
        ),
    };
    let mag =
        mag.ok_or_else(|| format!("Missing {} magnitude; enter it manually", band.label()))?;

    let air = airmass(i, t)?;

    let depth = (rp * 0.10045 / rstar).powi(2);

    let area = std::f64::consts::PI * (ap / 2.0).powi(2);
    let star = i.zero_point_photons_cm2_s
        * area
        * i.throughput
        * i.qe
        * i.exposure_s
        * 10f64.powf(-0.4 * (mag + i.extinction_mag_airmass * (air - 1.0)));

    let extraction_arcsec2 = std::f64::consts::PI * (1.5 * i.seeing_arcsec).powi(2);
    let pixels = extraction_arcsec2 / i.pixel_scale.powi(2);
    let sky = i.zero_point_photons_cm2_s
        * area
        * i.throughput
        * i.qe
        * i.exposure_s
        * 10f64.powf(-0.4 * i.sky_mag_arcsec2)
        * extraction_arcsec2;
    let variance = star + sky + pixels * (i.read_e.powi(2) + i.dark_e_s * i.exposure_s);
    let photon = variance.sqrt() / star;

    let scint = 0.09 * ap.powf(-2.0 / 3.0) * air.powf(1.75) * (2.0 * i.exposure_s).powf(-0.5);

    let nin = (dur * 3600.0 / i.cadence_s).floor().max(1.0);
    let nout = (i.comparison_hours * 3600.0 / i.cadence_s).floor().max(1.0);

    let white = (photon.powi(2) + scint.powi(2)) * (1.0 / nin + 1.0 / nout) / i.transits;

    let blocks_in = (dur * 60.0 / i.correlation_minutes).max(1.0);
    let blocks_out = (i.comparison_hours * 60.0 / i.correlation_minutes).max(1.0);
    let sys = (i.systematic_ppm * 1e-6).powi(2) * (1.0 / blocks_in + 1.0 / blocks_out) / i.transits;

    let snr = depth / (white + sys).sqrt();

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
        depth_ppm: depth * 1e6,
        photon_ppm: photon * 1e6,
        scint_ppm: scint * 1e6,
        systematic_ppm: i.systematic_ppm,
        air,
        n_in: nin,
        n_out: nout,
        min_aperture_cm,
        note: note.into(),
    })
}
