/*
use  ShareNats
parse shareNats -> convert to ShareMin
Convert SharePGInsertable into ShareMin
Convert ShareNats into ShareMin
probably just feed in values into calc_score instead of a share

use DPPLNSBlockNats
parse DPPLNSBlockNats and work with it as is

fix enums to use from_i8 and from_string and mabye to_i8 and to_string?

use EarningMySQLInsertable OR EarningPGInsertable or both i guess...
create genertic Earning that can be to_EarningMYSQLInsertable and to_EarningPGINsertable

final goal (ShareMin => min usable data for a share that will go into the queue)
  - load in SharePG from PG -> convert to ShareMin -> Process (add to queue/dict)
  - Receive ShareNats from Nats -> convert to ShareMin -> Process (add to queue/dict)
  - Trim Queue of ShareMin and AlmightyDict
  - Receive DPPLNSBlockNats from Nats -> Calc earning and generate Vec<Earning>
    - convert Vec<Earning> to Vec<EarningMYSQLInsertable> -> Insert into mysql
    - convert Vec<Earning> create Vec<EarningPGInsertable> -> Insert into PG

*/

const DEBUG_MODE: bool = false;

extern crate shared;
use diesel::prelude::*;
use shared::db_mysql::{
  establish_mysql_connection, helpers::earnings::insert_earnings_mysql,
  models::EarningMYSQLInsertable, MysqlPool,
};
use shared::nats::{
  establish_nats_connection,
  models::{DPPLNSBlockNats, ShareNats},
};

use shared::db_pg::{
  establish_pg_connection, helpers::shares::select_shares_newer_pg, models::SharePg,
};

// use shared::enums::*;
use sentry::{capture_message, integrations::failure::capture_error, Level};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{interval_at, Duration, Instant};

// constants
const NORMAL_FEE: f64 = 0.01;
const SOLO_FEE: f64 = 0.02;
const PARTY_FEE: f64 = 0.02;
const WINDOW_LENGTH: u64 = 300; //2 * 60 * 60;  //s
                                //const DECAY_COUNT: u64 = WINDOW_LENGTH / DECAY_INTERVAL; // 1/decay_count cant be infiniti repeating
                                //const DECAY_FACTOR: f64 = 1.0 / DECAY_COUNT as f64;
const DECAY_INTERVAL: u64 = 15; // s
                                // Minimum data required to be stored in the queue
                                // share minified is used to update the hashmap
#[derive(Debug, Clone)]
struct ShareMinified {
  user_id: i32,
  coin_id: i16,
  algo: i16,
  time: i64,
  share_payout: f64,
  mode: i16,
  party_pass: String,
  // decay_counter: i16,
}
// convert incoming shareNats into shareminified
impl From<ShareNats> for ShareMinified {
  fn from(s: ShareNats) -> Self {
    let score = calc_score(
      s.mode,
      s.coin_id,
      s.user_id,
      s.difficulty,
      s.share_diff,
      s.block_diff,
      s.block_reward,
    );
    ShareMinified {
      user_id: s.user_id,
      coin_id: s.coin_id,
      algo: s.algo,
      time: s.timestamp,
      share_payout: score,
      mode: s.mode,
      party_pass: s.party_pass,
      // decay_counter: 0,
    }
  }
}
// convert incoming shares from PG to share minified
impl From<SharePg> for ShareMinified {
  fn from(s: SharePg) -> Self {
    let score = calc_score(
      s.mode,
      s.coin_id,
      s.user_id,
      s.difficulty,
      s.share_diff,
      s.block_diff,
      s.block_reward,
    );
    ShareMinified {
      user_id: s.user_id,
      coin_id: s.coin_id,
      algo: s.algo,
      time: s.time,
      share_payout: score,
      mode: s.mode,
      party_pass: s.party_pass,
      // decay_counter: 0,
    }
  }
}

// generic earning - to be converted into PG and MYSQL earnings
#[derive(Debug, PartialEq)]
struct Earning {
  user_id: i32,
  coin_id: i32,
  block_id: i32,
  create_time: i64,
  status: i32,
  amount: f64,
  mode: i32,
  algo: i16,
  stratum_id: i16,
  party_pass: String,
}
impl Earning {
  // fn to_EarningMYSQLInsertable() -> EarningMYSQLInsertable {}
  // fn to_EarningPGINsertable() -> EarningPGInsertable {}
}

