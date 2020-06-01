extern crate shared;

use shared::db_pg::establish_pg_connection;
use shared::db_pg::{
  helpers::shares::{delete_shares_older_than, insert_shares_pg},
  models::SharePGInsertable,
};
use shared::nats::establish_nats_connection;
use shared::nats::models::ShareNats;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
const INSERTINTERVAL: u64 = 50;
const DELETEINTERVAL: u64 = 2000;
const WINDOW_LENGTH: u64 = 2 * 60 * 60;

#[tokio::main]
async fn main() {
  let mut tasks = Vec::new();
  //setup nats
  let nc = establish_nats_connection();
  // let coins: Vec<i32> = vec![2422, 1234];
  let pg_pool = establish_pg_connection();
  let shares: VecDeque<SharePGInsertable> = VecDeque::new();
  let shares = Arc::new(Mutex::new(shares));

  //-----------------------SHARES LISTENER--------------------------------
  {
    // for coin in coins {
    // setup nats channel
    let channel = format!("shares.>");
    let sub = match nc.queue_subscribe(&channel, "shares_to_pg_worker") {
      Ok(sub) => sub,
      Err(err) => panic!("Queue Sub coin failed: {}"),
    };
    // prep queue to be used in a thread
    let shares = shares.clone();

    // spawn a thread for this channel to listen to shares
    let share_task = tokio::spawn(async move {
      for msg in sub.messages() {
        let share = match parse_share(&msg.data) {
          Ok(val) => val,
          Err(err) => {
            println!("Error parsing share: {}", err);
            continue;
          }
        };
        let mut shares = shares.lock().unwrap();
        shares.push_back(share);
      }
    });
    tasks.push(share_task);
    // }
  }

  //----------------------------INSERT SHARES-------------------------------------
  {
    let pg_pool = pg_pool.clone();
    let shares = shares.clone();

    let insert_task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(INSERTINTERVAL));
      loop {
        interval.tick().await;

        // lock the shares queue
        let mut shares = shares.lock().unwrap();

        // create a new vec for insertable shares
        let mut shares_vec: Vec<SharePGInsertable> = Vec::new();
        if shares.len() > 0 {
          println!("Shares Moved from queue to vec {}", shares.len());
        }

        // empty the queue into the vec
        while shares.len() > 0 {
          shares_vec.push(shares.pop_front().unwrap());
        }
        if shares_vec.len() > 0 {
          println!("Shares to be inserted {}", shares_vec.len());
        }
        let pg_pool = pg_pool.clone();

        tokio::spawn(async move {
          let conn = match pg_pool.get() {
            Ok(conn) => conn,
            Err(err) => panic!("error getting mysql connection: {}", err),
          };

          if shares_vec.len() > 0 {
            // insert the array
            insert_shares_pg(&conn, shares_vec).expect("Share insert failed");
          }
        });
      }
    });
    tasks.push(insert_task);
  }
  //----------------------------CLEAN SHARES FROM PG------------------------------
  {
    let pg_pool = pg_pool.clone();

    let clean_task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(DELETEINTERVAL));
      loop {
        interval.tick().await;
        let conn = match pg_pool.get() {
          Ok(conn) => conn,
          Err(err) => panic!("error getting mysql connection: {}", err),
        };
        let time_window_start = SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_secs()
          - WINDOW_LENGTH;
        //println!("DELETING SHARES");
        match delete_shares_older_than(&conn, time_window_start as i64) {
          Ok(_) => (),
          Err(err) => println!("Deleting shares failed: {}", err),
        };
      }
    });
    tasks.push(clean_task);
  }

  for handle in tasks {
    handle.await.unwrap();
  }
}
fn parse_share(msg: &Vec<u8>) -> Result<SharePGInsertable, rmp_serde::decode::Error> {
  // Some JSON input data as a &str. Maybe this comes from the user.
  // Parse the string of data into serde_json::Value.
  // println!("msg: {:?}", msg);
  let s: ShareNats = match rmp_serde::from_read_ref(&msg) {
    Ok(s) => s,
    Err(err) => return Err(err),
  };
  // println!("Share: {:?}", &s);
  let share = sharenats_to_sharepginsertable(s);
  Ok(share)
}
fn sharenats_to_sharepginsertable(s: ShareNats) -> SharePGInsertable {
  SharePGInsertable {
    user_id: s.user_id,
    worker_id: s.worker_id,
    coin_id: s.coin_id as i16,
    time: s.timestamp,
    difficulty: s.difficulty,
    // share_diff: s.share_diff,
    block_diff: s.block_diff,
    algo: s.algo as i16,
    mode: s.mode as i16,
    block_reward: s.block_reward,
    party_pass: s.party_pass,
    stratum_id: s.stratum_id,
  }
}
