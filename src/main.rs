// mod database_utils;
mod entities;
mod errors;
mod geocoding;
mod weather;

use crate::errors::IntoHttpError;
use crate::weather::entities::{ProtoAdapter as _, Units};
use crate::weather::methods::do_weather_query;

use actix_web::{get, web, App, HttpServer, Responder};
use entities::Location;
use kiddo::float::kdtree::KdTree;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use tracing::info;
use weather::entities::CachedData;

mod weather_proto {
    include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
}

#[derive(Debug)]
struct AppState {
    kdtree: Mutex<KdTree<f64, usize, 3, 32, u32>>,
    cached_data: Mutex<HashMap<usize, CachedData>>,
}

#[derive(Deserialize, Debug)]
struct APIKey {
    owm_key: String,
}

#[tracing::instrument]
fn get_api_key_from_env() -> APIKey {
    info!("Fetching OWM API key");
    // Load openweathermap api key and return it.
    let owm_key = env::var("OWM_KEY").expect("OWM_KEY not set");
    // kill the process if the key is empty and return the key if it is not.
    if owm_key.is_empty() {
        panic!("OWM_KEY is empty!");
    } else {
        APIKey { owm_key }
    }
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[get("/v1/api/weather/{latitude}/{longitude}/{units}")]
#[tracing::instrument(skip(data))]
async fn parse_lat_long(
    full_query: web::Path<(f64, f64, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let lat = full_query.0;
    let long = full_query.1;
    let units: Units = full_query.2.to_owned().into();
    // figure out a better way to find a location in the database that is more efficient than looping over the entire db
    // anyway figure out if the db contains a location within 2km of the received lat/long using `math::haversine`
    // if contains, format the json with the relevant entry from the db.
    // if not, query owm and store the result of the api call in the db, then return the information
    // the client needs.
    let keys = get_api_key_from_env();
    let full_proto_response = do_weather_query(
        keys,
        Location {
            latitude: lat,
            longitude: long,
        },
        units,
        data,
    )
    .await;
    full_proto_response.http_internal_error("could not fetch weather")
}

#[get("/v1/api/geocode/{place}")]
#[tracing::instrument]
async fn geocode(full_query: web::Path<String>) -> impl Responder {
    let keys = get_api_key_from_env();
    let response = geocoding::methods::do_geocode(&keys, full_query.into_inner()).await;
    match response {
        Ok(resp) => format!("{}, {}", resp.latitude, resp.longitude),
        Err(_) => String::from("something went wrong"),
    }
}

#[get("/v1/api/reversegeocode/{latitude}/{longitude}")]
#[tracing::instrument(skip(data))]
async fn reverse_geocode(
    full_query: web::Path<(f64, f64)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let loc_tup = full_query.into_inner();
    let loc = Location {
        latitude: loc_tup.0,
        longitude: loc_tup.1,
    };

    let keys = get_api_key_from_env();
    let resp = geocoding::methods::do_reverse_geocode(&keys, &loc, &data).await;
    match resp {
        Ok(response) => response.to_proto().to_string(),
        Err(_) => String::from("something went wrong"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // install global tracing collector configured based on RUST_LOG env var
    tracing_subscriber::fmt::init();
    info!("Initialized tracing");

    info!("Starting HTTP server on port 8080");
    let web_data = web::Data::new(AppState {
        kdtree: Mutex::new(KdTree::new()),
        cached_data: Mutex::new(HashMap::new()),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(web_data.clone())
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .service(greet)
            .service(geocode)
            .service(reverse_geocode)
            .service(parse_lat_long)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

// TODO: probably write some unit tests for the env var function.
