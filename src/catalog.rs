// Cache for the NASA Exoplanet Archive lookups
use crate::state::{Cache, Target};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn cache_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_default()
        .join("transit_catalog_cache.json")
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn load_cache() -> Cache {
    fs::read(cache_path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// Query the NASA Exoplanet Archive for a planet by name
pub fn fetch_target(name: &str) -> Result<Target, String> {
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
        ra_deg:                n("ra"),
        dec_deg:               n("dec"),
        vmag:                  n("sy_vmag"),
        rmag:                  n("sy_rmag"),
        imag:                  n("sy_imag"),
        star_radius_solar:     n("st_rad"),
        planet_radius_jupiter: n("pl_radj"),
        duration_hours:        n("pl_trandur"),
        period_days:           n("pl_orbper"),
        epoch_bjd:             n("pl_tranmid"),
    })
}