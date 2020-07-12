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
const INSERTINTERVAL: u64 = 1000;
const DELETEINTERVAL: u64 = 2000;
const WINDOW_LENGTH: u64 = 24 * 60 * 60;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
  let mut tasks = Vec::new();

  //TODO add sentry

  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      // crash and sentry ULTRA IMPORTANT
      panic!("Nats did not connect: {}", e);
    }
  };
  // establish PG pool
  let pg_pool = match establish_pg_connection() {
    Ok(p) => p,
    Err(e) => {
      // crash and sentry ULTRA IMPORTANT
      panic!("PG pool failed to connect: {}", e)
    }
  };

  // setup our shares queueu
  let shares: VecDeque<SharePGInsertable> = VecDeque::new();
  let shares = Arc::new(Mutex::new(shares));

  //-----------------------SHARES LISTENER--------------------------------
  {
    // for coin in coins {
    // setup nats channel
    let channel = format!("shares.>");
    let sub = match nc.queue_subscribe(&channel, "shares_to_pg_worker") {
      Ok(sub) => sub,
      Err(err) => {
        // crash and sentry ULTRA IMPORTANT
        panic!("Queue Sub coin failed: {}", err)
      }
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
        // if shares.len() > 0 {
        //   println!("Shares Moved from queue to vec {}", shares.len());
        // }

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
            Err(err) => {
              // crash and sentry BIG ISSUE
              panic!("error getting PG connection: {}", err)
            }
          };

          if shares_vec.len() > 0 {
            // insert the array
            match insert_shares_pg(&conn, shares_vec) {
              Ok(_) => (),
              Err(err) => {
                // sentry that we failed to insert shares
                println!("Failed to insert shares. err: {}", err);
              }
            }
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
          Err(err) => {
            // crash - sentry, why cant we get a connection
            panic!("error getting PG connection: {}", err)
          }
        };
        let time_window_start = SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_secs()
          - WINDOW_LENGTH;
        //println!("DELETING SHARES");
        match delete_shares_older_than(&conn, time_window_start as i64) {
          Ok(_) => (),
          Err(err) => {
            // move on and sentry - why cant we delete?
            println!("Deleting shares failed: {}", err)
          }
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
    coin_id: s.coin_id,
    time: s.timestamp,
    difficulty: s.difficulty,
    share_diff: s.share_diff,
    block_diff: s.block_diff,
    algo: s.algo,
    mode: s.mode,
    block_reward: s.block_reward,
    party_pass: s.party_pass,
    stratum_id: s.stratum_id,
  }
}
