//astronomy helpers

use crate::state::{Input, Target};

/// Local sidereal time at the given Julian date, in degrees.
pub fn julian_sidereal(jd: f64) -> f64 {
    (280.46061837 + 360.98564736629 * (jd - 2_451_545.0)).rem_euclid(360.0)
}

/// Airmass at the observation time using the Kasten–Young approximation.
pub fn airmass(i: &Input, t: &Target) -> Result<f64, String> {
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
    Ok(1.0
        / ((alt + 244.0 / (165.0 + 47.0 * alt.powf(1.1)))
            .to_radians()
            .sin()))
}