// hashmap to hold userid's with the current dpplns scores
type UserScoreMapType = HashMap<String, HashMap<i32, f64>>;
// queue to hold shares (queue length of dpplns window)
type ShareQueueType = VecDeque<ShareMinified>;
// hashmap to hold earnings for each block
type EarningMapType = HashMap<i32, f64>;
/*
let algorithm = match algorithms.get(&b.algo) {
  Some(a) => b.id,
  None => panic!("why does this algo not exist"),
};
let mode = match modes.get(&b.mode) {
  Some(m) => m.id,
  None => panic!("why does this mode not exist"),
};
*/
#[tokio::main]
async fn main() {
  // TODO: This should be moved to shared so each module / project has one spot to initialize
  let _guard =
    sentry::init("https://689607b053ac4fbb81ee82a08a8aa18a@sentry.watlab.icemining.ca/9");

  // create base structs to be used across threads
  let shares_queue = Arc::new(Mutex::new(ShareQueueType::new()));
  let user_scores_map = Arc::new(Mutex::new(UserScoreMapType::new()));
  // lock the shares and scores, load last window from database
  {
    let mut sha = shares_queue.lock().unwrap();
    let mut sco = user_scores_map.lock().unwrap();
    load_shares_from_db(&mut *sha, &mut *sco);
    rebuild_decayed_map_and_trim_queue(&mut *sco, &mut *sha)
  }

  capture_message("DPPLNS loaded shares and is now live", Level::Info);

  // println!("{:?}", user_scores_map);

  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      panic!("Nats did not connect: {}", e);
    }
  };

  // setup threads array so the program doesnt end right away
  let mut tasks = Vec::new();

  //-----------------------SHARES LISTENER--------------------------------
  {
    // for coin in coins {
    // let channel = format!("shares.>");
    let sub = match nc.subscribe("shares.>") {
      Ok(s) => s,
      Err(e) => panic!("Nats sub to shares failed: {}", e),
    };
    let shares = shares_queue.clone();
    let user_scores = user_scores_map.clone();
    let share_task = tokio::spawn(async move {
      for msg in sub.messages() {
        let share = match parse_share(&msg.data) {
          Ok(share) => {
            let mut sha = shares.lock().unwrap();
            let mut sco = user_scores.lock().unwrap();
            handle_share(&mut *sco, &mut *sha, share);
          }
          Err(err) => {
            println!("share parse failed: {}", err);
            capture_message(&format!("Share parse failed: {}", err), Level::Error);
            ()
          }
        };
      }
    });
    tasks.push(share_task);
    // }
  }
  //-----------------------BLOCKS LISTENER----------------------------
  {
    // subscribe to the nats blocks channel
    let block_sub = match nc.queue_subscribe("blocks", "dpplns_worker") {
      Ok(b) => b,
      Err(e) => panic!("Nats sub to blocks failed: {}", e),
    };

    //setup msqyl
    let mysql_pool = match establish_mysql_connection() {
      Ok(p) => p,
      Err(e) => panic!("MYSQL FAILED: {}", e),
    };

    // grab a copy of user_users to be passed into listener thread
    let user_scores = user_scores_map.clone();

    // spawn a new task to listen for new blocks
    let block_task = tokio::spawn({
      async move {
        // loop through the blocking messages() listener waiting for blocks
        for msg in block_sub.messages() {
          // grab a copy fo the pool to passed into the thread
          let mysql_pool = mysql_pool.clone();

          // grab a copy of user_users to be passed into processing thread
          let user_scores = user_scores.clone();

          tokio::spawn(async move {
            // try to parse the block
            let sco = user_scores.lock().unwrap();
            match handle_block(&msg, &sco, mysql_pool) {
              Ok(_) => (),
              Err(err) => println!("block failed"),
            }
            // match parse_block(&msg.data) {
            //   Ok(block) => {
            //     // block parsed fine, lock the scores map
            //     let sco = user_scores.lock().unwrap();

            //     // generate earnings_dict, if none returned, encounted a weird issue
            //     match dpplns(&block, &*sco) {
            //       Some(earnings_dict) => {
            //         // drop the scores map to be used elsewhere
            //         drop(sco);

            //         // grab a mysql pool connection and insert earnings
            //         match mysql_pool.get() {
            //           Ok(mut conn) => insert_earnings(&block, earnings_dict, &mut conn),
            //           Err(err) => {
            //             capture_message(&format!("Mysql failed to get a pool connection to insert earnings for blockid: {},: {}", &block.id,err), Level::Error);
            //             ()
            //           }
            //         }
            //       }
            //       None => {
            //         capture_message(
            //           &format!("Block was not in dict: {}", &block.id),
            //           Level::Error,
            //         );
            //         ()
            //       }
            //     }
            //   }
            //   Err(err) => {
            //     capture_message(&format!("Block parse failed: {}", err), Level::Error);
            //     ()
            //   }
            // };
          });
        }
      }
    });
    tasks.push(block_task);
  }

  //--------------------------UPDATE MAP TIMER----------------------------
  {
    let shares = shares_queue.clone();
    let user_scores = user_scores_map.clone();
    let decay_task = tokio::spawn(async move {
      let mut interval = interval_at(
        Instant::now() + Duration::from_millis(DECAY_INTERVAL * 1000),
        Duration::from_millis(DECAY_INTERVAL * 1000),
      );

      loop {
        interval.tick().await;

        let mut sha = shares.lock().unwrap();
        let mut sco = user_scores.lock().unwrap();
        rebuild_decayed_map_and_trim_queue(&mut *sco, &mut *sha);
      }
    });
    tasks.push(decay_task);
  }

  for handle in tasks {
    handle.await.unwrap();
  }
}

