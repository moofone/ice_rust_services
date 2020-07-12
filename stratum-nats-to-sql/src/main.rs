extern crate shared;

// use sentry::{capture_message, integrations::failure::capture_error, Level};
use shared::db_mysql::{
  establish_mysql_connection, helpers::kdablocks::insert_kdablocks_mysql,
  models::KDABlockMYSQLInsertable, MysqlPool,
};
use shared::nats::establish_nats_connection;
use shared::nats::models::StratumAuthNats;
// use std::time::{Duration, SystemTime, UNIX_EPOCH};
// use tokio::time;

// const INSERTINTERVAL: u64 = 50;
// const DELETEINTERVAL: u64 = 2000;
// const WINDOW_LENGTH: u64 = 2 * 60 * 60;

fn handle_msg_auth(msg: &Vec<u8>) {
  let auth: StratumAuthNats = match rmp_serde::from_read_ref(&msg) {
    Ok(auth) => auth,
    Err(e) => panic!("Error parsing Startum auth nats. e: {}", e),
  };
}
// fn parse_
fn parse_password(password: &String) {}
// fn parse_ip(ip: &String)-> {

// }
fn handle_msg_subscribe() {}

fn handle_msg_diff_update() {}

fn handle_msg_disconnect() {}

#[tokio::main]
async fn main() {
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

  // capture_message("KDA Blocks listening", Level::Info);

  //-----------------------KDA BLOCKS LISTENER--------------------------------
  {
    // setup nats channel
    let subject = format!("kdablocks");
    let sub = match nc.queue_subscribe(&subject, "kdablocks_worker") {
      // let sub = match nc.subscribe(&subject) {
      Ok(sub) => sub,
      Err(e) => panic!("Queue kdablock coin failed: {}", e),
    };

    println!("spawning block task");
    // spawn a thread for this channel to listen to shares
    tasks.push(tokio::task::spawn(async move {
      // grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();
      println!("about to listen loop sub");

      for msg in sub.messages() {
        // // grab a copy to be passed into the thread
        let mysql_pool = mysql_pool.clone();
        println!("about to spawn thread");

        // spawn a thread for the block
        tokio::task::spawn_blocking(move || {
          // grab a mysql pool connection
          let conn = match mysql_pool.get() {
            Ok(conn) => conn,
            Err(e) => {
              // crash and sentry BIG ISSUE
              println!("Error mysql conn. e: {}", e);
              panic!("error getting mysql connection e: {}",);
            }
          };
        });
      }
    }));
    // tasks.push(blocks_task);
    // }
  }
  // loop {}
  for handle in tasks {
    handle.await.unwrap();
  }
}

// converts nats message to KDABlockNats
fn parse_kdablock(msg: &Vec<u8>) -> Result<KDABlockMYSQLInsertable, rmp_serde::decode::Error> {
  let b: KDABlockNats = match rmp_serde::from_read_ref(&msg) {
    Ok(b) => b,
    Err(err) => return Err(err),
  };
  let block = kdablocknats_to_blockmysqlinsertable(b);
  Ok(block)
}
fn kdablocknats_to_blockmysqlinsertable(b: KDABlockNats) -> KDABlockMYSQLInsertable {
  KDABlockMYSQLInsertable {
    coin_id: b.coin_id as i32,
    height: b.height,
    time: b.time as i32,
    userid: b.userid,
    workerid: b.workerid,
    confirmations: b.confirmations,
    amount: b.amount,
    difficulty: b.difficulty,
    difficulty_user: b.difficulty_user,
    blockhash: b.blockhash,
    algo: b.algo,
    category: b.category,
    stratum_id: b.stratum_id,
    mode: b.mode,
    party_pass: b.party_pass,
    chainid: b.chainid,
    node_id: b.node_id,
  }
}