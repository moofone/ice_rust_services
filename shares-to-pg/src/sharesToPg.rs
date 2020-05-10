extern crate shared;

use shared::db_pg::establish_pg_connection;
use shared::db_pg::{
  helpers::shares::{insert_shares_pg, select_shares_count_pg, select_shares_newer_pg},
  models::SharePGInsertable,
};
use shared::nats::establish_nats_connection;
use shared::nats::models::ShareNats;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
static INSERTINTERVAL: u64 = 50;

#[tokio::main]
async fn main() {
  let mut tasks = Vec::new();
  //setup nats
  let nc = establish_nats_connection();
  let coins: Vec<i32> = vec![2422];
  let pg_pool = establish_pg_connection();
  let shares: VecDeque<SharePGInsertable> = VecDeque::new();
  let shares = Arc::new(Mutex::new(shares));

  //-----------------------SHARES LISTENER--------------------------------
  {
    for coin in coins {
      // setup nats channel
      let channel = format!("shares.{}", coin.to_string());
      let sub = nc.queue_subscribe(&channel, "shares_to_pg_worker").unwrap();
      // prep queue to be used in a thread
      let shares = shares.clone();

      // spawn a thread for this channel to listen to shares
      let share_task = tokio::spawn(async move {
        for msg in sub.messages() {
          let share = parse_share(&msg.data);
          let mut shares = shares.lock().unwrap();
          shares.push_back(share);
        }
      });
      tasks.push(share_task);
    }
  }

  //----------------------------INSERT SHARES-------------------------------------
  {
    let pg_pool = pg_pool.clone();
    let shares = shares.clone();

    let insert_task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(INSERTINTERVAL));
      let conn = pg_pool.get().unwrap();
      loop {
        interval.tick().await;

        // lock the shares queue
        let mut shares = shares.lock().unwrap();

        // create a new vec for insertable shares
        let mut shares_vec: Vec<SharePGInsertable> = Vec::new();
        println!("Shares Moved from queue to vec {}", shares.len());

        // empty the queue into the vec
        while shares.len() > 0 {
          shares_vec.push(shares.pop_front().unwrap());
        }
        println!("Shares to be inserted {}", shares_vec.len());

        // insert the array
        insert_shares_pg(&conn, shares_vec).expect("Share insert failed");
      }
    });
    tasks.push(insert_task);
  }
  for handle in tasks {
    handle.await.unwrap();
  }
}
fn parse_share(msg: &Vec<u8>) -> SharePGInsertable {
  // Some JSON input data as a &str. Maybe this comes from the user.
  // Parse the string of data into serde_json::Value.
  let s: ShareNats = serde_json::from_slice(&msg).unwrap();
  let share = sharenats_to_sharepginsertable(s);
  return share;
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
