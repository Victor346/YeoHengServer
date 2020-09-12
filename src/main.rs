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

async fn dbtest(client: web::Data<MongoClient>) -> impl Responder {
    let db = client.database("yeohengDev");

    for collection_name in db.list_collection_names(None).await.expect("Error found while listing collections") {
        println!("{}", collection_name);
    }

    HttpResponse::Ok().body("Hello db")
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
            .route("/testdb", web::get().to(dbtest))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:3000")?
    };

    server.run().await
}
