use std::env::{self, VarError};

use crate::{loginmanager::LogMngTrait, mydatastruct::MyData};
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use mongodb::{bson::doc, options::ClientOptions, Client, Database};

use tracing::{debug, info, log::warn};

use warp::{
    http,
    reply::{self, Json},
};

#[async_trait]
pub trait MongoDBProviderTrait: Send {
    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String>;
    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String>;
}
#[derive(Clone)]
pub struct MongoConnectionParameters {
    pub address: String,
    pub port: i32,
    pub user_name: String,
    pub password: String,
}

impl MongoConnectionParameters {
    pub fn to_string(&self) -> String {
        if self.user_name.is_empty() {
            format!("mongodb://{}:{}", self.address, self.port)
        } else {
            format!(
                "mongodb://{}:{}@{}:{}",
                self.user_name, self.password, self.address, self.port
            )
        }
    }
}

#[derive(Clone)]
pub struct MongoDBProvider {
    client: Client,
    database: Database,
}
impl MongoDBProvider {
    pub async fn new(params: MongoConnectionParameters) -> MongoDBProvider {
        //let client_options = ClientOptions::parse(format!("mongodb://root:example@my-mongo:27017"))
        //  .await
        //.unwrap();
        let client_options = ClientOptions::parse(params.to_string()).await.unwrap();
        info!("Creating Mongo client with options: {:#?}", client_options);
        let client = Client::with_options(client_options).unwrap();
        let database = client.database("mydata");
        MongoDBProvider { client, database }
    }
}
#[async_trait]
impl MongoDBProviderTrait for MongoDBProvider {
    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String> {
        let collection = self.database.collection::<MyData>("dobro");
        info!("Inserting struct to DB: {:#?}", data);
        match collection.insert_one(data, None).await {
            Ok(result) => {
                info!("Successful insertion with id {}", result.inserted_id);
                return futures_util::__private::Ok(());
            }
            Err(err) => {
                warn!("Insertion failed due to {}", err);
                return futures_util::__private::Err(err.to_string());
            }
        }
    }
    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String> {
        let collection = self.database.collection::<MyData>("dobro");
        info!("Searching for id {}", id);
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
                info!("Got {} result", vec_res.len());
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
    mngr: impl LogMngTrait + Clone + Sync,
    data: MyData,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("insert route");
    if !mngr.check_token(token, format!("Add data to DB {:?}", data.clone())) {
        return Ok(create_forb_rep());
    }
    match db.insert_struct_to_db(data).await {
        Ok(_) => Ok(reply::with_status(
            reply::json(&"Item successfully created".to_string()),
            http::StatusCode::CREATED,
        )),
        Err(err_str) => Ok(reply::with_status(
            reply::json(&err_str),
            warp::http::StatusCode::NOT_ACCEPTABLE,
        )),
    }
}

pub async fn get_by_id(
    db: impl MongoDBProviderTrait,
    mngr: impl LogMngTrait + Clone + Sync,
    id: String,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("get route");
    if !mngr.check_token(token, format!("Get data from DB by id {}", id.clone())) {
        return Ok(create_forb_rep());
    }
    match db.read_from(id).await {
        Ok(res) => {
            return Ok(reply::with_status(
                reply::json(&res),
                http::StatusCode::FOUND,
            ))
        }
        Err(err_str) => {
            return Ok(reply::with_status(
                reply::json(&err_str),
                http::StatusCode::NOT_FOUND,
            ))
        }
    }
}
pub fn get_db_address_from_env() -> Result<String, VarError> {
    match env::var("TEST_MONGO_ADDRESS") {
        Ok(val) => {
            info!("Got Mongo address from enviroment{}", val.clone());
            Ok(val)
        }
        Err(e) => {
            info!("Error:{}", e);
            Err(e)
        }
    }
}
fn create_forb_rep() -> reply::WithStatus<Json> {
    reply::with_status(
        reply::json(&"Wrong token".to_string()),
        http::StatusCode::FORBIDDEN,
    )
}
