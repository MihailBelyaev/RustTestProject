use serde::{Serialize,Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MyData{
    id: String,
    first_name: String,
    age: i32,
    sex: Sex
    }

#[derive(Debug, Serialize, Deserialize)]   
pub enum Sex{
    Male,
    Female
    }