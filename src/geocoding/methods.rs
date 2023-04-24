use reqwest::StatusCode;

use crate::{data_types, APIKey, Location};

use super::entities::DoGeocodeResp;

pub(crate) async fn do_geocode(keys: &APIKey, place_name: String) -> Location {
    if !keys.owm_key.is_empty() {
        let owm_query = format!(
            "http://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
            place_name, keys.owm_key
        );
        let result = reqwest::get(owm_query).await;
        if let Ok(response) = result {
            if !StatusCode::is_success(&response.status()) {
                // Our request failed for some reason, we will try again later.
                return Location::default();
            }
            let response_mapping = response.json::<Vec<DoGeocodeResp>>().await.unwrap();
            if response_mapping.len() == 0 {
                return Location::default();
            }
            let first = response_mapping.get(0);
            if let Some(loc) = first {
                Location {
                    latitude: loc.lat,
                    longitude: loc.lon,
                }
            } else {
                Location::default()
            }
        } else {
            Location::default()
        }
    } else {
        Location::default()
    }
}

pub(crate) async fn do_reverse_geocode(
    keys: &APIKey,
    location: &Location,
) -> data_types::ReverseGeocode {
    if !keys.owm_key.is_empty() {
        let owm_query = format!(
            "http://api.openweathermap.org/geo/1.0/reverse?lat={}&lon={}&limit=1&appid={}",
            location.latitude, location.longitude, keys.owm_key
        );
        let result = reqwest::get(owm_query).await;
        if let Ok(response) = result {
            if !StatusCode::is_success(&response.status()) {
                // Our request failed for some reason, we will try again later.
                return data_types::ReverseGeocode::default();
            }
            let response_mapping = response.json::<Vec<DoGeocodeResp>>().await.unwrap();
            let first = response_mapping.get(0);
            if let Some(loc) = first {
                data_types::ReverseGeocode {
                    name: loc.name.clone(),
                    country: loc.country.clone(),
                    state: loc.state.clone().unwrap_or(String::from("")),
                }
            } else {
                data_types::ReverseGeocode::default()
            }
        } else {
            data_types::ReverseGeocode::default()
        }
    } else {
        data_types::ReverseGeocode::default()
    }
}
