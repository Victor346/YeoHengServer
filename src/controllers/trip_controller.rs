use crate::models::trip::{Trip, TripCreate};
use crate::auth::check_user;
use crate::MongoDb;
use actix_web::{web, HttpResponse};
use serde::{Serialize, Deserialize};

pub async fn create_trip(db: web::Data<MongoDb>,
                         trip_json: web::Json<TripCreate>,
                         _: check_user::CheckLogin
) -> HttpResponse {
    let trip = trip_json.into_inner();

    let trip_id = Trip::create(trip, &db).await;

    HttpResponse::Created().json(trip_id)
}