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
use diesel::prelude::*;
use dotenv::dotenv;
use futures::join;
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::accounts::{get_account_by_username_mysql, insert_account_mysql},
  helpers::stratums::insert_stratum_mysql,
  helpers::workers::{
    get_disconnected_worker_by_worker_name_mysql, get_worker_by_uuid_mysql, insert_worker_mysql,
    update_worker_mysql, update_workers_on_stratum_connect_mysql,
  },
  models::{StratumMYSQLInsertable, WorkerMYSQL, WorkerMYSQLInsertable},
  MysqlPool,
};
use shared::nats::models::{
  StratumAuthNatsNIM, StratumAuthResponseNats, StratumDevfeeNats, StratumDifficultyNats,
  StratumDisconnectNats, StratumStartNats,
};
use shared::nats::{establish_nats_connection, NatsConnection};
use std::env;
// use std::error::Error;

// const INSERTINTERVAL: u64 = 50;
// const DELETEINTERVAL: u64 = 2000;
// const WINDOW_LENGTH: u64 = 2 * 60 * 60;
struct GenericAccount {
  // generic account that we can convert stratumauthnats into to use for workers
  pub owner_id: i32,
  pub owner_type: String,
  pub coin_id: i32,
}

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

  // //-----------------------stratum AUTH LISTENER--------------------------------
  let auth_listener = stratum_auth_listener(&env, &mysql_pool, &nc);

  // //-----------------------stratum Start listener--------------------------------
  let start_listener = stratum_start_listener(&env, &mysql_pool, &nc);

  // //-----------------------stratum difficutly listener--------------------------------
  let difficulty_listener = stratum_difficulty_listener(&env, &mysql_pool, &nc);

  // //-----------------------stratum difficutly listener--------------------------------
  let disconnect_listener = stratum_disconnect_listener(&env, &mysql_pool, &nc);

  // //-----------------------stratum devfee listener--------------------------------
  let devfee_listener = stratum_devfee_listener(&env, &mysql_pool, &nc);

  join!(
    auth_listener,
    start_listener,
    difficulty_listener,
    disconnect_listener,
    devfee_listener,
  );
}

fn stratum_auth_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.auth.2408");
  } else {
    subject = format!("stratum.auth.2408");
  }
  let sub = match nc.queue_subscribe(&subject, "stratum_auth_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum auth listener failed: {}", e),
  };

  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      // println!("Msg: {}", msg.subject);
      let stratum_auth_nats_nim = match parse_msg_auth(&msg.data) {
        Ok(a) => a,
        Err(e) => {
          println!("failed to parse stratum auth nats msg: {}", e);
          continue;
        }
      };

      // grab a mysql pool connection
      let conn = match mysql_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          panic!("error getting mysql connection e: {}",);
        }
      };

      // get or insert the account
      let gen_account = get_or_insert_account_nim(&conn, &stratum_auth_nats_nim);

      // insert or update the worker
      let worker = insert_or_update_worker(&conn, &gen_account, &stratum_auth_nats_nim).unwrap();

      let nats_response = StratumAuthResponseNats {
        owner_id: gen_account.owner_id,
        worker_id: worker.id,
        uuid: worker.uuid,
      };
      let response = rmp_serde::to_vec_named(&nats_response).unwrap();
      msg.respond(&response).unwrap();
    }
  })
}

fn stratum_start_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.start.2408");
  } else {
    subject = format!("stratum.start.2408");
  }
  let sub = match nc.queue_subscribe(&subject, "stratum_start_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum start listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      println!("Msg: {}", msg.subject);
      let stratum_start_nats = parse_msg_start(&msg.data).unwrap();

      // grab a mysql pool connection
      let conn = match mysql_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          panic!("error getting mysql connection e: {}",);
        }
      };
      handle_stratum_start(&conn, &stratum_start_nats);
    }
  })
}

