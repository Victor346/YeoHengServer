use crate::MongoClient;

use serde::de;
use serde::{Deserialize, Serialize, Deserializer};
use bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, InsertOneOptions};
use mongodb::bson::Document;
use std::fmt;
use log::kv::Visitor;

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
    #[serde(deserialize_with = "string_to_objectid")]
    user_id: ObjectId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventFilter {
    offset: i16,
    limit: i8,
    user_id: Option<String>,
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