

use mongodb::{Client, options::ClientOptions,Database, bson::doc};
use warp::{http};
use crate::mydatastruct::MyData;
use futures_util::stream::StreamExt;
pub struct MongoDBProvider{
    client: Client,
    database: Database
}
impl MongoDBProvider {
    pub async fn new(port :i32)->MongoDBProvider{
        let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let database=client.database("mydata");
        MongoDBProvider{client,database}
    }
    pub async fn add_to_db(&self,data:MyData)->Result<impl warp::Reply, warp::Rejection>{
        let collection = self.database.collection::<MyData>("dobro");
        collection.insert_one(data, None);
        Ok(warp::reply::with_status("Ok", http::StatusCode::CREATED))
    }
    pub async fn get_by_id(&self,id:String) ->Result<impl warp::Reply, warp::Rejection>{
        let collection=self.database.collection::<MyData>("dobro");
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