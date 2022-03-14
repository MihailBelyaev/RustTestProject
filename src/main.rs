use std::env;

use tracing::info;
use warp::Filter;
use rust_test_project::loginmanager::LoginManager;
use rust_test_project::mongodbprovider::{self, MongoConnectionParameters, MongoDBProvider};
use rust_test_project::routes::{
    delete_certain_user, get_certain_user, get_filter_fcn, get_history_fcn, get_users_fcn,
    insert_filter_fcn, login_filter_fcn, post_user_fcn, update_certain_user,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    info!("Program started");
    let mongo_address: String = mongodbprovider::get_db_address_from_env().unwrap();
    let db_name = env::var("MONGO_INITDB_ROOT_USERNAME").unwrap_or_else(|_| "".to_string());
    let db_pass = env::var("MONGO_INITDB_ROOT_PASSWORD").unwrap_or_else(|_| "".to_string());
    let db_provider = MongoDBProvider::new(MongoConnectionParameters {
        address: mongo_address,
        port: 27017,
        user_name: db_name,
        password: db_pass,
    })
    .await;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let login_manager = LoginManager::new(database_url);
    info!("Creating routes");
    let db_provider_clone = db_provider.clone();
    let insert_route = insert_filter_fcn(db_provider.clone(), login_manager.clone()).await;
    let get_route = get_filter_fcn(db_provider_clone.clone(), login_manager.clone()).await;
    let log_route = login_filter_fcn(login_manager.clone()).await;
    let users_get_route = get_users_fcn(login_manager.clone()).await;
    let users_insert_route = post_user_fcn(login_manager.clone()).await;
    let user_get_route = get_certain_user(login_manager.clone()).await;
    let user_update_route = update_certain_user(login_manager.clone()).await;
    let user_delete_route = delete_certain_user(login_manager.clone()).await;
    let get_history_route = get_history_fcn(login_manager.clone()).await;
    let data_path = warp::path("data");
    let data_path_routes = data_path
        .and(insert_route)
        .or(data_path.and(get_route))
        .or(log_route)
        .or(users_get_route)
        .or(users_insert_route)
        .or(user_get_route)
        .or(user_update_route)
        .or(user_delete_route)
        .or(get_history_route);
    info!("Starting server");
    warp::serve(data_path_routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
