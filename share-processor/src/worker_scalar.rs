/*
TODO

  - add hashmap of queues
    hashmap<&str, VecDeque<ShareMinified>>
      hashmap : {
        coin_id-algo: {
          vecdeque<share, share, share>
        },
        mwc-primary: {
          [,2,123123123,]
        }
      }
  - add hashmap of configs
      potentially receive this on service start?
      hashmap<&str, hashmap<&str, u32>>
      hashmap : {
        2408-c31: {
          window_length: 300//s
          algo_target: 75434349234,
        }
        2408-c29: {
          window_length: 300//s
          algo_target: 75434349234,
        }
        2423-argon2d: {
          window_length: 300,//s
          algo_target: 863400
        }
      }



 on service start...
   load up shares from PG
   then start listening to nats

  collect different window lengths for each coin
    for shares queue... use max window length
    window lengths ranging from e.g(5 to 30minutes)

  on interval
    trim the queue for each coin

  on interval
    loop through coins queue 1x per scalar value
    hold a struct (most like a dictionary) for each worker


  goal:
    update scalar values coin/algo basis (workers/accounts/subaccounts/users/coins)
    grab graph values from scalars on

  workers:
    loop through the shares queue looking back at the this coin/algo window
      for each worker id
        sum up the difficulty
        adding share count
        adding initial share time
        adding last share time

    after the looping
      loop through the dict, and compute each hashrate and average with previous hashrates
      if the share count < predetermined value... set hashrate to 0
      if the last share time - initial share time < predetermined... set hashrate to 0
      in the dict, save an array of hashrates

      for share in shares:
        add share to the workers dict[share.worker_id]


    accountsDict: {
      user_A_coin_A_algo_A:{
        sumOfDiff
      }
      user_A_coin_A_algo_B; {
        sumOfDiff
      }
      worker_C: {
        sumOfDiff
      }

    }
SERVICE SHAREPROCESSOR
    fn calc_raw_hashrate(){
      loop trhough queue, calc hashrate
      insert into workers (raw_hashrate) values (raw_hashrate)
    }

SERVICE SHAREPROCESS_TIMESERIES
    // snapshot of workers table
    fn save_raw_hashrate(){
      select * from workers
      insert into hashworkers (raw_hashrate) values (raw_hashrate, timestamp.now())
    }

    fn calc_average_hashrate_scalar(){
      select
        avg(1hr)
        avg(12hr)
      from hashworkers where worker_id = ....

      update workers set 1hr, 12hr where worker_id = ...
    }

    // dont insert 0's after last share time > 1 day
    fn interval_insert_zero(){
      select * from hashworker
      loop through hashworker
        if missing datapoint
          insert 0
    }

SERVICE SHAREPROCESSOR_SCALAR_ROLLUP

select name, worker, hashrate from workers where hashrate > 0 AND time > unix_timestamp(now() - interval 5 minute) and last_share_time < unix_timestamp(now()-interval 1 minute) limit 5;


*/

extern crate shared;

// use nats;

// use shared::db_mysql::models::Worker;
use dotenv::dotenv;
use futures::join;
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::workers::{update_worker_hashrate, update_worker_mysql},
  models::WorkerMYSQL,
  MysqlPool,
};

use shared::enums::*;
use shared::nats::models::{
  ShareNats, ShareProcessorConfigNats, ShareProcessorConfigObj, ShareProcessorHashMap,
};
use shared::nats::{establish_nats_connection, NatsConnection};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time;
const TRIM_INTERVAL: u64 = 1; //s
use tokio::time::{interval_at, Duration, Instant};
const WINDOW_LENGTH: u64 = 60 * 5; //s
const HASHRATE_INTERVAL: u64 = 1 * 15; //s
struct ShareQueueMinifiedObj {
  user_name: String,
  // user_id: i32,
  timestamp: i64,
  algo: String,
  // worker_id: i32,
  uuid: String,
  coin_id: i16,
  worker_name: String,
  difficulty: f64,
}
impl From<ShareNats> for ShareQueueMinifiedObj {
  fn from(s: ShareNats) -> Self {
    ShareQueueMinifiedObj {
      user_name: s.user_name.to_string(),
      // user_id: s.user_id,
      timestamp: s.timestamp,
      algo: s.algo.to_string(), //Algos::from_i16(s.algo),
      // worker_id: s.worker_id,
      uuid: s.worker_uuid.to_string(),
      coin_id: s.coin_id,
      worker_name: "".to_string(), // s.worker_name,
      difficulty: s.difficulty,
    }
  }
}

#[derive(Debug)]
struct WorkerDictObj {
  // worker_id: i32,
  uuid: String,
  start_time: i64,
  last_share_time: i64,
  count: i16,
  difficulty_sum: f64,
  difficulty: f64,
  worker_name: String,
  // user_id: i32,
  user_name: String,
  // algo: Algos,
  algo: String,
  coin_id: i16,
  hashrate: f64,
  shares_per_min: f64,
}

