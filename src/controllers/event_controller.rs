use crate::models::event::{Event, EventUpdate, EventFilter};
use crate::utils::external_services::create_presgigned_url;
use crate::auth::{check_user};
use crate::MongoDb;
use log::debug;
use actix_web::{web, HttpResponse};
use serde::{Serialize, Deserialize};

pub async fn create_event(db: web::Data<MongoDb>, event_json: web::Json<Event>,
                          _: check_user::CheckLogin) -> HttpResponse {

    let event = event_json.into_inner();
    let event_id = Event::create(event, &db).await;

    HttpResponse::Ok().json(event_id)
}

pub async fn get_all_events(db: web::Data<MongoDb>, event_json: web::Query<EventFilter>)
                            -> HttpResponse {

    let event_filter = event_json.into_inner();
    let events = Event::get_all(event_filter, &db).await;

    HttpResponse::Ok().json(events)
}

pub async fn update_event(db: web::Data<MongoDb>, event_json: web::Json<EventUpdate>,
                          _: check_user::CheckLogin) -> HttpResponse {

    let event = event_json.into_inner();
    match EventUpdate::update(event, &db).await {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => {
            println!("{}", e.clone());
            HttpResponse::BadRequest().body(e)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PresignedRequest {
    file_extension: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PresignedResponse {
    presigned_url: String,
    public_url: String,
}

pub async fn get_presigned_url(presigned_req_json: web::Query<PresignedRequest>, _: check_user::CheckLogin) -> HttpResponse {
    let req_info = presigned_req_json.into_inner();

    let presigned_url = create_presgigned_url(
        req_info.username,
        req_info.file_extension,
        "events".to_string())
        .await;

    match presigned_url {
        Ok((pre_url, pub_url)) => {
            let s3_url = match std::env::var("S3_URL") {
                Ok(su) => format!("{}/{}", su, pub_url),
                Err(_) => return HttpResponse::InternalServerError().body("Error creating presigned url"),
            };
            HttpResponse::Ok().json(PresignedResponse {
                presigned_url: pre_url,
                public_url: s3_url
            })
        },
        Err(e) => {
            debug!("{}", e);
            HttpResponse::InternalServerError().body("Error creating presigned url")
        }
    }
}