fn stratum_difficulty_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.difficulty.2408");
  } else {
    subject = format!("stratum.difficulty.2408");
  }
  let sub = match nc.queue_subscribe(&subject, "stratum_difficulty_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum difficulty listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      println!("Msg: {}", msg.subject);
      let stratum_difficulty_nats = parse_msg_difficulty(&msg.data).unwrap();

      // grab a mysql pool connection
      let conn = match mysql_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          panic!("error getting mysql connection e: {}",);
        }
      };
      handle_difficulty_update(
        &conn,
        &stratum_difficulty_nats.uuid,
        stratum_difficulty_nats.difficulty,
      );
    }
  })
}
fn stratum_disconnect_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.disconnect.2408");
  } else {
    subject = format!("stratum.disconnect.2408");
  }
  let sub = match nc.queue_subscribe(&subject, "stratum_disconnect_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum disconnect listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      // println!("Msg: {}", msg.subject);
      let stratum_disconnect_nats = parse_msg_disconnect(&msg.data).unwrap();

      // grab a mysql pool connection
      let conn = match mysql_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          panic!("error getting mysql connection e: {}",);
        }
      };
      handle_worker_disconnect(&conn, &stratum_disconnect_nats.uuid);
    }
  })
}
fn stratum_devfee_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.devfee.2408");
  } else {
    subject = format!("stratum.devfee.2408");
  }
  let sub = match nc.queue_subscribe(&subject, "stratum_devfee_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum devfee listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      println!("Msg: {}", msg.subject);
      let stratum_devfee_nats = parse_msg_devfee(&msg.data).unwrap();

      // grab a mysql pool connection
      let conn = match mysql_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          panic!("error getting mysql connection e: {}",);
        }
      };
      handle_worker_devfee(&conn, &stratum_devfee_nats.uuid);
    }
  })
}

fn parse_msg_auth(msg: &Vec<u8>) -> Result<StratumAuthNatsNIM, rmp_serde::decode::Error> {
  rmp_serde::from_read_ref(&msg)
  // let auth: StratumAuthNatsNIM = match rmp_serde::from_read_ref(&msg) {
  //   Ok(auth) => auth,
  //   Err(e) => panic!("Error parsing Startum auth nats. e: {}", e),
  // };
  // // println!("stratum auth nats nim : {:?}", auth);
  // Ok(auth)
}
fn parse_msg_start(msg: &Vec<u8>) -> Result<StratumStartNats, rmp_serde::decode::Error> {
  let start: StratumStartNats = match rmp_serde::from_read_ref(&msg) {
    Ok(start) => start,
    Err(e) => panic!("Error parsing Startum start nats. e: {}", e),
  };
  println!("stratum start nats nim : {:?}", start);
  Ok(start)
}
fn parse_msg_difficulty(msg: &Vec<u8>) -> Result<StratumDifficultyNats, rmp_serde::decode::Error> {
  let difficulty: StratumDifficultyNats = match rmp_serde::from_read_ref(&msg) {
    Ok(difficulty) => difficulty,
    Err(e) => panic!("Error parsing Startum difficulty nats. e: {}", e),
  };
  println!("stratum difficulty nats nim : {:?}", difficulty);
  Ok(difficulty)
}
fn parse_msg_disconnect(msg: &Vec<u8>) -> Result<StratumDisconnectNats, rmp_serde::decode::Error> {
  let disconnect: StratumDisconnectNats = match rmp_serde::from_read_ref(&msg) {
    Ok(disconnect) => disconnect,
    Err(e) => panic!("Error parsing Startum disconnect nats. e: {}", e),
  };
  // println!("stratum disconnect nats nim : {:?}", disconnect);
  Ok(disconnect)
}
fn parse_msg_devfee(msg: &Vec<u8>) -> Result<StratumDevfeeNats, rmp_serde::decode::Error> {
  let devfee: StratumDevfeeNats = match rmp_serde::from_read_ref(&msg) {
    Ok(devfee) => devfee,
    Err(e) => panic!("Error parsing Startum devfee nats. e: {}", e),
  };
  println!("stratum devfee nats nim : {:?}", devfee);
  Ok(devfee)
}

fn get_or_insert_account_nim(
  pooled_conn: &MysqlConnection,
  new_msg: &StratumAuthNatsNIM,
) -> GenericAccount {
  let is_username = Some(new_msg.username.find('@'));
  let mut gen_account = GenericAccount {
    owner_id: 0,
    owner_type: "".to_string(),
    coin_id: 0,
  };
  // println!("{}", &new_msg.username);
  if is_username != None {
    let account = match get_account_by_username_mysql(pooled_conn, &new_msg.username) {
      Ok(a) => a, // found
      Err(e) => {
        println!("Account not found, inserting - {}", e);
        // not found
        match insert_account_mysql(pooled_conn, &new_msg.username, new_msg.coin_id as i32) {
          Ok(a) => a,
          Err(e) => panic!("insert failed. e: {}", e),
        }
      }
    };
    gen_account.owner_id = account.id;
    gen_account.owner_type = "account".to_string();
    gen_account.coin_id = account.coinid;
  }
  // println!("account: {}", gen_account.owner_id);
  gen_account
}

