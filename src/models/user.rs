use crate::MongoClient;

use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use regex::Regex;
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, InsertOneOptions, FindOptions};
use mongodb::bson::Document;
use argon2::{self, Config};
use futures::stream::StreamExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub _id: Option<ObjectId>,
    name: String,
    pub username: String,
    pub password: String,
    role: Option<String>,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLogin {
    email: String,
    password: String,
}

impl User {
    pub async fn validate(user_to_valdiate: User, client: &MongoClient) -> Result<User, String> {
        let db = client.database(std::env::var("DATABASE_NAME")
                                    .expect("Error retrieving database name")
                                        .as_str());
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

    pub async fn insert(user: User, client: &MongoClient) -> ObjectId {
        let db = client.database("yeohengDev");
        let user_collection = db.collection("users");
        (*user_collection
            .insert_one(user.to_doc().await, InsertOneOptions::default())
            .await
            .expect("Error in find user")
            .inserted_id
            .as_object_id()
            .unwrap())
            .clone()
    }

    pub async fn find_user(user_to_find: UserLogin, client: &MongoClient) -> Result<User, String> {
        let db = client.database(std::env::var("DATABASE_NAME")
                                    .expect("Error retrieving database name")
                                        .as_str());
        let user_collection = db.collection("users");
        let email = user_to_find.email.clone();
        let password = user_to_find.password.clone();

        let user_filter = doc!{"email": email};
        match user_collection.find_one(user_filter, FindOneOptions::default()).await.expect("Error in find user") {
            Some(user_found) => {
                match bson::from_bson::<User>(bson::Bson::Document(user_found)) {
                    Ok(user) => {
                        match argon2::verify_encoded(&user.password, password.as_bytes()).unwrap() {
                            true => Ok(user),
                            false => Err("Email or password mismatch".to_string())
                        }
                    },
                    Err(_e) => Err("Incorrect Struct".to_string()),
                }
            },
            None => Err("User not found".to_string())
        }
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