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

// const DEBUG_MODE: bool = false;

extern crate shared;
use diesel::prelude::*;
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::blocks::{get_blocks_unprocessed_mysql, update_block_to_processed_mysql},
  helpers::earnings::insert_earnings_mysql,
  models::{BlockMYSQL, EarningMYSQLInsertable},
  // MysqlPool,
};
use shared::nats::{establish_nats_connection, models::ShareNats};

use shared::db_pg::{
  establish_pg_connection, helpers::shares::select_shares_newer_pg, models::SharePg,
};

// use shared::enums::*;
// use sentry::{capture_message, integrations::failure::capture_error, Level};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{interval_at, Duration, Instant};

// constants
const NORMAL_FEE: f64 = 0.01;
const SOLO_FEE: f64 = 0.02;
const PARTY_FEE: f64 = 0.02;
const WINDOW_LENGTH: u64 = 2 * 60 * 60; //s
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
impl ShareMinified {
  fn mode_to_string(self) -> String {
    match self.mode {
      0 => "normal".to_string(),
      1 => "solo".to_string(),
      2 => "party".to_string(),
      _ => "invalid".to_string(),
    }
  }
  fn algo_to_string(self) -> String {
    match self.algo {
      0 => "blake2s".to_string(),
      _ => "invalid".to_string(),
    }
  }
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

// hashmap to hold userid's with the current dpplns scores
type UserScoreMapType = HashMap<String, HashMap<i32, f64>>;
// queue to hold shares (queue length of dpplns window)
type ShareQueueType = VecDeque<ShareMinified>;
// hashmap to hold earnings for each block
type EarningMapType = HashMap<i32, f64>;

#[tokio::main]
async fn main() {
  // let _guard =
  //   sentry::init("https://689607b053ac4fbb81ee82a08a8aa18a@sentry.watlab.icemining.ca/9");

  // create base structs to be used across threads
  let shares_queue = Arc::new(Mutex::new(ShareQueueType::new()));
  let user_scores_map = Arc::new(Mutex::new(UserScoreMapType::new()));

  {
    // lock the shares and scores, load last window from database
    let mut sha = shares_queue.lock().unwrap();
    let mut sco = user_scores_map.lock().unwrap();
    load_shares_from_db(&mut *sha, &mut *sco);
    rebuild_decayed_map_and_trim_queue(&mut *sco, &mut *sha)
  }

  // capture_message("DPPLNS loaded shares and is now live", Level::Info);

  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      panic!("Nats did not connect: {}", e);
    }
  };

  //setup msqyl
  let mysql_pool = match establish_mysql_connection() {
    Ok(p) => p,
    Err(e) => panic!("MYSQL FAILED: {}", e),
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
        match parse_share(&msg.data) {
          Ok(share) => {
            let mut sha = shares.lock().unwrap();
            let mut sco = user_scores.lock().unwrap();
            handle_share(&mut *sco, &mut *sha, share);
          }
          Err(err) => {
            println!("share parse failed: {}", err);
            // capture_message(&format!("Share parse failed: {}", err), Level::Error);
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
    // grab a copy of user_users to be passed into listener thread
    let user_scores = user_scores_map.clone();
    // listen to scheduled events
    let sub = match nc.queue_subscribe("events.dpplns", "dpplns_worker") {
      Ok(s) => s,
      Err(e) => panic!("Nats sub to shares failed: {}", e),
    };
    // spawn a new task to listen for new blocks
    let block_task = tokio::spawn({
      async move {
        for _ in sub.messages() {
          // ignore the message and run
          println!("dpplns event received, running");

          // grab a mysql pool connection
          let conn = match mysql_pool.get() {
            Ok(conn) => conn,
            Err(e) => {
              // crash and sentry BIG ISSUE
              println!("Error mysql conn. e: {}", e);
              panic!("error getting mysql connection. e: {}", e);
            }
          };

          // get new blocks within the hour
          let blocks: Vec<BlockMYSQL> = match get_blocks_unprocessed_mysql(&conn) {
            Ok(blocks) => blocks,
            Err(e) => {
              println!("Error getting blocks. e: {}", e);
              panic!("error... e: {}", e);
            }
          };

          for block in blocks {
            // set the block in mysql to unconfirmed, with 0 confirmations
            match update_block_to_processed_mysql(&conn, &block) {
              Ok(_) => (),
              Err(e) => println!("Update block failed. block: {}, e: {}", &block.id, e),
            }
            let sco = user_scores.lock().unwrap();
            match handle_block(&block, &sco, &conn) {
              Ok(_) => (),
              Err(e) => println!("block failed. e: {}", e),
            }
          }
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
fn dpplns(block: &BlockMYSQL, dict: &UserScoreMapType) -> Result<EarningMapType, String> {
  let mut f = NORMAL_FEE;
  let mut earnings_dict: EarningMapType = HashMap::new();
  // let log = "".to_string();

  match block.coin_id {
    2418 => f = 0.04,  // epic
    2426 => f = 0.12,  // atom
    2408 => f = 0.015, // nim
    2422 => f = 0.015, // mwc
    2423 => f = 0.02,  // kda
    2416 => f = 0.022, // arw
    2410 => f = 0.02,  // sin
    _ => f = NORMAL_FEE,
  }
  println!("f: {}", 1.0 - f);

  if &block.mode == "party" {
    f = 0.03;
  }
  if &block.mode == "solo" {
    f = 0.03;
  }

  // setup block fees and log
  match block.mode.as_str() {
    "normal" => {
      println!(
        "Normal Block: {} Reward: {}, Fee: {}%",
        block.coin_id,
        block.amount,
        f * 100.0
      );
    }
    "party" => {
      let share_payout = (1.0 - f as f64) * block.amount;
      let party_pass = &block.party_pass;
      //f = PARTY_FEE;
      println!(
        "Block Party: {} Payout: {}, Fee: {}%",
        &party_pass, share_payout, f
      );
    }
    "solo" => {
      let share_payout = (1.0 - f as f64) * block.amount;
      println!(
        "YOLO SOLO!! B: {} Payout: {}, Fee: {}%",
        block.amount,
        share_payout,
        f * 100.0
      );
      earnings_dict.insert(block.userid, share_payout);
      return Ok(earnings_dict);
    }
    _ => {
      println!("WHY DIDNT THIS BLOCK HAVE A MODE????");
    }
  }

  // copy proper dict over
  let key: String = dict_key_gen(&block.mode, block.coin_id, &block.algo, &block.party_pass);
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

  // let mut sum = 0.0;
  for (_, val) in earnings_dict.iter_mut() {
    let scaled_amount = *val * prop_factor;
    *val = scaled_amount;
    // sum += scaled_amount;
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
  block: &BlockMYSQL,
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
      mode: block.mode.clone(),
      stratum_id: 0, //block.stratum_id,
      algo: 0,       //block.algo,
      party_pass: block.party_pass.to_string(),
    });
  }

  // if DEBUG_MODE == false {
  let e = insert_earnings_mysql(pool_conn, earnings);
  match e {
    Ok(_) => (), //println!("Earnings Inserted"),
    Err(err) => {
      // capture_message(
      //   &format!(
      //     "Failed to insert earnings mysql for blockid: {},: {}",
      //     &block.id, err
      //   ),
      //   Level::Error,
      // );
      return Err(err);
    }
  }
  // }
  Ok(())
}

// generates a key from the share to be used in the user scores hashmap
fn dict_key_gen(mode: &String, coin_id: i32, algo: &String, party_pass: &String) -> String {
  let key;
  match mode.as_str() {
    "normal" => key = format!("N:{}-{}", coin_id.to_string(), algo.to_string()),
    "party" => {
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

// // converts nats message to DPPLNSBlockNats
// fn parse_block(msg: &Vec<u8>) -> Result<DPPLNSBlockNats, rmp_serde::decode::Error> {
//   let b: DPPLNSBlockNats = match rmp_serde::from_read_ref(&msg) {
//     Ok(b) => b,
//     Err(err) => return Err(err),
//   };
//   Ok(b)
// }

// add share to queue and add share to map
fn handle_share(dict: &mut UserScoreMapType, shares: &mut ShareQueueType, share: ShareMinified) {
  // pass reference of the share to map to be updated
  update_map_with_new_share(dict, &share);
  // move the share to the queue to be added
  add_share_to_queue(shares, share);
}

// handle block
fn handle_block(
  block: &BlockMYSQL, //msg: &nats::Message,
  sco: &UserScoreMapType,
  conn: &MysqlConnection,
) -> Result<(), Box<dyn Error>> {
  // parse the block
  //let block = parse_block(&msg.data)?;

  // generate earnings_dict, if none returned, encounted a weird issue
  let earnings_dict = dpplns(block, &*sco)?;

  // drop score map to be used elsewhere
  drop(sco);

  insert_earnings(block, earnings_dict, conn)?;
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
  let mut shares_loaded = 0;
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
    shares_loaded += new_shares.len();
    // println!("Queryied {}", new_shares.len());

    for share in new_shares {
      let share = ShareMinified::from(share);
      // decay the share initially
      // share.share_payout *= calc_decay_factor(time_now as i64, share.time);
      handle_share(user_scores_map, shares_queue, share);
    }
    println!("Loaded Shares Processed");
  }
  println!("Total Shares loaded: {}", shares_loaded);
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
  if mode == 1 {
    return 1.0;
  }

  // let mut f;
  // match coin_id {
  //   2418 => f = 0.04,  // epic
  //   2426 => f = 0.12,  // atom
  //   2408 => f = 0.015, // nim
  //   2422 => f = 0.015, // mwc
  //   2423 => f = 0.03,  // kda
  //   2416 => f = 0.022, // arw
  //   2410 => f = 0.02,  // sin
  //   _ => f = NORMAL_FEE,
  // }
  // println!("f: {}", 1.0 - f);

  // if mode == 2 {
  //   f = PARTY_FEE
  // }

  let diff = difficulty;
  let share_diff = share_diff;
  let block_diff = block_diff;
  let b = block_reward;
  // let s = diff / block_diff;
  //let s = sqrt(MIN(diff, block_diff) / work_diff) * work_diff / 2
  let user_id = user_id;

  let f = 0.0;
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
  // println!("share user_id: {}, payout: {}", user_id, share_payout);
  share_payout
}

// adds a minified share to the queue
fn add_share_to_queue(shares_queue: &mut ShareQueueType, share: ShareMinified) {
  shares_queue.push_back(share);
}

// add a share to the map but increaking the user's value
fn update_map_with_new_share(map: &mut UserScoreMapType, share_obj: &ShareMinified) {
  let mode = share_obj.clone().mode_to_string();
  let algo = share_obj.clone().algo_to_string();
  // generate a key for the dictionary based on the share
  let key: String = dict_key_gen(
    &mode,
    share_obj.coin_id as i32,
    &algo,
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
  // let start = time_current;
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

  // println!(
  //   "Done Decaying and Trimming, map-size: {}, queue-size: {}, took: {}ms",
  //   map.len(),
  //   shares.len(),
  //   SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs()
  //     - start,
  // );
  // println!("{:?}", map);
}
