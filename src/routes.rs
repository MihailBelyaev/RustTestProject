use warp::{Filter, Rejection, Reply};

use crate::{
    loginmanager::{self, LogMngTrait},
    mongodbprovider::{self, MongoDBProviderTrait},
};

pub async fn insert_filter_fcn(
    db_provider: impl MongoDBProviderTrait + Clone + Sync,
    mngr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path::end())
        .and(warp::any().map(move || db_provider.clone()))
        .and(warp::any().map(move || mngr.clone()))
        .and(warp::body::json())
        .and(warp::header::<String>("autorization"))
        .and_then(mongodbprovider::add_to_db)
}

pub async fn get_filter_fcn(
    db_provider: impl MongoDBProviderTrait + Clone + Sync,
    mngr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::get()
        .and(warp::any().map(move || db_provider.clone()))
        .and(warp::any().map(move || mngr.clone()))
        .and(warp::path::param())
        .and(warp::header::<String>("autorization"))
        .and(warp::path::end())
        .and_then(mongodbprovider::get_by_id)
}

pub async fn login_filter_fcn(
    login_mgr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("login")
        .and(warp::path::end())
        .and(warp::any().map(move || login_mgr.clone()))
        .and(warp::header::<String>("login"))
        .and(warp::header::<String>("password"))
        .and_then(loginmanager::check_login_data)
}

pub async fn get_users_fcn(
    mngr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("users")
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::any().map(move || mngr.clone()))
        .and(warp::header::<String>("autorization"))
        .and_then(loginmanager::get_users_list)
}

pub async fn post_user_fcn(
    mngr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("users")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::any().map(move || mngr.clone()))
        .and(warp::body::json())
        .and(warp::header::<String>("autorization"))
        .and_then(loginmanager::insert_user)
}

pub async fn get_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("users")
        .and(warp::get())
        .and(warp::any().map(move || mngr.clone()))
        .and(warp::path::param())
        .and(warp::header::<String>("autorization"))
        .and(warp::path::end())
        .and_then(loginmanager::get_certain_user)
}

pub async fn update_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("users")
        .and(warp::put())
        .and(warp::any().map(move || mngr.clone()))
        .and(warp::path::param())
        .and(warp::body::json())
        .and(warp::header::<String>("autorization"))
        .and(warp::path::end())
        .and_then(loginmanager::update_certain_user)
}

pub async fn delete_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("users")
        .and(warp::delete())
        .and(warp::any().map(move || mngr.clone()))
        .and(warp::path::param())
        .and(warp::header::<String>("autorization"))
        .and(warp::path::end())
        .and_then(loginmanager::delete_certain_user)
}

pub async fn get_history_fcn(
    mngr: impl LogMngTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("history")
        .and(warp::get())
        .and(warp::any().map(move || mngr.clone()))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(loginmanager::get_history_for_user)
}
