extern crate shared;

use diesel::prelude::*;
use dotenv::dotenv;
use futures::join;
use shared::db_mysql::establish_mysql_connection;
use shared::nats::{establish_nats_connection, NatsConnection};
use std::env;
mod stratums;
mod workers;

#[tokio::main]
async fn main() {
  dotenv().ok();
  let env = env::var("ENVIRONMENT_MODE").unwrap_or("".to_string());
  // let _guard =
  //   sentry::init("https://f8ee06fb619843b1ae923d9111d855a9@sentry.watlab.icemining.ca/10");

  // let mut tasks = Vec::new();
  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      // crash and sentry BIG
      panic!("Nats did not connect: {}", e);
    }
  };
  //setup msqyl
  let mysql_pool = match establish_mysql_connection() {
    Ok(p) => p,
    Err(e) => {
      // crash and sentry BIG
      panic!("MYSQL FAILED: {}", e)
    }
  };

  let _workers = workers::run_jobs(&env, &mysql_pool, &nc);
  let _stratums = stratums::run_jobs(&env, &mysql_pool, &nc);
  join!(_workers, _stratums);
  // // //-----------------------workers cleanup--------------------------------
  // let _mysql_workers_cleanup_listener = mysql_workers_cleanup_listener(&env, &mysql_pool, &nc);

  // // //-----------------------stratums cleanup--------------------------------
  // let _mysql_stratums_cleanup_listener = mysql_stratums_cleanup_listener(&env, &mysql_pool, &nc);

  // join!(
  //   _mysql_workers_cleanup_listener,
  //   _mysql_stratums_cleanup_listener
  // );
}
