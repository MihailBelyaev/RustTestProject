

use mongodb::{Client, options::ClientOptions,Database, bson::{doc, Document}};
use warp::{http};
use crate::mydatastruct::MyData;
use futures_util::stream::StreamExt;
use async_trait::async_trait;
#[async_trait]
pub trait MongoDBProviderTrait {
    async fn new(port:i32)->Self;
    async fn insert_struct_to_db(&self,data:MyData)->Result<(),String>;
    async fn read_from(&self,id:String)-> Result<Vec<MyData>, String>;
}

#[derive(Clone)]
pub struct MongoDBProvider{
    client: Client,
    database: Database
}
#[async_trait]
impl MongoDBProviderTrait for MongoDBProvider {
     async fn new(port :i32)->MongoDBProvider{
        let client_options = ClientOptions::parse(format!("mongodb://localhost:{}", port)).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let database=client.database("mydata");
        MongoDBProvider{client,database}
    }

    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String> {
        let collection = self.database.collection::<MyData>("dobro");
        match collection.insert_one(data, None).await {
            Ok(result) => {
                return futures_util::__private::Ok(());}
            Err (err)=>{
                return futures_util::__private::Err(err.to_string())
            }

    }
    }
    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String> {
        let collection=self.database.collection::<MyData>("dobro");
        let search_result=collection.find(doc!{"_id":id}, None).await;
        if search_result.is_ok() {
            let mut vec_res:Vec<MyData>=Vec::new();
            let mut cursor: mongodb::Cursor<MyData>=search_result.unwrap();
            while let Some(dt) = cursor.next().await{
                if dt.is_ok() {
                    vec_res.push(dt.unwrap());
                }
                else{
                    return Err("Internal Error".to_string())
                }
            }
            return Ok(vec_res)
        }
        else {
            return Err("Search Error".to_string())
        }
    }

}
  pub  async fn add_to_db(db: impl MongoDBProviderTrait, data: MyData) -> Result<impl warp::Reply, warp::Rejection> {
        match db.insert_struct_to_db(data).await{
            Ok(_)=>{
                Ok(warp::reply::with_status(warp::reply::json(&"Item successfully created".to_string()), http::StatusCode::CREATED))
            },
            Err(err_str )=>{
                Ok(warp::reply::with_status(warp::reply::json(&err_str),warp::http::StatusCode::NOT_ACCEPTABLE))
            }
        }
    }

  pub  async fn get_by_id(db: impl MongoDBProviderTrait, id: String) -> Result<impl warp::Reply, warp::Rejection> {
        match db.read_from(id).await{
            Ok(res)=>{
                return Ok(warp::reply::with_status(warp::reply::json(&res), http::StatusCode::FOUND))
            },
            Err(err_str)=>{
                return Ok(warp::reply::with_status(warp::reply::json(&err_str), http::StatusCode::NOT_FOUND))
            }
        }
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
       // let mongo_node = docker.run();

    }

    //TODO: test REST routes with FakeMongo

    }
