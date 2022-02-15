use crate::mongodbprovider::*;
use crate::mydatastruct::*;
use async_trait::async_trait;
#[cfg(test)]

struct FakeMongoDbProvider<'a> {
    provider: MongoDBProvider,
    node:Container<'a, Cli,GenericImage>
}
#[cfg(test)]
impl FakeMongoDbProvider<'_>{
    pub async fn new(docker:&Cli,port: i32) -> FakeMongoDbProvider<'_> {
        
        let run_arg=RunArgs::default()
            .with_name("mongo_test")
            .with_mapped_port((port as u16,27017));
        let generic_mongodb = GenericImage::new("mongo:5.0")
            .with_wait_for(WaitFor::message_on_stdout("LogicalSessionCacheRefresh"));

        let node = docker.run_with_args(generic_mongodb, run_arg);
        
        let provider = MongoDBProvider::new(port).await;
        FakeMongoDbProvider { provider, node }
    }
} 

#[cfg(test)]
#[async_trait]
impl MongoDBProviderTrait for FakeMongoDbProvider<'_>{
    
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
        let docker = clients::Cli::default();
        let fake_mongo = FakeMongoDbProvider::new(&docker,27017).await;
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
use testcontainers::clients::Cli;
use testcontainers::images::generic::GenericImage;
use testcontainers::images::generic::WaitFor;
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