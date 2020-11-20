use crate::MongoDb;
use crate::utils::custom_visitors::ObjectIdVisitor;

use serde::{de, Deserialize, Serialize};
use bson::oid::ObjectId;
use mongodb::bson::{doc, Bson, Document};
use mongodb::options::{
    FindOneOptions,
    FindOptions,
    InsertOneOptions,
    FindOneAndUpdateOptions,
    UpdateOptions,
    CountOptions,
    DeleteOptions,
    ReturnDocument
};
use std::borrow::Borrow;
use std::clone::Clone;
use futures::stream::StreamExt;
use std::error::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventEntry {
    _id: ObjectId,
    event_id: ObjectId,
    start_date: String,
    start_hour: String,
    budget: f32,
    duration: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trip {
    _id: ObjectId,
    pub name: String,
    start_date: String,
    end_date: String,
    budget: f32,
    destination: String,
    events: Vec<EventEntry>,
    private: bool,
    user_id: ObjectId
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TripFilter {
    pub offset: i64,
    pub limit: i64,
    pub budget_gt: Option<f32>,
    pub budget_lt: Option<f32>,
    pub user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTrip {
    #[serde(deserialize_with = "string_to_objectid")]
    id: ObjectId,
    events: Vec<EventEntry>
}

impl Trip {
    pub async fn get_trip(trip_id: String, db: &MongoDb) -> Result<Trip, String> {
        let trip_collection = db.collection("trips");

        match ObjectId::with_string(trip_id.as_str().as_ref()) {
            Ok(oi) => {
                match trip_collection.find_one(
                    doc! {"_id": oi},
                    FindOneOptions::default()
                ).await.expect("Error finding trip"){
                    Some(trip_found) => {
                        match bson::from_bson::<Trip>(bson::Bson::Document(trip_found)) {
                            Ok(trip) => Ok(trip),
                            Err(_e) => Err("Incorrect Struct".to_string()),
                        }
                    },
                    None => Err("Trip not found".to_string()),
                }
            },
            Err(_) => Err("Cannot convert given string to ObjectId".to_string()),
        }
    }

    pub async fn get_filtered_trips(trip_filter: TripFilter, db: &MongoDb) -> Vec<Trip> {
        let trip_collection = db.collection("trips");

        // Create a custom find option
        let find_options = FindOptions::builder()
            .limit(trip_filter.limit)
            .skip(trip_filter.offset)
            .build();

        // Get custom filter
        let filter = get_find_filter(trip_filter);

        let mut cursor = trip_collection.find(filter, find_options).await.expect("Error finding collection");
        let mut trips = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) =>
                    match bson::from_bson::<Trip>(bson::Bson::Document(document)) {
                        Ok(trip) => trips.push(trip),
                        Err(e) => println!("{:?}", e),
                    },
                Err(_) => println!("Error retrieving Document"),
            }
        }

        trips
    }

    pub async fn count_filtered_trips(trip_filter: TripFilter, db: &MongoDb) -> Result<i64, String> {
        let trip_collection = db.collection("trips");

        // Create a custom find option
        let count_options = CountOptions::builder()
            .limit(trip_filter.limit)
            .skip(trip_filter.offset)
            .build();

        // Get custom filter
        let filter = get_find_filter(trip_filter);

        match trip_collection.count_documents(filter, count_options).await {
            Ok(count) => Ok(count),
            Err(_) => Err("Error counting document".to_string()),
        }
    }

    pub async fn create(trip: TripCreate, db: &MongoDb) -> ObjectId {
        let trip_collection = db.collection("trips");

        (*trip_collection
            .insert_one(trip.to_doc(), InsertOneOptions::default())
            .await
            .expect("Error in insert user")
            .inserted_id
            .as_object_id()
            .unwrap())
            .clone()
    }

    pub async fn update(edit_info: TripEdit, db: &MongoDb) -> Result<Trip, String> {
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
        match edit_info.private {
            Some(b) => update_doc.insert("private", b),
            None => Some(Bson::default()),
        };

        let find_update_options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        match trip_collection.find_one_and_update(doc!{"_id": edit_info._id},
                                                  doc!{"$set": update_doc},
                                                        find_update_options
        ).await.expect("Error updating Trip") {
            Some(trip_updated) => {
                match bson::from_bson::<Trip>(bson::Bson::Document(trip_updated)) {
                    Ok(trip) => Ok(trip),
                    Err(_) => Err("Incorrect struct, expecting trip struct".to_string())
                }
            },
            None => Err("Trip not found".to_string()),
        }
    }

    pub async fn push_event_entry(event_entry: EventEntry, db: &MongoDb) -> Result<String, String> {
        let trip_collection = db.collection("trips");

        let update_query = doc! {
            "$push": {"events": event_entry.borrow().to_doc()},
            "$inc": {"budget": event_entry.budget}
        };

        match trip_collection.update_one(doc! {"_id": event_entry._id.clone()},
                                         update_query,
                                         UpdateOptions::default()
        ).await {
            Ok(_) => Ok("Event successfully added".to_string()),
            Err(_) => Err("Event not found".to_string())
        }
    }

    pub async fn pull_event_entry(event_entry: EventEntry, db: &MongoDb) -> Result<String, String> {
        let trip_collection = db.collection("trips");

        let update_query = doc! {
            "$push": {"events": event_entry.borrow().to_doc()},
            "$inc": {"budget": -event_entry.budget}
        };

        match trip_collection.update_one(doc! {"_id": event_entry._id.clone()},
                                         update_query,
                                         UpdateOptions::default()
        ).await {
            Ok(_) => Ok("Event successfully removed".to_string()),
            Err(_) => Err("Event not found".to_string())
        }
    }

    pub async fn delete_trip(trip_id: String, db: &MongoDb) -> Result<i64, String> {
        let trip_collection = db.collection("trips");

        match ObjectId::with_string(trip_id.as_str().as_ref()) {
            Ok(oi) => {
                match trip_collection.delete_one(doc! {"_id": oi}, DeleteOptions::default()).await {
                    Ok(result) => Ok(result.deleted_count),
                    Err(_) => Err("Trip not found".to_string()),
                }
            },
            Err(_) => Err("Cannot convert given string to ObjectId".to_string()),
        }
    }

    pub async fn fork(trip_fork: TripFork, user_id: String, db: &MongoDb) -> Result<ObjectId, String> {
        let trip_collection = db.collection("trips");

        match trip_collection.find_one(
            doc! {"_id": trip_fork.to_fork_trip_id},
            FindOneOptions::default()
        ).await.expect("Error finding trip") {
            Some(trip_found) => {
                match bson::from_bson::<Trip>(bson::Bson::Document(trip_found)) {
                    Ok(trip) => {
                        println!("{}", user_id);
                        let new_user_id = ObjectId::with_string(user_id.as_str().as_ref())
                            .expect("Cannot convert given string to ObjectId");

                        let new_trip = TripCreate {
                            name: trip_fork.name,
                            start_date: trip_fork.start_date,
                            end_date: trip_fork.end_date,
                            budget: trip.budget,
                            destination: trip.destination,
                            private: trip.private,
                            user_id: new_user_id.clone(),
                        };

                        let new_trip_id = (*trip_collection.insert_one(
                            new_trip.to_doc(),
                            InsertOneOptions::default(),
                        ).await
                           .expect("Error inserting Trip")
                           .inserted_id
                           .as_object_id()
                           .unwrap()
                        ).clone();

                        let new_events = trip.events.into_iter()
                                                                     .map(|event_entry| event_entry
                                                                         .to_doc_new_id(new_user_id.clone())
                                                                     )
                                                                     .collect::<Vec<Document>>();
                        match trip_collection.update_one(doc! {"_id": new_trip_id.clone()},
                                         doc! {"$set": {"events": new_events}},
                                         UpdateOptions::default()
                        ).await {
                            Ok(_) => Ok(new_trip_id),
                            Err(_e) => Err("Error updating Trip".to_string()),
                        }
                    },
                    Err(_e) => Err("Incorrect Struct".to_string()),
                }
            },
            None => Err("Trip not found".to_string()),
        }
    }

    pub async fn force_private(trip_id: String, admin_id: String, db: &MongoDb) -> Result<String, String> {
        let event_collection = db.collection("events");
        let user_collection = db.collection("users");
        let trip_oid = ObjectId::with_string(trip_id.as_str().as_ref())
            .expect("Cannot convert given string to ObjectId");
        let admin_oid = ObjectId::with_string(admin_id.as_str().as_ref())
            .expect("Cannot convert given string to ObjectId");

        match user_collection.find_one(
            doc!{"_id": admin_oid},
            FindOneOptions::default()
        ).await.expect("Error finding user") {
            Some(admin_found) => {
                let admin_role = admin_found.get_str("role").expect("Error getting admin role");

                match admin_role {
                    "superadmin" | "admin" => {
                        match event_collection.update_one(
                            doc!{"_id": trip_oid},
                            doc!{"$set": {"private": true}},
                            UpdateOptions::default()
                        ).await {
                            Ok(_) => Ok("Successfully changed trip to private".to_string()),
                            Err(_) => Err("Error changing trip to private".to_string())
                        }
                    },
                    _ => Err("Access Denied: user don't have sufficient privileges".to_string())
                }
            },
            None => Err("User not found".to_string())
        }
    }
}

