use std::env;

use diesel::{sqlite::SqliteConnection, Connection};
use tracing::{info, warn};
use warp::{http, Rejection};

use crate::models::User;

pub trait LogMngTrait: Send {
    fn check_user(&self, user: String, pass: String) -> bool;
    fn get_users_list(&self) -> Result<Vec<User>, diesel::result::Error>;
    fn insert_new_user(&self, new_user: User) -> bool;
}

#[derive(Clone)]
pub struct LoginManager {
    db_url: String,
}

impl LoginManager {
    pub fn get_security_key() -> String {
        return "Toad".to_string();
    }
    pub fn new(db_url: String) -> Self {
        SqliteConnection::establish(&db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));
        Self { db_url }
    }
}

impl LogMngTrait for LoginManager {
    fn check_user(&self, user: String, pass: String) -> bool {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        match User::by_login(user.clone(), &conn) {
            Some(res) => res.password == pass,
            None => false,
        }
    }

    fn get_users_list(&self) -> Result<Vec<User>, diesel::result::Error> {
        let conn = SqliteConnection::establish(&self.db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", self.db_url));
        User::get_list(&conn)
    }
    fn insert_new_user(&self, new_user: User) -> bool {
        let conn = SqliteConnection::establish(&self.db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", self.db_url));
        User::insert_new_user(&conn, new_user)
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
