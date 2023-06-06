use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use crate::{geocoding::entities::ReverseGeocode, weather_proto::weather_message};
use chrono::{DateTime, Utc};
use protobuf::EnumOrUnknown;

pub(crate) trait ProtoAdapter {
    type ProtoType;
    fn to_proto(&self) -> Self::ProtoType;
}

pub(crate) enum Conditions {
    Rainy,
    Cloudy,
    Clear,
    Snow,
    Overcast,
    Storm,
}

impl ProtoAdapter for Conditions {
    type ProtoType = EnumOrUnknown<weather_message::Conditions>;

    fn to_proto(&self) -> Self::ProtoType {
        match self {
            Conditions::Rainy => weather_message::Conditions::RAINY,
            Conditions::Cloudy => weather_message::Conditions::CLOUDY,
            Conditions::Clear => weather_message::Conditions::CLEAR,
            Conditions::Snow => weather_message::Conditions::SNOW,
            Conditions::Overcast => weather_message::Conditions::OVERCAST,
            Conditions::Storm => weather_message::Conditions::STORM,
        }
        .into()
    }
}

pub(crate) struct OneDayForecast {
    pub(crate) high_temp: f64,
    pub(crate) low_temp: f64,
    pub(crate) condition: Conditions,
    pub(crate) time: i64,
    pub(crate) sunrise: i64,
    pub(crate) sunset: i64,
    pub(crate) rain: f64,
}

impl ProtoAdapter for OneDayForecast {
    type ProtoType = weather_message::OneDayForecast;

    fn to_proto(&self) -> Self::ProtoType {
        weather_message::OneDayForecast {
            high_temp: self.high_temp,
            low_temp: self.low_temp,
            condition: self.condition.to_proto(),
            time: self.time,
            sunrise: self.sunrise,
            sunset: self.sunset,
            rain: self.rain,
            ..Default::default()
        }
    }
}

pub(crate) struct HourlyWeather {
    pub(crate) temp: f64,
    pub(crate) feels_like: f64,
    pub(crate) condition: Conditions,
    pub(crate) time: i64,
}

impl ProtoAdapter for HourlyWeather {
    type ProtoType = weather_message::HourlyWeather;

    fn to_proto(&self) -> Self::ProtoType {
        weather_message::HourlyWeather {
            temp: self.temp,
            feels_like: self.feels_like,
            condition: self.condition.to_proto(),
            time: self.time,
            ..Default::default()
        }
    }
}

struct WeatherInfo {
    forecasts: Vec<OneDayForecast>,
    hour_forecasts: Vec<HourlyWeather>,
    aqi: String,
    wind_speed: f32,
    weather_alerts: String,
}

impl ProtoAdapter for WeatherInfo {
    type ProtoType = weather_message::WeatherInfo;

    fn to_proto(&self) -> Self::ProtoType {
        weather_message::WeatherInfo {
            forecasts: self.forecasts.iter().map(|x| x.to_proto()).collect(),
            hour_forecasts: self.hour_forecasts.iter().map(|x| x.to_proto()).collect(),
            aqi: self.aqi.clone(),
            wind_speed: self.wind_speed,
            weather_alerts: self.weather_alerts.clone(),
            ..Default::default()
        }
    }
}

pub(crate) enum Units {
    Metric,
    Imperial,
    Standard,
}

impl Display for Units {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Units::Metric => f.write_str("metric"),
            Units::Imperial => f.write_str("imperial"),
            Units::Standard => f.write_str("standard"),
        }
    }
}

impl From<String> for Units {
    fn from(unit: String) -> Self {
        match unit.as_str() {
            "metric" => Units::Metric,
            "imperial" => Units::Imperial,
            _ => Units::Standard,
        }
    }
}

// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::WeatherResponse;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: WeatherResponse = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};

use super::utils::convert_id_to_condition;

