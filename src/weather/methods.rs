use std::collections::HashMap;

use actix_web::Responder;
use protobuf::Message;
use reqwest::StatusCode;
use serde_json::Value;

use crate::{APIKey, entities::Location, weather::{utils::{process_daily_weather, convert_aqi_to_string}, entities::ProtoAdapter}, weather_proto::weather_message, geocoding};

use super::{entities::{HourlyWeather, Units}, utils::convert_id_to_condition};

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

pub(crate) fn process_current_weather(
    current_weather_mapping: HashMap<String, Value>,
) -> HourlyWeather {
    let current_condition = {
        // Reference: https://openweathermap.org/weather-conditions#Weather-Condition-Codes-2
        let current_weather_weather: HashMap<String, Value> = serde_json::from_value(
            current_weather_mapping
                .get("weather")
                .unwrap()
                .as_array()
                .unwrap()[0]
                .clone(),
        )
        .unwrap();
        let current_weather_id = current_weather_weather.get("id").unwrap().as_i64().unwrap();
        convert_id_to_condition(current_weather_id)
    };
    HourlyWeather {
        temp: current_weather_mapping
            .get("temp")
            .unwrap()
            .as_f64()
            .unwrap(),
        feels_like: current_weather_mapping
            .get("feels_like")
            .unwrap()
            .as_f64()
            .unwrap(),
        time: current_weather_mapping.get("dt").unwrap().as_i64().unwrap(),
        condition: current_condition,
    }
}

pub(crate) fn process_hourly_weather(
    hourly_weather_mapping: Vec<HashMap<String, Value>>,
) -> Vec<HourlyWeather> {
    let mut result = vec![];
    for hourly_weather in hourly_weather_mapping {
        result.push(process_current_weather(hourly_weather));
    }
    result.sort_by(|e, e2| e.time.partial_cmp(&e2.time).unwrap());
    result
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
        let response_mapping: HashMap<String, Value> = response.json().await.unwrap();
        eprintln!(
            "current weather: {}",
            response_mapping.get("current").unwrap()
        );
        let current_weather_mapping: HashMap<String, Value> = serde_json::from_str(
            response_mapping
                .get("current")
                .unwrap()
                .to_string()
                .as_str(),
        )
        .unwrap();
        let current_weather = process_current_weather(current_weather_mapping);
        let hourly_weather_mapping =
            serde_json::from_value(response_mapping.get("hourly").unwrap().clone()).unwrap();
        let hourly_weather = process_hourly_weather(hourly_weather_mapping);
        let daily_weather_mapping =
            serde_json::from_value(response_mapping.get("daily").unwrap().clone()).unwrap();
        let daily_weather = process_daily_weather(daily_weather_mapping);
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