use crate::MongoClient;

use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, InsertOneOptions};
use mongodb::bson::Document;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    _id: Option<ObjectId>,
    name: String,
    description: String,
    tags: Vec<String>,
    personal_type: String,
    rating: f32,
    country: String,
    city: String,
    location: Vec<f64>,
    image: String,
}

impl Event {
    pub async fn get_all() {

    }

    pub async fn get_one(){

    }

    pub async fn create(event: Event, client: &MongoClient) -> ObjectId {
        let db = client.database("yeohengDev");
        let event_collection = db.collection("events");
        (*event_collection
            .insert_one(event.to_doc().await, InsertOneOptions::default())
            .await
            .expect("Error in find user")
            .inserted_id
            .as_object_id()
            .unwrap())
            .clone()
    }

    pub async fn delete(){

    }

    pub async fn update(){

    }

    pub async fn to_doc(&self) -> Document {
        doc! {
            "name": self.name.clone(),
            "description": self.description.clone(),
            "tags": self.tags.clone(),
            "personal_type": self.personal_type.clone(),
            "rating": self.rating.clone(),
            "country": self.country.clone(),
            "city": self.city.clone(),
            "location": self.location.clone(),
            "image": self.image.clone(),
        }
    }
}