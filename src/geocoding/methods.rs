use actix_web::web;
use anyhow::{anyhow, Ok};
use chrono::Utc;
use reqwest::StatusCode;
use tracing::info;

use crate::{entities::Location, weather::utils::haversine, APIKey, AppState};

use super::entities::{DoGeocodeResp, ReverseGeocode};

pub(crate) async fn do_geocode(keys: &APIKey, place_name: String) -> anyhow::Result<Location> {
    let owm_query = format!(
        "http://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
        place_name, keys.owm_key
    );
    let response = reqwest::get(owm_query).await?;

    if !StatusCode::is_success(&response.status()) {
        // Our request failed for some reason, we will try again later.
        return Ok(Location::default());
    }

    let response_mapping = response.json::<Vec<DoGeocodeResp>>().await?;

    let loc = response_mapping
        .get(0)
        .ok_or(anyhow!("response vec is empty"))?;
    Ok(Location {
        latitude: loc.lat,
        longitude: loc.lon,
    })
}

pub(crate) async fn do_reverse_geocode(
    keys: &APIKey,
    location: &Location,
    data: &web::Data<AppState>,
) -> anyhow::Result<ReverseGeocode> {
    {
        let kdtree = data.kdtree.lock().unwrap();
        let rev = data.cached_data.lock().unwrap();

        let query = [location.latitude, location.longitude];
        let (dist, nearest_idx) = kdtree.nearest_one(&query, &haversine);
        info!("Distance from given point {}km", dist);
        info!("Points in kdtree {}", kdtree.size());
        if dist < 10.0 {
            // return result from hashmap
            if let Some(cached_res) = rev.get(&nearest_idx) {
                if cached_res.expiry > Utc::now() {
                    return Ok(cached_res.reverse_geocode.clone());
                }
            }
        }
    }
    // construct query URL
    let owm_query = format!(
        "http://api.openweathermap.org/geo/1.0/reverse?lat={}&lon={}&limit=1&appid={}",
        location.latitude, location.longitude, keys.owm_key
    );

    // make request
    let response = reqwest::get(owm_query).await?;

    // Our request failed for some reason, we will try again later.
    if !StatusCode::is_success(&response.status()) {
        return Ok(ReverseGeocode::default());
    }

    // deserialize response
    let response_mapping = response.json::<Vec<DoGeocodeResp>>().await.unwrap();

    // return first response
    let loc = response_mapping
        .get(0)
        .ok_or(anyhow!("response vec is empty"))?;

    Ok(ReverseGeocode {
        name: loc.name.clone(),
        country: loc.country.clone(),
        state: loc.state.clone().unwrap_or(String::from("")),
        latitude: loc.lat,
        longitude: loc.lon,
    })
}
