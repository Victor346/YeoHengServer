use crate::MongoClient;
use crate::utils::custom_visitors::ObjectIdVisitor;

use serde::{de, Deserialize, Serialize};
use bson::oid::ObjectId;
use mongodb::bson::{Bson, doc, Document};
use mongodb::options::{InsertOneOptions, FindOptions, FindOneAndUpdateOptions, ReturnDocument};
use futures::stream::StreamExt;

// Event struct to Retrieve and Create
#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    _id: Option<ObjectId>,
    name: String,
    description: String,
    tags: Vec<String>,
    personal_type: String,
    rating: Option<f32>,
    country: String,
    city: String,
    price: f32,
    duration: String,
    location: Option<Vec<f64>>,
    image: String,
    #[serde(deserialize_with = "string_to_objectid")]
    user_id: ObjectId,
}

// Event struct to Update and Delete
#[derive(Serialize, Deserialize, Debug)]
pub struct EventUpdate {
    #[serde(deserialize_with = "string_to_objectid")]
    _id: ObjectId,
    name: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    personal_type: Option<String>,
    rating: Option<f32>,
    country: Option<String>,
    city: Option<String>,
    price: Option<f32>,
    duration: Option<String>,
    location: Option<Vec<f64>>,
    image: Option<String>,
    #[serde(deserialize_with = "string_to_objectid")]
    user_id: ObjectId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventFilter {
    offset: i64,
    limit: i64,
    user_id: Option<String>,
}

impl Event {
    pub async fn get_all(event_filter: EventFilter, client: &MongoClient) -> Vec<Event>{
        let db = client.database(std::env::var("DATABASE_NAME")
                        .expect("Error retrieving database name")
                        .as_str());
        let event_collection = db.collection("events");

        // Create a custom find option
        let find_options = FindOptions::builder()
                            .limit(event_filter.limit)
                            .skip(event_filter.offset)
                            .build();

        // Check if the user is logged to filter his events
        let filter = match event_filter.user_id {
            Some(s) => {
                match ObjectId::with_string(s.as_str().as_ref()) {
                    Ok(oi) => doc! { "user_id": oi },
                    Err(_) => doc! {},
                }
            },
            None => doc! {},
        };

        let mut cursor = event_collection.find(filter, find_options).await.expect("Error finding collection");
        let mut events = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) =>
                    match bson::from_bson::<Event>(bson::Bson::Document(document)) {
                        Ok(event) => events.push(event),
                        Err(e) => println!("{:?}", e),
                    },
                Err(_) => println!("Error retrieving Document"),
            }
        }

        events
    }

    pub async fn create(mut event: Event, client: &MongoClient) -> ObjectId {
        let db = client.database(std::env::var("DATABASE_NAME")
            .expect("Error retrieving database name")
            .as_str());
        let event_collection = db.collection("events");

        // If the Event location is empty create a default one
        match event.location.clone() {
            None => event.location = Some(vec![0.0, 0.0]),
            Some(_) => (),
        }

        (*event_collection
            .insert_one(event.to_doc().await, InsertOneOptions::default())
            .await
            .expect("Error inserting Event")
            .inserted_id
            .as_object_id()
            .unwrap()
        ).clone()
    }

    pub async fn to_doc(&self) -> Document {
        doc! {
            "name": self.name.clone(),
            "description": self.description.clone(),
            "tags": self.tags.clone(),
            "personal_type": self.personal_type.clone(),
            "rating": self.rating.unwrap_or_else(|| 5.0).clone(),
            "country": self.country.clone(),
            "city": self.city.clone(),
            "price": self.price.clone(),
            "duration": self.duration.clone(),
            "location": self.location.as_ref().unwrap().clone(),
            "image": self.image.clone(),
            "user_id": self.user_id.clone(),
        }
    }
}

impl EventUpdate {
        pub async fn update(event: EventUpdate, client: &MongoClient) -> Result<Event, String> {
        let db = client.database(std::env::var("DATABASE_NAME")
            .expect("Error retrieving database name")
            .as_str());
        let event_collection = db.collection("events");

        // Check which field is being updated
        let mut update = doc! {};
        match event.name {
            Some(s) => update.insert("name", s),
            None => Some(Bson::default())
        };
        match event.description {
            Some(s) => update.insert("description", s),
            None => Some(Bson::default())
        };
        match event.tags {
            Some(v) => update.insert("tags", v),
            None => Some(Bson::default())
        };
        match event.personal_type {
            Some(s) => update.insert("personal_type", s),
            None => Some(Bson::default())
        };
        match event.rating {
            Some(f) => update.insert("rating", f),
            None => Some(Bson::default())
        };
        match event.country {
            Some(s) => update.insert("country", s),
            None => Some(Bson::default())
        };
        match event.city {
            Some(s) => update.insert("city", s),
            None => Some(Bson::default())
        };
        match event.price {
            Some(f) => update.insert("price", f),
            None => Some(Bson::default())
        };
        match event.duration {
            Some(s) => update.insert("duration", s),
            None => Some(Bson::default())
        };
        match event.location {
            Some(v) => update.insert("location", v),
            None => Some(Bson::default())
        };
        match event.image {
            Some(s) => update.insert("image", s),
            None => Some(Bson::default())
        };

        let find_update_options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        match event_collection.find_one_and_update(doc! {"_id": event._id}, doc! {"$set": update},
                                                   find_update_options)
            .await.expect("Error updating Event") {

            Some(event_updated) => {
                match bson::from_bson::<Event>(bson::Bson::Document(event_updated)) {
                    Ok(event) => Ok(event),
                    Err(_e) => Err("Incorrect struct, expecting event struct".to_string()),
                }
            },
            None => Err("Event not found".to_string())
        }
    }
}

// Deserialize the String and convert it to ObjectId
fn string_to_objectid<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
    where
        D: de::Deserializer<'de>,
{
    // Deserialize using a custom visitor
    deserializer.deserialize_any(ObjectIdVisitor)
}