type ShareQueueType = VecDeque<ShareQueueMinifiedObj>;
type ShareQueuesMapType = HashMap<String, ShareQueueType>;
type ShareQueuesMapArcType = Arc<Mutex<ShareQueuesMapType>>;

// type ShareQueueArc = Arc<Mutex<VecDeque<ShareQueueMinifiedObj>>>;

type WorkerDict = HashMap<String, WorkerDictObj>;

pub async fn run_jobs(env: &String, mysql_pool: &MysqlPool, nc: &NatsConnection) {
  let mut config_nats = ShareProcessorConfigNats {
    config: HashMap::new(),
  };
  config_nats.config.insert(
    "2408-argon2d".to_string(),
    ShareProcessorConfigObj {
      window_length: 300,
      algo_target: 65536000,
      // min_share_count: 0,
    },
  );
  config_nats.config.insert(
    "2423-blake2s".to_string(),
    ShareProcessorConfigObj {
      window_length: 300,
      algo_target: 1000,
    },
  );
  let config = config_nats.config;

  let mut share_queues_map = Arc::new(Mutex::new(ShareQueuesMapType::new()));

  //-----------------------SHARES LISTENER--------------------------------
  let _share_listener = share_listener(&env, &nc, &mut share_queues_map);

  //-----------------------QUEUE TRIM INTERVAL --------------------------------
  let _trim_queue = trim_queue(config.clone(), &mut share_queues_map);

  //-----------------------CALC RAW WORKER HASHRATE INTERVAL --------------------------------
  let _calc_raw_hashrate = calc_raw_hashrate(config.clone(), &mut share_queues_map, &mysql_pool);

  // -----------------------CALC RAW WORKER HASHRATE INTERVAL --------------------------------
  // let _trim_disconnected_workers_listener = trim_disconnected_workers(&mysql_pool);

  join!(
    _share_listener,
    _trim_queue,
    _calc_raw_hashrate,
    // _trim_disconnected_workers_listener,
  );
}

// listen for shares and push to queue
fn share_listener(
  env: &String,
  nc: &NatsConnection,
  share_queues_map: &ShareQueuesMapArcType,
) -> tokio::task::JoinHandle<()> {
  // connect to the nats channel
  let subject;
  if env == "dev" {
    subject = format!("dev.shares.>");
  } else {
    subject = format!("shares.2428");
  }
  let sub = match nc.subscribe(&subject) {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum auth listener failed: {}", e),
  };

  // start a thread to listen for messages
  let share_queues_map = share_queues_map.clone();
  tokio::task::spawn(async move {
    println!("Listening for shares in worker_scalar");
    for msg in sub.messages() {
      println!("share: {}", msg.subject);
      // parse the share into a minified object, then push it to the queue
      if let Ok(share) = parse_share_msg_into_minified(&msg.data) {
        let mut share_queues_map = share_queues_map.lock().unwrap();
        // get the key for the share
        let key = keygen(share.coin_id, &share.algo);

        // add or get the queue for this coin
        let share_queue = share_queues_map.entry(key).or_insert(ShareQueueType::new());

        share_queue.push_back(share);
      }
    }
  })
}

fn trim_queue(
  config: ShareProcessorHashMap,
  share_queues_map: &ShareQueuesMapArcType,
) -> tokio::task::JoinHandle<()> {
  let share_queues_map = share_queues_map.clone();
  println!("starting worker_scalar trim queue");
  tokio::task::spawn(async move {
    let mut interval = interval_at(
      Instant::now() + Duration::from_millis(TRIM_INTERVAL * 1000),
      Duration::from_millis(TRIM_INTERVAL * 1000),
    );

    loop {
      interval.tick().await;

      let mut share_queues_map = share_queues_map.lock().unwrap();
      for (k, share_queue) in share_queues_map.iter_mut() {
        // get the window length from the config, defeault to 300
        // println!("KEY: {}", k);
        let window_length = match config.get(k) {
          Some(val) => val.window_length,
          None => 300,
        } as u64;
        let time_current = SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_secs();
        let time_window_start = time_current - window_length;
        // trim the queue first to avoid adding shares we dont want
        if share_queue.len() < 1 {
          println!("shares queue empty, nothign to trim");
          continue;
        }
        let mut time = share_queue.front().unwrap().timestamp;
        let mut share: ShareQueueMinifiedObj;

        // println!("time: {}, time_window_start: {}", time, time_window_start);
        while time < time_window_start as i64 && share_queue.len() > 0 {
          share = share_queue.pop_front().unwrap();
          time = share.timestamp;
        }

        println!("Done Trimming - {}, queue-size: {}", k, share_queue.len());
      }
    }
  })
}