// main dpplns function, takes in a block and generates the earnings for each user
fn dpplns(block: &DPPLNSBlockNats, dict: &UserScoreMapType) -> Result<EarningMapType, String> {
  let mut f = NORMAL_FEE;
  let mut earnings_dict: EarningMapType = HashMap::new();
  // let log = "".to_string();

  // setup block fees and log
  match block.mode {
    0 => {
      println!(
        "Normal Block: {} Reward: {}, Fee: {}%",
        block.coin_id,
        block.amount,
        f * 100.0
      );
    }
    1 => {
      let share_payout = (1.0 - PARTY_FEE as f64) * block.amount;
      let party_pass = &block.party_pass;
      f = PARTY_FEE;
      println!(
        "Block Party: {} Payout: {}, Fee: {}%",
        &party_pass,
        share_payout,
        f * 100.0
      );
    }
    2 => {
      let share_payout = (1.0 - SOLO_FEE as f64) * block.amount;
      println!(
        "YOLO SOLO!! B: {} Payout: {}, Fee: {}%",
        block.amount,
        share_payout,
        SOLO_FEE * 100.0
      );
      earnings_dict.insert(block.userid, share_payout);
      return Ok(earnings_dict);
    }
    _ => {
      println!("WHY DIDNT THIS BLOCK HAVE A MODE????");
    }
  }

  // copy proper dict over
  let key: String = dict_key_gen(block.mode, block.coin_id, block.algo, &block.party_pass);
  let key_exists = dict.contains_key(&key);

  if key_exists == false {
    println!("block failed, invalid dict key: {}", &key);
    return Err(format!("Block Failed in dpplns, blockid: {}", block.id).to_string());
    // panic!("block failed");
  }
  earnings_dict = dict.get(&key).unwrap().clone();

  let mut total_earned = 0.0;
  let tgt_block_payout = (1.0 - f) * block.amount;
  //log += format!(" tgt: {}", tgt_block_payout).to_string();

  for (_, val) in earnings_dict.iter() {
    total_earned += val;
  }

  let prop_factor = tgt_block_payout / total_earned;

  let mut sum = 0.0;
  for (_, val) in earnings_dict.iter_mut() {
    let scaled_amount = *val * prop_factor;
    *val = scaled_amount;
    sum += scaled_amount;
  }

  // println!(
  //   "Payment Complete! {} Total Earnings: {} Target Block Payout: {} Elapsed Time: {}",
  //   earnings_dict.len(),
  //   sum,
  //   tgt_block_payout,
  //   0
  // );
  if earnings_dict.len() == 0 {
    panic!("earnings dict empty");
  };

  return Ok(earnings_dict);
}

// inserts earnings into the databases
fn insert_earnings(
  block: &DPPLNSBlockNats,
  earnings_dict: EarningMapType,
  pool_conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
  let mut earnings: Vec<EarningMYSQLInsertable> = Vec::new();
  let create_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();
  for (&user_id, val) in earnings_dict.iter() {
    earnings.push(EarningMYSQLInsertable {
      userid: user_id,
      coinid: block.coin_id as i32,
      blockid: block.id,
      create_time: create_time as i32,
      status: 0,
      amount: *val,
      mode: {
        match block.mode {
          0 => "normal".to_string(),
          1 => "party".to_string(),
          2 => "solo".to_string(),
          _ => {
            println!("WHY DIDNT THIS BLOCK HAVE A MODE????");
            "normal".to_string()
          }
        }
      },
      stratum_id: block.stratum_id,
      algo: block.algo,
      party_pass: block.party_pass.to_string(),
    });
  }

  if DEBUG_MODE == false {
    let e = insert_earnings_mysql(pool_conn, earnings);
    match e {
      Ok(_) => (), //println!("Earnings Inserted"),
      Err(err) => {
        capture_message(
          &format!(
            "Failed to insert earnings mysql for blockid: {},: {}",
            &block.id, err
          ),
          Level::Error,
        );
        return Err(err);
      }
    }
  }
  Ok(())
}

