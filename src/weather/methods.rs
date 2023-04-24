use std::collections::HashMap;

use actix_web::Responder;
use protobuf::Message;
use reqwest::StatusCode;
use serde_json::Value;

use crate::{APIKey, entities::Location, weather::{utils::{convert_aqi_to_string}, entities::{ProtoAdapter, WeatherResponse}}, weather_proto::weather_message, geocoding};

use super::entities::Units;

pub(crate) async fn do_aqi_query(keys: &APIKey, location: &Location) -> i32 {
    if !keys.owm_key.is_empty() {
        let owm_query = format!(
            "http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}",
            location.latitude, location.longitude, keys.owm_key
        );
        let result = reqwest::get(owm_query).await;
        if let Ok(response) = result {
            // Our request failed for some reason, we will try again later.
            let response_mapping: HashMap<String, Value> = response.json().await.unwrap();
            let response_list: Vec<HashMap<String, Value>> =
                serde_json::from_value(response_mapping.get("list").unwrap().clone()).unwrap();
            let main: HashMap<String, i32> =
                serde_json::from_value(response_list[0].get("main").unwrap().clone()).unwrap();
            *main.get("aqi").unwrap()
        } else {
            -1
        }
    } else {
        -1
    }
}

pub (crate) async fn do_weather_query(keys: APIKey, location: Location, units: Units) -> impl Responder {
    if !keys.owm_key.is_empty() {
        let owm_query = format!(
            "http://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&units={}",
            location.latitude, location.longitude, keys.owm_key, units
        );
        let result = reqwest::get(owm_query).await;
        if result.is_err() {
            println!("{:?}", result.err());
            // Our request failed for some reason, we will try again later.
            return b"request failed 1".to_vec();
        }
        let response = result.unwrap();
        if !StatusCode::is_success(&response.status()) {
            // Our request failed for some reason, we will try again later.
            return format!("request failed with statuscode: {}", &response.status())
                .as_bytes()
                .to_vec();
        }
        let response_mapping = response.json::<WeatherResponse>().await.unwrap();
        let current_weather = response_mapping.current;
        let hourly_weather = response_mapping.hourly;
        let daily_weather = response_mapping.daily;
        let final_weather = weather_message::WeatherInfo {
            hour_forecasts: hourly_weather.iter().map(|w| w.to_proto()).collect(),
            current_weather: Some(current_weather.to_proto()).into(),
            forecasts: daily_weather.iter().map(|w| w.to_proto()).collect(),
            aqi: convert_aqi_to_string(do_aqi_query(&keys, &location).await),
            geocode: Some(geocoding::methods::do_reverse_geocode(&keys, &location).await.to_proto()).into(),
            ..Default::default()
        };
        return final_weather.write_to_bytes().unwrap();
    }
    b"no key".to_vec()
}