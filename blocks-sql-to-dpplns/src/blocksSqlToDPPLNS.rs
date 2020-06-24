extern crate shared;

use sentry::{capture_message, integrations::failure::capture_error, Level};
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::algorithms::get_algorithms_mysql,
  helpers::blocks::{get_blocks_unprocessed_mysql, update_block_to_unconfirmed_mysql},
  helpers::modes::get_modes_mysql,
  models::{AlgorithmMYSQL, BlockMYSQL, ModeMYSQL},
  MysqlPool,
};
use shared::nats::establish_nats_connection;
use shared::nats::models::DPPLNSBlockNats;
use shared::nats::models::KDABlockNats;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
const RUN_INTERVAL: u64 = 10;
use tokio::time::{interval_at, Duration, Instant};

#[tokio::main]
async fn main() {
  let _guard =
    sentry::init("https://f8ee06fb619843b1ae923d9111d855a9@sentry.watlab.icemining.ca/10");

  let mut tasks = Vec::new();
  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      panic!("Nats did not connect: {}", e);
    }
  };
  // let coins: Vec<i32> = vec![2422, 1234];
  //setup msqyl
  let mysql_pool = match establish_mysql_connection() {
    Ok(p) => p,
    Err(e) => panic!("MYSQL FAILED: {}", e),
  };

  capture_message("Moving blocks from sql to dpplns nats", Level::Info);

  // ------------------------SQL BLOCKS SELECT INTERVAL---------------------------
  {
    let block_select_task = tokio::spawn(async move {
      let mut interval = interval_at(
        Instant::now() + Duration::from_millis(RUN_INTERVAL * 1000),
        Duration::from_millis(RUN_INTERVAL * 1000),
      );

      loop {
        interval.tick().await;
        // grab a copy fo the pool to passed into the thread
        let mysql_pool = mysql_pool.clone();

        // grab a mysql pool connection
        let conn = match mysql_pool.get() {
          Ok(c) => c,
          Err(e) => {
            return Err(format!("Mysql connection failed")).unwrap();
          }
        };
        // get new blocks within the hour
        let blocks: Vec<BlockMYSQL> = match get_blocks_unprocessed_mysql(&conn) {
          Ok(blocks) => blocks,
          Err(e) => {
            panic!("no new blocks");
          }
        };

        for block in blocks {
          // set the block in mysql to unconfirmed, with 0 confirmations
          update_block_to_unconfirmed_mysql(&conn, &block);

          // pass block to dpplns
          let b = blockmysql_to_dpplnsblocknats(block);
          let msgpack_data = rmp_serde::to_vec(&b).unwrap();
          println!("PUBLISHING");
          match nc.publish("dpplns", msgpack_data) {
            Ok(val) => (),
            Err(err) => println!("err: {}", err),
          }
        }
      }
    });
    tasks.push(block_select_task);
  }

  for handle in tasks {
    handle.await.unwrap();
  }
}
// convert mysqlblock into natsblock
fn blockmysql_to_dpplnsblocknats(b: BlockMYSQL) -> DPPLNSBlockNats {
  DPPLNSBlockNats {
    id: b.id,
    coin_id: b.coin_id,
    height: b.height,
    time: b.time,
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
  }
}
