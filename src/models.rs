use crate::schema;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::users;
use super::schema::users::dsl::users as user_dsl;

use super::schema::history;
use super::schema::history::dsl::history as history_dsl;

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, Clone)]
#[table_name = "users"]
pub struct User {
    pub login: String,
    pub password: String,
    pub token: String,
}
impl User {
    pub fn by_login(login: String, conn: &SqliteConnection) -> Option<Self> {
        if let Ok(record) = user_dsl.find(login).get_result::<User>(conn) {
            Some(record)
        } else {
            None
        }
    }
    pub fn by_token(token: String, conn: &SqliteConnection) -> Option<Self> {
        let dsl_filter = schema::users::dsl::users.filter(schema::users::token.eq(token));
        if let Ok(record) = dsl_filter.first::<User>(conn) {
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
    pub fn update_user_password(conn: &SqliteConnection, new_user: User) -> bool {
        let dsl_filter = schema::users::dsl::users.filter(schema::users::login.eq(new_user.login));
        let res = diesel::update(dsl_filter)
            .set(schema::users::password.eq(new_user.password))
            .execute(conn);
        if let Ok(result)=res {
            result != 0 
        } else {
            false
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, Clone)]
#[table_name = "history"]
pub struct History {
    pub id: String,
    pub login: String,
    pub request: String,
    pub tms: NaiveDateTime,
}

impl History {
    pub fn add_element(conn: &SqliteConnection, element: History) {
        let _res = diesel::insert_into(history_dsl)
            .values(&element)
            .execute(conn);
    }
    pub fn get_by_login(
        conn: &SqliteConnection,
        user_name: String,
    ) -> Result<Vec<History>, diesel::result::Error> {
        let dsl_filter = schema::history::dsl::history.filter(schema::history::login.eq(user_name));
        dsl_filter.load::<History>(conn)
    }
}