// Keep up to date with https://openweathermap.org/api/one-call-3#parameter.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub(crate) lat: f64,
    pub(crate) lon: f64,
    pub(crate) timezone: String,
    pub(crate) timezone_offset: i64,
    pub(crate) current: Current,
    pub(crate) minutely: Vec<Minutely>,
    pub(crate) hourly: Vec<Current>,
    pub(crate) daily: Vec<Daily>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Current {
    pub(crate) dt: i64,
    pub(crate) sunrise: Option<i64>,
    pub(crate) sunset: Option<i64>,
    pub(crate) temp: f64,
    pub(crate) feels_like: f64,
    pub(crate) pressure: i64,
    pub(crate) humidity: i64,
    pub(crate) dew_point: f64,
    pub(crate) clouds: i64,
    pub(crate) uvi: f64,
    pub(crate) visibility: i64,
    pub(crate) wind_speed: f64,
    pub(crate) wind_deg: i64,
    pub(crate) wind_gust: Option<f64>,
    pub(crate) weather: Vec<Weather>,
    pub(crate) rain: Option<RainSnow>,
    pub(crate) snow: Option<RainSnow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RainSnow {
    pub(crate) r1_h: Option<f64>,
}

impl ProtoAdapter for Current {
    type ProtoType = weather_message::HourlyWeather;

    fn to_proto(&self) -> Self::ProtoType {
        weather_message::HourlyWeather {
            temp: self.temp,
            feels_like: self.feels_like,
            condition: convert_id_to_condition(self.weather[0].id).to_proto(),
            time: self.dt,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    pub(crate) id: i64,
    pub(crate) main: Main,
    pub(crate) description: Description,
    pub(crate) icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Daily {
    pub(crate) dt: i64,
    pub(crate) sunrise: i64,
    pub(crate) sunset: i64,
    pub(crate) moonrise: i64,
    pub(crate) moonset: i64,
    pub(crate) moon_phase: f64,
    pub(crate) temp: Temp,
    pub(crate) feels_like: FeelsLike,
    pub(crate) pressure: i64,
    pub(crate) humidity: i64,
    pub(crate) dew_point: f64,
    pub(crate) wind_speed: f64,
    pub(crate) wind_deg: i64,
    pub(crate) wind_gust: f64,
    pub(crate) weather: Vec<Weather>,
    pub(crate) clouds: i64,
    pub(crate) pop: f64,
    pub(crate) uvi: f64,
    pub(crate) rain: Option<f64>,
    pub(crate) summary: String,
}

impl ProtoAdapter for Daily {
    type ProtoType = weather_message::OneDayForecast;

    fn to_proto(&self) -> Self::ProtoType {
        weather_message::OneDayForecast {
            high_temp: self.temp.max,
            low_temp: self.temp.min,
            condition: convert_id_to_condition(self.weather[0].id).to_proto(),
            time: self.dt,
            sunrise: self.sunrise,
            sunset: self.sunset,
            rain: self.rain.unwrap_or(0f64),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeelsLike {
    pub(crate) day: f64,
    pub(crate) night: f64,
    pub(crate) eve: f64,
    pub(crate) morn: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Temp {
    pub(crate) day: f64,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) night: f64,
    pub(crate) eve: f64,
    pub(crate) morn: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minutely {
    pub(crate) dt: i64,
    pub(crate) precipitation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Description {
    #[serde(rename = "broken clouds")]
    BrokenClouds,
    #[serde(rename = "clear sky")]
    ClearSky,
    #[serde(rename = "few clouds")]
    FewClouds,
    #[serde(rename = "light rain")]
    LightRain,
    #[serde(rename = "moderate rain")]
    ModerateRain,
    #[serde(rename = "heavy intensity rain")]
    HeavyIntensityRain,
    #[serde(rename = "mist")]
    Mist,
    #[serde(rename = "overcast clouds")]
    OvercastClouds,
    #[serde(rename = "scattered clouds")]
    ScatteredClouds,
    #[serde(rename = "haze")]
    Haze,
    #[serde(rename = "smoke")]
    Smoke,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Main {
    Clear,
    Clouds,
    Haze,
    Mist,
    Rain,
    Smoke,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AqiResponse {
    pub(crate) coord: Coord,
    pub(crate) list: Vec<List>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coord {
    pub(crate) lon: f64,
    pub(crate) lat: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub(crate) main: AqiMain,
    pub(crate) components: HashMap<String, f64>,
    pub(crate) dt: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AqiMain {
    pub(crate) aqi: i64,
}

#[derive(Debug)]
pub struct CachedData {
    pub weather: weather_message::WeatherInfo,
    pub reverse_geocode: ReverseGeocode,
    pub expiry: DateTime<Utc>,
}
