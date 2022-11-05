use actix_web::{get, web, App, HttpServer, Responder};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[get("/v1/api/{latitude}/{longitude}")]
async fn parse_lat_long(lat_long: web::Path<(f64, f64)>) -> impl Responder {
    let lat = lat_long.0;
    let long = lat_long.1;
    format!("Latitude: {lat}, Longitude {long}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
