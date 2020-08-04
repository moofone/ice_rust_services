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
  helpers::workers::{delete_stale_workers_mysql, update_worker_hashrate, update_worker_mysql},
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
  user_id: i32,
  timestamp: i64,
  algo: String,
  worker_id: i32,
  coin_id: i16,
  worker_name: String,
  difficulty: f64,
}
impl From<ShareNats> for ShareQueueMinifiedObj {
  fn from(s: ShareNats) -> Self {
    ShareQueueMinifiedObj {
      user_id: s.user_id,
      timestamp: s.timestamp,
      algo: "nim".to_string(), //Algos::from_i16(s.algo),
      worker_id: s.worker_id,
      coin_id: s.coin_id,
      worker_name: "".to_string(), // s.worker_name,
      difficulty: s.difficulty,
    }
  }
}

#[derive(Debug)]
struct WorkerDictObj {
  worker_id: i32,
  start_time: i64,
  last_share_time: i64,
  count: i16,
  difficulty_sum: f64,
  difficulty: f64,
  worker_name: String,
  user_id: i32,
  // algo: Algos,
  algo: String,
  coin_id: i16,
  hashrate: f64,
  shares_per_min: f64,
}
// impl WorkerDictObj {
//   fn calc_hashrate(self) -> f64 {
//     let target = Algos::get_target(&self.algo) as f64;
//     let interval = (self.end_time - self.start_time) as f64;
//     (self.difficulty * target) / interval / 1000.0
//   }
//   fn new(
//     start: i64,
//     end: i64,
//     worker_name: &String,
//     user_id: i32,
//     algo: Algos,
//     coin_id: i16,
//   ) -> WorkerDictObj {
//     WorkerDictObj {
//       start_time: start,
//       end_time: end,
//       count: 0,
//       difficulty: 0.0,
//       worker_name: worker_name.to_string(),
//       user_id: user_id,
//       algo: algo,
//       coin_id: coin_id,
//       hashrate: 0.0,
//     }
//   }
// }

type ShareQueueType = VecDeque<ShareQueueMinifiedObj>;
type ShareQueuesMapType = HashMap<String, ShareQueueType>;
type ShareQueuesMapArcType = Arc<Mutex<ShareQueuesMapType>>;

// type ShareQueueArc = Arc<Mutex<VecDeque<ShareQueueMinifiedObj>>>;

type WorkerDict = HashMap<i32, WorkerDictObj>;

#[tokio::main]
async fn main() {
  dotenv().ok();
  let env = env::var("ENVIRONMENT_MODE").unwrap_or("".to_string());

  let mut config_nats = ShareProcessorConfigNats {
    config: HashMap::new(),
  };
  config_nats.config.insert(
    "2408-argon2d".to_string(),
    ShareProcessorConfigObj {
      window_length: 300,
      algo_target: 65536000,
    },
  );
  let config = config_nats.config;

  let mut share_queues_map = Arc::new(Mutex::new(ShareQueuesMapType::new()));

  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      // crash and sentry BIG
      panic!("Nats did not connect: {}", e);
    }
  };
  //setup msqyl
  let mysql_pool = match establish_mysql_connection() {
    Ok(p) => p,
    Err(e) => {
      // crash and sentry BIG
      panic!("MYSQL FAILED: {}", e)
    }
  };
  //-----------------------SHARES LISTENER--------------------------------
  let _share_listener = share_listener(&env, &nc, &mut share_queues_map);

  //-----------------------QUEUE TRIM INTERVAL --------------------------------
  let _trim_queue = trim_queue(config.clone(), &mut share_queues_map);

  //-----------------------CALC RAW WORKER HASHRATE INTERVAL --------------------------------
  let _calc_raw_hashrate = calc_raw_hashrate(config.clone(), &mut share_queues_map, &mysql_pool);

  // -----------------------CALC RAW WORKER HASHRATE INTERVAL --------------------------------
  let _trim_disconnected_workers_listener = trim_disconnected_workers(&mysql_pool);

  join!(
    _share_listener,
    _trim_queue,
    _calc_raw_hashrate,
    _trim_disconnected_workers_listener,
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
    subject = format!("dev.shares.2408");
  } else {
    subject = format!("shares.>");
  }
  let sub = match nc.subscribe(&subject) {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum auth listener failed: {}", e),
  };

  // start a thread to listen for messages
  let share_queues_map = share_queues_map.clone();
  tokio::task::spawn(async move {
    for msg in sub.messages() {
      // println!("share: {}", msg.subject);
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
      // // TODO loop throguh the queue hashmap and do this for each
      // let mut shares = shares_queue.lock().unwrap();
      // let time_current = SystemTime::now()
      //   .duration_since(UNIX_EPOCH)
      //   .unwrap()
      //   .as_secs();
      // let time_window_start = time_current - WINDOW_LENGTH;

      // // trim the queue first to avoid adding shares we dont want
      // if shares.len() < 1 {
      //   println!("shares queue empty, nothign to trim");
      //   continue;
      // }
      // let mut time = shares.front().unwrap().timestamp;
      // let mut share: ShareQueueMinifiedObj;
      // //todo
      // /*
      //    calc the shortest trim time
      //    while time < shortest trim time
      //       check time against coin_id trim time
      //       trim if needed
      // */
      // // println!("time: {}, time_window_start: {}", time, time_window_start);
      // while time < time_window_start as i64 && shares.len() > 0 {
      //   share = shares.pop_front().unwrap();
      //   time = share.timestamp;
      // }

      // println!("Done Trimming, queue-size: {}", shares.len());
    }
  })
}

