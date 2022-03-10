use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;
use std::sync::RwLock;

use crate::loginmanager::LogMngTrait;
use crate::loginmanager::SimplifiedUser;
use crate::mongodbprovider;
use crate::mongodbprovider::*;
use crate::mydatastruct;
use crate::mydatastruct::*;
use crate::routes::get_filter_fcn;
use crate::routes::insert_filter_fcn;
use async_trait::async_trait;

struct FakeMongoDbProvider<'a> {
    provider: MongoDBProvider,
    node: Arc<Container<'a, Cli, GenericImage>>,
}

impl FakeMongoDbProvider<'_> {
    pub async fn new(docker: &Cli, port: i32) -> FakeMongoDbProvider<'_> {
        let run_arg = RunArgs::default()
            .with_name("mongo_test")
            .with_mapped_port((port as u16, 27017));
        let generic_mongodb = GenericImage::new("mongo:5.0")
            .with_wait_for(WaitFor::message_on_stdout("LogicalSessionCacheRefresh"));

        let node = Arc::new(docker.run_with_args(generic_mongodb, run_arg));
        let mon_addr = "localhost".to_string();//mongodbprovider::get_db_address_from_env().unwrap();
        let db_name = "".to_string();//env::var("MONGO_INITDB_ROOT_USERNAME").unwrap_or_else(|_| "".to_string());
        let db_pass = "".to_string();//env::var("MONGO_INITDB_ROOT_PASSWORD").unwrap_or_else(|_| "".to_string());
        let provider = MongoDBProvider::new(MongoConnectionParameters{ address: mon_addr, port, user_name: db_name, password: db_pass }).await;
        FakeMongoDbProvider { provider, node }
    }
}

#[async_trait]
impl MongoDBProviderTrait for FakeMongoDbProvider<'_> {
    async fn insert_struct_to_db(&self, data: MyData) -> Result<(), String> {
        self.provider.insert_struct_to_db(data).await
    }
    async fn read_from(&self, id: String) -> Result<Vec<MyData>, String> {
        self.provider.read_from(id).await
    }
}

mod tests {
    use testcontainers::{clients, Docker};

    use crate::mydatastruct;

    use super::{FakeMongoDbProvider, MongoDBProviderTrait};

    //TODO: test mongo methods with testcontainers lib
    #[tokio::test]
    async fn mongo_add_and_read_test() {
        let docker = clients::Cli::default();
        let fake_mongo = FakeMongoDbProvider::new(&docker, 27017).await;
        let test_stuct = mydatastruct::create_my_struct(
            "test".to_string(),
            "AAA".to_string(),
            53,
            mydatastruct::Sex::Female,
        );
        let insert_res = fake_mongo.insert_struct_to_db(test_stuct.clone()).await;
        assert_eq!(insert_res.is_ok(), true);
        let second_insert = fake_mongo.insert_struct_to_db(test_stuct.clone()).await;
        assert_eq!(second_insert.is_err(), true);
        let read_res_vec = fake_mongo.read_from("test".to_string()).await;
        assert_eq!(read_res_vec.is_ok(), true);
        let vec_unw = read_res_vec.unwrap();
        assert_eq!(vec_unw.len(), 1);
        assert_eq!(vec_unw[0], test_stuct);
    }

    //TODO: test REST routes with FakeMongo
}
use crate::testlogin::MockLogMngr;
use ::testcontainers::*;
use serde_json::json;
use testcontainers::clients::Cli;
use testcontainers::images::generic::GenericImage;
use testcontainers::images::generic::WaitFor;
use warp::hyper::StatusCode;
use warp::Filter;

#[tokio::test]
async fn insert_route_test() {
    let docker = clients::Cli::default();
    let db_provider = FakeMongoDbProvider::new(&docker, 27017).await;
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = SimplifiedUser {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    let insert_route = insert_filter_fcn(db_provider.provider.clone(), mngr.clone()).await;
    let data_path = warp::path("data");
    let data_path_routes = data_path.and(insert_route);

    let test_stuct = mydatastruct::create_my_struct(
        "test".to_string(),
        "AAA".to_string(),
        53,
        mydatastruct::Sex::Female,
    );

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

    let req_test = warp::test::request()
        .path("/data")
        .method("POST")
        .header("autorization", "123")
        .body(serde_json::to_string(&test_stuct).unwrap())
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::NOT_ACCEPTABLE);
}

#[tokio::test]
async fn get_route_test() {
    let docker = clients::Cli::default();
    let db_provider = FakeMongoDbProvider::new(&docker, 27017).await;
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = SimplifiedUser {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    let get_route = get_filter_fcn(db_provider.provider.clone(), mngr.clone()).await;
    let data_path = warp::path("data");
    let data_path_routes = data_path.and(get_route);

    let test_stuct = mydatastruct::create_my_struct(
        "test".to_string(),
        "AAA".to_string(),
        53,
        mydatastruct::Sex::Female,
    );

    let req_test = warp::test::request()
        .path("/data/test")
        .method("GET")
        .header("autorization", "123")
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::NOT_FOUND);

    let _insert_res = db_provider
        .provider
        .insert_struct_to_db(test_stuct.clone())
        .await;
    assert_eq!(_insert_res.is_ok(), true);

    let req_test = warp::test::request()
        .path("/data/test")
        .method("GET")
        .header("autorization", "123")
        .reply(&data_path_routes.clone())
        .await;

    assert_eq!(req_test.status(), StatusCode::FOUND);

    let body = req_test.into_body();

    let encoded = std::str::from_utf8(&body).unwrap();

    let test_vec = vec![test_stuct];
    assert_eq!(encoded, serde_json::to_string(&test_vec).unwrap());
}
