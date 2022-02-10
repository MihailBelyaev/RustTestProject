

use mongodb::{Client, options::ClientOptions,Database, bson::doc};
use warp::{http, Rejection, Reply};
use crate::mydatastruct::MyData;
use futures_util::stream::StreamExt;
#[derive(Clone)]
pub struct MongoDBProvider{
    client: Client,
    database: Database
}
impl MongoDBProvider {
    pub async fn new(port :i32)->MongoDBProvider{
        let client_options = ClientOptions::parse(format!("mongodb://localhost:{}", port)).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let database=client.database("mydata");
        MongoDBProvider{client,database}
    }

    pub async fn insert_struct_to_db(&self, data: MyData) -> Result<(), std::Error> {
        todo!()
    }

    pub async fn read_from(&self, id: String) -> Result<MyData, std::Error> {
        todo!()
    }

    //todo: move to trait impl
    pub async fn add_to_db(db:MongoDBProvider,data:MyData)->Result<impl warp::Reply, warp::Rejection>{
        let collection = db.database.collection::<MyData>("dobro");
        collection.insert_one(data, None).await.unwrap();
        Ok(warp::reply::with_status("Ok", http::StatusCode::CREATED))
    }
    //todo: move to trait impl
    pub async fn get_by_id(db:MongoDBProvider,id:String) ->Result<impl warp::Reply, warp::Rejection>{
        let collection=db.database.collection::<MyData>("dobro");
        let search_result=collection.find(doc!{"id":id}, None).await;
        if search_result.is_ok() {
            let mut vec_res:Vec<MyData>=Vec::new();
            let mut cursor: mongodb::Cursor<MyData>=search_result.unwrap();
            while let Some(dt) = cursor.next().await{
                if dt.is_ok() {
                    vec_res.push(dt.unwrap());
                }
                else{
                    return Err(warp::reject::reject())
                }
            }
            return Ok(warp::reply::with_status(warp::reply::json(&vec_res), http::StatusCode::OK))

        }
        else {
            return Err(warp::reject::reject())
        }
    }
}

impl MongoDBProviderTrait for MongoDBProvider {
    async fn add_to_db(db: MongoDBProvider, data: MyData) -> Result<impl Reply, Rejection> {
        todo!()
    }

    async fn get_by_id(db: MongoDBProvider, id: String) -> Result<impl Reply, Rejection> {
        todo!()
    }
}

trait MongoDBProviderTrait {
    pub async fn add_to_db(db:MongoDBProvider,data:MyData)->Result<impl warp::Reply, warp::Rejection> {};
    pub async fn get_by_id(db:MongoDBProvider,id:String) ->Result<impl warp::Reply, warp::Rejection> {};
}

#[cfg(test)]
struct FakeMongoDbProvider{}

#[cfg(test)]
//TODO: make realization Trait for fakeMongo

#[cfg(test)]
mod tests {
    use testcontainers::{clients, Docker};

    //TODO: test mongo methods with testcontainers lib
    #[tokio::test]
    async fn mongo_add_and_read_test() {
        let docker = clients::Cli::default();
        let mongo_node = docker.run();

    }

    //TODO: test REST routes with FakeMongo

    }
