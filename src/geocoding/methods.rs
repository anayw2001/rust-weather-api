use anyhow::{anyhow, Ok};
use reqwest::StatusCode;

use crate::{entities::Location, APIKey};

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
) -> anyhow::Result<ReverseGeocode> {
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
    })
}
