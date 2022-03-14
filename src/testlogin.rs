use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use warp::hyper::StatusCode;

use crate::{
    loginmanager::{LogMngTrait, SimplifiedUser},
    models::User,
    routes,
};
#[derive(Clone)]
pub struct MockLogMngr {
    pub inner: Arc<RwLock<BTreeMap<String, User>>>,
}
impl LogMngTrait for MockLogMngr {
    fn check_user(&self, user: String, pass: String) -> bool {
        let tmp = self.inner.read().unwrap();
        match tmp.get(&user) {
            None => false,
            Some(user) => pass == user.password,
        }
    }
    fn get_users_list(&self) -> Result<Vec<SimplifiedUser>, diesel::result::Error> {
        let mut res: Vec<SimplifiedUser> = Vec::new();
        let tmp = self.inner.read().unwrap();
        let iterat = tmp.iter();
        for (log, user) in iterat {
            res.push(SimplifiedUser {
                login: log.to_string(),
                password: user.password.to_string(),
            });
        }
        Ok(res)
    }
    fn insert_new_user(&self, new_user: SimplifiedUser) -> bool {
        let mut tmp = self.inner.write().unwrap();
        if tmp.contains_key(&new_user.login) {
            false
        } else {
            let tmp_user = new_user.clone();
            tmp.insert(
                new_user.login.clone(),
                User {
                    login: tmp_user.login,
                    password: tmp_user.password,
                    token: new_user.login,
                },
            );
            true
        }
    }
    fn get_by_login(&self, login: String) -> Option<SimplifiedUser> {
        let tmp = self.inner.read().unwrap();
        if tmp.contains_key(&login) {
            let pass = tmp.get(&login).unwrap();
            Some(SimplifiedUser {
                login,
                password: pass.password.to_string(),
            })
        } else {
            None
        }
    }
    fn update_password(&self, new_data: SimplifiedUser) -> bool {
        let mut tmp = self.inner.write().unwrap();
        if tmp.contains_key(&new_data.login) {
            let mut data = tmp.get_mut(&new_data.login).unwrap();
            data.password = new_data.password;
            true
        } else {
            false
        }
    }
    fn delete_user(&self, login: String) -> bool {
        let mut tmp = self.inner.write().unwrap();
        if tmp.contains_key(&login) {
            tmp.remove(&login);
            true
        } else {
            false
        }
    }

    fn get_security_key(&self, username: String) -> String {
        let tmp = self.inner.read().unwrap();
        tmp.get(&username).unwrap().token.clone()
    }

    fn check_token(&self, _token: String, _req: String) -> bool {
        true
    }

    fn get_history(
        &self,
        login: String,
    ) -> Result<Vec<crate::models::History>, diesel::result::Error> {
        let tmp = self.inner.read().unwrap();
        if tmp.contains_key(&login) {
            Ok(Vec::new())
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }
}
#[tokio::test]
async fn login_route_test() {
    tracing_subscriber::fmt().try_init();
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = SimplifiedUser {
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
        mngr.get_security_key(test_stuct.login.clone())
    );
    let test_login = test_stuct.login.clone();
    let req_test = warp::test::request()
        .path("/login")
        .header("login", &test_login)
        .header("password", "ABC")
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_users_route_test() {
    tracing_subscriber::fmt().try_init();
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = SimplifiedUser {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    let test_struct = SimplifiedUser {
        login: "ABC".to_string(),
        password: "CBA".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    mngr.insert_new_user(test_struct.clone());
    let data_path_routes = routes::get_users_fcn(mngr.clone()).await;
    let req_test = warp::test::request()
        .path("/users")
        .method("GET")
        .header("autorization", "123")
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
    tracing_subscriber::fmt().try_init();
    let mngr = MockLogMngr {
        inner: Arc::new(RwLock::new(BTreeMap::new())),
    };
    let test_stuct = SimplifiedUser {
        login: "123".to_string(),
        password: "321".to_string(),
    };
    let test_struct = SimplifiedUser {
        login: "ABC".to_string(),
        password: "CBA".to_string(),
    };
    mngr.insert_new_user(test_stuct.clone());
    let data_path_routes = routes::post_user_fcn(mngr.clone()).await;
    let req_test = warp::test::request()
        .path("/users")
        .method("POST")
        .header("autorization", "123")
        .body(serde_json::to_string(&test_stuct).unwrap())
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::NOT_FOUND);

    let req_test = warp::test::request()
        .path("/users")
        .method("POST")
        .header("autorization", "123")
        .body(serde_json::to_string(&test_struct).unwrap())
        .reply(&data_path_routes.clone())
        .await;
    assert_eq!(req_test.status(), StatusCode::OK);
}
