extern crate shared;

use sentry::{capture_message, integrations::failure::capture_error, Level};
use shared::db_mysql::{
  establish_mysql_connection, helpers::kdablocks::insert_kdablocks_mysql,
  models::KDABlockMYSQLInsertable, MysqlPool,
};
use shared::nats::establish_nats_connection;
use shared::nats::models::KDABlockNats;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;

const INSERTINTERVAL: u64 = 50;
const DELETEINTERVAL: u64 = 2000;
const WINDOW_LENGTH: u64 = 2 * 60 * 60;

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

  capture_message("KDA Blocks listening", Level::Info);

  //-----------------------KDA BLOCKS LISTENER--------------------------------
  {
    // for coin in coins {
    // setup nats channel
    let channel = format!("kdablocks");
    let sub = match nc.queue_subscribe(&channel, "kdablock_worker") {
      Ok(sub) => sub,
      Err(err) => panic!("Queue kdablock coin failed: {}"),
    };

    // spawn a thread for this channel to listen to shares
    let block_task = tokio::spawn(async move {
      // grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();

      for msg in sub.messages() {
        println!("kdablock");

        // parse the block
        let kdablock = match parse_kdablock(&msg.data) {
          Ok(val) => val,
          Err(err) => {
            println!("Error parsing kdablock: {}", err);
            continue;
          }
        };

        // grab a mysql pool connection
        let conn = match mysql_pool.get() {
          Ok(c) => c,
          Err(e) => {
            return Err(format!(
              "Mysql connection failed on block: {}",
              kdablock.height
            ))
            .unwrap();
          }
        };

        // create a queue of blocks ( incase we want to scale or bulk insert)
        let mut kdablocks: Vec<KDABlockMYSQLInsertable> = Vec::new();
        kdablocks.push(kdablock);
        insert_kdablocks_mysql(&conn, kdablocks);
        println!("block inserted");
        // let mut shares = shares.lock().unwrap();
        // shares.push_back(share);
      }
    });
    tasks.push(block_task);
    // }
  }

  // //----------------------------INSERT SHARES-------------------------------------
  // {
  //   let pg_pool = pg_pool.clone();
  //   let shares = shares.clone();

  //   let insert_task = tokio::spawn(async move {
  //     let mut interval = time::interval(Duration::from_millis(INSERTINTERVAL));
  //     loop {
  //       interval.tick().await;

  //       // lock the shares queue
  //       let mut shares = shares.lock().unwrap();

  //       // create a new vec for insertable shares
  //       let mut shares_vec: Vec<SharePGInsertable> = Vec::new();
  //       if shares.len() > 0 {
  //         println!("Shares Moved from queue to vec {}", shares.len());
  //       }

  //       // empty the queue into the vec
  //       while shares.len() > 0 {
  //         shares_vec.push(shares.pop_front().unwrap());
  //       }
  //       if shares_vec.len() > 0 {
  //         println!("Shares to be inserted {}", shares_vec.len());
  //       }
  //       let pg_pool = pg_pool.clone();

  //       tokio::spawn(async move {
  //         let conn = match pg_pool.get() {
  //           Ok(conn) => conn,
  //           Err(err) => panic!("error getting mysql connection: {}", err),
  //         };

  //         if shares_vec.len() > 0 {
  //           // insert the array
  //           insert_shares_pg(&conn, shares_vec).expect("Share insert failed");
  //         }
  //       });
  //     }
  //   });
  //   tasks.push(insert_task);
  // }

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
