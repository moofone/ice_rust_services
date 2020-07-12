// #[macro_use]
extern crate log;

#[macro_use]
extern crate diesel;

extern crate rmp;
extern crate rmp_serde as rmps;
extern crate serde;

extern crate dotenv;
extern crate lazy_static;
extern crate r2d2;

pub mod db_mysql;
pub mod db_pg;
pub mod enums;
pub mod nats;