// move to database-maintenance
fn trim_disconnected_workers(mysql_pool: &MysqlPool) -> tokio::task::JoinHandle<()> {
  let mysql_pool = mysql_pool.clone();
  tokio::task::spawn(async move {
    let mut interval = interval_at(
      Instant::now() + Duration::from_millis(TRIM_INTERVAL * 30000),
      Duration::from_millis(TRIM_INTERVAL * 30000),
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
      let lookback_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 60 * 60 * 30;
      println!("Deleting stale workers");
      delete_stale_workers_mysql(&conn, 2423, lookback_time as i32);
      delete_stale_workers_mysql(&conn, 2408, lookback_time as i32);
    }
  })
}

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
          let worker = worker_dict.entry(share.worker_id).or_insert(WorkerDictObj {
            worker_id: share.worker_id,
            start_time: share.timestamp,
            last_share_time: share.timestamp,
            count: 1,
            difficulty_sum: share.difficulty,
            difficulty: share.difficulty,
            worker_name: share.worker_name.clone(),
            user_id: share.user_id,
            algo: "nim".to_string(),
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
          worker.hashrate =
            worker.difficulty_sum * algo_target as f64 / WINDOW_LENGTH as f64 / 1000.0;
          worker.shares_per_min = worker.count as f64 / (WINDOW_LENGTH as f64 / 60.0);
          update_worker_hashrate(
            &conn,
            worker.worker_id,
            worker.last_share_time as i32,
            worker.shares_per_min,
            worker.hashrate,
            worker.difficulty,
          )
          .unwrap();
        }
        println!("Done updating, map size: {}", worker_dict.len());
      }

      // let shares_queue = shares_queue.lock().unwrap();
      // let mut worker_dict = WorkerDict::new();
      // // add each share from the queue to the worker dict
      // for share in shares_queue.iter() {
      //   let worker = worker_dict.entry(share.worker_id).or_insert(WorkerDictObj {
      //     worker_id: share.worker_id,
      //     start_time: share.timestamp,
      //     last_share_time: share.timestamp,
      //     count: 1,
      //     difficulty_sum: share.difficulty,
      //     difficulty: share.difficulty,
      //     worker_name: share.worker_name.clone(),
      //     user_id: share.user_id,
      //     algo: "nim".to_string(),
      //     coin_id: share.coin_id,
      //     hashrate: 0.0,
      //     shares_per_min: 0.0,
      //   });
      //   worker.count += 1;
      //   worker.last_share_time = share.timestamp;
      //   worker.difficulty_sum += share.difficulty;
      //   worker.difficulty = share.difficulty;
      // }

      // let mut counter = 0;
      // //todo change to multithread the updates
      // for mut worker in worker_dict.values_mut() {
      //   worker.hashrate = worker.difficulty_sum * 65536000.0 / WINDOW_LENGTH as f64 / 1000.0;
      //   worker.shares_per_min = worker.count as f64 / (WINDOW_LENGTH as f64 / 60.0);
      //   update_worker_hashrate(
      //     &conn,
      //     worker.worker_id,
      //     worker.last_share_time as i32,
      //     worker.shares_per_min,
      //     worker.hashrate,
      //     worker.difficulty,
      //   )
      //   .unwrap();
      // }
      // println!("Done updating, map size: {}", worker_dict.len());
      // // println!("Worker 1: {}", worker_dict.get();
      // // println!("worker dict: {:?}", worker_dict.keys());
    }
  })
}

fn keygen(coin_id: i16, algo: &String) -> String {
  format!("{}-{}", coin_id, algo)
  // return coin_id + algo
}
// fn generate_key(coin_id: i16, algo: Algos, user_id: i32, worker_name: &String) -> String {
//   format!("{}-{}-{}-{}", coin_id, algo, user_id, worker_name,)
// }

// fn handle_share(
//   //worker_dict: &mut WorkerDict,
//   shares: &mut ShareQueue,
//   share: ShareQueueMinifiedObj,
// ) {
//   // update the hasmap
//   //update_dict_with_new_share(worker_dict, &share);
//   // add to the queue
//   shares.push_back(share);
// }
// fn update_dict_with_new_share(worker_dict: &mut WorkerDict, share: &ShareQueueMinifiedObj) {
//   let key: String = generate_key(share.coin_id, share.algo, share.user_id, &share.worker_name);

//   if let Some(worker) = worker_dict.get_mut(&key) {
//     // worker exists already
//     worker.end_time = share.timestamp;
//     worker.count += 1;
//     worker.difficulty += share.difficulty;
//   // worker.hashrate = worker.calc_hashrate();
//   } else {
//     // new worker
//     let worker = WorkerDictObj::new(
//       share.timestamp,
//       share.timestamp,
//       &share.worker_name,
//       share.user_id,
//       share.algo,
//       share.coin_id,
//     );
//     worker_dict.insert(key.to_string(), worker);
//   }
// }

// fn update_dict_by_removing_share(worker_dict: &mut WorkerDict, share: &ShareQueueMinifiedObj){
//     let key: String = generate_key(share.coin_id, share.algo, share.user_id, &share.worker_name);

//       // set the user_scores dict to the proper key
//   if !worker_dict.contains_key(&key) {
//     panic!("WTF HOW DID WE GET HERE GREG????");
//   }

//   if let Some(worker) = worker_dict.get_mut(&key){
//       // somehow remove the start time and move the starttime of the next share ?
//       worker.difficulty -= share.difficulty;

//   }

// }
