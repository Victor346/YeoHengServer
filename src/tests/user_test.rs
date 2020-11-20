#[cfg(test)]
mod test {
    use super::*;
    use crate::models::user::{User};
    use crate::MongoDb;

    use mongodb::{Client, options::ClientOptions};
    use log::kv::Source;
    use mongodb::options::ResolverConfig;
    use bson::oid::ObjectId;

    fn type_of<T>(_: &T) -> &str {
        std::any::type_name::<T>()
    }

    async fn get_mongo_db() -> MongoDb {
        dotenv::dotenv().ok();

        let mut mongo_options = ClientOptions::parse_with_resolver_config(
            std::env::var("MONGO_URL").expect("Error in Mongo URL").as_str(),
            ResolverConfig::cloudflare()
        ).await.expect("Error found while creating client options");
        mongo_options.app_name = Some("YeoHengServer".to_string());
        let mongo_client = Client::with_options(mongo_options).expect("Error found while creating mongo client");
        mongo_client.database(std::env::var("TEST_DATABASE_NAME")
            .expect("Error retrieving database name")
            .as_str())
    }

    #[actix_rt::test]
    async fn test_insert_user() {
        let mongo_db = get_mongo_db().await;

        let user = User {
            _id: None,
            name: String::from("Test"),
            username: String::from("test"),
            password: String::from("test"),
            role: None,
            email: String::from("test@test.com"),
        };

        let response = User::insert(user, &mongo_db).await;

        assert_eq!(type_of(&ObjectId::new()), type_of(&response));
    }

    #[actix_rt::test]
    async fn test_get_all() {
        let mongo_db = get_mongo_db().await;

        let user = User {
            _id: None,
            name: String::from("TestGet"),
            username: String::from("test"),
            password: String::from("test"),
            role: None,
            email: String::from("test@test.com"),
        };

        let response = User::get_all_like_user("te".to_string(),
                                               "5fb7437c00058e520064685f".to_string(),
                                               &mongo_db).await.expect("Error: test get all usr");

        assert_eq!(true, response.iter().count() > 0);
    }
}