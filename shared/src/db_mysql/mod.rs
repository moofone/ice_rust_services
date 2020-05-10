pub mod helpers;
pub mod models;
pub mod schema;
pub mod util;

use diesel::{
  mysql::MysqlConnection,
  // prelude::*,
  r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
};

use dotenv::dotenv;
use std::env;

pub type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
pub type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

fn init_mysql_pool(database_url: &str) -> Result<MysqlPool, PoolError> {
  let manager = ConnectionManager::<MysqlConnection>::new(database_url);
  Pool::builder()
    .max_size(16)
    .test_on_check_out(true)
    .build(manager)
  // .map_err(|e| e.into())
}

pub fn establish_mysql_connection() -> MysqlPool {
  dotenv().ok();

  let database_url = env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL must be set");
  init_mysql_pool(&database_url).expect("Failed to create pool")
}
