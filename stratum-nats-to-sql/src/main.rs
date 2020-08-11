extern crate shared;
/*
  channel will be stratum.auth.2408

  worker table gets owner_id: i32, owner_type: String,

  worker names are the only thing stiching workers together
  were going to gruop any worker with the same name and same owner_id

  worker connects , gets a uuid, discconects, loses a uuid

  on stratum startup, clear workers table with stratum_id of unnamed workers to eliminate 2x issue

  on worker connect, if named worker...
    check the table for worker name
    if worker name exists...
      check active
        if active is true...
          insert a new row - this will be the case for duplicate names, or edge case disconnects
        if active is false...
          update the row
    if worker name does not exists
      add a row, set state to active, set uuid
  if not named worker
    inser a row with a uuid and set state to active

  on worker disconnect...
  set state to disconnected where uuid = uuid

  worker states
  - new
  - active
  - disconnected
  - idle (active but no share for awhile)\


  TODO LIST
  - send owner_id, worker_id, uuid back to stratum on auth
*/
// use sentry::{capture_message, integrations::failure::capture_error, Level};
use dotenv::dotenv;
use futures::join;
use shared::db_mysql::establish_mysql_connection;

use shared::nats::establish_nats_connection;
use std::env;
// mod auth;
mod stratum;
mod worker;
// use std::error::Error;

// const INSERTINTERVAL: u64 = 50;
// const DELETEINTERVAL: u64 = 2000;
// const WINDOW_LENGTH: u64 = 2 * 60 * 60;
// struct GenericAccount {
//   // generic account that we can convert stratumauthnats into to use for workers
//   pub owner_id: i32,
//   pub owner_type: String,
//   pub coin_id: i32,
// }

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

  // let auth_listeners = auth::run_listeners(&env, &mysql_pool, &nc);
  let stratum_listeners = stratum::run_listeners(&env, &mysql_pool, &nc);
  let worker_listeners = worker::run_listeners(&env, &mysql_pool, &nc);

  join!(stratum_listeners, worker_listeners);
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_handle_msg_auth() {
    // let data = rmp_serde::to_vec("hi there tester").unwrap();

    // let res = handle_msg_auth(&data);
    // assert_eq!(res, data);
  }

  // #[test]
  // fn test_parse_password() {
  //   let password = "password";
  //   assert_eq!(parse_password(&password.to_string()), true);
  // }
  //   #[test]
  //   fn test_get_or_insert_account_nim() {
  //     let mysql_pool_conn = establish_mysql_connection().unwrap().get().unwrap();
  //     let stratum_auth_nim = StratumAuthNatsNIM {
  //       username: "NQ11DGPGDBTX8H4J23QDL4A2D5DF7PDF9MAL".to_string(),
  //       coin_id: 2408,
  //       ip: "209.226.65.130".to_string(),
  //       difficulty: 64.0,
  //       version: "noncerpro-cuda-3.3.0".to_string(),
  //       consensus_mode: "dumb".to_string(),
  //       worker_name: "tesla".to_string(),
  //       uuid: "f6b5089d-75ee-4635-a5f2-74401c1d517e".to_string(),
  //       algo: "argon2d".to_string(),
  //       time: 1595121619,
  //       stratum_id: "2".to_string(),
  //       mode: "normal".to_string(),
  //       password: "".to_string(),
  //       party_pass: "".to_string(),
  //       pid: 20796,
  //     };
  //     get_or_insert_account_nim(&mysql_pool_conn, &stratum_auth_nim);
  //     let account =
  //       get_account_by_username_mysql(&mysql_pool_conn, &stratum_auth_nim.username).unwrap();
  //     assert_eq!(account.username, "NQ11DGPGDBTX8H4J23QDL4A2D5DF7PDF9MAL");
  //   }
  // }

  // #[test]
  // fn test_disconnect_worker() {
  //   let mysql_pool_conn = establish_mysql_connection().unwrap().get().unwrap();
  //   let worker = WorkerMYSQLInsertable {
  //     coinid: 1234,
  //     userid: 1234,
  //     worker: "test_worker".to_string(),
  //     hashrate: 0.0,
  //     owner_id: 1234,
  //     difficulty: 0.0,
  //     owner_type: "test_type".to_string(),
  //     uuid: "test_uuid".to_string(),
  //     state: "new".to_string(),
  //     ip: "192.test".to_string(),
  //     version: "test".to_string(),
  //     password: "test".to_string(),
  //     algo: "test".to_string(),
  //     mode: "test".to_string(),
  //     stratum_id: "test".to_string(),
  //   };
  //   insert_worker_mysql(&mysql_pool_conn, worker).unwrap();

  //   let uuid = "test_uuid".to_string();
  //   handle_worker_disconnect(&mysql_pool_conn, &uuid);
  //   let worker = get_worker_by_uuid_mysql(&mysql_pool_conn, &uuid).unwrap();
  //   assert_eq!(worker.state, "disconnected".to_string());
  // }

  // #[test]
  // fn test_stratum_start() {
  //   let mysql_pool_conn = establish_mysql_connection().unwrap().get().unwrap();
  //   let stratum_start_nats = StratumStartNats {
  //     pid: 12,
  //     time: 12,
  //     started: 12,
  //     algo: "test".to_string(),
  //     workers: 0,
  //     port: 12,
  //     symbol: "test".to_string(),
  //     stratum_id: "test".to_string(),
  //     coin_id: 0,
  //   };
  //   handle_stratum_start(&mysql_pool_conn, &stratum_start_nats);
  //   assert_eq!(1, 1);
  // }

  // #[test]
  // fn test_difficulty_update() {
  //   let mysql_pool_conn = establish_mysql_connection().unwrap().get().unwrap();
  //   let worker = WorkerMYSQLInsertable {
  //     coinid: 1234,
  //     userid: 1234,
  //     worker: "test_worker".to_string(),
  //     hashrate: 0.0,
  //     owner_id: 1234,
  //     owner_type: "test_type".to_string(),
  //     uuid: "test_uuid".to_string(),
  //     state: "new".to_string(),
  //     ip: "192.test".to_string(),
  //     version: "test".to_string(),
  //     password: "test".to_string(),
  //     algo: "test".to_string(),
  //     mode: "test".to_string(),
  //     stratum_id: "test".to_string(),
  //     difficulty: 0.0,
  //   };
  //   insert_worker_mysql(&mysql_pool_conn, worker).unwrap();

  //   let uuid = "test_uuid".to_string();

  //   handle_difficulty_update(&mysql_pool_conn, &uuid, 100.0);
  //   let worker = get_worker_by_uuid_mysql(&mysql_pool_conn, &uuid).unwrap();
  //   assert_eq!(worker.difficulty, 100.0);
}
