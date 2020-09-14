use crate::models::user::User;

use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use argon2::{self, Config};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation,
    EncodingKey, DecodingKey};

#[derive(Debug, Serialize, Deserialize)]
struct MyClaims {
    iss: String,
    sub: String,
    exp: usize,
}

pub fn salt_password(password: String) -> String {
    let pass = password.as_bytes();
    let salt = std::env::var("SALT_SECRET")
        .expect("Error retrieving salt");
    let salt = salt.as_bytes();

    let config  = Config::default();
    argon2::hash_encoded(pass, salt, &config).unwrap()
}

pub fn generate_jwt(user: User) -> String{
    let claims = MyClaims {
        iss: "yeoheng-server.com".to_string(),
        sub: user._id.unwrap().to_hex(),
        exp: 4552,
    };
    "agfsf".to_string()
}