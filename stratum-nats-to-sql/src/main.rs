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
  - idle (active but no share for awhile)
*/
// use sentry::{capture_message, integrations::failure::capture_error, Level};
use diesel::prelude::*;

// use dotenv::dotenv;
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::accounts::{get_account_by_username_mysql, insert_account_mysql},
  helpers::kdablocks::insert_kdablocks_mysql,
  helpers::workers::{
    get_worker_by_uuid_mysql, get_worker_by_worker_name_mysql, insert_worker_mysql,
    update_worker_mysql,
  },
  models::{AccountMYSQL, KDABlockMYSQLInsertable, WorkerMYSQL, WorkerMYSQLInsertable},
  MysqlPool,
};
use shared::nats::establish_nats_connection;
use shared::nats::models::{StratumAuthNatsNIM, StratumStartNats};
// use std::env;
use std::error::Error;
// use std::time::{Duration, SystemTime, UNIX_EPOCH};
// use tokio::time;

// const INSERTINTERVAL: u64 = 50;
// const DELETEINTERVAL: u64 = 2000;
// const WINDOW_LENGTH: u64 = 2 * 60 * 60;
struct generic_account {
  // generic account that we can convert stratumauthnats into to use for workers
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  // let _guard =
  //   sentry::init("https://f8ee06fb619843b1ae923d9111d855a9@sentry.watlab.icemining.ca/10");

  let mut tasks = Vec::new();
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
  {
    let subject = format!("stratum.auth.2408");
    let sub = match nc.queue_subscribe(&subject, "stratum_auth_worker") {
      // let sub = match nc.subscribe(&subject) {
      Ok(sub) => sub,
      Err(e) => panic!("Queue stratum auth listener failed: {}", e),
    };
    tasks.push(tokio::task::spawn(async move {
      //  grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();

      for msg in sub.messages() {
        println!("Msg: {}", msg.subject);
        let stratumAuthNIM = parse_msg_auth(&msg.data);

        // grab a mysql pool connection
        let conn = match mysql_pool.get() {
          Ok(conn) => conn,
          Err(e) => {
            // crash and sentry BIG ISSUE
            println!("Error mysql conn. e: {}", e);
            panic!("error getting mysql connection e: {}",);
          }
        };

        get_or_insert_account_nim(&conn, &stratumAuthNIM);
      }
    }))
  }

  // //-----------------------stratum Start listener--------------------------------
  {
    let subject = format!("stratum.start");
    let sub = match nc.queue_subscribe(&subject, "stratum_start_worker") {
      // let sub = match nc.subscribe(&subject) {
      Ok(sub) => sub,
      Err(e) => panic!("Queue stratum start listener failed: {}", e),
    };
    tasks.push(tokio::task::spawn(async move {
      //  grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();

      for msg in sub.messages() {
        println!("Msg: {}", msg.subject);
        let StratumStartNats = parse_msg_start(&msg.data);

        // grab a mysql pool connection
        let conn = match mysql_pool.get() {
          Ok(conn) => conn,
          Err(e) => {
            // crash and sentry BIG ISSUE
            println!("Error mysql conn. e: {}", e);
            panic!("error getting mysql connection e: {}",);
          }
        };

        // get_or_insert_account_nim(&conn, &stratumAuthNIM);
      }
    }))
  }

  // //-----------------------stratum difficutly listener--------------------------------
  {
    let subject = format!("stratum.difficulty.2408");
    let sub = match nc.queue_subscribe(&subject, "stratum_difficulty_worker") {
      // let sub = match nc.subscribe(&subject) {
      Ok(sub) => sub,
      Err(e) => panic!("Queue stratum difficulty listener failed: {}", e),
    };
    tasks.push(tokio::task::spawn(async move {
      //  grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();

      for msg in sub.messages() {
        println!("Msg: {}", msg.subject);
        let StratumStartNats = parse_msg_start(&msg.data);

        // grab a mysql pool connection
        let conn = match mysql_pool.get() {
          Ok(conn) => conn,
          Err(e) => {
            // crash and sentry BIG ISSUE
            println!("Error mysql conn. e: {}", e);
            panic!("error getting mysql connection e: {}",);
          }
        };

        // get_or_insert_account_nim(&conn, &stratumAuthNIM);
      }
    }))
  }

  for handle in tasks {
    handle.await.unwrap();
  }
  // //-----------------------KDA BLOCKS LISTENER--------------------------------
  // {
  //   // setup nats channel
  //   let subject = format!("kdablocks");
  //   let sub = match nc.queue_subscribe(&subject, "kdablocks_worker") {
  //     // let sub = match nc.subscribe(&subject) {
  //     Ok(sub) => sub,
  //     Err(e) => panic!("Queue kdablock coin failed: {}", e),
  //   };

  //   println!("spawning block task");
  //   // spawn a thread for this channel to listen to shares
  //   tasks.push(tokio::task::spawn(async move {
  //     // grab a copy fo the pool to passed into the thread
  //     let mysql_pool = mysql_pool.clone();
  //     println!("about to listen loop sub");

  //     for msg in sub.messages() {
  //       // // grab a copy to be passed into the thread
  //       let mysql_pool = mysql_pool.clone();
  //       println!("about to spawn thread");

  //       // spawn a thread for the block
  //       tokio::task::spawn_blocking(move || {
  //         // grab a mysql pool connection
  //         let conn = match mysql_pool.get() {
  //           Ok(conn) => conn,
  //           Err(e) => {
  //             // crash and sentry BIG ISSUE
  //             println!("Error mysql conn. e: {}", e);
  //             panic!("error getting mysql connection e: {}",);
  //           }
  //         };
  //       });
  //     }
  //   }));
  //   // tasks.push(blocks_task);
  //   // }
  // }
  // // loop {}
  // for handle in tasks {
  //   handle.await.unwrap();
  // }
}

