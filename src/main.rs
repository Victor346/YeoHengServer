mod models;
mod auth;
mod controllers;

extern crate argon2;

use crate::controllers::{user_controller, event_controller};
use actix_web::{web, middleware, App, HttpServer};
use mongodb::{Client, options::ClientOptions};
use mongodb::options::ResolverConfig;
use serde::{Serialize, Deserialize};
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client, PutObjectRequest};
use rusoto_s3::util::PreSignedRequest;
use rusoto_core::credential::{DefaultCredentialsProvider, ProvideAwsCredentials};

type MongoClient = mongodb::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    let mut s3_client = S3Client::new(Region::UsEast1);

    let resp = s3_client.list_buckets().await;

    let resp = resp.unwrap();
    for bucket in resp.buckets.unwrap().iter() {
        println!("{}", bucket.name.as_ref().unwrap());
    }


    let req = PutObjectRequest {
        bucket: "yeoheng-itesm-2020".to_string(),
        key: "temp/hasofoasdhf.jpg".to_string(),
        ..Default::default()
    };

    let credential = DefaultCredentialsProvider::new()
        .unwrap()
        .credentials()
        .await
        .unwrap();

    let presigned_url = req.get_presigned_url(&Region::UsEast1, &credential, &Default::default());
    println!("{:?}", presigned_url);

    let mut mongo_options = ClientOptions::parse_with_resolver_config(
        std::env::var("MONGO_URL").expect("Error in Mongo URL").as_str(),
        ResolverConfig::cloudflare()
    ).await.expect("Error found while creating client options");
    mongo_options.app_name = Some("YeoHengServer".to_string());
    let mongo_client = Client::with_options(mongo_options).expect("Error found while creating mongo client");

    let server = HttpServer::new(move || {
        App::new()
            .data(mongo_client.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(user_controller::index))
            .route("/login", web::post().to(user_controller::login))
            .route("/signup", web::post().to(user_controller::register))
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
