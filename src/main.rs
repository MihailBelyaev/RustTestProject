use mongodbprovider::MongoDBProvider;
use mydatastruct::MyData;
use warp::Filter;
mod mongodbprovider;
mod mydatastruct;
#[tokio::main]
async fn main() {

    let dbProvider=MongoDBProvider::new(27017).await;
    let db_provider_clone=dbProvider.clone();
    let data_path = warp::path("data");
    let data_path_routes=data_path
            .and(warp::post())
            .and(warp::any().map(move||dbProvider.clone()))
            .and(warp::body::json())
            .and_then(MongoDBProvider::add_to_db)
       .or(data_path
            .and(warp::get())
            .and(warp::any().map(move||db_provider_clone.clone()))
            .and(warp::path::param())
            .and_then(MongoDBProvider::get_by_id));

    warp::serve(data_path_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}