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

mod auth;
mod worker_scalar;
mod worker_scalar_rework;

const WINDOW_LENGTH: u64 = 60 * 5; //s
const HASHRATE_INTERVAL: u64 = 1 * 15; //s

// struct ShareProcesserServer {
//   env: String,
//   nc: nats::Connection,
//   mysql_pool: MysqlPool,
//   config: HashMap<String, ShareProcessorConfigObj>,
//   test: Arc<Mutex<Vec<String>>>,
// }
// impl ShareProcesserServer {
//   fn new() -> ShareProcesserServer {
//     // setup dotenv
//     dotenv().ok();

//     // setup nats
//     let nc = match establish_nats_connection() {
//       Ok(n) => n,
//       Err(e) => {
//         println!("Nats did not connect: {}", e);
//         panic!("Nats did not connect: {}", e);
//       }
//     };

//     //setup msqyl
//     let mysql_pool = match establish_mysql_connection() {
//       Ok(p) => p,
//       Err(e) => panic!("MYSQL FAILED: {}", e),
//     };
//     let mut config_nats = ShareProcessorConfigNats {
//       config: HashMap::new(),
//     };
//     config_nats.config.insert(
//       "2408-argon2d".to_string(),
//       ShareProcessorConfigObj {
//         window_length: 300,
//         algo_target: 65536000,
//       },
//     );
//     config_nats.config.insert(
//       "2423-blake2s".to_string(),
//       ShareProcessorConfigObj {
//         window_length: 300,
//         algo_target: 1000,
//       },
//     );
//     let config = config_nats.config;
//     // return the initialized dpplns server
//     ShareProcesserServer {
//       env: env::var("ENVIRONMENT_MODE").expect("ENVIRONMENT_MODE not set"),
//       nc: nc,
//       config: config,
//       mysql_pool: mysql_pool,
//       test: Arc::new(Mutex::new(Vec::new())),
//     }
//   }
//   async fn run(&self) -> Result<(), std::io::Error> {
//     let worker_scalar_job = worker_scalar::run_jobs(&self.env, &self.mysql_pool, &self.nc);
//     let auth_job = auth::run_jobs(&self.env, &self.mysql_pool, &self.nc);
//     join!(worker_scalar_job, auth_job);
//     Ok(())
//   }
// }
// #[tokio::main]
// async fn main() -> Result<(), std::io::Error> {
//   let share_processor = ShareProcesserServer::new();
//   share_processor.run().await?;
//   Ok(())
// }
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
  config_nats.config.insert(
    "2423-blake2s".to_string(),
    ShareProcessorConfigObj {
      window_length: 300,
      algo_target: 1000,
    },
  );
  let config = config_nats.config;

  // let mut share_queues_map = Arc::new(Mutex::new(ShareQueuesMapType::new()));

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

  let worker_scalar_job = worker_scalar::run_jobs(&env, &mysql_pool, &nc);
  // let worker_scalar_job = worker_scalar_rework::run_jobs(&env, mysql_pool.clone(), nc.clone());
  let auth_job = auth::run_jobs(&env, &mysql_pool, &nc);

  join!(worker_scalar_job, auth_job);
  // join!(auth_job);
}
