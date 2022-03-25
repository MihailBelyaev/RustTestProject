use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, QueryDsl, RunQueryDsl,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
use warp::{http, Rejection};

use crate::{
    models::{History, User},
    schema,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimplifiedUser {
    pub login: String,
    pub password: String,
}

pub trait LogMngTrait: Send {
    fn check_user(&self, user: String, pass: String) -> bool;
    fn get_users_list(&self) -> Result<Vec<SimplifiedUser>, diesel::result::Error>;
    fn insert_new_user(&self, new_user: SimplifiedUser) -> bool;
    fn get_by_login(&self, login: String) -> Option<SimplifiedUser>;
    fn update_password(&self, new_data: SimplifiedUser) -> bool;
    fn delete_user(&self, login: String) -> bool;

    fn get_security_key(&self, username: String) -> String;
    fn check_token(&self, token: String, req: String) -> bool;
    fn get_history(&self, login: String) -> Result<Vec<History>, diesel::result::Error>;
}

#[derive(Clone)]
pub struct LoginManager {
    db_pool: Pool<ConnectionManager<PgConnection>>,
}

impl LoginManager {
    pub fn new(db_url: String) -> Self {
        let pool = Pool::builder()
            .max_size(15)
            .build(ConnectionManager::<PgConnection>::new(db_url))
            .unwrap();
        Self { db_pool: pool }
    }
}

impl LogMngTrait for LoginManager {
    fn check_user(&self, user: String, pass: String) -> bool {
        let conn = self
            .db_pool
            .get()
            .unwrap_or_else(|_| panic!("Error connecting to DB"));
        match User::by_login(user, &conn) {
            Some(res) => res.password == pass,
            None => false,
        }
    }

    fn get_users_list(&self) -> Result<Vec<SimplifiedUser>, diesel::result::Error> {
        let res = User::get_list(&self.db_pool.get().unwrap());
        match res {
            Ok(res_vec) => {
                let mut res = Vec::<SimplifiedUser>::new();
                for (log, pas) in res_vec
                    .iter()
                    .map(|x| (x.login.clone(), x.password.clone()))
                {
                    res.push(SimplifiedUser {
                        login: log,
                        password: pas,
                    });
                }
                Ok(res)
            }
            Err(err) => Err(err),
        }
    }
    fn insert_new_user(&self, new_user: SimplifiedUser) -> bool {
        User::insert_new_user(
            &self.db_pool.get().unwrap(),
            User {
                login: new_user.login,
                password: new_user.password,
                token: Uuid::new_v4().to_string(),
            },
        )
    }
    fn get_by_login(&self, login: String) -> Option<SimplifiedUser> {
        if let Some(user) = User::by_login(login, &self.db_pool.get().unwrap()) {
            Some(SimplifiedUser {
                login: user.login,
                password: user.password,
            })
        } else {
            None
        }
    }

    fn update_password(&self, new_data: SimplifiedUser) -> bool {
        User::update_user_password(
            &self.db_pool.get().unwrap(),
            User {
                login: new_data.login,
                password: new_data.password,
                token: Uuid::new_v4().to_string(),
            },
        )
    }

    fn delete_user(&self, login: String) -> bool {
        let dsl_filter = schema::users::dsl::users.filter(schema::users::login.eq(login));
        let res = diesel::delete(dsl_filter).execute(&self.db_pool.get().unwrap());
        res.is_ok()
    }

    fn get_security_key(&self, username: String) -> String {
        let conn = self
            .db_pool
            .get()
            .unwrap_or_else(|_| panic!("Error connecting to DB"));
        let tmp_usr = User::by_login(username, &conn).unwrap();
        tmp_usr.token
    }
    fn check_token(&self, token: String, req: String) -> bool {
        let conn = self
            .db_pool
            .get()
            .unwrap_or_else(|_| panic!("Error connecting to DB"));
        let res = User::by_token(token, &conn);
        if let Some(val) = res {
            let elem = History {
                id: Uuid::new_v4().to_string(),
                login: val.login,
                request: req,
                tms: chrono::Utc::now().naive_utc(),
            };
            History::add_element(&conn, elem);
            true
        } else {
            false
        }
    }
    fn get_history(&self, login: String) -> Result<Vec<History>, diesel::result::Error> {
        let conn = self
            .db_pool
            .get()
            .unwrap_or_else(|_| panic!("Error connecting to DB"));
        History::get_by_login(&conn, login)
    }
}

pub async fn check_login_data(
    mngr: impl LogMngTrait + Clone + Sync,
    log: String,
    pas: String,
) -> Result<impl warp::Reply, Rejection> {
    if mngr.check_user(log.clone(), pas.clone()) {
        info!("Got user {}:{}", log, pas);
        Ok(warp::reply::with_status(
            warp::reply::with_header(warp::reply(), "token", mngr.get_security_key(log)),
            http::StatusCode::OK,
        ))
    } else {
        info!("No user {}:{}", log, pas);
        Err(warp::reject())
    }
}

pub async fn get_users_list(
    mngr: impl LogMngTrait + Clone + Sync,
    token: String,
) -> Result<impl warp::Reply, Rejection> {
    if !mngr.check_token(token, "Get users list".to_string()) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Wrong token".to_string()),
            http::StatusCode::FORBIDDEN,
        ));
    }

    let res = mngr.get_users_list();
    match res {
        Ok(users_vec) => Ok(warp::reply::with_status(
            warp::reply::json(&users_vec),
            http::StatusCode::OK,
        )),
        Err(err) => {
            warn!("Error while getting user list {}", err);
            Err(warp::reject())
        }
    }
}

