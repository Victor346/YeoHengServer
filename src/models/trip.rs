use crate::MongoDb;

use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, FindOptions, InsertOneOptions};
use mongodb::bson::Document;

#[derive(Serialize, Deserialize, Debug)]
pub struct EventEntry {
    event_id: ObjectId,
    start_date: String,
    start_hour: String,
    duration: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trip {
    _id: ObjectId,
    name: String,
    start_date: String,
    end_date: String,
    budget: f32,
    destination: String,
    events: Vec<EventEntry>,
}

impl Trip {
    pub async fn create(mut trip: TripCreate, db: &MongoDb) -> ObjectId {
        let trip_collection = db.collection("trips");
        let events: Vec<EventEntry> = Vec::new();
        (*trip_collection
            .insert_one(trip.to_doc().await, InsertOneOptions::default())
            .await
            .expect("Error in insert user")
            .inserted_id
            .as_object_id()
            .unwrap())
            .clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TripCreate {
    name: String,
    start_date: String,
    end_date: String,
    budget: f32,
    destination: String,
}

impl TripCreate {
    pub async fn to_doc(&self) -> Document {
        doc! {
            "name": self.name.clone(),
            "start_date": self.start_date.clone(),
            "end_date": self.end_date.clone(),
            "budget": self.budget,
            "destination": self.destination.clone(),
        }
    }
}
