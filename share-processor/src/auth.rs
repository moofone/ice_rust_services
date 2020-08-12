// use diesel::prelude::*;
use futures::join;
use shared::db_mysql::{
  helpers::accounts::{get_account_by_username_mysql, insert_account_mysql},
  helpers::workers::{
    get_disconnected_worker_by_worker_name_mysql, insert_worker_mysql, update_worker_mysql,
  },
  models::{WorkerMYSQL, WorkerMYSQLInsertable},
  MysqlPool, MysqlPooledConnection,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use shared::nats::models::{ShareNats, StratumAuthNatsNIM, StratumAuthResponseNats};
use shared::nats::NatsConnection;

const SHARE_THRESHHOLD: i16 = 3;
pub struct GenericAccount {
  // generic account that we can convert stratumauthnats into to use for workers
  pub user_name: String,
  pub owner_type: String,
  pub coin_id: i16,
  pub share_count: i16,
}
type AccountQueue = Arc<Mutex<HashMap<String, GenericAccount>>>;

pub async fn run_jobs(env: &String, mysql_pool: &MysqlPool, nc: &NatsConnection) {
  let account_queue = Arc::new(Mutex::new(HashMap::new()));

  //-----------------------auth LISTENER--------------------------------
  let auth = stratum_auth_listener(env, &mysql_pool, nc, &account_queue.clone());
  //-----------------------SHARES LISTENER--------------------------------
  let _share_listener = share_listener(env, &mysql_pool, nc, &account_queue.clone());

  join!(auth, _share_listener);
}
pub fn stratum_auth_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
  account_queue: &AccountQueue,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  println!("waiting on auth message");
  let mysql_pool = mysql_pool.clone();
  let subject;
  if env == "dev" {
    subject = format!("dev.stratum.auth.2408");
  } else {
    subject = format!("stratum.auth.2408");
  }
  let sub = match nc.queue_subscribe(&subject, "stratum_auth_worker") {
    // let sub = match nc.subscribe(&subject) {
    Ok(sub) => sub,
    Err(e) => panic!("Queue stratum auth listener failed: {}", e),
  };

  let account_queue = account_queue.clone();
  tokio::task::spawn(async move {
    //  grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();
    let mut counter = 0;
    for msg in sub.messages() {
      // counter += 1;
      // if counter % 100 == 0 {
      println!("Msg: {} (printing every 100)", msg.subject);
      // counter = 0;
      // }

      //  grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();
      // println!("MSG: {}", &msg);

      let stratum_auth_nats_nim = match parse_msg_auth(&msg.data) {
        Ok(a) => a,
        Err(e) => {
          println!("failed to parse stratum auth nats msg: {}", e);
          continue;
        }
      };

      let mut account_queue = account_queue.clone();

      tokio::task::spawn(async move {
        // println!("Msg: {}", msg.subject);

        // grab a mysql pool connection
        let conn = match mysql_pool.get() {
          Ok(conn) => conn,
          Err(e) => {
            // crash and sentry BIG ISSUE
            println!("Error mysql conn. e: {}", e);
            panic!("error getting mysql connection e: {}",);
          }
        };

        // get or insert the account
        get_or_insert_account_nim(&conn, &stratum_auth_nats_nim, &mut account_queue);

        // // insert or update the worker
        // match insert_or_update_worker(&conn, &gen_account, &stratum_auth_nats_nim) {
        //   Ok(worker) => {
        //     let nats_response = StratumAuthResponseNats {
        //       owner_id: gen_account.owner_id,
        //       worker_id: worker.id,
        //       uuid: worker.uuid,
        //     };
        //     // println!(
        //     //   "nats response: {} - {} - {}",
        //     //   nats_response.owner_id,
        //     //   nats_response.worker_id,
        //     //   nats_response.uuid.len()
        //     // );
        //     let response = rmp_serde::to_vec_named(&nats_response).unwrap();
        //     match msg.respond(&response) {
        //       Ok(_) => (),
        //       Err(e) => println!("Failed to send response: {}", e),
        //     }

        //     let elapsed = SystemTime::now()
        //       .duration_since(UNIX_EPOCH)
        //       .unwrap()
        //       .as_millis()
        //       - time_current;
        //     if elapsed > 1000 {
        //       println!("Done Sending worker: {},  took: {}ms", worker.id, elapsed);
        //     }
        //   }
        //   Err(e) => {
        //     println!("Error inserting worker: {}", e);
        //   }
        // }
        // let worker = match insert_or_update_worker(&conn, &gen_account, &stratum_auth_nats_nim){
        //   Ok(w)=> w,
        //   Err(e)=> println!("insert or update worker failed: {}",e)
        // };
      });
    }
  })
}

fn parse_msg_auth(msg: &Vec<u8>) -> Result<StratumAuthNatsNIM, rmp_serde::decode::Error> {
  rmp_serde::from_read_ref(&msg)
  // let auth: StratumAuthNatsNIM = match rmp_serde::from_read_ref(&msg) {
  //   Ok(auth) => auth,
  //   Err(e) => panic!("Error parsing Startum auth nats. e: {}", e),
  // };
  // // println!("stratum auth nats nim : {:?}", auth);
  // Ok(auth)
}

