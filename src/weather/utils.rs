use std::collections::HashMap;

use serde_json::Value;

use super::entities::OneDayForecast;

use crate::weather::entities::Conditions;

pub(crate) fn convert_aqi_to_string(aqi: i32) -> String {
    String::from(match aqi {
        1 => "Good",
        2 => "Fair",
        3 => "Moderate",
        4 => "Poor",
        _ => "Very Poor",
    })
}

pub(crate) fn convert_id_to_condition(current_weather_id: i64) -> Conditions {
    if current_weather_id == 800 {
        Conditions::Clear
    } else if current_weather_id > 800 && current_weather_id < 805 {
        if current_weather_id == 804 {
            Conditions::Overcast
        } else {
            Conditions::Cloudy
        }
    } else if current_weather_id > 599 && current_weather_id < 700 {
        Conditions::Snow
    } else if current_weather_id > 199 && current_weather_id < 300 {
        Conditions::Storm
    } else {
        Conditions::Rainy
    }
}

pub(crate) fn process_daily_weather(
    daily_weather_mapping: Vec<HashMap<String, Value>>,
) -> Vec<OneDayForecast> {
    let mut result = vec![];
    for daily_weather in daily_weather_mapping {
        println!("daily_weather: {:?}", daily_weather);
        let temp_mapping: HashMap<String, f64> =
            serde_json::from_value(daily_weather.get("temp").unwrap().clone()).unwrap();
        let high_temp = *temp_mapping.get("max").unwrap();
        let low_temp = *temp_mapping.get("min").unwrap();
        let current_condition = {
            // Reference: https://openweathermap.org/weather-conditions#Weather-Condition-Codes-2
            let current_weather_weather: HashMap<String, Value> = serde_json::from_value(
                daily_weather.get("weather").unwrap().as_array().unwrap()[0].clone(),
            )
            .unwrap();
            let current_weather_id = current_weather_weather.get("id").unwrap().as_i64().unwrap();
            convert_id_to_condition(current_weather_id)
        };
        result.push(OneDayForecast {
            high_temp,
            low_temp,
            condition: current_condition,
            time: daily_weather.get("dt").unwrap().as_i64().unwrap(),
            sunrise: daily_weather.get("sunrise").unwrap().as_i64().unwrap(),
            sunset: daily_weather.get("sunset").unwrap().as_i64().unwrap(),
            rain: daily_weather
                .get("rain")
                .unwrap_or(&serde_json::Value::from(0f64))
                .as_f64()
                .unwrap(),
        })
    }
    result
}