fn insert_or_update_worker(
  pooled_conn: &MysqlConnection,
  account: &GenericAccount,
  new_msg: &StratumAuthNatsNIM,
) -> Result<WorkerMYSQL, Box<dyn std::error::Error>> {
  if new_msg.worker_name.len() > 0 {
    if let Ok(mut worker) = get_disconnected_worker_by_worker_name_mysql(
      pooled_conn,
      account.owner_id,
      &account.owner_type,
      &new_msg.worker_name,
    ) {
      if worker.state != "connected" && worker.state != "active" {
        worker.state = "connected".to_string();
        worker.uuid = new_msg.uuid.to_string();
        println!("Updating worker: {}", worker.worker);
        update_worker_mysql(pooled_conn, &worker).unwrap();
        return Ok(worker);
        // Ok(worker);
      }
    }
  }
  let worker = WorkerMYSQLInsertable {
    coinid: new_msg.coin_id,
    userid: account.owner_id,
    worker: new_msg.worker_name.to_string(),
    hashrate: 0.0,
    difficulty: 0.0,
    owner_id: account.owner_id,
    owner_type: account.owner_type.to_string(),
    uuid: new_msg.uuid.to_string(),
    state: "connected".to_string(),
    ip: new_msg.ip.to_string(),
    version: new_msg.version.to_string(),
    password: new_msg.password.to_string(),
    algo: new_msg.algo.to_string(),
    mode: new_msg.mode.to_string(),
    stratum_id: new_msg.stratum_id.to_string(),
  };
  let new_worker = match insert_worker_mysql(pooled_conn, worker) {
    Ok(w) => w,
    Err(e) => return Err(format!("Failed to insert worker. e: {}", e))?,
  };
  println!("Inserting new worker: {}", new_worker.worker);
  Ok(new_worker)
}

fn handle_worker_disconnect(pooled_conn: &MysqlConnection, uuid: &String) {
  // go into the table and set uuid row to state disconnected
  if let Ok(mut worker) = get_worker_by_uuid_mysql(pooled_conn, uuid) {
    if worker.state == "devfee".to_string() {
      return;
    }
    worker.state = "disconnected".to_string();
    update_worker_mysql(pooled_conn, &worker).unwrap();
  }
}
fn handle_worker_devfee(pooled_conn: &MysqlConnection, uuid: &String) {
  // go into the table and set uuid row to state disconnected
  if let Ok(mut worker) = get_worker_by_uuid_mysql(pooled_conn, uuid) {
    worker.state = "devfee".to_string();
    update_worker_mysql(pooled_conn, &worker).unwrap();
  }
}
fn handle_difficulty_update(pooled_conn: &MysqlConnection, uuid: &String, difficulty: f64) {
  if let Ok(mut worker) = get_worker_by_uuid_mysql(pooled_conn, uuid) {
    worker.difficulty = difficulty;
    update_worker_mysql(pooled_conn, &worker).unwrap();
  }
}

fn handle_stratum_start(pooled_conn: &MysqlConnection, new_msg: &StratumStartNats) {
  let stratum_row = StratumMYSQLInsertable {
    pid: new_msg.pid,
    time: new_msg.time,
    started: new_msg.started,
    algo: new_msg.algo.to_string(),
    workers: 0,
    port: new_msg.port,
    symbol: new_msg.algo.to_string(),
    stratum_id: new_msg.stratum_id.to_string(),
  };
  match insert_stratum_mysql(pooled_conn, &stratum_row) {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  };
  update_workers_on_stratum_connect_mysql(pooled_conn, &stratum_row.stratum_id, new_msg.coin_id)
    .unwrap();
}

// fn parse_
// fn parse_password(password: &String) -> bool {
//   password.eq("password")
// }
// fn parse_ip(ip: &String)-> {

// // }
// fn handle_msg_subscribe() {}

// fn handle_msg_diff_update() {}

