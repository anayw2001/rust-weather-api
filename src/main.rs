mod database_utils;
mod math;

use actix_web::{get, web, App, HttpServer, Responder};
use serde::Deserialize;
use std::fs;

struct Location {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize, Debug)]
struct APIKey {
    key: String,
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[get("/v1/api/{latitude}/{longitude}")]
async fn parse_lat_long(lat_long: web::Path<(f64, f64)>) -> impl Responder {
    let lat = lat_long.0;
    let long = lat_long.1;
    // figure out a better way to find a location in the database that is more efficient than looping over the entire db
    // anyway figure out if the db contains a location within 2km of the received lat/long using `math::haversine`
    // if contains, format the json with the relevant entry from the db.
    // if not, query owm and store the result of the api call in the db, then return the information
    // the client needs.
    format!("Latitude: {lat}, Longitude {long}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load openweathermap api key.
    let owm_credentials_raw = fs::read_to_string("creds.json").expect("No creds.json file found.");
    let owm_key: APIKey = serde_json::from_str(&owm_credentials_raw).expect("bad json");
    println!("owm_api_key: {owm_key:?}");
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
