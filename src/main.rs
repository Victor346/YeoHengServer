mod models;
mod auth;

extern crate argon2;

use crate::models::user::User;
use crate::auth::authentication;
use actix_web::{web, middleware, App, HttpResponse, Responder, HttpServer};
use mongodb::{Client, options::ClientOptions};
use listenfd::ListenFd;
use mongodb::options::ResolverConfig;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn index2() -> impl Responder {
    HttpResponse::Ok().body("Hello world 3")
}

async fn register(client: web::Data<MongoClient>, user_json: web::Json<User>) -> impl Responder {
    let user = user_json.into_inner();

    println!("{:?}", user);

    match User::validate(user, &client).await {
        Ok(mut validated_user) => {
            println!("{:?}", validated_user);
            let salted_pass = authentication::salt_password(validated_user.password.clone());
            validated_user.password = salted_pass;
            User::insert(validated_user, &client).await;

            HttpResponse::Ok().body("Todo chido")
        },
        Err(e) => {
            println!("{}", e.clone());
            HttpResponse::BadRequest().body(e)
        },
    }
}

type MongoClient = mongodb::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    let mut mongo_options = ClientOptions::parse_with_resolver_config(
        std::env::var("MONGO_URL").expect("Error in Mongo URL").as_str(),
        ResolverConfig::cloudflare()
    ).await.expect("Error found while creating client options");
    mongo_options.app_name = Some("YeoHengServer".to_string());
    let mongo_client = Client::with_options(mongo_options).expect("Error found while creating mongo client");

    let mut listenfd = ListenFd::from_env();
    let  mut server = HttpServer::new(move || {
        App::new()
            .data(mongo_client.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .route("/again", web::get().to(index2))
            .route("/signup", web::get().to(register))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:3000")?
    };

    server.run().await
}
