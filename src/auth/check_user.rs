use crate::auth::authentication::MyClaims;

use actix_web::error::ErrorUnauthorized;
use actix_web::{dev, Error, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

pub struct CheckLogin;

impl FromRequest for CheckLogin {
    type Error = Error;
    type Future = Ready<Result<CheckLogin, Error>>;
    type Config = ();

    fn from_request(_req: &HttpRequest, _playload: &mut dev::Payload) -> Self::Future {
        let _auth = _req.headers().get("Authorization");
        match _auth {
            Some(_) => {
                let _split: Vec<&str> = _auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = _split[1].trim();
                let _var = std::env::var("JWT_SECRET")
                             .expect("Error retrieving jwt secret");
                let key = _var.as_ref();
                match decode::<MyClaims>(
                    token,
                    &DecodingKey::from_secret(key),
                    &Validation::new(Algorithm::HS512),
                ) {
                    Ok(_token) => ok(CheckLogin),
                    Err(_e) => err(ErrorUnauthorized("invalid token!")),
                }
            }
            None => err(ErrorUnauthorized("blocked!")),
        }
    }
}