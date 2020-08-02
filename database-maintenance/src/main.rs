extern crate shared;

use diesel::prelude::*;
use dotenv::dotenv;
use futures::join;
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::accounts::{get_account_by_username_mysql, insert_account_mysql},
  helpers::stratums::{insert_stratum_mysql, update_stratum_by_pid_mysql},
  helpers::workers::{
    get_disconnected_worker_by_worker_name_mysql, get_worker_by_uuid_mysql, insert_worker_mysql,
    update_worker_mysql, update_workers_on_stratum_connect_mysql,
  },
  models::{StratumMYSQLInsertable, WorkerMYSQL, WorkerMYSQLInsertable},
  MysqlPool,
};
use shared::nats::models::{
  StratumAuthNatsNIM, StratumAuthResponseNats, StratumDevfeeNats, StratumDifficultyNats,
  StratumDisconnectNats, StratumHeartbeatNats, StratumStartNats,
};
use shared::nats::{establish_nats_connection, NatsConnection};
use std::env;
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

  // //-----------------------workers cleanup--------------------------------
  let _mysql_workers_cleanup_listener = mysql_workers_cleanup_listener(&env, &mysql_pool, &nc);

  // //-----------------------stratums cleanup--------------------------------
  let _mysql_stratums_cleanup_listener = mysql_stratums_cleanup_listener(&env, &mysql_pool, &nc);

  join!(
    _mysql_workers_cleanup_listener,
    _mysql_stratums_cleanup_listener
  );
}

fn mysql_workers_cleanup_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.heartbeat.2408");
  } else {
    subject = format!("stratum.heartbeat.2408");
  }
  let sub = match nc.queue_subscribe(&subject, "stratum_heartbeat_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum start listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      println!("Msg: {}", msg.subject);

      // DELETE FROM WORKERS WHERE PID IS NOT IN THE STRATUM TABLE

      // DELETE FROM WORKERS WHERE WORKER IS STALE
    }
  })
}
fn mysql_stratums_cleanup_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.heartbeat.2408");
  } else {
    subject = format!("stratum.heartbeat.2408");
  }
  let sub = match nc.queue_subscribe(&subject, "stratum_heartbeat_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum start listener failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      println!("Msg: {}", msg.subject);

      // DELETE FROM STRATUMS WHERE STRATUMS ARE STALE
    }
  })
}
