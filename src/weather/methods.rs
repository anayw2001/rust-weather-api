
use protobuf::Message;
use reqwest::StatusCode;

use crate::{
    entities::Location,
    geocoding,
    weather::{
        entities::{ProtoAdapter, WeatherResponse},
        utils::convert_aqi_to_string,
    },
    weather_proto::weather_message,
    APIKey,
};

use super::entities::{AqiResponse, Units};

pub(crate) async fn do_aqi_query(keys: &APIKey, location: &Location) -> anyhow::Result<i64> {
    let owm_query = format!(
        "http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}",
        location.latitude, location.longitude, keys.owm_key
    );
    let result = reqwest::get(owm_query).await?;
    let response_mapping = result.json::<AqiResponse>().await.unwrap();
    let aqi = response_mapping.list[0].main.aqi.try_into()?;
    Ok(aqi)
}

pub(crate) async fn do_weather_query(
    keys: APIKey,
    location: Location,
    units: Units,
) -> anyhow::Result<Vec<u8>> {
    let owm_query = format!(
        "http://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&units={}",
        location.latitude, location.longitude, keys.owm_key, units
    );
    let response = reqwest::get(owm_query).await?;
    if !StatusCode::is_success(&response.status()) {
        // Our request failed for some reason, we will try again later.
        return Ok(
            format!("request failed with statuscode: {}", &response.status())
                .as_bytes()
                .to_vec(),
        );
    }
    
    let response_mapping = response.json::<WeatherResponse>().await.unwrap();

    let final_weather = weather_message::WeatherInfo {
        hour_forecasts: response_mapping.hourly.iter().map(|w| w.to_proto()).collect(),
        current_weather: Some(response_mapping.current.to_proto()).into(),
        forecasts: response_mapping.daily.iter().map(|w| w.to_proto()).collect(),
        aqi: convert_aqi_to_string(do_aqi_query(&keys, &location).await?),
        geocode: Some(
            geocoding::methods::do_reverse_geocode(&keys, &location)
                .await?
                .to_proto(),
        )
        .into(),
        ..Default::default()
    };

    Ok(final_weather.write_to_bytes().unwrap())
}
