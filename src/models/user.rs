use crate::{MongoClient, MongoDb};

use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use regex::Regex;
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, InsertOneOptions, UpdateOptions, FindOptions};
use mongodb::bson::Document;
use argon2::{self};
use futures::StreamExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub _id: Option<ObjectId>,
    name: String,
    pub username: String,
    pub password: String,
    pub role: Option<String>,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    _id: ObjectId,
    name: String,
    username: String,
    role: String,
    email: String
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
        let db = client.database(std::env::var("DATABASE_NAME")
                        .expect("Error retrieving database name")
                        .as_str());
        let user_collection = db.collection("users");
        (*user_collection
            .insert_one(user.to_doc().await, InsertOneOptions::default())
            .await
            .expect("Error inserting User")
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

    pub async fn promote_user(user_id: String, admin_id: String, db: &MongoDb) -> Result<String, String> {
        let user_collection = db.collection("users");
        let user_oid = ObjectId::with_string(user_id.as_str().as_ref())
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
                    "superadmin" => {
                        match user_collection.update_one(
                            doc!{"_id": user_oid, "role": "user"},
                            doc!{"$set": {"role": "admin"}},
                            UpdateOptions::default()
                        ).await {
                            Ok(_) => Ok("Successfully promoted user role".to_string()),
                            Err(_) => Err("Error promoting user role".to_string())
                        }
                    },
                    _ => Err("Access Denied: user don't have sufficient privileges".to_string())
                }
            },
            None => Err("User not found".to_string())
        }
    }

    pub async fn demote_user(user_id: String, admin_id: String, db: &MongoDb) -> Result<String, String> {
        let user_collection = db.collection("users");
        let user_oid = ObjectId::with_string(user_id.as_str().as_ref())
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
                    "superadmin" => {
                        match user_collection.update_one(
                            doc!{"_id": user_oid, "role": "admin"},
                            doc!{"$set": {"role": "user"}},
                            UpdateOptions::default()
                        ).await {
                            Ok(_) => Ok("Successfully demoted user role".to_string()),
                            Err(_) => Err("Error demoting user role".to_string())
                        }
                    },
                    _ => Err("Access Denied: user don't have sufficient privileges".to_string())
                }
            },
            None => Err("User not found".to_string())
        }
    }

    pub async fn get_all_like_user(search_str: String, admin_id: String, db: &MongoDb) -> Result<Vec<UserProfile>, String> {
        let user_collection = db.collection("users");
        let admin_oid = ObjectId::with_string(admin_id.as_str().as_ref())
            .expect("Cannot convert given string to ObjectId");

        match user_collection.find_one(
            doc!{"_id": admin_oid},
            FindOneOptions::default()
        ).await.expect("Error finding user") {
            Some(admin_found) => {
                let admin_role = admin_found.get_str("role").expect("Error getting admin role");

                match admin_role {
                    "superadmin" => {
                        match user_collection.find(doc!{"username": {"$regex": search_str, "$options": "i"}},
                                                   FindOptions::default()
                        ).await {
                            Ok(mut cursor) => {
                                let mut users = Vec::<UserProfile>::new();

                                while let Some(result) = cursor.next().await {
                                    match result {
                                        Ok(document) =>
                                            match bson::from_bson::<UserProfile>(bson::Bson::Document(document)) {
                                                Ok(user) => users.push(user),
                                                Err(e) => println!("Error retrieving User"),
                                            },
                                        Err(e) => println!("{:?}", e),
                                    };
                                }

                                Ok(users)
                            },
                            Err(e) => Err("Error getting users".to_string())
                        }
                    },
                    _ => Err("Access Denied: user don't have sufficient privileges".to_string())
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