/*
use  ShareNats
parse shareNats -> convert to ShareMin
Convert SharePGInsertable into ShareMin
Convert ShareNats into ShareMin
probably just feed in values into calc_score instead of a share

use BlockNats
parse blockNats and work with it as is

fix enums to use from_i8 and from_string and mabye to_i8 and to_string?

use EarningMySQLInsertable OR EarningPGInsertable or both i guess...
create genertic Earning that can be to_EarningMYSQLInsertable and to_EarningPGINsertable

final goal (ShareMin => min usable data for a share that will go into the queue)
  - load in SharePG from PG -> convert to ShareMin -> Process (add to queue/dict)
  - Receive ShareNats from Nats -> convert to ShareMin -> Process (add to queue/dict)
  - Trim Queue of ShareMin and AlmightyDict
  - Receive BlockNats from Nats -> Calc earning and generate Vec<Earning>
    - convert Vec<Earning> to Vec<EarningMYSQLInsertable> -> Insert into mysql
    - convert Vec<Earning> create Vec<EarningPGInsertable> -> Insert into PG

*/

extern crate shared;
use diesel::prelude::*;
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::earnings::insert_earnings_mysql,
  models::{EarningMYSQLInsertable, ShareMYSQLInsertable},
};
use shared::nats::{
  establish_nats_connection,
  models::{BlockNats, ShareNats},
};

use shared::db_pg::{
  establish_pg_connection, helpers::shares::select_shares_newer_pg, models::SharePg,
};

use shared::enums::*;

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
// constants
const NORMAL_FEE: f64 = 0.01;
const SOLO_FEE: f64 = 0.02;
const PARTY_FEE: f64 = 0.02;
const WINDOW_LENGTH: u64 = 2 * 60 * 60;

// Minimum data required to be stored in the queue
// share minified is used to update the hashmap
#[derive(Debug, Clone)]
struct ShareMinified {
  user_id: i32,
  coin_id: i16,
  algo: Algos,
  time: i64,
  share_payout: f64,
  mode: ShareModes,
  party_pass: String,
}
// convert incoming shareNats into shareminified
impl From<ShareNats> for ShareMinified {
  fn from(s: ShareNats) -> Self {
    let score = calc_score(
      ShareModes::from_i16(s.mode),
      s.coin_id,
      s.user_id,
      s.difficulty,
      s.block_diff,
      s.block_reward,
    );
    ShareMinified {
      user_id: s.user_id,
      coin_id: s.coin_id,
      algo: Algos::from_i16(s.algo),
      time: s.timestamp,
      share_payout: score,
      mode: ShareModes::from_i16(s.mode),
      party_pass: s.party_pass,
    }
  }
}
// convert incoming shares from PG to share minified
impl From<SharePg> for ShareMinified {
  fn from(s: SharePg) -> Self {
    let score = calc_score(
      ShareModes::from_i16(s.mode),
      s.coin_id,
      s.user_id,
      s.difficulty,
      s.block_diff,
      s.block_reward,
    );
    ShareMinified {
      user_id: s.user_id,
      coin_id: s.coin_id,
      algo: Algos::from_i16(s.algo),
      time: s.time,
      share_payout: score,
      mode: ShareModes::from_i16(s.mode),
      party_pass: s.party_pass,
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

#[tokio::main]
async fn main() {
  // let _guard =
  //   sentry::init("hhttps://3741efe24e524656911739231c935b7b@sentry.watlab.icemining.ca/4");
  // sentry::capture_message("Hello World!", sentry::Level::Info);

  // create base structs to be used across threads
  let shares_queue = Arc::new(Mutex::new(ShareQueueType::new()));
  let user_scores_map = Arc::new(Mutex::new(UserScoreMapType::new()));

  {
    // lock the shares and scores, load last window from database
    let mut sha = shares_queue.lock().unwrap();
    let mut sco = user_scores_map.lock().unwrap();
    load_shares_from_db(&mut *sha, &mut *sco);
  }

  let nc = establish_nats_connection();
  let coins: Vec<i32> = vec![2422, 2122];
  // add each of the subscriptions in
  // let mut share_subs: Vec<nats::subscription::Subscription> = Vec::new();
  // setup threads array
  let mut tasks = Vec::new();
  //-----------------------SHARES LISTENER--------------------------------
  {
    for coin in coins {
      let channel = format!("shares.{}", coin.to_string());
      let sub = nc.subscribe(&channel).unwrap();
      let shares = shares_queue.clone();
      let user_scores = user_scores_map.clone();
      let share_task = tokio::spawn(async move {
        for msg in sub.messages() {
          let share = parse_share(&msg.data);
          let mut sha = shares.lock().unwrap();
          let mut sco = user_scores.lock().unwrap();
          handle_share(&mut *sco, &mut *sha, share);
        }
        // let mut interval = time::interval(Duration::from_millis(1));
        // loop {
        //   if let Some(msg) = sub.try_next() {
        //     let share = parse_share(&msg.data);
        //     let mut sha = shares.lock().unwrap();
        //     let mut sco = user_scores.lock().unwrap();
        //     handle_share(&mut *sco, &mut *sha, share);
        //   } else {
        //     interval.tick().await;
        //   }
        // }
      });
      tasks.push(share_task);
    }
  }
  //-----------------------BLOCKS LISTENER----------------------------
  {
    // subscribe to the nats blocks channel
    let block_sub = nc.queue_subscribe("blocks", "dpplns_worker").unwrap();

    //setup msqyl
    let mysql_pool = establish_mysql_connection();
    // grab a copy of user_users to be passed into listener thread
    let user_scores = user_scores_map.clone();

    // spawn a new task to listen for new blocks
    let block_task = tokio::spawn({
      async move {
        // loop through the blocking messages() listener waiting for blocks
        for msg in block_sub.messages() {
          // grab a new
          let mysql_pool = mysql_pool.clone();

          // grab a copy of user_users to be passed into processing thread
          let user_scores = user_scores.clone();
          tokio::spawn({
            async move {
              let block = parse_block(&msg.data);
              let sco = user_scores.lock().unwrap();
              let earnings_dict = dpplns(&block, &*sco);
              drop(sco);

              let mut conn = mysql_pool.get().unwrap();
              insert_earnings(&block, earnings_dict, &mut conn);
            }
          });
        }
      }
    });
    tasks.push(block_task);
  }
  //--------------------------TRIM TIMER----------------------------
  {
    let shares = shares_queue.clone();
    let user_scores = user_scores_map.clone();
    let trim_task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(2000));
      loop {
        interval.tick().await;

        let mut sha = shares.lock().unwrap();
        let mut sco = user_scores.lock().unwrap();
        trim_shares_from_queue_and_map(&mut *sco, &mut *sha);
      }
    });
    tasks.push(trim_task);
  }