// generates a key from the share to be used in the user scores hashmap
fn dict_key_gen(mode: i16, coin_id: i16, algo: i16, party_pass: &String) -> String {
  let key;
  match mode {
    0 => key = format!("N:{}-{}", coin_id.to_string(), algo.to_string()),
    1 => {
      key = format!(
        "P:{}-{}-{}",
        coin_id.to_string(),
        algo.to_string(),
        party_pass.to_string(),
      )
    }
    _ => key = "".to_string(),
  }
  return key;
}

// converts nats message to sharenats and then to share minified
fn parse_share(msg: &Vec<u8>) -> Result<ShareMinified, rmp_serde::decode::Error> {
  // Some JSON input data as a &str. Maybe this comes from the user.
  // Parse the string of data into serde_json::Value.
  let s: ShareNats = match rmp_serde::from_read_ref(&msg) {
    Ok(s) => s,
    Err(err) => return Err(err),
  };
  let share = ShareMinified::from(s);
  Ok(share)
}

// converts nats message to DPPLNSBlockNats
fn parse_block(msg: &Vec<u8>) -> Result<DPPLNSBlockNats, rmp_serde::decode::Error> {
  let b: DPPLNSBlockNats = match rmp_serde::from_read_ref(&msg) {
    Ok(b) => b,
    Err(err) => return Err(err),
  };
  Ok(b)
}

// add share to queue and add share to map
fn handle_share(dict: &mut UserScoreMapType, shares: &mut ShareQueueType, share: ShareMinified) {
  // pass reference of the share to map to be updated
  update_map_with_new_share(dict, &share);
  // move the share to the queue to be added
  add_share_to_queue(shares, share);
}

// handle block
fn handle_block(
  msg: &nats::Message,
  sco: &UserScoreMapType,
  mysql_pool: MysqlPool,
) -> Result<(), Box<dyn Error>> {
  // parse the block
  let block = parse_block(&msg.data)?;

  // generate earnings_dict, if none returned, encounted a weird issue
  let earnings_dict = dpplns(&block, &*sco)?;

  // drop score map to be used elsewhere
  drop(sco);

  // grab a mysql pool connection and insert earnings
  let conn = match mysql_pool.get() {
    Ok(c) => c,
    Err(e) => return Err(format!("Mysql connection failed on block: {}", block.id))?,
  };
  insert_earnings(&block, earnings_dict, &conn)?;
  Ok(())
}

// load shares in from postgres on restart of dpplns service
fn load_shares_from_db(
  shares_queue: &mut VecDeque<ShareMinified>,
  user_scores_map: &mut UserScoreMapType,
) {
  // establish PG pool
  let pg_pool = match establish_pg_connection() {
    Ok(p) => p,
    Err(e) => panic!("PG Pool to load shares failed: {}", e),
  };
  // get a pooled connection
  let pg_conn = match pg_pool.get() {
    Ok(p) => p,
    Err(e) => panic!("PG Pool Connection to load shares failed: {}", e),
  };

  // get the window time with its steps setup
  // steps required as a full window would be a 26M row query
  let mut time_window_start = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs()
    - WINDOW_LENGTH;
  let time_now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();
  let step_count = 10;
  let step_size = (time_now - time_window_start) / step_count;

  // loop through selecting 1/step_count windows of shares and load them
  for _ in 0..step_count {
    let new_shares: Vec<SharePg> = match select_shares_newer_pg(
      &pg_conn,
      time_window_start as i64,
      (time_window_start + step_size) as i64,
    ) {
      Ok(s) => s,
      Err(e) => panic!("Failed to load shares from PG: {}", e),
    };
    time_window_start += step_size;
    // println!("Queryied {}", new_shares.len());

    for share in new_shares {
      let share = ShareMinified::from(share);
      // decay the share initially
      // share.share_payout *= calc_decay_factor(time_now as i64, share.time);
      handle_share(user_scores_map, shares_queue, share);
    }
    println!("Loaded Shares Processed");
  }
  // println!("Queue size {}", shares_queue.len());
  // println!("{:?}", map);
}

