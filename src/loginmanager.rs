use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;
use warp::{http, Reply, Rejection};

#[async_trait]
pub trait LogMngTrait: Send {
    async fn check_user(&self, user: String, pass: String) -> bool;
}

#[derive(Clone)]
pub struct LoginManager {
    pub inner: Arc<RwLock<BTreeMap<String, String>>>,
}

impl LoginManager {
    pub fn get_security_key() -> String {
        return "Toad".to_string();
    }
    pub fn new() -> Self {
        let mut little_db: BTreeMap<String, String> = BTreeMap::new();
        little_db.insert("admin".to_string(), "admin".to_string());
        Self {
            inner: Arc::new(RwLock::new(little_db)),
        }
    }
    
}

#[async_trait]
impl LogMngTrait for LoginManager{
    async fn check_user(&self, user: String, pass: String) -> bool {
        self.inner.read().await.contains_key(&user)
            && (self.inner.read().await.get(&user) == futures_util::__private::Some(&pass))
    }
}

pub async fn check_login_data(
    mngr: impl LogMngTrait +Clone+Sync,
    log: String,
    pas: String,
) -> Result<impl warp::Reply, Rejection> {
    
    if mngr.check_user(log, pas).await {
        return Ok(warp::reply::with_status(
            warp::reply::with_header(warp::reply(), "token", LoginManager::get_security_key()),
            http::StatusCode::OK,
        ));
    } else {
        return Err(warp::reject());
    }
}
