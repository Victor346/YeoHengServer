use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use argon2::{self, Config};
use jsonwebtoken::{encode, Header, Algorithm, EncodingKey};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct MyClaims {
    pub iss: String,
    pub sub: String,
    pub exp: i64,
}

pub fn salt_password(password: String) -> String {
    let pass = password.as_bytes();
    let salt = std::env::var("SALT_SECRET")
        .expect("Error retrieving salt");
    let salt = salt.as_bytes();

    let config  = Config::default();
    argon2::hash_encoded(pass, salt, &config).unwrap()
}

pub fn generate_jwt(user_id: ObjectId) -> String{
    let milisecond_in_day = 24 * 60 * 60 * 1000; 

    let claims = MyClaims {
        iss: "yeoheng-server.com".to_string(),
        sub: user_id.to_hex(),
        exp: Utc::now().timestamp() + milisecond_in_day,
    };

    let header = Header::new(Algorithm::HS512);
    let jwt_secret = std::env::var("JWT_SECRET").expect("Error retrieving jwt secret");
    encode(&header, &claims, &EncodingKey::from_secret(jwt_secret.as_ref())).unwrap()
}