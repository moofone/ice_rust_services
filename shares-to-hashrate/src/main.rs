/*
values needed from each share - min struct
    - user_id
    - timestamp - of share
    - algo
    - worker_id
    - coin_id
    - worker_name
    - difficulty

hashrate calc requires:
    - summed difficulty of shares in window
    - algo for the target
    - window length
sharesPerMin requires:
    - count of shares in window
    - window length

dict requires
    - key
        - worker_name - if empty, replace with n/a
        - user_id - if empty, replace with 0
        - algo
        - coin_id
    - start_time
    - end_time
    - count
    - difficulty


*/

extern crate shared;

// use nats;

// use shared::db_mysql::models::Worker;
use shared::enums::*;
use shared::nats::establish_nats_connection;
use shared::nats::models::ShareNats;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;

struct ShareQueueMinifiedObj {
    user_id: i32,
    timestamp: i64,
    algo: Algos,
    worker_id: i32,
    coin_id: i16,
    worker_name: String,
    difficulty: f64,
}

struct WorkerDictObj {
    start_time: i64,
    end_time: i64,
    count: i16,
    difficulty: f64,
    worker_name: String,
    user_id: i32,
    algo: Algos,
    coin_id: i16,
    hashrate: f64,
}
impl WorkerDictObj {
    fn calc_hashrate(self) -> f64 {
        let target = Algos::get_target(&self.algo) as f64;
        let interval = (self.end_time - self.start_time) as f64;
        (self.difficulty * target) / interval / 1000.0
    }
    fn new(
        start: i64,
        end: i64,
        worker_name: &String,
        user_id: i32,
        algo: Algos,
        coin_id: i16,
    ) -> WorkerDictObj {
        WorkerDictObj {
            start_time: start,
            end_time: end,
            count: 0,
            difficulty: 0.0,
            worker_name: worker_name.to_string(),
            user_id: user_id,
            algo: algo,
            coin_id: coin_id,
            hashrate: 0.0,
        }
    }
}
impl From<ShareNats> for ShareQueueMinifiedObj {
    fn from(s: ShareNats) -> Self {
        ShareQueueMinifiedObj {
            user_id: s.user_id,
            timestamp: s.timestamp,
            algo: Algos::from_i16(s.algo),
            worker_id: s.worker_id,
            coin_id: s.coin_id,
            worker_name: "".to_string(), // s.worker_name,
            difficulty: s.difficulty,
        }
    }
}

type ShareQueue = VecDeque<ShareQueueMinifiedObj>;
type WorkerDict = HashMap<String, WorkerDictObj>;

#[tokio::main]
async fn main() {
    let mut tasks = Vec::new();
    //setup nats
    let nc = establish_nats_connection();
    let coins: Vec<i32> = vec![2422, 2122];

    let shares = Arc::new(Mutex::new(ShareQueue::new()));

    //-----------------------SHARES LISTENER--------------------------------
    // loads shares into the queue when received
    // shares are stored in a HashMap for that worker
    // key is in the form of coinid-algo-user_id-worker_name
    {
        for coin in coins {
            // setup nats channel
            let channel = format!("shares.{}", coin.to_string());
            let sub = nc.subscribe(&channel).unwrap();
            // prep queue to be used in a thread
            let shares = shares.clone();
            //let worker_dict = worker_dict.clone();

            // spawn a thread for this channel to listen to shares
            let share_task = tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_millis(50));
                loop {
                    // check if a share is ready
                    if let Some(msg) = sub.try_next() {
                        // prase the share, loc the queue, and add it
                        let share = parse_share(&msg.data);
                        let mut sha = shares.lock().unwrap();
                        //let mut dict = worker_dict.lock().unwrap();
                        handle_share(
                            //&mut *dict,
                            &mut *sha, share,
                        );
                    // shares.push_back(share);
                    } else {
                        interval.tick().await;
                    }
                }
            });
            tasks.push(share_task);
        }
    }

    //-----------------------PROCESS HASHRATES--------------------------------
    // build a HashMap of workers from the queue
    //
    {
        let shares = shares.clone();

        // spawn a thread for this channel to listen to shares
        let process_task = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(50));
            loop {
                interval.tick().await;

                // createa  hashmap to hold workers
                let mut worker_dict = WorkerDict::new();
                let shares = shares.lock().unwrap();

                // add each share from the queue to the worker dict
                for share in shares.iter() {
                    update_dict_with_new_share(&mut worker_dict, &share);
                }

                // prcess hashrates
                for (_, worker) in worker_dict.iter_mut() {
                    // *worker.hashrate = worker.calc_hashrate();
                }
            }
        });
        tasks.push(process_task);
    }

    for handle in tasks {
        handle.await.unwrap();
    }
}
fn parse_share(msg: &Vec<u8>) -> ShareQueueMinifiedObj {
    // Some JSON input data as a &str. Maybe this comes from the user.
    // Parse the string of data into serde_json::Value.
    let s: ShareNats = serde_json::from_slice(&msg).unwrap();
    ShareQueueMinifiedObj::from(s)
}

fn generate_key(coin_id: i16, algo: Algos, user_id: i32, worker_name: &String) -> String {
    format!("{}-{}-{}-{}", coin_id, algo, user_id, worker_name,)
}

fn handle_share(
    //worker_dict: &mut WorkerDict,
    shares: &mut ShareQueue,
    share: ShareQueueMinifiedObj,
) {
    // update the hasmap
    //update_dict_with_new_share(worker_dict, &share);
    // add to the queue
    shares.push_back(share);
}
fn update_dict_with_new_share(worker_dict: &mut WorkerDict, share: &ShareQueueMinifiedObj) {
    let key: String = generate_key(share.coin_id, share.algo, share.user_id, &share.worker_name);

    if let Some(worker) = worker_dict.get_mut(&key) {
        // worker exists already
        worker.end_time = share.timestamp;
        worker.count += 1;
        worker.difficulty += share.difficulty;
    // worker.hashrate = worker.calc_hashrate();
    } else {
        // new worker
        let worker = WorkerDictObj::new(
            share.timestamp,
            share.timestamp,
            &share.worker_name,
            share.user_id,
            share.algo,
            share.coin_id,
        );
        worker_dict.insert(key.to_string(), worker);
    }
}

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