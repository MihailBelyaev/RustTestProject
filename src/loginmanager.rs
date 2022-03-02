use std::{env, sync::Arc};

use diesel::{sqlite::SqliteConnection, Connection, ExpressionMethods, QueryDsl, RunQueryDsl, r2d2::{Pool, ConnectionManager}};
use tracing::{info, warn};
use warp::{http, Rejection};

use crate::{models::User, schema};

pub trait LogMngTrait:Send {
    fn check_user(&self, user: String, pass: String) -> bool;
    fn get_users_list(&self) -> Result<Vec<User>, diesel::result::Error>;
    fn insert_new_user(&self, new_user: User) -> bool;
    fn get_by_login(&self, login: String) -> Option<User>;
    fn update_password(&self, new_data: User) -> bool;
    fn delete_user(&self, login: String) -> bool;
}

#[derive(Clone)]
pub struct LoginManager {
    db_pool:Pool<ConnectionManager<SqliteConnection>>
}

impl LoginManager {
    pub fn get_security_key() -> String {
        return "Toad".to_string();
    }
    pub fn new(db_url: String) -> Self {   
         let pool=Pool::builder().max_size(15).build(ConnectionManager::<SqliteConnection>::new(db_url)).unwrap();
        Self {db_pool:pool }
    }
}

impl LogMngTrait for LoginManager {
    fn check_user(&self, user: String, pass: String) -> bool {
        let conn = self.db_pool.get()
            .unwrap_or_else(|_| panic!("Error connecting to DB"));
        match User::by_login(user.clone(), &conn) {
            Some(res) => res.password == pass,
            None => false,
        }
    }

    fn get_users_list(&self) -> Result<Vec<User>, diesel::result::Error> {
        User::get_list(&self.db_pool.get().unwrap())
    }
    fn insert_new_user(&self, new_user: User) -> bool {
        
        User::insert_new_user(&self.db_pool.get().unwrap(), new_user)
    }
    fn get_by_login(&self, login: String) -> Option<User> {
       
        User::by_login(login, &self.db_pool.get().unwrap())
    }

    fn update_password(&self, new_data: User) -> bool {
        User::update_user_password(&self.db_pool.get().unwrap(), new_data.clone())
    }

    fn delete_user(&self, login: String) -> bool {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        let dsl_filter = schema::users::dsl::users.filter(schema::users::login.eq(login));
        let res = diesel::delete(dsl_filter).execute(&conn);
        res.is_ok()
    }
}

pub async fn check_login_data(
    mngr: impl LogMngTrait + Clone + Sync,
    log: String,
    pas: String,
) -> Result<impl warp::Reply, Rejection> {
    if mngr.check_user(log.clone(), pas.clone()) {
        info!("Got user {}:{}", log, pas);
        return Ok(warp::reply::with_status(
            warp::reply::with_header(warp::reply(), "token", LoginManager::get_security_key()),
            http::StatusCode::OK,
        ));
    } else {
        info!("No user {}:{}", log, pas);
        return Err(warp::reject());
    }
}

pub async fn get_users_list(
    mngr: impl LogMngTrait + Clone + Sync,
) -> Result<impl warp::Reply, Rejection> {
    let res = mngr.get_users_list();
    match res {
        Ok(users_vec) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&users_vec),
                http::StatusCode::OK,
            ))
        }
        Err(err) => {
            warn!("Error while getting user list {}", err);
            return Err(warp::reject());
        }
    }
}

pub async fn insert_user(
    mngr: impl LogMngTrait + Clone + Sync,
    new_user: User,
) -> Result<impl warp::Reply, Rejection> {
    if mngr.insert_new_user(new_user) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Success!".to_string()),
            http::StatusCode::OK,
        ));
    } else {
        return Err(warp::reject());
    }
}

pub async fn get_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
    user_id: String,
) -> Result<impl warp::Reply, Rejection> {
    let res = mngr.get_by_login(user_id);
    if res.is_some() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&res.unwrap()),
            http::StatusCode::OK,
        ));
    } else {
        return Err(warp::reject());
    }
}

pub async fn update_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
    user_id: String,
    new_data: User,
) -> Result<impl warp::Reply, Rejection> {
    if new_data.login != user_id {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"login mismatch!".to_string()),
            http::StatusCode::BAD_REQUEST,
        ));
    } else {
        if mngr.update_password(new_data) {
            return Ok(warp::reply::with_status(
                warp::reply::json(&"Success!".to_string()),
                http::StatusCode::OK,
            ));
        } else {
            return Err(warp::reject());
        }
    }
}

pub async fn delete_certain_user(
    mngr: impl LogMngTrait + Clone + Sync,
    user_id: String,
) -> Result<impl warp::Reply, Rejection> {
    if mngr.delete_user(user_id) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Success!".to_string()),
            http::StatusCode::NO_CONTENT,
        ));
    } else {
        return Err(warp::reject());
    }
}
