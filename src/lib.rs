#[macro_use]
extern crate diesel;
pub mod loginmanager;
pub mod models;
pub mod mongodbprovider;
#[cfg(test)]
mod my_tests;
pub mod mydatastruct;
pub mod routes;
pub mod schema;
#[cfg(test)]
pub mod testsmongo;
#[cfg(test)]
pub mod testlogin;