  for handle in tasks {
    handle.await.unwrap();
  }
}

// main dpplns function, takes in a block and generates the earnings for each user
fn dpplns(block: &BlockNats, dict: &UserScoreMapType) -> EarningMapType {
  let mut f = NORMAL_FEE;
  let mut earnings_dict: EarningMapType = HashMap::new();
  // let log = "".to_string();

  // setup block fees and log
  match ShareModes::from_i16(block.mode) {
    ShareModes::NORMAL => {
      println!(
        "Normal Block: {} Reward: {}, Fee: {}%",
        block.coin_id,
        block.amount,
        f * 100.0
      );
    }
    ShareModes::PARTY => {
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
    ShareModes::SOLO => {
      let share_payout = (1.0 - SOLO_FEE as f64) * block.amount;
      println!(
        "YOLO SOLO!! B: {} Payout: {}, Fee: {}%",
        block.amount,
        share_payout,
        SOLO_FEE * 100.0
      );
      earnings_dict.insert(block.userid, share_payout);
      return earnings_dict;
    }
  }

  // copy proper dict over
  let key: String = dict_key_gen(
    &ShareModes::from_i16(block.mode),
    block.coin_id,
    &Algos::from_i16(block.algo),
    &block.party_pass,
  );
  let key_exists = dict.contains_key(&key);
  if key_exists == false {
    println!("block failed, invalid dict key: {}", &key);
    panic!("block failed");
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

  println!(
    "Payment Complete! {} Total Earnings: {} Target Block Payout: {} Elapsed Time: {}",
    earnings_dict.len(),
    sum,
    tgt_block_payout,
    0
  );
  if earnings_dict.len() == 0 {
    panic!("earnings dict empty");
  };

  return earnings_dict;
}

// inserts earnings into the databases
fn insert_earnings(block: &BlockNats, earnings_dict: EarningMapType, pool_conn: &MysqlConnection) {
  let mut earnings: Vec<EarningMYSQLInsertable> = Vec::new();
  for (&user_id, val) in earnings_dict.iter() {
    earnings.push(EarningMYSQLInsertable {
      userid: user_id,
      coinid: block.coin_id as i32,
      blockid: block.id,
      // createtime: block.time,
      status: 0,
      amount: *val,
      mode: ShareModes::from_i16(block.mode).to_string(),
      stratum_id: block.stratum_id,
    });
  }
  let e = insert_earnings_mysql(pool_conn, earnings);
  match e {
    Ok(success) => println!("Earnings Inserted"),
    Err(e) => println!("Earnings Insert Failed: {}", e),
  }
}

// generates a key from the share to be used in the user scores hashmap
fn dict_key_gen(mode: &ShareModes, coin_id: i16, algo: &Algos, party_pass: &String) -> String {
  let key;
  match mode {
    ShareModes::NORMAL => key = format!("N:{}-{}", coin_id.to_string(), algo.to_string()),
    ShareModes::PARTY => {
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

// TODO: MsgPack with lz4 compression for shares
// converts nats message to sharenats and then to share minified
fn parse_share(msg: &Vec<u8>) -> ShareMinified {
  // Some JSON input data as a &str. Maybe this comes from the user.
  // Parse the string of data into serde_json::Value.
  let s: ShareNats = serde_json::from_slice(&msg).unwrap();
  let share = ShareMinified::from(s);
  return share;
}

// converts nats message to blocknats
fn parse_block(msg: &Vec<u8>) -> BlockNats {
  let b: BlockNats = serde_json::from_slice(&msg).unwrap();
  return b;
}

// add share to queue and add share to map
fn handle_share(dict: &mut UserScoreMapType, shares: &mut ShareQueueType, share: ShareMinified) {
  add_share_to_queue(shares, share.clone());
  update_map_with_new_share(dict, share.clone());
}

// load shares in from postgres on restart of dpplns service
fn load_shares_from_db(
  shares_queue: &mut VecDeque<ShareMinified>,
  user_scores_map: &mut UserScoreMapType,
) {
  // establish PG pool
  let pg_pool = establish_pg_connection();
  // get a pooled connection
  let pg_conn = pg_pool.get().unwrap();

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
    let new_shares: Vec<SharePg> = select_shares_newer_pg(
      &pg_conn,
      time_window_start as i64,
      (time_window_start + step_size) as i64,
    );
    time_window_start += step_size;
    println!("Queryied {}", new_shares.len());

    for share in new_shares {
      let share = ShareMinified::from(share);
      handle_share(user_scores_map, shares_queue, share);
    }
    // println!("Shares Processed");
  }
  println!("Queue size {}", shares_queue.len());
}

// calculate the base share value
fn calc_score(
  mode: ShareModes,
  coin_id: i16,
  user_id: i32,
  difficulty: f64,
  block_diff: f64,
  block_reward: f64,
) -> f64 {
  // solo shares dont need to keep score
  if mode == ShareModes::SOLO {
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

  if mode as i8 == ShareModes::PARTY as i8 {
    f = PARTY_FEE
  }

  let diff = difficulty;
  let b = block_reward;
  let s = diff / block_diff;
  let user_id = user_id;

  let mut share_payout = (1.0 - f) * (s * b);
  match user_id {
    52892 => share_payout *= 4.0,  // myatomwallet
    53443 => share_payout *= 49.0, // nimiq
    51779 => share_payout *= 12.0, // mymwc666
    50432 => share_payout *= 0.94, // mwcdevelsoft
    _ => (),
  }

  // return the share_payout
  share_payout
}

// adds a minified share to the queue
fn add_share_to_queue(shares_queue: &mut ShareQueueType, share: ShareMinified) {
  shares_queue.push_back(share);
}

// add a share to the map but increaking the user's value
fn update_map_with_new_share(map: &mut UserScoreMapType, share_obj: ShareMinified) {
  // generate a key for the dictionary based on the share
  let key: String = dict_key_gen(
    &share_obj.mode,
    share_obj.coin_id,
    &share_obj.algo,
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

// removes a share from the map by updating its value
fn update_map_by_removing_share(map: &mut UserScoreMapType, share_obj: ShareMinified) {
  // generate a key for the dictionary based on the share
  let key: String = dict_key_gen(
    &share_obj.mode,
    share_obj.coin_id,
    &share_obj.algo,
    &share_obj.party_pass,
  );

  // set the user_scores map to the proper key
  if !map.contains_key(&key) {
    panic!("WTF HOW DID WE GET HERE GREG????");
  }
  let user_scores = map.get_mut(&key).unwrap();

  // update user score
  if let Some(score) = user_scores.get_mut(&share_obj.user_id) {
    *score -= share_obj.share_payout;
    // println!("{}", 1e-9);
    // remove user if down to 0
    if *score <= 1e-9 {
      user_scores.remove(&share_obj.user_id);
    }
  }

  // drop the entire key if its empty
  if user_scores.len() == 0 {
    map.remove(&key);
  }
}

// remove shares from the queue and update the value in the map to reflect
// looks back to the beginning of the window length
fn trim_shares_from_queue_and_map(mut map: &mut UserScoreMapType, shares: &mut ShareQueueType) {
  let now = SystemTime::now();
  let start = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
  let time_window_start = now.duration_since(UNIX_EPOCH).unwrap().as_secs() - WINDOW_LENGTH;
  if shares.len() == 0 {
    return;
  }
  let mut trimmed = 0;
  let mut time = shares.front().unwrap().time;
  let mut share: ShareMinified;
  while time < time_window_start as i64 && shares.len() > 0 {
    share = shares.pop_front().unwrap();
    time = share.time;
    update_map_by_removing_share(&mut map, share);
    trimmed += 1;
  }
  println!(
    "Done Trimming, map-size: {}, queue-size: {}, took: {}ms, trimmed: {}",
    map.len(),
    shares.len(),
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs()
      - start,
    trimmed
  );
  // println!("{:?}", map);
}
