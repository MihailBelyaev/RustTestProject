use crate::mydatastruct::MyData;
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Database,
};
use testcontainers::{clients, images, Docker, RunArgs, core::Port};
use warp::http;
#[async_trait]
pub trait MongoDBProviderTrait: Send + 'static {
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
        let client_options = ClientOptions::parse(format!("mongodb://0.0.0.0:{}", port))
            .await
            .unwrap();
        let client = Client::with_options(client_options).unwrap();
        let database = client.database("mydata");
        MongoDBProvider { client, database }
    }
}
#[async_trait]
impl MongoDBProviderTrait for MongoDBProvider {
    

    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String> {
        let collection = self.database.collection::<MyData>("dobro");
        match collection.insert_one(data, None).await {
            Ok(result) => {
                return futures_util::__private::Ok(());
            }
            Err(err) => return futures_util::__private::Err(err.to_string()),
        }
    }
    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String> {
        let collection = self.database.collection::<MyData>("dobro");
        let search_result = collection.find(doc! {"_id":id}, None).await;
        if search_result.is_ok() {
            let mut vec_res: Vec<MyData> = Vec::new();
            let mut cursor: mongodb::Cursor<MyData> = search_result.unwrap();
            while let Some(dt) = cursor.next().await {
                if dt.is_ok() {
                    vec_res.push(dt.unwrap());
                } else {
                    return Err("Internal Error".to_string());
                }
            }
            return Ok(vec_res);
        } else {
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
        ))
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

#[cfg(test)]
struct FakeMongoDbProvider {
    provider: MongoDBProvider,
}
#[cfg(test)]
impl FakeMongoDbProvider{
    pub async fn new(port: i32) -> FakeMongoDbProvider {
        let docker = clients::Cli::default();
        //let run_arg:RunArgs;
        //let run_arg=run_arg.with_name("mongo_test");
        //run_arg.with_network("HostNetwork");
        //run_arg.with_mapped_port(Port{local:27017,internal:27017});
        let node = docker.run(images::mongo::Mongo::default());
        let host_port = node.get_host_port(port.try_into().unwrap());
        let provider = MongoDBProvider::new(host_port.unwrap().into()).await;
        FakeMongoDbProvider { provider }
    }
} 

#[cfg(test)]
#[async_trait]
impl MongoDBProviderTrait for FakeMongoDbProvider {
    
    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String> {
        self.provider.insert_struct_to_db(data).await
    }
    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String> {
        self.provider.read_from(id).await
    }
}

#[cfg(test)]
mod tests {
    use testcontainers::{clients, Docker};

    use crate::mydatastruct;

    use super::{FakeMongoDbProvider, MongoDBProviderTrait};

    //TODO: test mongo methods with testcontainers lib
    #[tokio::test]
    async fn mongo_add_and_read_test() {
        let fake_mongo = FakeMongoDbProvider::new(27017).await;
        let test_stuct = mydatastruct::create_my_struct(
            "test".to_string(),
            "AAA".to_string(),
            53,
            mydatastruct::Sex::Female,
        );
        let insert_res=fake_mongo.insert_struct_to_db(test_stuct.clone()).await;
        assert_eq!(insert_res.is_ok(),true);
        let second_insert=fake_mongo.insert_struct_to_db(test_stuct.clone()).await;
        assert_eq!(second_insert.is_err(),true);
        let read_res_vec=fake_mongo.read_from("test".to_string()).await;
        assert_eq!(read_res_vec.is_ok(),true);
        let vec_unw=read_res_vec.unwrap();
        assert_eq!(vec_unw.len(),1);
        assert_eq!(vec_unw[0],test_stuct);
    }

    //TODO: test REST routes with FakeMongo
}
use postgres::{Client as PostClient, NoTls};
use::testcontainers::*;
#[test]
fn postgres_one_plus_one() {
    let docker = clients::Cli::default();
    let postgres_image = images::postgres::Postgres::default();
    let node = docker.run(postgres_image);

    let connection_string = &format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    let mut conn = PostClient::connect(connection_string, NoTls).unwrap();

    for row in conn.query("SELECT 1 + 1", &[]).unwrap() {
        let first_column: i32 = row.get(0);
        assert_eq!(first_column, 2);
    }
}