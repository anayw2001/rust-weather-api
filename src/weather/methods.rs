use actix_web::web;
use chrono::{Duration, Utc};
use kiddo::distance::squared_euclidean;
use protobuf::Message;
use reqwest::StatusCode;
use tracing::debug;

use crate::{
    entities::Location,
    geocoding,
    weather::{
        entities::{ProtoAdapter, WeatherResponse},
        utils::{convert_aqi_to_string, unit_sphere_squared_euclidean_to_kilometres},
    },
    weather_proto::weather_message,
    APIKey, AppState,
};

use super::{
    entities::{AqiResponse, CachedData, Units},
    utils::degrees_lat_lng_to_unit_sphere,
};

pub(crate) async fn do_aqi_query(keys: &APIKey, location: &Location) -> anyhow::Result<i64> {
    let owm_query = format!(
        "http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}",
        location.latitude, location.longitude, keys.owm_key
    );
    let result = reqwest::get(owm_query).await?;
    let response_mapping = result.json::<AqiResponse>().await.unwrap();
    let aqi = response_mapping.list[0].main.aqi;
    Ok(aqi)
}

pub(crate) async fn do_weather_query(
    keys: APIKey,
    location: Location,
    units: Units,
    data: web::Data<AppState>,
) -> anyhow::Result<Vec<u8>> {
    {
        // lock kdtree and cached data hashmap
        let mut kdtree = data.kdtree.lock().unwrap();
        let mut cached_data = data.cached_data.lock().unwrap();
        debug!("Locked kdtree and cache hashmap");

        // convert point to 3d coordinates
        let query = degrees_lat_lng_to_unit_sphere(location.latitude, location.longitude);
        debug!("Converted lat,lon to 3d coordinates");

        // query nearest single point
        let (dist, nearest_idx) = kdtree.nearest_one(&query, &squared_euclidean);

        // convert euclidean square distance to kilometres
        let dist_km = unit_sphere_squared_euclidean_to_kilometres(dist);
        debug!("Distance from given point {}km", dist_km);
        debug!("Points in kdtree {}", kdtree.size());

        // if nearest point is less than 10km away, we can use it
        if dist_km < 10.0 {
            debug!("Nearest point is less than 10km");
            // return result from hashmap
            if let Some(cached_res) = cached_data.get(&nearest_idx) {
                debug!("Hashmap contains data for this index");
                // if data is not yet stale, return it
                if cached_res.expiry > Utc::now() {
                    debug!("Returned data is not yet stale");
                    return Ok(cached_res.weather.write_to_bytes().unwrap());
                } else {
                    debug!("Returned data is stale. Clearing cache.");
                    kdtree.remove(&query, nearest_idx);
                    cached_data.remove(&nearest_idx);
                }
            }
        }
        debug!("Nearest point is more than 10kmm querying OWM");
    }

    // otherwise, query OWM for weather data
    let owm_query = format!(
        "http://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&units={}",
        location.latitude, location.longitude, keys.owm_key, units
    );

    debug!("OWM query: {}", owm_query);
    let response = reqwest::get(owm_query).await?;

    debug!("Got response from OWM");

    if !StatusCode::is_success(&response.status()) {
        // Our request failed for some reason, we will try again later.
        return Ok(
            format!("request failed with statuscode: {}", &response.status())
                .as_bytes()
                .to_vec(),
        );
    }

    // deserialize OWM response
    let response_mapping = response.json::<WeatherResponse>().await.unwrap();

    debug!("Deserialized response");

    // fetch reverse geocode for location
    let reverse_geocode = geocoding::methods::do_reverse_geocode(&keys, &location, &data).await?;

    debug!("Fetched reverse geocode");

    // construct response
    let final_weather = weather_message::WeatherInfo {
        hour_forecasts: response_mapping
            .hourly
            .iter()
            .map(|w| w.to_proto())
            .collect(),
        current_weather: Some(response_mapping.current.to_proto()).into(),
        forecasts: response_mapping
            .daily
            .iter()
            .map(|w| w.to_proto())
            .collect(),
        aqi: convert_aqi_to_string(do_aqi_query(&keys, &location).await?),
        geocode: Some(reverse_geocode.to_proto()).into(),
        alerts: response_mapping
            .alerts
            .unwrap_or(vec![])
            .iter()
            .map(|a| a.to_proto())
            .collect(),
        ..Default::default()
    };

    // re-acquire locks, we need to write some data to cache
    let mut kdtree = data.kdtree.lock().unwrap();
    let mut cached_data = data.cached_data.lock().unwrap();

    // generate incremental index
    let index = cached_data.len() + 1;

    // insert index and cached data into hashmap
    cached_data.insert(
        index,
        CachedData {
            weather: final_weather.clone(),
            reverse_geocode,
            expiry: Utc::now() + Duration::hours(1),
        },
    );

    // add index to kdtree
    kdtree.add(
        &degrees_lat_lng_to_unit_sphere(location.latitude, location.longitude),
        index,
    );

    Ok(final_weather.write_to_bytes().unwrap())
}
