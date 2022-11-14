#![feature(future_join)]

// mod database_utils;
mod data_types;
mod math;

use crate::data_types::{Conditions, ProtoAdapter as _};

use crate::weather_proto::weather_message;
use actix_web::cookie::time::format_description::modifier::Hour;
use actix_web::{get, web, App, HttpServer, Responder};
use lazy_static::lazy_static;
use protobuf::MessageField;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read as _};
use subtle::ConstantTimeEq as _;
use surrealdb::Session;

mod weather_proto {
    include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
}

struct Location {
    latitude: f64,
    longitude: f64,
}

enum Units {
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

#[derive(Deserialize, Debug)]
struct APIKey {
    owm_key: String,
}

fn convert_id_to_condition(current_weather_id: i64) -> Conditions {
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

fn process_current_weather(
    current_weather_mapping: HashMap<String, Value>,
) -> data_types::HourlyWeather {
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
    data_types::HourlyWeather {
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

fn process_hourly_weather(
    hourly_weather_mapping: Vec<HashMap<String, Value>>,
) -> Vec<data_types::HourlyWeather> {
    let mut result = vec![];
    for hourly_weather in hourly_weather_mapping {
        result.push(process_current_weather(hourly_weather));
    }
    result
}

async fn do_weather_query(keys: APIKey, location: Location, units: Units) -> String {
    if !keys.owm_key.is_empty() {
        let owm_query = format!(
            "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&units={}",
            location.latitude, location.longitude, keys.owm_key, units
        );
        let result = reqwest::get(owm_query).await;
        if result.is_err() {
            // Our request failed for some reason, we will try again later.
            return "request failed 1".to_string();
        }
        let response = result.unwrap();
        if !StatusCode::is_success(&response.status()) {
            // Our request failed for some reason, we will try again later.
            return format!("request failed with statuscode: {}", &response.status());
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
        let final_weather = weather_message::WeatherInfo {
            hour_forecasts: hourly_weather.iter().map(|w| w.to_proto()).collect(),
            current_weather: Some(current_weather.to_proto()).into(),
            ..Default::default()
        };
        return final_weather.to_string();
    }
    "no key".to_string()
}

fn get_credential_digest() -> Vec<u8> {
    let input = File::open("creds.json").unwrap();
    let mut reader = BufReader::new(input);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    hasher.finalize().to_vec()
}

async fn get_api_key_from_json() -> APIKey {
    // Confirm that creds.json has not been modified, otherwise panic
    // let ds =
    // let mut transaction = ds.transaction(false, false).await.unwrap();
    // Should have been created in main()
    // let stored_digest = transaction.get("credential_sha").await.unwrap().unwrap();
    // let current_digest = get_credential_digest();
    // if stored_digest.ct_eq(current_digest.as_slice()).into() {
    // Load openweathermap api key.
    let credentials_raw = fs::read_to_string("creds.json").expect("No creds.json file found.");
    serde_json::from_str(&credentials_raw).expect("bad json")
    // }
    // panic!("Credentials may have been modified while this API was running! Check for attackers!")
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[get("/v1/api/{latitude}/{longitude}/{units}")]
async fn parse_lat_long(full_query: web::Path<(f64, f64, String)>) -> impl Responder {
    let lat = full_query.0;
    let long = full_query.1;
    let units: Units = full_query.2.to_owned().into();
    // figure out a better way to find a location in the database that is more efficient than looping over the entire db
    // anyway figure out if the db contains a location within 2km of the received lat/long using `math::haversine`
    // if contains, format the json with the relevant entry from the db.
    // if not, query owm and store the result of the api call in the db, then return the information
    // the client needs.
    let keys = get_api_key_from_json().await;
    let full_proto_response = do_weather_query(
        keys,
        Location {
            latitude: lat,
            longitude: long,
        },
        units,
    )
    .await;
    // let ds = surrealdb::Datastore::new(format!("file://{DB_PATH}").as_str()).await;
    // if ds.is_err() {
    //     return Response::new(StatusCode::EXPECTATION_FAILED);
    // }
    // let store = ds.unwrap();
    // let session = Session::for_kv();
    // let statement = "SELECT * FROM locations";
    // let res = store.execute(statement, &session, None, false);
    full_proto_response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Store the SHA-256 hash of creds.json into the database so that we don't run into an issue
    // where an attacker can introduce a TOCTOU vuln.
    // let ds = surrealdb::Datastore::new(format!("file://{DB_PATH}").as_str())
    //     .await
    //     .map_err(|_| {
    //         std::io::Error::new(std::io::ErrorKind::Other, "unable to create datastore")
    //     })?;
    // let mut transaction = ds
    //     .transaction(true, true)
    //     .await
    //     .expect("unable to start transaction");
    // // SHA-256 of creds.json
    // let digest = get_credential_digest();
    // transaction
    //     .set("credential_sha", digest.as_slice())
    //     .await
    //     .expect("failed to write hash to store");
    // transaction.commit().await.expect("failed to commit");
    HttpServer::new(|| {
        App::new()
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .service(greet)
            .service(parse_lat_long)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
