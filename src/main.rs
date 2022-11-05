mod database_utils;
mod math;

use actix_web::{get, web, App, HttpServer, Responder};
use reqwest::StatusCode;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read as _};
use subtle::ConstantTimeEq as _;

const DB_PATH: &str = "weathurber.db";

mod weather_proto {
    include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
}

struct Location {
    latitude: f64,
    longitude: f64,
}

enum Units {
    METRIC,
    IMPERIAL,
    STANDARD,
}

impl Display for Units {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Units::METRIC => f.write_str("metric"),
            Units::IMPERIAL => f.write_str("imperial"),
            Units::STANDARD => f.write_str("standard"),
        }
    }
}

impl From<String> for Units {
    fn from(unit: String) -> Self {
        match unit.as_str() {
            "metric" => Units::METRIC,
            "imperial" => Units::IMPERIAL,
            _ => Units::STANDARD,
        }
    }
}

#[derive(Deserialize, Debug)]
struct APIKey {
    owm_key: String,
}

async fn do_weather_query(keys: APIKey, location: Location, units: Units) {
    if !keys.owm_key.is_empty() {
        let owm_query = format!(
            "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&units={}",
            location.latitude, location.longitude, keys.owm_key, units
        );
        let result = reqwest::get(owm_query).await;
        if result.is_err() {
            // Our request failed for some reason, we will try again later.
            return;
        }
        let response = result.unwrap();
        if StatusCode::is_success(&response.status()) {
            // Our request failed for some reason, we will try again later.
            return;
        }
    }
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
    let ds = surrealdb::Datastore::new(format!("file://{DB_PATH}").as_str())
        .await
        .expect("unable to create datastore");
    let mut transaction = ds.transaction(false, false).await.unwrap();
    // Should have been created in main()
    let stored_digest = transaction.get("credential_sha").await.unwrap().unwrap();
    let current_digest = get_credential_digest();
    if stored_digest.ct_eq(current_digest.as_slice()).into() {
        // Load openweathermap api key.
        let credentials_raw = fs::read_to_string("creds.json").expect("No creds.json file found.");
        serde_json::from_str(&credentials_raw).expect("bad json")
    }
    panic!("Credentials may have been modified while this API was running! Check for attackers!")
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
    do_weather_query(
        keys,
        Location {
            latitude: lat,
            longitude: long,
        },
        units,
    )
    .await;
    format!("Latitude: {lat}, Longitude {long}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Store the SHA-256 hash of creds.json into the database so that we don't run into an issue
    // where an attacker can introduce a TOCTOU vuln.
    let ds = surrealdb::Datastore::new(format!("file://{DB_PATH}").as_str())
        .await
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "unable to create datastore")
        })?;
    let mut transaction = ds
        .transaction(true, true)
        .await
        .expect("unable to start transaction");
    // SHA-256 of creds.json
    let digest = get_credential_digest();
    transaction
        .set("credential_sha", digest.as_slice())
        .await
        .expect("failed to write hash to store");
    transaction.commit().await.expect("failed to commit");
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
