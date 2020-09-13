use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use regex::Regex;
use mongodb::bson::doc;
use crate::MongoClient;
use mongodb::options::{FindOneOptions, InsertOneOptions};
use mongodb::bson::Document;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: Option<ObjectId>,
    name: String,
    username: String,
    pub password: String,
    role: Option<String>,
    email: String,
}

impl User {
    pub async fn validate(user_to_valdiate: User, client: &MongoClient) -> Result<User, String> {
        let db = client.database("yeohengDev");
        let user_collection = db.collection("users");
        let mail = user_to_valdiate.email.clone();
        let username = user_to_valdiate.username.clone();

        let re = Regex::new(r".+@[a-zA-Z0-9]+\.([a-zA-Z]{2,3}|[0-9]{1,3})").unwrap();
        if !re.is_match(mail.as_str()) {
            return Err("Invalid email".to_string())
        }

        let user_filter = doc!{"$or": [ {"email": mail}, {"username": username}]};
        match user_collection.find_one(user_filter, FindOneOptions::default()).await.expect("Error in find user") {
            Some(_) => Err("User is already registered".to_string()),
            None => {

                Ok(user_to_valdiate)
            }
        }
    }

    pub async fn insert(user: User, client: &MongoClient) {
        let db = client.database("yeohengDev");
        let user_collection = db.collection("users");
        user_collection
            .insert_one(user.to_doc().await, InsertOneOptions::default())
            .await
            .expect("Error in find user");
    }

    pub async fn to_doc(&self) -> Document {
        doc! {
            "name": self.name.clone(),
            "username": self.username.clone(),
            "password": self.password.clone(),
            "role": "user",
            "email": self.email.clone()
        }
    }
}

