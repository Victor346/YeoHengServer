use crate::models::event::{Event};
use crate::utils::external_services::create_presgigned_url;
use crate::auth::{authentication, check_user};
use crate::MongoClient;
use log::debug;
use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};

pub async fn create_event(client: web::Data<MongoClient>, event_json: web::Json<Event>, _: check_user::CheckLogin) -> HttpResponse {
    let event = event_json.into_inner();

    let event_id = Event::create(event, &client).await;

    HttpResponse::Ok().json(event_id)
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
                Err(e) => return HttpResponse::InternalServerError().body("Error creating presigned url"),
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