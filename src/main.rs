mod models;
mod auth;
mod controllers;
mod utils;
mod tests;

extern crate argon2;

use crate::controllers::{user_controller, event_controller, trip_controller};
use actix_web::{web, middleware, App, HttpServer, HttpResponse};
use mongodb::{Database ,Client, options::ClientOptions};
use mongodb::options::ResolverConfig;
use rusoto_core::Region;
use rusoto_s3::S3Client;
use actix_cors::Cors;

type MongoClient = mongodb::Client;
type MongoDb = mongodb::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    let s3_client = S3Client::new(Region::UsEast1);

    let mut mongo_options = ClientOptions::parse_with_resolver_config(
        std::env::var("MONGO_URL").expect("Error in Mongo URL").as_str(),
        ResolverConfig::cloudflare()
    ).await.expect("Error found while creating client options");
    mongo_options.app_name = Some("YeoHengServer".to_string());
    let mongo_client = Client::with_options(mongo_options).expect("Error found while creating mongo client");
    let mongo_db = mongo_client.database(std::env::var("DATABASE_NAME")
        .expect("Error retrieving database name")
        .as_str());
    let server = HttpServer::new(move || {
        App::new()
            .data(mongo_client.clone())
            .data(mongo_db.clone())
            .data(s3_client.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .send_wildcard()
                    .finish()
            )
            .route("/", web::get().to(user_controller::index))
            .route("/login", web::post().to(user_controller::login))
            .route("/signup", web::post().to(user_controller::register))
            .service(
                web::scope("/event")
                    .route("", web::get().to(event_controller::get_events))
                    .route("/count", web::get().to(event_controller::count_events))
                    .route("/presigned", web::get().to(event_controller::get_presigned_url))
                    .route("/create", web::post().to(event_controller::create_event))
                    .route("/update", web::put().to(event_controller::update_event))
                    .route("/{id}", web::get().to(event_controller::get_event))
            )
            .service(
                web::scope("/trip")
                    .route("", web::get().to(trip_controller::get_trips))
                    .route("", web::post().to(trip_controller::create_trip))
                    .route("", web::put().to(trip_controller::update_trip))
                    .route("/count", web::get().to(trip_controller::count_trips))
                    .route("/add", web::put().to(trip_controller::add_event_entry))
                    .route("/remove", web::put().to(trip_controller::remove_event_entry))
                    .route("/fork", web::post().to(trip_controller::fork_trip))
                    .route("/{id}", web::get().to(trip_controller::get_trip))
                    .route("/{id}", web::delete().to(trip_controller::delete_trip))
            )
            .service(
                web::scope("/user")
                    .route("/promote/{id}", web::put().to(user_controller::promote))
                    .route("/demote/{id}", web::put().to(user_controller::demote))
            )
            .default_service(
                web::route()
                    .to(|| HttpResponse::NotFound())
            )
    });

    let address = format!("0.0.0.0:{}",match std::env::var("PORT") {
        Ok(p) => p,
        Err(_e) => "3000".to_string(),
    });

    println!("{}", address);

    let server = server.bind(address)?;


    server.run().await
}
