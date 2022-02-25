
use std::{ env};

use async_trait::async_trait;
use tracing::{info, warn};
use warp::{http, Rejection};
use diesel::{sqlite::SqliteConnection, Connection};

use crate::models::{User, self};

#[async_trait]
pub trait LogMngTrait: Send {
    async fn check_user(&self, user: String, pass: String) -> bool;
}

#[derive(Clone)]
pub struct LoginManager {
}

impl LoginManager {
    pub fn get_security_key() -> String {
        return "Toad".to_string();
    }
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
        SqliteConnection::establish(&database_url)
          .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        Self {
        }
    }
}

#[async_trait]
impl LogMngTrait for LoginManager {
    async fn check_user(&self, user: String, pass: String) -> bool {
        let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
        let conn=SqliteConnection::establish(&database_url)
          .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        match User::by_login(user.clone(),&conn){
            Some(res)=>{
                res.password==pass
            },
            None => false
        }
        
    }
}

pub async fn check_login_data(
    mngr: impl LogMngTrait + Clone + Sync,
    log: String,
    pas: String,
) -> Result<impl warp::Reply, Rejection> {
    if mngr.check_user(log.clone(), pas.clone()).await {
        info!("Got user {}:{}",log,pas);
        return Ok(warp::reply::with_status(
            warp::reply::with_header(warp::reply(), "token", LoginManager::get_security_key()),
            http::StatusCode::OK,
        ));
    } else {
        info!("No user {}:{}",log,pas);
        return Err(warp::reject());
    }
}

pub async fn get_users_list()->Result<impl warp::Reply, Rejection>{
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
        let conn=SqliteConnection::establish(&database_url)
          .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
          let res=User::get_list(&conn);
    match res{
        Ok(users_vec)=>{
            return Ok(warp::reply::with_status(
                warp::reply::json(&users_vec),
                http::StatusCode::OK))
        },
        Err(err)=>{
            warn!("Error while getting user list {}",err);
            return Err(warp::reject());
        }
    }
}
