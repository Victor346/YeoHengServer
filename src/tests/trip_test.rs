use crate::models::trip::{TripCreate, TripFilter, Trip};

use mongodb::{Client, options::ClientOptions};
use mongodb::options::ResolverConfig;
use bson::oid::ObjectId;
use crate::MongoDb;

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::MongoDb;

    #[actix_rt::test]
    async fn test_create_trip() {
        let mongo_db = get_mongo_db().await;

        let event = TripCreate {
            name: String::from("Test"),
            start_date: String::from("Start"),
            end_date: String::from("End"),
            budget: 150.0,
            destination: String::from("Destination"),
            private: false,
            user_id: ObjectId::new(),
        };

        let response = Trip::create(event, &mongo_db).await;

        assert_eq!(type_of(&ObjectId::new()), type_of(&response));
    }

    #[actix_rt::test]
    async fn test_get_trip() {
        let mongo_db = get_mongo_db().await;

        let filter = TripFilter {
            offset: 0,
            limit: 5,
            budget_gt: None,
            budget_lt: None,
            user_id: None,
        };

        let response = Trip::get_filtered_trips(filter, &mongo_db).await;

        assert_eq!("Test", response[0].name);
    }

    #[actix_rt::test]
    async fn test_count_trip() {
        let mongo_db = get_mongo_db().await;

        let filter = TripFilter {
            offset: 0,
            limit: 1,
            budget_gt: None,
            budget_lt: None,
            user_id: None,
        };

        let response = Trip::count_filtered_trips(filter, &mongo_db)
            .await.expect("Error counting document");

        assert_eq!(1, response);
    }
}