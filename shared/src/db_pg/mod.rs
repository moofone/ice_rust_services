pub mod helpers;
pub mod models;
pub mod schema;
pub mod util;

use diesel::{
  // pg::Pg,
  prelude::*,
  r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
};

use dotenv::dotenv;
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

fn init_postgres_pool(database_url: &str) -> Result<PgPool, PoolError> {
  let manager = ConnectionManager::<PgConnection>::new(database_url);
  Pool::builder()
    .max_size(20)
    .test_on_check_out(true)
    .build(manager)
  // .map_err(|e| e.into())
}
pub fn establish_pg_connection() -> Result<PgPool, PoolError> {
  dotenv().ok();

  let database_url = env::var("PG_DATABASE_URL").expect("PG_DATABASE_URL must be set");
  println!("{}", &database_url);
  init_postgres_pool(&database_url)
}
