use std::collections::HashMap;

use reqwest::StatusCode;
use serde_json::Value;

use crate::{APIKey, Location, data_types};

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
            let response_mapping: Vec<HashMap<String, Value>> = response.json().await.unwrap();
            if response_mapping.len() == 0 {
                return Location::default();
            }
            Location {
                latitude: response_mapping
                    .get(0)
                    .unwrap()
                    .get("lat")
                    .unwrap()
                    .as_f64()
                    .unwrap(),
                longitude: response_mapping
                    .get(0)
                    .unwrap()
                    .get("lon")
                    .unwrap()
                    .as_f64()
                    .unwrap(),
            }
        } else {
            Location::default()
        }
    } else {
        Location::default()
    }
}

pub(crate) async fn do_reverse_geocode(keys: &APIKey, location: &Location) -> data_types::ReverseGeocode {
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
            let response_mapping: Vec<HashMap<String, Value>> = response.json().await.unwrap();
            let first_element: HashMap<String, Value> = response_mapping[0].clone();
            data_types::ReverseGeocode {
                name: first_element.get("name").unwrap().to_string(),
                country: first_element.get("country").unwrap().to_string(),
                state: {
                    if first_element.contains_key("state") {
                        first_element.get("state").unwrap().to_string()
                    } else {
                        "".to_string()
                    }
                },
            }
        } else {
            data_types::ReverseGeocode::default()
        }
    } else {
        data_types::ReverseGeocode::default()
    }
}
