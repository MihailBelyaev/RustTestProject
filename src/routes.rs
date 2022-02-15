use warp::{Filter, Rejection, Reply};

use crate::mongodbprovider::{self, MongoDBProviderTrait};

pub async fn insert_filter_fcn(
    db_provider: impl MongoDBProviderTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::any().map(move || db_provider.clone()))
        .and(warp::body::json())
        .and_then(mongodbprovider::add_to_db)
}

pub async fn get_filter_fcn(
    db_provider: impl MongoDBProviderTrait + Clone + Sync,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::get()
        .and(warp::any().map(move || db_provider.clone()))
        .and(warp::path::param())
        .and_then(mongodbprovider::get_by_id)
}
