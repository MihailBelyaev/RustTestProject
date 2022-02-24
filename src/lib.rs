#[macro_use]
extern crate diesel;
pub mod schema;
pub mod models;
pub mod loginmanager;
pub mod mongodbprovider;
#[cfg(test)]
mod my_tests;
pub mod mydatastruct;
pub mod routes;
#[cfg(test)]
pub mod testsmongo;

