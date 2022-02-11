use mongodbprovider::{MongoDBProvider, MongoDBProviderTrait};
use mydatastruct::MyData;
use warp::Filter;
mod mongodbprovider;
mod mydatastruct;
#[tokio::main]
async fn main() {
    let dbProvider = MongoDBProvider::new(27017).await;
    let db_provider_clone = dbProvider.clone();
    let data_path = warp::path("data");
    let data_path_routes = data_path
        .and(warp::post())
        .and(warp::any().map(move || dbProvider.clone()))
        .and(warp::body::json())
        .and_then(mongodbprovider::add_to_db)
        .or(data_path
            .and(warp::get())
            .and(warp::any().map(move || db_provider_clone.clone()))
            .and(warp::path::param())
            .and_then(mongodbprovider::get_by_id));

    warp::serve(data_path_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
