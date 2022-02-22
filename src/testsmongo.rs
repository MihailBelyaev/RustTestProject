use std::sync::Arc;

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
        let mon_addr=mongodbprovider::get_db_address_from_env().unwrap();
        let provider = MongoDBProvider::new(mon_addr,port).await;
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
use ::testcontainers::*;
use postgres::{Client as PostClient, NoTls};
use serde_json::json;
use testcontainers::clients::Cli;
use testcontainers::images::generic::GenericImage;
use testcontainers::images::generic::WaitFor;
use warp::hyper::StatusCode;
use warp::Filter;
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

#[tokio::test]
async fn insert_route_test() {
    let docker = clients::Cli::default();
    let db_provider = FakeMongoDbProvider::new(&docker, 27017).await;
    let insert_route = insert_filter_fcn(db_provider.provider.clone()).await;
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
            "id": "test",
            "first_name": "AAA",
            "age": 53,
            "sex": "Female"
        }
    );

    let req_test = warp::test::request()
        .path("/data")
        .method("POST")
        .body(test_body_request.as_str().unwrap())
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::CREATED);

    let req_test = warp::test::request()
        .path("/data")
        .method("POST")
        .body(serde_json::to_string(&test_stuct).unwrap())
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::NOT_ACCEPTABLE);
}

#[tokio::test]
async fn get_route_test() {
    let docker = clients::Cli::default();
    let db_provider = FakeMongoDbProvider::new(&docker, 27017).await;
    let get_route = get_filter_fcn(db_provider.provider.clone()).await;
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
        .reply(&data_path_routes.clone())
        .await;

    assert_eq!(req_test.status(), StatusCode::FOUND);

    let body = req_test.into_body();

    let encoded = std::str::from_utf8(&body).unwrap();

    let test_vec = vec![test_stuct];
    assert_eq!(encoded, serde_json::to_string(&test_vec).unwrap());
}
