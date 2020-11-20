use crate::models::trip::{Trip, TripCreate, TripEdit, TripFilter, EventEntry, TripFork};
use crate::auth::check_user;
use crate::MongoDb;

use actix_web::{web, HttpResponse};
use actix_web::http::StatusCode;
use std::collections::HashMap;

pub async fn create_trip(db: web::Data<MongoDb>,
                         trip_json: web::Json<TripCreate>,
                         check: check_user::CheckLogin
) -> HttpResponse {
    let trip = trip_json.into_inner();

    let trip_id = Trip::create(trip, &db).await;

    HttpResponse::Created().json(trip_id)
}

pub async fn get_trip(db: web::Data<MongoDb>, trip_path: web::Path<String>) -> HttpResponse {
    let trip_id = trip_path.into_inner();

    match Trip::get_trip(trip_id, &db).await {
        Ok(trip) => HttpResponse::Ok().json(trip),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn get_trips(db: web::Data<MongoDb>, trip_json: web::Query<TripFilter>) -> HttpResponse {
    let trip_filter = trip_json.into_inner();
    let trips = Trip::get_filtered_trips(trip_filter, &db).await;

    HttpResponse::Ok().json(trips)
}

pub async fn count_trips(db: web::Data<MongoDb>, trip_json: web::Query<TripFilter>) -> HttpResponse {
    let trip_filter = trip_json.into_inner();

    match Trip::count_filtered_trips(trip_filter, &db).await {
        Ok(count) => {
            let mut map: HashMap<&str, i64> = HashMap::new();
            map.insert("event_count", count);
            HttpResponse::Ok().json(map)
        },
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn delete_trip(db: web::Data<MongoDb>, trip_path: web::Path<String>) -> HttpResponse {
    let trip_id = trip_path.into_inner();

    match Trip::delete_trip(trip_id, &db).await {
        Ok(_count) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn update_trip(db: web::Data<MongoDb>,
                        trip_json: web::Json<TripEdit>,
                        _: check_user::CheckLogin
) -> HttpResponse {
    let trip_edit = trip_json.into_inner();
    match Trip::update(trip_edit, &db).await {
        Ok(trip) => HttpResponse::Ok().json(trip),
        Err(e) => {
            println!("{}", e.clone());
            HttpResponse::BadRequest().body(e)
        }
    }
}

pub async fn add_event_entry(db: web::Data<MongoDb>,
                             entry_json: web::Json<EventEntry>,
                             _:check_user::CheckLogin
) -> HttpResponse {
    let event_entry = entry_json.into_inner();

    match Trip::push_event_entry(event_entry, &db).await {
        Ok(msg) => HttpResponse::Ok().body(msg),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn remove_event_entry(db: web::Data<MongoDb>,
                                entry_json:
                                web::Json<EventEntry>,
                                _:check_user::CheckLogin
) -> HttpResponse {
    let event_entry = entry_json.into_inner();

    match Trip::pull_event_entry(event_entry, &db).await {
        Ok(msg) => HttpResponse::Ok().body(msg),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn fork_trip(db: web::Data<MongoDb>,
                       entry_json: web::Json<TripFork>,
                       check: check_user::CheckLogin
) -> HttpResponse {
    let trip_fork = entry_json.into_inner();

    match Trip::fork(trip_fork, check.user_id, &db).await {
        Ok(oi) => HttpResponse::Created().json(oi),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn force_private(db: web::Data<MongoDb>,
                           trip_path: web::Path<String>,
                           check: check_user::CheckLogin
) -> HttpResponse {
    let trip_id = trip_path.into_inner();
    let admin_id = check.user_id;

    match Trip::force_private(trip_id, admin_id, &db).await {
        Ok(msg) => HttpResponse::Created().body(msg),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}