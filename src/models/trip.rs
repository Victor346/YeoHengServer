use crate::MongoDb;
use crate::utils::custom_visitors::ObjectIdVisitor;

use serde::{de, Deserialize, Serialize};
use bson::oid::ObjectId;
use mongodb::bson::{doc, Bson};
use mongodb::options::{FindOneOptions, FindOptions, InsertOneOptions, FindOneAndUpdateOptions, ReturnDocument};
use mongodb::bson::Document;

#[derive(Serialize, Deserialize, Debug)]
pub struct EventEntry {
    _id: ObjectId,
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

    pub async fn update(mut edit_info: TripEdit, db: &MongoDb) -> Result<Trip, String> {
        let trip_collection = db.collection("trips");
        let mut update_doc = doc!{};
        match edit_info.name {
            Some(s) => update_doc.insert("name", s),
            None => Some(Bson::default()),
        };
        match edit_info.start_date {
            Some(s) => update_doc.insert("start_date", s),
            None => Some(Bson::default()),
        };
        match edit_info.end_date {
            Some(s) => update_doc.insert("end_date", s),
            None => Some(Bson::default()),
        };
        match edit_info.budget {
            Some(f) => update_doc.insert("budget", f),
            None => Some(Bson::default()),
        };
        match edit_info.destination {
            Some(s) => update_doc.insert("destination", s),
            None => Some(Bson::default()),
        };

        let find_update_options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        match trip_collection.find_one_and_update(doc!{"_id": edit_info._id},
                                                  doc!{"$set": update_doc},
                                                        find_update_options)
            .await
            .expect("Error updating Trip") {
            Some(trip_updated) => {
                match bson::from_bson::<Trip>(bson::Bson::Document(trip_updated)) {
                    Ok(trip) => Ok(trip),
                    Err(_) => Err("Incorrect struct, expecting trip struct".to_string())
                }
            },
            None => Err("Trip not found".to_string()),
        }
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
            "events": [],
            "destination": self.destination.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TripEdit {
    #[serde(deserialize_with = "string_to_objectid")]
    _id: ObjectId,
    name: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    budget: Option<f32>,
    destination: Option<String>,
}

// Deserialize the String and convert it to ObjectId
fn string_to_objectid<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
    where
        D: de::Deserializer<'de>,
{
    // Deserialize using a custom visitor
    deserializer.deserialize_any(ObjectIdVisitor)
}