// // converts nats message to KDABlockNats
// fn parse_kdablock(msg: &Vec<u8>) -> Result<KDABlockMYSQLInsertable, rmp_serde::decode::Error> {
//   let b: KDABlockNats = match rmp_serde::from_read_ref(&msg) {
//     Ok(b) => b,
//     Err(err) => return Err(err),
//   };
//   let block = kdablocknats_to_blockmysqlinsertable(b);
//   Ok(block)
// }
// fn kdablocknats_to_blockmysqlinsertable(b: KDABlockNats) -> KDABlockMYSQLInsertable {
//   KDABlockMYSQLInsertable {
//     coin_id: b.coin_id as i32,
//     height: b.height,
//     time: b.time as i32,
//     userid: b.userid,
//     workerid: b.workerid,
//     confirmations: b.confirmations,
//     amount: b.amount,
//     difficulty: b.difficulty,
//     difficulty_user: b.difficulty_user,
//     blockhash: b.blockhash,
//     algo: b.algo,
//     category: b.category,
//     stratum_id: b.stratum_id,
//     mode: b.mode,
//     party_pass: b.party_pass,
//     chainid: b.chainid,
//     node_id: b.node_id,
//   }
// }

fn parse_msg_auth(msg: &Vec<u8>) -> StratumAuthNatsNIM {
  let auth: StratumAuthNatsNIM = match rmp_serde::from_read_ref(&msg) {
    Ok(auth) => auth,
    Err(e) => panic!("Error parsing Startum auth nats. e: {}", e),
  };
  println!("stratum auth nats nim : {:?}", auth);
  auth
}
fn parse_msg_start(msg: &Vec<u8>) -> StratumStartNats {
  let stratum: StratumStartNats = match rmp_serde::from_read_ref(&msg) {
    Ok(stratum) => stratum,
    Err(e) => panic!("Error parsing Startum auth nats. e: {}", e),
  };
  println!("stratum start nats nim : {:?}", stratum);
  stratum
}

fn get_or_insert_account_nim(
  pooled_conn: &MysqlConnection,
  new_msg: &StratumAuthNatsNIM,
) -> Result<(), Box<dyn Error>> {
  let is_username = Some(new_msg.username.find('@'));
  println!("{}", &new_msg.username);
  if is_username != None {
    let account = match get_account_by_username_mysql(pooled_conn, &new_msg.username) {
      Ok(a) => a, // found
      Err(e) => {
        println!("{}", e);
        // not found
        match insert_account_mysql(pooled_conn, &new_msg.username, new_msg.coin_id as i32) {
          Ok(a) => a,
          Err(e) => panic!("insert failed. e: {}", e),
        }
      }
    };
  }
  Ok(())
}

fn handle_worker_connect(
  pooled_conn: &MysqlConnection,
  account: &AccountMYSQL,
  new_msg: &StratumAuthNatsNIM,
) {
  // if new_msg.worker_name.len() > 0 {
  //   if let Ok(worker) = get_worker_by_worker_name_mysql(
  //     pooled_conn,
  //     account.owner_id,
  //     &account.owner_type,
  //     &new_msg.worker_name,
  //   ) {
  //     if worker.state != "active" {
  //       update_worker_mysql(pooled_conn, &worker).unwrap();
  //     }
  //   }
  // } else {
  //   let worker = WorkerMYSQLInsertable {
  //     coin_id: new_msg.coin_id,
  //     user_id: account.owner_id,
  //     worker: new_msg.worker_name.to_string(),
  //     hashrate: 0.0,
  //     owner_id: account.owner_id,
  //     owner_type: account.owner_type.to_string(),
  //     uuid: new_msg.uuid.to_string(),
  //     state: "new".to_string(),
  //     ip_address: new_msg.ip.to_string(),
  //     version: new_msg.version.to_string(),
  //     password: new_msg.password.to_string(),
  //     algo: new_msg.algo.to_string(),
  //     mode: new_msg.mode.to_string(),
  //   };
  //   insert_worker_mysql(pooled_conn, worker).unwrap();
  // }

  // on worker connect, if named worker...
  //   check the table for worker name
  //   if worker name exists...
  //     check active
  //       if active is true...
  //         insert a new row - this will be the case for duplicate names, or edge case disconnects
  //       if active is false...
  //         update the row
  //   if worker name does not exists
  //     add a row, set state to active, set uuid
  // if not named worker
  //   inser a row with a uuid and set state to active
}

fn handle_worker_disconnect(pooled_conn: &MysqlConnection, uuid: &String) {
  // go into the table and set uuid row to state disconnected
  if let Ok(mut worker) = get_worker_by_uuid_mysql(pooled_conn, uuid) {
    worker.state = "disconnected".to_string();
    update_worker_mysql(pooled_conn, &worker);
  }
}

// fn parse_
fn parse_password(password: &String) -> bool {
  password.eq("password")
}
// fn parse_ip(ip: &String)-> {

// }
fn handle_msg_subscribe() {}

fn handle_msg_diff_update() {}

fn handle_msg_disconnect() {}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_handle_msg_auth() {
    let data = rmp_serde::to_vec("hi there tester").unwrap();

    // let res = handle_msg_auth(&data);
    // assert_eq!(res, data);
  }

  #[test]
  fn test_parse_password() {
    let password = "password";
    assert_eq!(parse_password(&password.to_string()), true);
  }

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
      stratum_id: 2,
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
