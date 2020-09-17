use crate::models::user::{User, UserLogin};
use crate::auth::{authentication, check_user};
use crate::MongoClient;
use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct UserResponse{
    jwt: String,
    username: String,
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
            
            let response = UserResponse{
                jwt: jwt,
                username: username
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
            let salted_pass = authentication::salt_password(validated_user.password.clone());
            let username = validated_user.username.clone();
            validated_user.password = salted_pass;
            let user_id = User::insert(validated_user, &client).await;
            let jwt = authentication::generate_jwt(user_id);
            
            let response = UserResponse{
                jwt: jwt,
                username: username
            };

            HttpResponse::Created().json(response)
        },
        Err(e) => {
            println!("{}", e.clone());
            HttpResponse::BadRequest().body(e)
        },
    }
}

pub async fn protected(client: web::Data<MongoClient>, user_json: web::Json<User>, _: check_user::CheckLogin) -> HttpResponse {
    let user = user_json.into_inner();
    println!("{:?}", user);

    HttpResponse::Ok().json(user)
}