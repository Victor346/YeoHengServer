use crate::MongoClient;

use serde::de;
use serde::forward_to_deserialize_any;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, InsertOneOptions, FindOptions};
use mongodb::bson::Document;
use std::fmt;
use futures::stream::StreamExt;
use serde::ser::SerializeStruct;

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
    location: Option<Vec<f64>>,
    image: String,
    user_id: ObjectId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventCreate {
    _id: Option<ObjectId>,
    name: String,
    description: String,
    tags: Vec<String>,
    personal_type: String,
    rating: Option<f32>,
    country: String,
    city: String,
    location: Option<Vec<f64>>,
    image: String,
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
        let db = client.database("yeohengDev");
        let event_collection = db.collection("events");
        let find_options = FindOptions::builder().limit(event_filter.limit).skip(event_filter.offset).build();

        let filter = match event_filter.user_id {
            Some(s) => {
                match ObjectId::with_string(s.as_str().as_ref()) {
                    Ok(oi) => doc! { "user_id": oi },
                    Err(e) => doc! {},
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
                Err(e) => println!("Error retriving Document"),
            }
        }

        events
    }

    pub async fn create(event: EventCreate, client: &MongoClient) -> ObjectId {
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
}

impl EventCreate {
    pub async fn to_doc(&self) -> Document {
        doc! {
            "name": self.name.clone(),
            "description": self.description.clone(),
            "tags": self.tags.clone(),
            "personal_type": self.personal_type.clone(),
            "rating": self.rating.unwrap_or_else(|| 5.0).clone(),
            "country": self.country.clone(),
            "city": self.city.clone(),
            "location": self.location.as_ref().unwrap().clone(),
            "image": self.image.clone(),
            "user_id": self.user_id.clone(),
        }
    }
}

// Deserializa el String y la convierte en ObjectId
fn string_to_objectid<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct ObjectIdVisitor;

    impl<'de> de::Visitor<'de> for ObjectIdVisitor {
        type Value = ObjectId;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing object id")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            match ObjectId::with_string(v) {
                Ok(oi) => Ok(oi),
                Err(e) => Err(E::custom("Not a ObjectId format")),
            }
        }
    }

    deserializer.deserialize_any(ObjectIdVisitor)
}