fn calc_decay_factor(time_current: i64, time_share: i64) -> f64 {
  // current - share time / window length gives us a value between 0 and 1
  let x_axis_value = std::cmp::max((time_current - time_share) / WINDOW_LENGTH as i64, 1) as f64;
  // y = -x + 1  (linear decay over the full window_length)
  1.0 - (-x_axis_value + 1.0)
}

// calculate the base share value
fn calc_score(
  mode: i16,
  coin_id: i16,
  user_id: i32,
  difficulty: f64,
  share_diff: f64,
  block_diff: f64,
  block_reward: f64,
) -> f64 {
  // solo shares dont need to keep score
  if mode == 2 {
    return 1.0;
  }

  let mut f;
  match coin_id {
    2418 => f = 0.04,  // epic
    2426 => f = 0.12,  // atom
    2408 => f = 0.015, // nim
    2422 => f = 0.015, // mwc
    2423 => f = 0.03,  // kda
    2416 => f = 0.022, // arw
    2410 => f = 0.02,  // sin
    _ => f = NORMAL_FEE,
  }

  if mode == 1 {
    f = PARTY_FEE
  }

  let diff = difficulty;
  let share_diff = share_diff;
  let block_diff = block_diff;
  let b = block_reward;
  // let s = diff / block_diff;
  //let s = sqrt(MIN(diff, block_diff) / work_diff) * work_diff / 2
  let user_id = user_id;

  let s = (share_diff.min(block_diff) / diff).sqrt() * diff;
  let mut share_payout = (1.0 - f) * (s * b);
  match user_id {
    52892 => share_payout *= 4.0,  // myatomwallet
    53443 => share_payout *= 49.0, // nimiq
    51779 => share_payout *= 12.0, // mymwc666
    50432 => share_payout *= 0.94, // mwcdevelsoft
    _ => (),
  }

  // return the share_payout
  //10.0
  println!("share user_id: {}, payout: {}", user_id, share_payout);
  share_payout
}

// adds a minified share to the queue
fn add_share_to_queue(shares_queue: &mut ShareQueueType, share: ShareMinified) {
  shares_queue.push_back(share);
}

// add a share to the map but increaking the user's value
fn update_map_with_new_share(map: &mut UserScoreMapType, share_obj: &ShareMinified) {
  // generate a key for the dictionary based on the share
  let key: String = dict_key_gen(
    share_obj.mode,
    share_obj.coin_id,
    share_obj.algo,
    &share_obj.party_pass,
  );

  // add coin-algo if it doesnt already exist
  if !map.contains_key(&key) {
    let init_map: HashMap<i32, f64> = HashMap::new();
    map.insert(key.to_string(), init_map);
  }

  // set the user_scores map to the proper key
  let user_scores = map.get_mut(&key).unwrap();

  // update user score
  if let Some(user) = user_scores.get_mut(&share_obj.user_id) {
    *user += share_obj.share_payout
  } else {
    user_scores.insert(share_obj.user_id, share_obj.share_payout);
  }
}

// decay shares queue
// loop through the queue and rebuild the map with new values
fn rebuild_decayed_map_and_trim_queue(map: &mut UserScoreMapType, shares: &mut ShareQueueType) {
  let time_current = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();
  let start = time_current;
  let time_window_start = time_current - WINDOW_LENGTH;
  if shares.len() == 0 {
    println!("No shares to decay in queue");
    return;
  }

  *map = UserScoreMapType::new();

  // trim the queue first to avoid adding shares we dont want
  let mut time = shares.front().unwrap().time;
  let mut share: ShareMinified;
  while time < time_window_start as i64 && shares.len() > 0 {
    share = shares.pop_front().unwrap();
    time = share.time;
    // update_map_by_removing_share(&mut map, share);
    // trimmed += 1;
  }

  // loop through the shares and add to the map with the new decay'ed score
  for share in shares.iter() {
    // println!("payout: {}", share.share_payout);
    let mut new_share = share.clone();
    new_share.share_payout *= calc_decay_factor(time_current as i64, new_share.time as i64);
    // println!("share:{:?}", new_share);

    update_map_with_new_share(map, &new_share)
  }

  println!(
    "Done Decaying and Trimming, map-size: {}, queue-size: {}, took: {}ms",
    map.len(),
    shares.len(),
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs()
      - start,
  );
  println!("{:?}", map);
}
// //--------------------------TRIM TIMER----------------------------
// {
//   let shares = shares_queue.clone();
//   let user_scores = user_scores_map.clone();
//   let trim_task = tokio::spawn(async move {
//     let mut interval = time::interval(Duration::from_millis(1000));
//     loop {
//       interval.tick().await;

