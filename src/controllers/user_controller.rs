use crate::models::user::{User, UserLogin, ProvidedGoogleUser};
use crate::auth::check_user;
use crate::auth::{authentication};
use crate::MongoDb;

use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use ureq;

#[derive(Debug, Serialize, Deserialize)]
struct UserResponse{
    jwt: String,
    username: String,
    id: String,
    role: String,
}

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

pub async fn login(db: web::Data<MongoDb>, user_form: web::Form<UserLogin>) -> impl Responder {
    let user_login = user_form.into_inner();

    match User::find_user(user_login, &db).await {
        Ok(validated_user) => {
            let user_id = validated_user._id.clone().unwrap();
            let username = validated_user.username.clone();
            let jwt = authentication::generate_jwt(user_id);
            let user_id = validated_user._id.clone().unwrap().to_hex();
            let role = validated_user.role.clone().unwrap();
            
            let response = UserResponse{
                jwt,
                username,
                id: user_id,
                role,
            };

            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            println!("{}", e.clone());
            HttpResponse::BadRequest().body(e)
        }
    }
}

pub async fn register(db: web::Data<MongoDb>, user_json: web::Json<User>) -> impl Responder {
    let user = user_json.into_inner();

    match User::validate(user, &db).await {
        Ok(mut validated_user) => {
            println!("{:?}", validated_user);
            validated_user.role = Some("user".to_string());
            let salted_pass = authentication::salt_password(validated_user.password.clone());
            let username = validated_user.username.clone();
            let role = validated_user.role.clone().unwrap();
            validated_user.password = salted_pass;
            let user_id = User::insert(validated_user, &db).await;
            let id = user_id.clone().to_hex();
            let jwt = authentication::generate_jwt(user_id);
            let response = UserResponse{
                jwt,
                username,
                id,
                role,
            };

            HttpResponse::Created().json(response)
        },
        Err(e) => {
            println!("{}", e.clone());
            HttpResponse::BadRequest().body(e)
        },
    }
}

pub async fn get_all_like_user(db: web::Data<MongoDb>,
                               search_path: web::Path<String>,
                               check: check_user::CheckLogin
) -> HttpResponse {
    let search_str = search_path.into_inner();
    let admin_id = check.user_id;

    match User::get_all_like_user(search_str, admin_id, &db).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn promote(db: web::Data<MongoDb>,
                     user_path: web::Path<String>,
                     check: check_user::CheckLogin
) -> HttpResponse {
    let user_id = user_path.into_inner();
    let admin_id = check.user_id;

    match User::promote_user(user_id, admin_id, &db).await {
        Ok(msg) => HttpResponse::Created().body(msg),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn demote(db: web::Data<MongoDb>,
                     user_path: web::Path<String>,
                     check: check_user::CheckLogin
) -> HttpResponse {
    let user_id = user_path.into_inner();
    let admin_id = check.user_id;

    match User::demote_user(user_id, admin_id, &db).await {
        Ok(msg) => HttpResponse::Created().body(msg),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

pub async fn register_from_google(user_json: web::Json<ProvidedGoogleUser>, db: web::Data<MongoDb>)
    -> HttpResponse {
    // Validate that token_id is valid
    let new_user = user_json.into_inner();
    let resp = ureq::get(format!("https://oauth2.googleapis.com/tokeninfo?id_token={}", new_user.token_id).as_str())
        .call();
    if resp.ok() {
        match User::validate_google(new_user, &db).await {
            Ok(mut validated_user) => {
                println!("{:?}", validated_user);
                let role = "user".to_string();
                let username = validated_user.email.clone();
                let user_id = User::insert_google(validated_user, &db).await;
                let id = user_id.clone().to_hex();
                let jwt = authentication::generate_jwt(user_id);
                let response = UserResponse{
                    jwt,
                    username,
                    id,
                    role,
                };

                HttpResponse::Created().json(response)
            },
            Err(e) => {
                println!("{}", e.clone());
                HttpResponse::BadRequest().body(e)
            },
        }
    } else {
        HttpResponse::BadRequest().body("Invalid token")
    }
}

pub async fn login_from_google(user_json: web::Json<ProvidedGoogleUser>, db: web::Data<MongoDb>)
                               -> HttpResponse {
    let user_login = user_json.into_inner();

    let resp = ureq::get(format!("https://oauth2.googleapis.com/tokeninfo?id_token={}", user_login.token_id).as_str())
        .call();

    if resp.ok() {
        match User::find_google_user(user_login, &db).await {
            Ok(validated_user) => {
                let user_id = validated_user._id.clone().unwrap();
                let username = validated_user.username.clone();
                let jwt = authentication::generate_jwt(user_id);
                let user_id = validated_user._id.clone().unwrap().to_hex();
                let role = validated_user.role.clone().unwrap();

                let response = UserResponse {
                    jwt,
                    username,
                    id: user_id,
                    role,
                };

                HttpResponse::Ok().json(response)
            },
            Err(e) => {
                println!("{}", e.clone());
                HttpResponse::BadRequest().body(e)
            }
        }
    } else {
        HttpResponse::BadRequest().body("Invalid token")
    }
}
