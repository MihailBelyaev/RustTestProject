use RustTestProject::mongodbprovider::{MongoDBProvider, MongoDBProviderTrait};
use RustTestProject::routes::{get_filter_fcn, insert_filter_fcn};

use tracing::info;
use warp::{Filter, Rejection, Reply};

#[tokio::main]
async fn main() {
    info!("Programm started");
    let db_provider = MongoDBProvider::new(27017).await;
    info!("Creating routes");
    let db_provider_clone = db_provider.clone();
    let insert_route = insert_filter_fcn(db_provider.clone()).await;
    let get_route = get_filter_fcn(db_provider_clone.clone()).await;
    let data_path = warp::path("data");
    let data_path_routes = data_path.and(insert_route).or(data_path.and(get_route));
    info!("Starting server");
    warp::serve(data_path_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
