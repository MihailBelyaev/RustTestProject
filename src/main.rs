use mongodbprovider::{MongoDBProvider, MongoDBProviderTrait};

use warp::{Filter};
mod mongodbprovider;
mod mydatastruct;
#[tokio::main]
async fn main() {
    let db_provider = MongoDBProvider::new(27017).await;
    let db_provider_clone = db_provider.clone();
    let data_path = warp::path("data");
    let data_path_routes = data_path
        .and(warp::post())
        .and(warp::any().map(move || db_provider.clone()))
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
//async fn insert_filter_fcn(db_provider: impl MongoDBProviderTrait + Clone + Sync) {
    //let dbProvider = Arc::new(db_provider);
    //let data_path = warp::path("data");
 //   warp::post()
   //     .and(warp::any().map(move || db_provider.clone()))
     //   .and(warp::body::json())
       // .and_then(mongodbprovider::add_to_db)
//}

