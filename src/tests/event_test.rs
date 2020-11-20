#[cfg(test)]
mod test {
    use super::*;
    use crate::MongoDb;
    use crate::models::event::{Event, EventFilter};

    use mongodb::{Client, options::ClientOptions};
    use mongodb::options::ResolverConfig;
    use bson::oid::ObjectId;

    fn type_of<T>(_: &T) -> &str { std::any::type_name::<T>() }

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
    async fn test_create_event() {
        let mongo_db = get_mongo_db().await;

        let event = Event {
            _id: Some(ObjectId::new()),
            name: String::from("Test"),
            description: String::from("Description"),
            tags: vec! [String::from("tag1"), String::from("tag2")],
            personal_type: String::from("Type"),
            rating: Some(5.0),
            country: String::from("Country"),
            city: String::from("City"),
            price: 100.0,
            duration: String::from("Duration"),
            location: Some(vec! [0.0, 0.0]),
            image: String::from("Image"),
            private: false,
            user_id: ObjectId::new(),
        };

        let response = Event::create(event, &mongo_db).await;

        assert_eq!(type_of(&ObjectId::new()), type_of(&response));
    }

    #[actix_rt::test]
    async fn test_get_event() {
        let mongo_db = get_mongo_db().await;

        let filter = EventFilter {
            offset: 0,
            limit: 5,
            tags: None,
            personal_type: None,
            rating: None,
            country: None,
            city: None,
            user_id: None,
            include_private: None,
        };

        let response = Event::get_filtered_events(filter, &mongo_db).await;

        assert_eq!("Test", response[0].name);
    }

    #[actix_rt::test]
    async fn test_count_event() {
        let mongo_db = get_mongo_db().await;

        let filter = EventFilter {
            offset: 0,
            limit: 1,
            tags: None,
            personal_type: None,
            rating: None,
            country: None,
            city: None,
            user_id: None,
            include_private: Some(false),
        };

        let response = Event::count_filtered_events(filter, &mongo_db)
            .await.expect("Error counting document");

        assert_eq!(1, response);
    }
}