// // move to database-maintenance
// fn trim_disconnected_workers(mysql_pool: &MysqlPool) -> tokio::task::JoinHandle<()> {
//   let mysql_pool = mysql_pool.clone();
//   tokio::task::spawn(async move {
//     let mut interval = interval_at(
//       Instant::now() + Duration::from_millis(TRIM_INTERVAL * 30000),
//       Duration::from_millis(TRIM_INTERVAL * 30000),
//     );

//     loop {
//       interval.tick().await;

//       let conn = match mysql_pool.get() {
//         Ok(conn) => conn,
//         Err(e) => {
//           // crash and sentry BIG ISSUE
//           println!("Error mysql conn. e: {}", e);
//           continue;
//           // panic!("error getting mysql connection e: {}",);
//         }
//       };
//       let lookback_time = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_secs()
//         - 60 * 60 * 1;
//       println!("Deleting stale workers");
//       delete_stale_workers_mysql(&conn, 2423, lookback_time as i32);
//       delete_stale_workers_mysql(&conn, 2408, lookback_time as i32);
//     }
//   })
// }

fn parse_share_msg_into_minified(
  msg: &Vec<u8>,
) -> Result<ShareQueueMinifiedObj, rmp_serde::decode::Error> {
  let share: ShareNats = match rmp_serde::from_read_ref(&msg) {
    Ok(share) => share,
    Err(e) => {
      println!("Error parsing share. e: {}", e);
      return Err(e);
    }
  };
  Ok(ShareQueueMinifiedObj::from(share))
}

async fn calc_raw_hashrate(
  config: ShareProcessorHashMap,
  share_queues_map: &ShareQueuesMapArcType,
  mysql_pool: &MysqlPool,
) -> tokio::task::JoinHandle<()> {
  let share_queues_map = share_queues_map.clone();
  let mysql_pool = mysql_pool.clone();
  tokio::task::spawn(async move {
    let mut interval = interval_at(
      Instant::now() + Duration::from_millis(HASHRATE_INTERVAL * 1000),
      Duration::from_millis(HASHRATE_INTERVAL * 1000),
    );

    loop {
      interval.tick().await;

      let conn = match mysql_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          continue;
          // panic!("error getting mysql connection e: {}",);
        }
      };

      let mut share_queues_map = share_queues_map.lock().unwrap();
      for (k, share_queue) in share_queues_map.iter_mut() {
        // get the target from the config, defeault to 1000? why not
        let algo_target = match config.get(k) {
          Some(val) => val.algo_target,
          None => 1000,
        } as u64;
        let mut worker_dict = WorkerDict::new();
        // add each share from the queue to the worker dict
        for share in share_queue.iter() {
          // let worker = worker_dict.entry(share.worker_id).or_insert(WorkerDictObj {
          let worker = worker_dict
            .entry(share.uuid.to_string())
            .or_insert(WorkerDictObj {
              // worker_id: share.worker_id,
              uuid: share.uuid.to_string(),
              start_time: share.timestamp,
              last_share_time: share.timestamp,
              count: 1,
              difficulty_sum: share.difficulty,
              difficulty: share.difficulty,
              worker_name: share.worker_name.clone(),
              // user_id: share.user_id,
              user_name: share.user_name.to_string(),
              algo: share.algo.to_string(),
              coin_id: share.coin_id,
              hashrate: 0.0,
              shares_per_min: 0.0,
            });
          worker.count += 1;
          worker.last_share_time = share.timestamp;
          worker.difficulty_sum += share.difficulty;
          worker.difficulty = share.difficulty;
        }
        //todo change to multithread the updates
        for mut worker in worker_dict.values_mut() {
          // println!("worker difficulty sum: {}", worker.difficulty_sum);
          worker.hashrate =
            (worker.difficulty_sum * (algo_target as f64) / WINDOW_LENGTH as f64) / 1000.0;
          worker.shares_per_min = worker.count as f64 / (WINDOW_LENGTH as f64 / 60.0);
          // println!(
          //   "worker uuid: {}, hashrate: {}, algo target: {}, window_length: {},",
          //   &worker.uuid, &worker.hashrate, algo_target, WINDOW_LENGTH,
          // );
          update_worker_hashrate(
            &conn,
            &worker.uuid,
            worker.last_share_time as i32,
            worker.shares_per_min,
            worker.hashrate,
            worker.difficulty,
          )
          .unwrap();
        }
        println!("Done updating, map size: {}", worker_dict.len());

        // TODO - update workers set hashrate = 0 where last_share_time > window length
      }
    }
  })
}

fn keygen(coin_id: i16, algo: &String) -> String {
  format!("{}-{}", coin_id, algo)
  // return coin_id + algo
}