pub async fn insert_user(
    mngr: impl LogMngTrait + Clone + Sync,
    new_user: SimplifiedUser,
    token: String,
) -> Result<impl warp::Reply, Rejection> {
    if !mngr.check_token(token, format!("Insert user {:?}", new_user)) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Wrong token".to_string()),
            http::StatusCode::FORBIDDEN,
        ));
    }
    if mngr.insert_new_user(new_user) {
        Ok(warp::reply::with_status(
            warp::reply::json(&"Success!".to_string()),
            http::StatusCode::OK,
        ))
    } else {
        Err(warp::reject())
    }
}

pub async fn get_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
    user_id: String,
    token: String,
) -> Result<impl warp::Reply, Rejection> {
    if !mngr.check_token(token, format!("Get user with id{}", user_id)) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Wrong token".to_string()),
            http::StatusCode::FORBIDDEN,
        ));
    }

    let res = mngr.get_by_login(user_id);
    if res.is_some() {
        Ok(warp::reply::with_status(
            warp::reply::json(&res.unwrap()),
            http::StatusCode::OK,
        ))
    } else {
        Err(warp::reject())
    }
}

pub async fn update_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
    user_id: String,
    new_data: SimplifiedUser,
    token: String,
) -> Result<impl warp::Reply, Rejection> {
    if !mngr.check_token(
        token,
        format!("Update user {} with {:?}", user_id, new_data),
    ) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Wrong token".to_string()),
            http::StatusCode::FORBIDDEN,
        ));
    }

    if new_data.login != user_id {
        Ok(warp::reply::with_status(
            warp::reply::json(&"login mismatch!".to_string()),
            http::StatusCode::BAD_REQUEST,
        ))
    } else if mngr.update_password(new_data) {
        Ok(warp::reply::with_status(
            warp::reply::json(&"Success!".to_string()),
            http::StatusCode::OK,
        ))
    } else {
        Err(warp::reject())
    }
}

pub async fn delete_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
    user_id: String,
    token: String,
) -> Result<impl warp::Reply, Rejection> {
    if !mngr.check_token(token, format!("Delete user {:?}", user_id)) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Wrong token".to_string()),
            http::StatusCode::FORBIDDEN,
        ));
    }

    if mngr.delete_user(user_id) {
        Ok(warp::reply::with_status(
            warp::reply::json(&"Success!".to_string()),
            http::StatusCode::NO_CONTENT,
        ))
    } else {
        Err(warp::reject())
    }
}

pub async fn get_history_for_user(
    mngr: impl LogMngTrait + Clone + Sync,
    user_id: String,
) -> Result<impl warp::Reply, Rejection> {
    match mngr.get_history(user_id) {
        Ok(res) => Ok(warp::reply::with_status(
            warp::reply::json(&res),
            http::StatusCode::OK,
        )),
        Err(_) => Err(warp::reject()),
    }
}