impl EventEntry {
    pub fn to_doc(&self) -> Document {
        doc! {
            "_id": self._id.clone(),
            "event_id": self.event_id.clone(),
            "start_date": self.start_date.clone(),
            "start_hour": self.start_hour.clone(),
            "budget": self.budget.clone(),
            "duration": self.duration.clone(),
        }
    }

    pub fn to_doc_new_id(&self, id: ObjectId) -> Document {
        doc! {
            "_id": id.clone(),
            "event_id": self.event_id.clone(),
            "start_date": self.start_date.clone(),
            "start_hour": self.start_hour.clone(),
            "budget": self.budget.clone(),
            "duration": self.duration.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TripCreate {
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub budget: f32,
    pub destination: String,
    pub private: bool,
    #[serde(deserialize_with = "string_to_objectid")]
    pub user_id: ObjectId
}

impl TripCreate {
    pub fn to_doc(&self) -> Document {
        doc! {
            "name": self.name.clone(),
            "start_date": self.start_date.clone(),
            "end_date": self.end_date.clone(),
            "budget": self.budget,
            "events": [],
            "destination": self.destination.clone(),
            "private": self.private.clone(),
            "user_id": self.user_id.clone(),
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
    private: Option<bool>,
    destination: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TripFork {
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(deserialize_with = "string_to_objectid")]
    pub to_fork_trip_id: ObjectId,
}

fn get_find_filter(trip_filter: TripFilter) -> Document {
    let mut filter = doc! {};
    match trip_filter.user_id {
        Some(s) => {
            match ObjectId::with_string(s.as_str().as_ref()) {
                Ok(oi) => filter.insert("user_id", oi),
                Err(_) => filter.insert("private", false),
            }
        },
        None => filter.insert("private", false),
    };
    match trip_filter.budget_gt {
        Some(f) => filter.insert("budget", doc! {"$gte": f}),
        None => Some(Bson::default()),
    };
    match trip_filter.budget_lt {
        Some(f) => filter.insert("budget", doc! {"$lte": f}),
        None => Some(Bson::default()),
    };

    filter
}

// Deserialize the String and convert it to ObjectId
fn string_to_objectid<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
    where
        D: de::Deserializer<'de>,
{
    // Deserialize using a custom visitor
    deserializer.deserialize_any(ObjectIdVisitor)
}