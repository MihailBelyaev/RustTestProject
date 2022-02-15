use futures_util::TryFutureExt;
use warp::{Filter, Rejection, Reply};
use crate::mongodbprovider::{add_to_db, MongoDBProviderTrait};


pub fn server_routes(
    data_path: String,
    mongodb_provider: impl MongoDBProviderTrait + Send + Sync + Clone
) -> impl Filter<Extract = impl Reply, Error = Rejection> {
        get_data_route(data_path.clone(), mongodb_provider.clone())
        // .or(post_data_route(data_path.clone(), mongodb_provider.clone()))
}

pub fn get_data_route(
    data_path: String,
    mongodb_provider: impl MongoDBProviderTrait + Send + Sync + Clone
) -> impl Filter<Extract = impl Reply, Error = Rejection> {
    warp::path(data_path)
        .and(warp::get())
        .and(warp::any().map(move || mongodb_provider.clone()))
        .and(warp::body::json())
        .and_then(add_to_db)
}

// pub fn post_data_route(
//     data_path: String,
//     mongodb_provider: impl MongoDBProviderTrait + Send + Sync + Clone
// ) -> impl Filter<Extract = impl Reply, Error = Rejection> {
//     todo!()
// }

#[cfg(test)]
mod tests{
    use serde_json::json;
    use warp::http::StatusCode;
    use crate::mongodbprovider::MongoDBProviderTrait;
    use crate::mydatastruct::{create_my_struct, MyData, Sex};
    use crate::server::get_data_route;
    use async_trait::async_trait;

    #[derive(Default, Clone)]
    struct FakeMongoProvider{
    }

    #[async_trait]
    impl MongoDBProviderTrait for FakeMongoProvider {
        async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String> {
            Ok(())
        }

        async fn read_from(&self, id: String) -> Result<Vec<MyData>, String> {
            Ok(vec![create_my_struct("123".to_string(), "".to_string(), 0, Sex::Male)])
        }
    }

    #[tokio::test]
    async fn test_get_route() {
        let fake_mongo_provider = FakeMongoProvider::default();
        let route = get_data_route("data".to_string(), fake_mongo_provider);

        let value = warp::test::request()
            .method("GET")
            .path("data...") // your path
            .reply(&route)
            .await;

        assert_eq!(value.status(), StatusCode::OK);

        let body = value.into_body();

        let encoded = std::str::from_utf8(&body).unwrap();

        let expected = json!(
            {
                "_id": "123",
                "asdasd": "32323"
            }
        );

        assert_eq!(encoded, expected.as_str().unwrap());
    }
}