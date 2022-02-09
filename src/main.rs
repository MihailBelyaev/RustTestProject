use mongodbprovider::MongoDBProvider;
use mydatastruct::MyData;
use warp::Filter;
mod mongodbprovider;
mod mydatastruct;
#[tokio::main]
async fn main() {

    let dbProvider=MongoDBProvider::new(27017).await;
    let data_path = warp::path("data");
    let data_path_routes=data_path
            .and(warp::post())
            .and(warp::body::json())
            .map(|my_data:MyData|{dbProvider.add_to_db(my_data)})
        .or(data_path
            .and(warp::get())
            .and(warp::path::param())
            .map(|id:String|{dbProvider.get_by_id(id)}));

    warp::serve(data_path_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}