fn get_or_insert_account_nim(
  pooled_conn: &MysqlPooledConnection,
  new_msg: &StratumAuthNatsNIM,
  account_queue: &mut AccountQueue,
) {
  let is_username = Some(new_msg.username.find('@'));
  let gen_account = GenericAccount {
    user_name: new_msg.username.to_string(),
    owner_type: "account".to_string(), //new_msg.owner_type,
    coin_id: new_msg.coin_id,
    share_count: 0,
  };
  // println!("{}", &new_msg.username);
  if is_username != None {
    match get_account_by_username_mysql(pooled_conn, &gen_account.user_name) {
      Ok(_) => (), // found
      Err(e) => {
        println!("Account not found, adding to queue - {}", e);
        // not found
        // add it to the queue to be added
        let mut account_queue = account_queue.lock().unwrap();
        account_queue
          .entry(gen_account.user_name.to_string())
          .or_insert(gen_account);
        // account_queue.push(gen_account);
        // match insert_account_mysql(pooled_conn, &new_msg.username, new_msg.coin_id as i32) {
        //   Ok(a) => a,
        //   Err(e) => panic!("insert failed. e: {}", e),
        // }
      }
    };
    // gen_account.owner_id = account.id;
    // gen_account.owner_type = "account".to_string();
    // gen_account.coin_id = account.coinid;
  }
  // println!("account: {}", gen_account.owner_id);
  // gen_account
}

// listen for shares and push to queue
fn share_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
  account_queue: &AccountQueue,
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

  let mysql_pool = mysql_pool.clone();
  // start a thread to listen for messages
  let account_queue = account_queue.clone();
  tokio::task::spawn(async move {
    println!("Listening for shares");
    for msg in sub.messages() {
      println!("SHARE!!!!!!");
      //TODO parse the share, check the map, if its needs to be inserted into the accounts table, do taht too
      if let Ok(share) = parse_share_msg(&msg.data) {
        let mut account_queue = account_queue.lock().unwrap();
        if !account_queue.is_empty() {
          if let Some(account) = account_queue.get_mut(&share.user_name) {
            account.share_count += 1;

            if account.share_count >= SHARE_THRESHHOLD {
              account_queue.remove(&share.user_name);
              drop(account_queue);

              // grab a mysql pool connection
              let conn = match mysql_pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                  // crash and sentry BIG ISSUE
                  println!("Error mysql conn. e: {}", e);
                  panic!("error getting mysql connection e: {}",);
                }
              };
              match insert_account_mysql(&conn, &share.user_name, share.coin_id as i32) {
                Ok(_) => (),
                Err(e) => println!("Error inserting account: {}, e : {}", share.user_name, e),
              };
            }
          }
        }
      }
    }
  })
}

fn parse_share_msg(msg: &Vec<u8>) -> Result<ShareNats, rmp_serde::decode::Error> {
  let share: ShareNats = match rmp_serde::from_read_ref(&msg) {
    Ok(share) => share,
    Err(e) => {
      println!("Error parsing share. e: {}", e);
      return Err(e);
    }
  };
  Ok(share)
}

// fn insert_or_update_worker(
//   pooled_conn: &MysqlPooledConnection,
//   account: &GenericAccount,
//   new_msg: &StratumAuthNatsNIM,
// ) -> Result<WorkerMYSQL, Box<dyn std::error::Error>> {
//   // println!(
//   //   "gen account: {}, {}, {}",
//   //   account.owner_id, account.owner_type, new_msg.worker_name
//   // );
//   if new_msg.worker_name.len() > 0 {
//     if let Ok(mut worker) = get_disconnected_worker_by_worker_name_mysql(
//       pooled_conn,
//       account.owner_id,
//       &account.owner_type,
//       &new_msg.worker_name,
//     ) {
//       // println!("worker: {:?}", worker.id);
//       if worker.state != "connected" && worker.state != "active" {
//         worker.state = "connected".to_string();
//         worker.uuid = new_msg.uuid.to_string();
//         worker.time = Some(new_msg.time);
//         worker.pid = Some(new_msg.pid);
//         worker.stratum_id = new_msg.stratum_id.to_string();
//         // println!("Updating worker: {}", worker.worker);
//         match update_worker_mysql(pooled_conn, &worker) {
//           Ok(w) => w,
//           Err(e) => {
//             println!("Failed to update worker. e: {}", e);
//             return Err(format!("Failed to update worker. e: {}", e))?;
//           }
//         }
//         return Ok(worker);
//         // Ok(worker);
//       }
//     }
//   }
//   let worker = WorkerMYSQLInsertable {
//     coinid: new_msg.coin_id,
//     userid: account.owner_id,
//     worker: new_msg.worker_name.to_string(),
//     hashrate: 0.0,
//     difficulty: 0.0,
//     owner_id: account.owner_id,
//     owner_type: account.owner_type.to_string(),
//     uuid: new_msg.uuid.to_string(),
//     state: "connected".to_string(),
//     ip: new_msg.ip.to_string(),
//     version: new_msg.version.to_string(),
//     password: new_msg.consensus_mode.to_string(),
//     algo: new_msg.algo.to_string(),
//     mode: new_msg.mode.to_string(),
//     stratum_id: new_msg.stratum_id.to_string(),
//     time: new_msg.time,
//     pid: new_msg.pid,
//     name: new_msg.username.to_string(),
//     last_share_time: None,
//     shares_per_min: None,
//   };
//   let new_worker = match insert_worker_mysql(pooled_conn, worker) {
//     Ok(w) => w,
//     Err(e) => {
//       println!("Failed to insert worker. e: {}", e);
//       return Err(format!("Failed to insert worker. e: {}", e))?;
//     }
//   };
//   // println!("Inserting new worker: {}", new_worker.worker);
//   Ok(new_worker)
// }
