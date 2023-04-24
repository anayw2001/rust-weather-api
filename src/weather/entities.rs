use std::fmt::{Display, Formatter};

use crate::weather_proto::weather_message;
use protobuf::EnumOrUnknown;

pub(crate) trait ProtoAdapter {
    type ProtoType;
    fn to_proto(&self) -> Self::ProtoType;
}

pub(crate) enum Conditions {
    Unknown,
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
            Conditions::Unknown => weather_message::Conditions::UNKNOWN,
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

pub (crate) enum Units {
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
