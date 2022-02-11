use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MyData {
    #[serde(rename = "_id")]
    id: String,
    first_name: String,
    age: i32,
    sex: Sex,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Sex {
    Male,
    Female,
}

pub fn create_my_struct(id: String, first_name: String, age: i32, sex: Sex) -> MyData {
    MyData {
        id,
        first_name,
        age,
        sex,
    }
}
