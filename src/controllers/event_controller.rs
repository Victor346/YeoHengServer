use crate::models::event::{Event, EventCreate, EventFilter};
use crate::auth::{authentication, check_user};
use crate::MongoClient;
use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};

pub async fn create_event(client: web::Data<MongoClient>, event_json: web::Json<EventCreate>, _: check_user::CheckLogin) -> HttpResponse {
    let event = event_json.into_inner();

    let event_id = Event::create(event, &client).await;

    HttpResponse::Ok().json(event_id)
}

pub async fn get_all_events(client: web::Data<MongoClient>, event_json: web::Json<EventFilter>, _: check_user::CheckLogin) -> HttpResponse {
    let event_filter = event_json.into_inner();

    let events = Event::get_all(event_filter, &client).await;

    HttpResponse::Ok().json(events)
}