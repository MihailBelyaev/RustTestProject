use std::env;

use tracing::info;
use warp::{Filter, Rejection, Reply};
use RustTestProject::loginmanager::LoginManager;
use RustTestProject::mongodbprovider::{MongoDBProvider, MongoDBProviderTrait, self};
use RustTestProject::routes::{get_filter_fcn, insert_filter_fcn, login_filter_fcn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    info!("Program started");
    let mongo_address:String=mongodbprovider::get_db_address();
    let db_provider = MongoDBProvider::new(mongo_address,27017).await;
    let login_manager = LoginManager::new();
    info!("Creating routes");
    let db_provider_clone = db_provider.clone();
    let insert_route = insert_filter_fcn(db_provider.clone()).await;
    let get_route = get_filter_fcn(db_provider_clone.clone()).await;
    let log_route = login_filter_fcn(login_manager.clone()).await;
    let data_path = warp::path("data");
    let data_path_routes = data_path
        .and(insert_route)
        .or(data_path.and(get_route))
        .or(log_route);
    info!("Starting server");
    warp::serve(data_path_routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}