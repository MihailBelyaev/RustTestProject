use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use tracing::info;
use warp::hyper::StatusCode;

use crate::{
    loginmanager::{self, LogMngTrait, LoginManager},
    models::User,
    routes,
};
#[derive(Clone)]
struct MockLogMngr {
    pub inner: Arc<RwLock<BTreeMap<String, String>>>,
}
impl LogMngTrait for MockLogMngr {
    fn check_user(&self, user: String, pass: String) -> bool {
        let tmp = self.inner.read().unwrap();
        match tmp.get(&user) {
            None => false,
            Some(password) => &pass == password,
        }
    }
    fn get_users_list(&self) -> Result<Vec<User>, diesel::result::Error> {
        let mut res: Vec<User> = Vec::new();
        let tmp = self.inner.read().unwrap();
        let iterat = tmp.iter();
        for (log, pass) in iterat {
            res.push(User {
                login: log.to_string(),
                password: pass.to_string(),
                token: todo!(),
            });
        }
        Ok(res)
    }
    fn insert_new_user(&self, new_user: User) -> bool {
        let mut tmp = self.inner.write().unwrap();
        if tmp.contains_key(&new_user.login) {
            false
        } else {
            tmp.insert(new_user.login, new_user.password);
            true
        }
    }
    fn get_by_login(&self, login: String) -> Option<User> {
        let tmp = self.inner.read().unwrap();
        if tmp.contains_key(&login.clone()) {
            let pass = tmp.get(&login).unwrap();
            Some(User {
                login,
                password: pass.to_string(),
            })
        } else {
            None
        }
    }
    fn update_password(&self, new_data: User) -> bool {
        let mut tmp = self.inner.write().unwrap();
        if tmp.contains_key(&new_data.login) {
            let mut data = tmp.get_mut(&new_data.login).unwrap();
            *data = new_data.password;
            return true;
        } else {
            return false;
        }
    }
    fn delete_user(&self, login: String) -> bool {
        let mut tmp = self.inner.write().unwrap();
        if tmp.contains_key(&login) {
            tmp.remove(&login);
            return true;
        } else {
            return false;
        }
    }
}
#[tokio::test]
async fn login_route_test() {
    tracing_subscriber::fmt().init();
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = User {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    let data_path_routes = routes::login_filter_fcn(mngr.clone()).await;

    let req_test = warp::test::request()
        .path("/login")
        .header("login", &test_stuct.login)
        .header("password", &test_stuct.password)
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::OK);
    assert_eq!(
        req_test.headers().get("token").unwrap().to_str().unwrap(),
        LoginManager::get_security_key()
    );

    let req_test = warp::test::request()
        .path("/login")
        .header("login", &test_stuct.login)
        .header("password", "ABC")
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_users_route_test() {
    tracing_subscriber::fmt().init();
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = User {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    let test_struct = User {
        login: "ABC".to_string(),
        password: "CBA".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    mngr.insert_new_user(test_struct.clone());
    let data_path_routes = routes::get_users_fcn(mngr.clone()).await;
    let req_test = warp::test::request()
        .path("/users")
        .method("GET")
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::OK);
    let body = req_test.into_body();

    let encoded = std::str::from_utf8(&body).unwrap();

    let test_vec = vec![test_stuct, test_struct];
    assert_eq!(encoded, serde_json::to_string(&test_vec).unwrap());
}

#[tokio::test]
async fn post_users_route_test() {
    tracing_subscriber::fmt().init();
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = User {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    let test_struct = User {
        login: "ABC".to_string(),
        password: "CBA".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    let data_path_routes = routes::post_user_fcn(mngr.clone()).await;
    let req_test = warp::test::request()
        .path("/users")
        .method("POST")
        .body(serde_json::to_string(&test_stuct).unwrap())
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::NOT_FOUND);

    let req_test = warp::test::request()
        .path("/users")
        .method("POST")
        .body(serde_json::to_string(&test_struct).unwrap())
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::OK);
}
