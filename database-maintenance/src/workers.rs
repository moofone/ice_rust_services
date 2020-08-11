extern crate shared;

use futures::join;
use shared::db_mysql::{
  helpers::workers::{delete_disconnected_workers_mysql, delete_stale_workers_mysql},
  MysqlPool,
};
use shared::nats::NatsConnection;
use std::time::{SystemTime, UNIX_EPOCH};
pub async fn run_jobs(env: &String, mysql_pool: &MysqlPool, nc: &NatsConnection) {
  let _mysql_workers_cleanup_listener = mysql_workers_cleanup_listener(&env, &mysql_pool, &nc);

  join!(_mysql_workers_cleanup_listener);
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
    subject = format!("dev.events.maintenance.workers");
  } else {
    subject = format!("events.maintenance.workers");
  }
  let sub = match nc.queue_subscribe(&subject, "events_maintenance_workers_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue maintenance workers table failed: {}", e),
  };
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();

    for msg in sub.messages() {
      println!("Msg: {}", msg.subject);
      let conn = match mysql_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          continue;
          // panic!("error getting mysql connection e: {}",);
        }
      };
      // DELETE FROM WORKERS WHERE PID IS NOT IN THE STRATUM TABLE

      // DELETE FROM WORKERS WHERE WORKER IS STALE
      let lookback_disconnected_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 60 * 5; //30 minutes

      let lookback_stale_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 60 * 15; // 30 minutes
      println!("Deleting stale workers");
      delete_stale_workers_mysql(&conn, 2423, lookback_stale_time as i32);
      delete_stale_workers_mysql(&conn, 2408, lookback_stale_time as i32);
      delete_disconnected_workers_mysql(&conn, 2408, lookback_disconnected_time as i32);
      delete_disconnected_workers_mysql(&conn, 2408, lookback_disconnected_time as i32);
    }
  })
}
