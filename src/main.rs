mod models;
mod auth;
mod controllers;
mod utils;

extern crate argon2;

use crate::controllers::{user_controller, event_controller};
use actix_web::{web, middleware, App, HttpServer};
use mongodb::{Client, options::ClientOptions};
use mongodb::options::ResolverConfig;
use serde::{Serialize, Deserialize};
use rusoto_core::Region;
use rusoto_s3::S3Client;

type MongoClient = mongodb::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    let mut s3_client = S3Client::new(Region::UsEast1);

    let mut mongo_options = ClientOptions::parse_with_resolver_config(
        std::env::var("MONGO_URL").expect("Error in Mongo URL").as_str(),
        ResolverConfig::cloudflare()
    ).await.expect("Error found while creating client options");
    mongo_options.app_name = Some("YeoHengServer".to_string());
    let mongo_client = Client::with_options(mongo_options).expect("Error found while creating mongo client");

    let server = HttpServer::new(move || {
        App::new()
            .data(mongo_client.clone())
            .data(s3_client.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(user_controller::index))
            .route("/login", web::post().to(user_controller::login))
            .route("/signup", web::post().to(user_controller::register))
            .route("event/presigned", web::get().to(event_controller::get_presigned_url))
            .service(web::resource("/protected")
                .route(web::post().to(event_controller::create_event)))
    });

    let address = format!("0.0.0.0:{}",match std::env::var("PORT") {
        Ok(p) => p,
        Err(_e) => "3000".to_string(),
    });

    println!("{}", address);

    let server = server.bind(address)?;


    server.run().await
}
