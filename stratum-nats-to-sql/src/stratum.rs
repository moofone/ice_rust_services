extern crate shared;

use diesel::prelude::*;
use futures::join;
use shared::db_mysql::{
  helpers::stratums::{insert_stratum_mysql, update_stratum_by_pid_mysql},
  helpers::workers::update_workers_on_stratum_connect_mysql,
  models::StratumMYSQLInsertable,
  MysqlPool,
};
use shared::nats::models::{StratumHeartbeatNats, StratumStartNats};
use shared::nats::NatsConnection;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn run_listeners(env: &String, mysql_pool: &MysqlPool, nc: &NatsConnection) {
  let start = stratum_start_listener(env, mysql_pool, nc);
  let heartbeat = stratum_heartbeat_listener(env, mysql_pool, nc);
  join!(start, heartbeat);
}
pub fn stratum_start_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.start.>");
  } else {
    subject = format!("stratum.start.>");
  }
  println!("listening to stratum starts");

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
pub fn stratum_heartbeat_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.heartbeat.>");
  } else {
    subject = format!("stratum.heartbeat.>");
  }
  println!("listening to heartbeats");
  let sub = match nc.queue_subscribe(&subject, "stratum_heartbeat_worker") {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum start listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      println!("Msg: {}", msg.subject);
      let stratum_heartbeat_nats = parse_msg_heartbeat(&msg.data).unwrap();

      // grab a mysql pool connection
      let conn = match mysql_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          panic!("error getting mysql connection e: {}",);
        }
      };
      handle_stratum_heartbeat(&conn, &stratum_heartbeat_nats);
    }
  })
}

fn parse_msg_start(msg: &Vec<u8>) -> Result<StratumStartNats, rmp_serde::decode::Error> {
  let start: StratumStartNats = match rmp_serde::from_read_ref(&msg) {
    Ok(start) => start,
    Err(e) => panic!("Error parsing Startum start nats. e: {}", e),
  };
  // println!("stratum start nats nim : {:?}", start);
  Ok(start)
}
fn parse_msg_heartbeat(msg: &Vec<u8>) -> Result<StratumHeartbeatNats, rmp_serde::decode::Error> {
  let heartbeat: StratumHeartbeatNats = match rmp_serde::from_read_ref(&msg) {
    Ok(heartbeat) => heartbeat,
    Err(e) => panic!("Error parsing Startum heartbeat nats. e: {}", e),
  };
  // println!("stratum heartbeat nats nim : {:?}", heartbeat);
  Ok(heartbeat)
}

fn handle_stratum_start(pooled_conn: &MysqlConnection, new_msg: &StratumStartNats) {
  let stratum_row = StratumMYSQLInsertable {
    pid: new_msg.pid,
    time: new_msg.time,
    started: new_msg.started,
    algo: new_msg.algo.to_string(),
    workers: 0,
    port: new_msg.port,
    symbol: new_msg.symbol.to_string(),
    stratum_id: new_msg.stratum_id.to_string(),
  };
  match insert_stratum_mysql(pooled_conn, &stratum_row) {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  };
  update_workers_on_stratum_connect_mysql(pooled_conn, &stratum_row.stratum_id, new_msg.coin_id)
    .unwrap();
}

fn handle_stratum_heartbeat(pooled_conn: &MysqlConnection, new_msg: &StratumHeartbeatNats) {
  // let stratum_row = StratumMYSQLInsertable {
  //   pid: new_msg.pid,
  //   time: new_msg.time,
  //   started: new_msg.started,
  //   algo: new_msg.algo.to_string(),
  //   workers: 0,
  //   port: new_msg.port,
  //   symbol: new_msg.algo.to_string(),
  //   stratum_id: new_msg.stratum_id.to_string(),
  // };
  // match insert_stratum_mysql(pooled_conn, &stratum_row) {
  //   Ok(_) => (),
  //   Err(err) => println!("Error: {}", err),
  // };
  let heartbeat_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i32;
  match update_stratum_by_pid_mysql(
    pooled_conn,
    new_msg.pid,
    &new_msg.stratum_id,
    &new_msg.symbol,
    heartbeat_time,
    new_msg.worker_count,
  ) {
    Ok(_) => (),
    Err(e) => println!(
      "Failed to update stratum with heartbeat of pid: {}, e: {}",
      new_msg.pid, e
    ),
  }
  // update_workers_on_stratum_connect_mysql(pooled_conn, &stratum_row.stratum_id, new_msg.coin_id)
  //   .unwrap();
}
