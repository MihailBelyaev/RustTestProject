use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::users;
use super::schema::users::dsl::users as user_dsl;
#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, Clone)]
#[table_name = "users"]
pub struct User {
    pub login: String,
    pub password: String,
}
impl User {
    pub fn by_login(login: String, conn: &SqliteConnection) -> Option<Self> {
        if let Ok(record) = user_dsl.find(login).get_result::<User>(conn) {
            Some(record)
        } else {
            None
        }
    }

    pub fn get_list(conn: &SqliteConnection) -> Result<Vec<User>, diesel::result::Error> {
        user_dsl.load::<User>(conn) //expect("Error while loading users list")
    }
    pub fn insert_new_user(conn: &SqliteConnection, new_user: User) -> bool {
        let res = diesel::insert_into(user_dsl)
            .values(&new_user)
            .execute(conn);
        res.is_ok()
    }
}