//       let mut sha = shares.lock().unwrap();
//       let mut sco = user_scores.lock().unwrap();
//       rebuild_decayed_map_and_trim_queue(&mut *sco, &mut *sha)
//       // trim_shares_from_queue_and_map(&mut *sco, &mut *sha);
//     }
//   });
//   tasks.push(trim_task);
// }
// // remove shares from the queue and update the value in the map to reflect
// // looks back to the beginning of the window length
// fn trim_shares_from_queue_and_map(mut map: &mut UserScoreMapType, shares: &mut ShareQueueType) {
//   let now = SystemTime::now();
//   let start = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
//   let time_window_start = now.duration_since(UNIX_EPOCH).unwrap().as_secs() - WINDOW_LENGTH;
//   // if shares.len() == 0 {
//   //   return;
//   // }
//   let mut trimmed = 0;
//   // let mut time = shares.front().unwrap().time;
//   // let mut share: ShareMinified;

//   // while time < time_window_start as i64 && shares.len() > 0 {
//   //   share = shares.pop_front().unwrap();
//   //   time = share.time;
//   //   // update_map_by_removing_share(&mut map, share);
//   //   // trimmed += 1;
//   // }

//   //decay_queue_and_map(&mut map, &shares);

//   // println!(
//   //   "Done Trimming, map-size: {}, queue-size: {}, took: {}ms, trimmed: {}",
//   //   map.len(),
//   //   shares.len(),
//   //   SystemTime::now()
//   //     .duration_since(UNIX_EPOCH)
//   //     .unwrap()
//   //     .as_secs()
//   //     - start,
//   //   trimmed
//   // );
//   // println!("{:?}", map);
// }

// // removes a share from the map by updating its value
// fn update_map_by_removing_share(map: &mut UserScoreMapType, share_obj: ShareMinified) {
//   // generate a key for the dictionary based on the share
//   let key: String = dict_key_gen(
//     share_obj.mode,
//     share_obj.coin_id,
//     share_obj.algo,
//     &share_obj.party_pass,
//   );

//   // set the user_scores map to the proper key
//   if !map.contains_key(&key) {
//     panic!("WTF HOW DID WE GET HERE GREG????");
//   }
//   let user_scores = map.get_mut(&key).unwrap();

//   // update user score
//   if let Some(score) = user_scores.get_mut(&share_obj.user_id) {
//     *score -= share_obj.share_payout;
//     // println!("{}", 1e-9);
//     // remove user if down to 0
//     if *score <= 1e-9 {
//       user_scores.remove(&share_obj.user_id);
//     }
//   }

//   // drop the entire key if its empty
//   if user_scores.len() == 0 {
//     map.remove(&key);
//   }
// }
// // removes a share from the map by updating its value
// fn update_map_by_decaying_share(
//   map: &mut UserScoreMapType,
//   share_obj: &mut ShareMinified,
//   time_current: i64,
// ) {
//   // generate a key for the dictionary based on the share
//   let key: String = dict_key_gen(
//     share_obj.mode,
//     share_obj.coin_id,
//     share_obj.algo,
//     &share_obj.party_pass,
//   );

//   // set the user_scores map to the proper key
//   if !map.contains_key(&key) {
//     // key depleted already
//     return;
//     // panic!("WTF HOW DID WE GET HERE GREG????");
//   }
//   let user_scores = map.get_mut(&key).unwrap();
//   // println!(
//   //   "subtract value  - {}",
//   //   share_obj.share_payout * DECAY_FACTOR
//   // );

//   let decay_factor = calc_decay_factor(share_obj.decay_counter);
//   // update user score
//   if let Some(score) = user_scores.get_mut(&share_obj.user_id) {
//     *score -= share_obj.share_payout * decay_factor;
//     // println!("{}", 1e-9);
//     // remove user if down to 0
//     if *score <= 1e-9 {
//       user_scores.remove(&share_obj.user_id);
//     }
//   }

//   share_obj.decay_counter += 1;

//   // drop the entire key if its empty
//   if user_scores.len() == 0 {
//     map.remove(&key);
//   }
// }
