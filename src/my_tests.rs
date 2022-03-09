use crate::loginmanager::{LogMngTrait, SimplifiedUser};
use crate::mongodbprovider::{self, MongoDBProvider, MongoDBProviderTrait};
use crate::mydatastruct;
use crate::mydatastruct::MyData;
use crate::routes::{get_filter_fcn, insert_filter_fcn};
use crate::testlogin::MockLogMngr;
use async_trait::async_trait;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use testcontainers::clients::Cli;
use testcontainers::images::generic::{GenericImage, WaitFor};
use testcontainers::{clients, Container, Docker, RunArgs}; 
use tokio::sync::RwLock as TokioRwLock;
use warp::http::StatusCode;
use warp::Filter;

#[derive(Clone, Default)]
struct FakeMongoProvider2 {
    pub inner: Arc<TokioRwLock<BTreeMap<String, MyData>>>,
}

impl FakeMongoProvider2 {
    fn new() -> Self {
        Self {
            inner: Arc::new(TokioRwLock::new(BTreeMap::new())),
        }
    }
}

#[async_trait]
impl MongoDBProviderTrait for FakeMongoProvider2 {
    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String> {
        let mut inner = self.inner.write().await;
        inner.insert(data.id_getter(), data);
        Ok(())
    }

    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String> {
        let inner = self.inner.write().await;
        match inner.get(&id) {
            None => Err("err".to_string()),
            Some(data) => Ok(vec![data.clone()]),
        }
    }
}

pub fn mongo_setup(docker: &Cli, port: u16) -> Container<Cli, GenericImage> {
    let mut container_name = "mongo_test".to_string();
    container_name.push_str(port.to_string().as_str());
    let run_args = RunArgs::default()
        .with_name(container_name)
        .with_mapped_port((port as u16, 27017));
    let generic_mongodb_image = GenericImage::new("mongo:5.0")
        .with_wait_for(WaitFor::message_on_stdout("LogicalSessionCacheRefresh"));

    docker.run_with_args(generic_mongodb_image, run_args)
}

#[tokio::test]
async fn mongo_insert_and_read_test() {
    let docker = clients::Cli::default();
    let container = mongo_setup(&docker, 27018);
    let mongo_addr = "localhost".to_string();mongodbprovider::get_db_address_from_env().unwrap();
    let db_name = "".to_string();//env::var("MONGO_INITDB_ROOT_USERNAME").unwrap_or_else(|_| "".to_string());
    let db_pass = "".to_string();//env::var("MONGO_INITDB_ROOT_PASSWORD").unwrap_or_else(|_| "".to_string());
    let mongo_provider = MongoDBProvider::new(mongodbprovider::MongoConnectionParameters { address: mongo_addr, port: 27018, user_name: db_name, password: db_pass }).await;

    let test_struct = mydatastruct::create_my_struct(
        "test".to_string(),
        "AAA".to_string(),
        53,
        mydatastruct::Sex::Female,
    );

    let insertion = mongo_provider
        .insert_struct_to_db(test_struct.clone())
        .await;
    assert!(insertion.is_ok());

    let read = mongo_provider.read_from("test".to_string()).await.unwrap();
    assert_eq!(read.len(), 1);
    assert_eq!(read[0], test_struct);
}

#[tokio::test]
async fn mongo_upsert_test() {
    let docker = clients::Cli::default();
    let container = mongo_setup(&docker, 27019);
    let mongo_addr = "localhost".to_string();mongodbprovider::get_db_address_from_env().unwrap();
    let db_name = "".to_string();//env::var("MONGO_INITDB_ROOT_USERNAME").unwrap_or_else(|_| "".to_string());
    let db_pass = "".to_string();//env::var("MONGO_INITDB_ROOT_PASSWORD").unwrap_or_else(|_| "".to_string());
    let mongo_provider = MongoDBProvider::new(mongodbprovider::MongoConnectionParameters { address: mongo_addr, port: 27019, user_name: db_name, password: db_pass }).await;

    let test_struct = mydatastruct::create_my_struct(
        "test".to_string(),
        "AAA".to_string(),
        53,
        mydatastruct::Sex::Female,
    );

    let insertion = mongo_provider
        .insert_struct_to_db(test_struct.clone())
        .await;
    assert!(insertion.is_ok());
    let upsertion = mongo_provider
        .insert_struct_to_db(test_struct.clone())
        .await;
    assert!(upsertion.is_err());
}

#[tokio::test]
async fn rest_post_insert_data_test() {
    let db_provider = FakeMongoProvider2::default();
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = SimplifiedUser {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    let insert_route = insert_filter_fcn(db_provider.clone(), mngr.clone()).await;
    let data_path = warp::path("data");
    let data_path_routes = data_path.and(insert_route);

    let test_body_request = json!(
        {
            "_id": "test",
            "first_name": "AAA",
            "age": 53,
            "sex": "Female"
        }
    );

    let req_test = warp::test::request()
        .path("/data")
        .method("POST")
        .header("autorization", "123")
        .body(serde_json::to_string(&test_body_request).unwrap())
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::CREATED);

    let inserted_data = db_provider.read_from("test".to_string()).await.unwrap();
    assert_eq!(
        inserted_data[0],
        mydatastruct::create_my_struct(
            "test".to_string(),
            "AAA".to_string(),
            53,
            mydatastruct::Sex::Female
        )
    );
}

#[tokio::test]
async fn rest_get_read_data_with_data_contains_test() {
    let db_provider = FakeMongoProvider2::default();

    let test_struct = mydatastruct::create_my_struct(
        "test".to_string(),
        "AAA".to_string(),
        53,
        mydatastruct::Sex::Female,
    );

    db_provider.insert_struct_to_db(test_struct).await.unwrap();
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = SimplifiedUser {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    let insert_route = get_filter_fcn(db_provider, mngr.clone()).await;
    let data_path = warp::path("data");
    let data_path_routes = data_path.and(insert_route);

    let req_test = warp::test::request()
        .path("/data/test")
        .method("GET")
        .header("autorization", "123")
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::FOUND);

    let assert_body = json!(
        [{
            "_id": "test",
            "first_name": "AAA",
            "age": 53,
            "sex": "Female"
        }]
    );

    let body = req_test.into_body();
    let body = std::str::from_utf8(&body).unwrap();

    assert_eq!(body, serde_json::to_string(&assert_body).unwrap().as_str());
}

#[tokio::test]
async fn rest_get_read_data_without_data_contains_test() {}
