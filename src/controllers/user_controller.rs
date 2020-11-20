use crate::models::user::{User, UserLogin};
use crate::auth::check_user;
use crate::auth::{authentication};
use crate::MongoClient;
use crate::MongoDb;

use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};

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

pub async fn login(client: web::Data<MongoClient>, user_form: web::Form<UserLogin>) -> impl Responder {
    let user_login = user_form.into_inner();

    match User::find_user(user_login, &client).await {
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

pub async fn register(client: web::Data<MongoClient>, user_json: web::Json<User>) -> impl Responder {
    let user = user_json.into_inner();

    match User::validate(user, &client).await {
        Ok(mut validated_user) => {
            println!("{:?}", validated_user);
            validated_user.role = Some("user".to_string());
            let salted_pass = authentication::salt_password(validated_user.password.clone());
            let username = validated_user.username.clone();
            let role = validated_user.role.clone().unwrap();
            validated_user.password = salted_pass;
            let user_id = User::insert(validated_user, &client).await;
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