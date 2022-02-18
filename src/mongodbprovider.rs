use crate::mydatastruct::MyData;
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use mongodb::{bson::doc, options::ClientOptions, Client, Database};

use tracing::{info, log::warn};
use warp::http;
#[async_trait]
pub trait MongoDBProviderTrait: Send {
    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String>;
    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String>;
}

#[derive(Clone)]
pub struct MongoDBProvider {
    client: Client,
    database: Database,
}
impl MongoDBProvider {
    pub async fn new(port: i32) -> MongoDBProvider {
        let client_options = ClientOptions::parse(format!("mongodb://localhost:{}", port))
            .await
            .unwrap();
        //println!("Options:{:#?}",client_options);
        info!("Creating Mongo client with options: {:#?}",client_options);
        let client = Client::with_options(client_options).unwrap();
        let database = client.database("mydata");
        MongoDBProvider { client, database }
    }
}
#[async_trait]
impl MongoDBProviderTrait for MongoDBProvider {
    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String> {
        let collection = self.database.collection::<MyData>("dobro");
        info!("Inserting struct to DB: {:#?}",data);
        match collection.insert_one(data, None).await {
            Ok(result) => {
                info!("Successful insertion with id {}",result.inserted_id);
                return futures_util::__private::Ok(());
            }
            Err(err) => {
                warn!("Insertion failed due to {}",err);
                return futures_util::__private::Err(err.to_string())
            },
        }
    }
    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String> {
        let collection = self.database.collection::<MyData>("dobro");
        info!("Searching for id {}",id);
        let search_result = collection.find(doc! {"_id":id}, None).await;
        if search_result.is_ok() {
            let mut vec_res: Vec<MyData> = Vec::new();
            let mut cursor: mongodb::Cursor<MyData> = search_result.unwrap();
            while let Some(dt) = cursor.next().await {
                if dt.is_ok() {
                    vec_res.push(dt.unwrap());
                } else {
                    warn!("Internal search error!");
                    return Err("Internal Error".to_string());
                }
            }
            if vec_res.len() > 0 {
                info!("Got {} result",vec_res.len());
                return Ok(vec_res);
            } else {
                warn!("Not Found!");
                return Err("Not Found!".to_string());
            }
        } else {
            warn!("Internal search error!");
            return Err("Search Error".to_string());
        }
    }
}
pub async fn add_to_db(
    db: impl MongoDBProviderTrait + Clone + Sync,
    data: MyData,
) -> Result<impl warp::Reply, warp::Rejection> {
    match db.insert_struct_to_db(data).await {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&"Item successfully created".to_string()),
            http::StatusCode::CREATED,
        )),
        Err(err_str) => Ok(warp::reply::with_status(
            warp::reply::json(&err_str),
            warp::http::StatusCode::NOT_ACCEPTABLE,
        )),
    }
}

pub async fn get_by_id(
    db: impl MongoDBProviderTrait,
    id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    match db.read_from(id).await {
        Ok(res) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&res),
                http::StatusCode::FOUND,
            ))
        }
        Err(err_str) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&err_str),
                http::StatusCode::NOT_FOUND,
            ))
        }
    }
}
