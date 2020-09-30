extern crate shared;
use diesel::prelude::*;
use futures::join;
use shared::db_mysql::{
  helpers::workers::{get_worker_by_uuid_mysql, update_worker_mysql},
  MysqlPool,
};
use shared::nats::models::{StratumDevfeeNats, StratumDisconnectNats};
use shared::nats::NatsConnection;

pub async fn run_listeners(env: &String, mysql_pool: &MysqlPool, nc: &NatsConnection) {
  let dc = stratum_disconnect_listener(env, mysql_pool, nc);
  let devfee = stratum_devfee_listener(env, mysql_pool, nc);
  join!(dc, devfee);
}
pub fn stratum_disconnect_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "prod" {
    subject = format!("stratum.disconnect.>");
  } else {
    subject = format!("{}.stratum.disconnect.>", env);
  }
  println!("listening to worker disconnects");

  let sub = match nc.queue_subscribe(&subject, "stratum_disconnect_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum disconnect listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    let mut counter = 0;
    for msg in sub.messages() {
      println!("Msg: {} (printing every 100)", msg.subject);

      counter += 1;
      if counter % 100 == 0 {
        println!("Msg: {} (printing every 100)", msg.subject);
        counter = 0;
      }
      // println!("Msg: {}", msg.subject);
      if let Ok(stratum_disconnect_nats) = parse_msg_disconnect(&msg.data) {
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
    }
  })
}
pub fn stratum_devfee_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "prod" {
    subject = format!("stratum.devfee.>");
  } else {
    subject = format!("{}.stratum.devfee.>", env);
  }
  println!("listening to worker devfees");

  let sub = match nc.queue_subscribe(&subject, "stratum_devfee_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum devfee listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    let mut counter = 0;
    for msg in sub.messages() {
      counter += 1;
      if counter % 100 == 0 {
        println!("Msg: {} (printing every 100)", msg.subject);
        counter = 0;
      }
      // println!("Msg: {}", msg.subject);
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
fn parse_msg_disconnect(msg: &Vec<u8>) -> Result<StratumDisconnectNats, rmp_serde::decode::Error> {
  let disconnect: StratumDisconnectNats = match rmp_serde::from_read_ref(&msg) {
    Ok(disconnect) => disconnect,
    Err(e) => return Err(e), //panic!("Error parsing Startum disconnect nats. e: {}", e),
  };
  // println!("stratum disconnect nats nim : {:?}", disconnect);
  Ok(disconnect)
}
fn parse_msg_devfee(msg: &Vec<u8>) -> Result<StratumDevfeeNats, rmp_serde::decode::Error> {
  let devfee: StratumDevfeeNats = match rmp_serde::from_read_ref(&msg) {
    Ok(devfee) => devfee,
    Err(e) => panic!("Error parsing Startum devfee nats. e: {}", e),
  };
  // println!("stratum devfee nats nim : {:?}", devfee);
  Ok(devfee)
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