// fn handle_msg_disconnect() {}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_handle_msg_auth() {
    // let data = rmp_serde::to_vec("hi there tester").unwrap();

    // let res = handle_msg_auth(&data);
    // assert_eq!(res, data);
  }

  #[test]
  // fn test_parse_password() {
  //   let password = "password";
  //   assert_eq!(parse_password(&password.to_string()), true);
  // }
  #[test]
  fn test_get_or_insert_account_nim() {
    let mysql_pool_conn = establish_mysql_connection().unwrap().get().unwrap();
    let stratum_auth_nim = StratumAuthNatsNIM {
      username: "NQ11DGPGDBTX8H4J23QDL4A2D5DF7PDF9MAL".to_string(),
      coin_id: 2408,
      ip: "209.226.65.130".to_string(),
      difficulty: 64.0,
      version: "noncerpro-cuda-3.3.0".to_string(),
      consensus_mode: "dumb".to_string(),
      worker_name: "tesla".to_string(),
      uuid: "f6b5089d-75ee-4635-a5f2-74401c1d517e".to_string(),
      algo: "argon2d".to_string(),
      time: 1595121619,
      stratum_id: "2".to_string(),
      mode: "normal".to_string(),
      password: "".to_string(),
      party_pass: "".to_string(),
      pid: 20796,
    };
    get_or_insert_account_nim(&mysql_pool_conn, &stratum_auth_nim);
    let account =
      get_account_by_username_mysql(&mysql_pool_conn, &stratum_auth_nim.username).unwrap();
    assert_eq!(account.username, "NQ11DGPGDBTX8H4J23QDL4A2D5DF7PDF9MAL");
  }
}

#[test]
fn test_disconnect_worker() {
  let mysql_pool_conn = establish_mysql_connection().unwrap().get().unwrap();
  let worker = WorkerMYSQLInsertable {
    coinid: 1234,
    userid: 1234,
    worker: "test_worker".to_string(),
    hashrate: 0.0,
    owner_id: 1234,
    difficulty: 0.0,
    owner_type: "test_type".to_string(),
    uuid: "test_uuid".to_string(),
    state: "new".to_string(),
    ip: "192.test".to_string(),
    version: "test".to_string(),
    password: "test".to_string(),
    algo: "test".to_string(),
    mode: "test".to_string(),
    stratum_id: "test".to_string(),
  };
  insert_worker_mysql(&mysql_pool_conn, worker).unwrap();

  let uuid = "test_uuid".to_string();
  handle_worker_disconnect(&mysql_pool_conn, &uuid);
  let worker = get_worker_by_uuid_mysql(&mysql_pool_conn, &uuid).unwrap();
  assert_eq!(worker.state, "disconnected".to_string());
}

#[test]
fn test_stratum_start() {
  let mysql_pool_conn = establish_mysql_connection().unwrap().get().unwrap();
  let stratum_start_nats = StratumStartNats {
    pid: 12,
    time: 12,
    started: 12,
    algo: "test".to_string(),
    workers: 0,
    port: 12,
    symbol: "test".to_string(),
    stratum_id: "test".to_string(),
    coin_id: 0,
  };
  handle_stratum_start(&mysql_pool_conn, &stratum_start_nats);
  assert_eq!(1, 1);
}

#[test]
fn test_difficulty_update() {
  let mysql_pool_conn = establish_mysql_connection().unwrap().get().unwrap();
  let worker = WorkerMYSQLInsertable {
    coinid: 1234,
    userid: 1234,
    worker: "test_worker".to_string(),
    hashrate: 0.0,
    owner_id: 1234,
    owner_type: "test_type".to_string(),
    uuid: "test_uuid".to_string(),
    state: "new".to_string(),
    ip: "192.test".to_string(),
    version: "test".to_string(),
    password: "test".to_string(),
    algo: "test".to_string(),
    mode: "test".to_string(),
    stratum_id: "test".to_string(),
    difficulty: 0.0,
  };
  insert_worker_mysql(&mysql_pool_conn, worker).unwrap();

  let uuid = "test_uuid".to_string();

  handle_difficulty_update(&mysql_pool_conn, &uuid, 100.0);
  let worker = get_worker_by_uuid_mysql(&mysql_pool_conn, &uuid).unwrap();
  assert_eq!(worker.difficulty, 100.0);
}
