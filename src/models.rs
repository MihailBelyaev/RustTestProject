use serde::{Deserialize, Serialize};
use diesel::prelude::*;

use super::schema::users;
use super::schema::users::dsl::users as user_dsl;
#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub login: String,
    pub password: String,
}
impl User{
    pub fn by_login(login: String, conn: &SqliteConnection) -> Option<Self> {
        if let Ok(record) = user_dsl.find(login).get_result::<User>(conn) {
            Some(record)
        } else {
            None
        }
    }   
}