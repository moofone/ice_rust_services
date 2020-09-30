// use diesel::prelude::*;
use futures::join;
use shared::db_mysql::{
  helpers::accounts::{get_account_by_username_mysql, insert_account_mysql},
  helpers::workers::{
    get_disconnected_worker_by_worker_name_mysql, insert_worker_mysql, update_worker_mysql,
  },
  models::{AccountMYSQL, WorkerMYSQL, WorkerMYSQLInsertable},
  MysqlPool, MysqlPooledConnection,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{interval_at, Duration, Instant};

use shared::nats::models::{ShareNats, StratumAuthNatsNIM};
use shared::nats::NatsConnection;

const SHARE_THRESHOLD: i8 = 1;
const TRIM_INTERVAL: u64 = 60; //s
const TRIM_THRESHOLD: i32 = 60; //s

pub struct AuthStruct {
  pub msg: StratumAuthNatsNIM,
  pub share_count: i8,
}
pub struct GenericAccount {
  // generic account that we can convert stratumauthnats into to use for workers
  pub owner_id: i32,
  pub user_name: String,
  pub owner_type: String,
  pub coin_id: i32,
  // pub share_count: i16,
  // pub create_time: u64,
}
type AccountQueue = Arc<Mutex<HashMap<String, AuthStruct>>>;

pub async fn run_jobs(env: &String, mysql_pool: &MysqlPool, nc: &NatsConnection) {
  let account_queue = Arc::new(Mutex::new(HashMap::new()));

  //-----------------------auth LISTENER--------------------------------
  let auth = stratum_auth_listener(env, &mysql_pool, nc, &account_queue.clone());
  //-----------------------SHARES LISTENER--------------------------------
  let _share_listener = share_listener(env, &mysql_pool, nc, &account_queue.clone());
  //-----------------------Expired accounts job--------------------------------
  let _expired_job = expired_job(env, &mut account_queue.clone());

  join!(auth, _share_listener, _expired_job);
}
pub fn stratum_auth_listener(
  env: &String,
  mysql_pool: &MysqlPool,
  nc: &NatsConnection,
  account_queue: &AccountQueue,
) -> tokio::task::JoinHandle<()> {
  //  grab a copy fo the pool to passed into the thread
  // println!("waiting on auth message");
  let mysql_pool = mysql_pool.clone();

  let subject;
  if env == "prod" {
    subject = format!("stratum.auth.>");
  } else {
    subject = format!("{}.stratum.auth.>", env);
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
    println!("waiting for auth messages");
    for msg in sub.messages() {
      counter += 1;
      if counter % 100 == 0 {
        println!("Msg: {} (printing every 100)", msg.subject);
        counter = 0;
      }

      //  grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();
      // println!("MSG: {}", &msg);
      // println!("pool cloned");
      let stratum_auth_nats_nim = match parse_msg_auth(&msg.data) {
        Ok(a) => a,
        Err(e) => {
          println!("failed to parse stratum auth nats msg: {}", e);
          continue;
        }
      };
      // println!("message parsed");
      // println!("auth message: {:?}", stratum_auth_nats_nim);

      // println!("querue cloned");
      let mut account_queue = account_queue.clone();

      // tokio::task::spawn(async move {
      // println!("Msg: {}", msg.subject);
      // grab a mysql pool connection
      let conn = match mysql_pool.get() {
        Ok(conn) => {
          // println!("got conn");
          conn
        }
        Err(e) => {
          // crash and sentry BIG ISSUE
          println!("Error mysql conn. e: {}", e);
          panic!("error getting mysql connection e: {}",);
        }
      };

      // println!("Abouty to get or insert account");
      // get or insert the account
      if let Some(gen_account) =
        get_account_or_add_to_queue(&conn, &stratum_auth_nats_nim, &mut account_queue)
      {
        insert_or_update_worker(&conn, &gen_account, &stratum_auth_nats_nim).unwrap();
      }
      // });
      // println!("thread over");
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

fn get_account_or_add_to_queue(
  pooled_conn: &MysqlPooledConnection,
  auth_msg: &StratumAuthNatsNIM,
  account_queue: &mut AccountQueue,
) -> Option<GenericAccount> {
  let is_username = Some(auth_msg.user_name.find('@'));
  // let gen_account = GenericAccount {
  //   user_name: auth_msg.username.to_string(),
  //   owner_type: "account".to_string(), //new_msg.owner_type,
  //   coin_id: auth_msg.coin_id,
  //   share_count: 0,
  //   create_time: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs(),
  // };
  // println!("{}", &new_msg.username);
  if is_username != None {
    match get_account_by_username_mysql(pooled_conn, &auth_msg.user_name) {
      Ok(a) => {
        return Some(GenericAccount {
          owner_id: a.id,
          user_name: a.username,
          coin_id: a.coinid,
          owner_type: "account".to_string(),
        });
        // return Some(a)
      } // found
      Err(e) => {
        // println!(
        //   "Account not found, adding to queue - {},{},",
        //   &auth_msg.user_name, e
        // );
        // not found
        // add it to the queue to be added
        // let user_name
        let auth = AuthStruct {
          msg: auth_msg.clone(),
          share_count: 0,
        };
        let mut account_queue = account_queue.lock().unwrap();
        account_queue
          .entry(auth.msg.user_name.to_string())
          .or_insert(auth);
        // println!("account queue length : {}", account_queue.len());
      }
    }
  // gen_account.owner_id = account.id;
  // gen_account.owner_type = "account".to_string();
  // gen_account.coin_id = account.coinid;
  } else {
    println!("its a username?");
  };
  return None;
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
  if env == "prod" {
    subject = format!("stratum.shares.>");
  } else {
    subject = format!("{}.stratum.shares.>", env);
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
    println!("Listening for shares for auth");
    for msg in sub.messages() {
      // println!("SHARE!!!!!!: {}", msg.subject);
      //TODO parse the share, check the map, if its needs to be inserted into the accounts table, do taht too
      if let Ok(share) = parse_share_msg(&msg.data) {
        let mut account_queue = account_queue.lock().unwrap();
        if !account_queue.is_empty() {
          // println!(
          //   "account queue not empty, share  being checked: {}",
          //   &share.user_name
          // );
          // println!("Share username: {}", share.user_name);
          if let Some(account) = account_queue.get_mut(&share.user_name) {
            // println!("share found in accoutn queue");
            account.share_count += 1;

            if account.share_count >= SHARE_THRESHOLD {
              let account = account_queue.remove(&share.user_name).unwrap();
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
              // println!("Share threshold met, insertin accoutn: {}", share.user_name);
              match insert_account_mysql(&conn, &share.user_name, share.coin_id as i32) {
                Ok(a) => {
                  let gen_account = GenericAccount {
                    owner_id: a.id,
                    user_name: a.username,
                    coin_id: a.coinid,
                    owner_type: "account".to_string(),
                  };
                  insert_or_update_worker(&conn, &gen_account, &account.msg).unwrap();
                  // return Some(GenericAccount {
                  //   owner_id: a.id,
                  //   user_name: a.username,
                  //   coin_id: a.coinid,
                  //   owner_type: "account".to_string(),
                  // });
                  // return Some(a)
                } // found
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
      println!("Error parsing share. e: {}, bytes: {:?}", e, &msg);
      return Err(e);
    }
  };
  Ok(share)
}

fn expired_job(env: &String, account_queue: &mut AccountQueue) -> tokio::task::JoinHandle<()> {
  let account_queue = account_queue.clone();
  tokio::task::spawn(async move {
    let mut interval = interval_at(
      Instant::now() + Duration::from_millis(TRIM_INTERVAL * 1000),
      Duration::from_millis(TRIM_INTERVAL * 1000),
    );

    loop {
      interval.tick().await;
      let expired_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32
        - TRIM_THRESHOLD;
      let mut account_queue = account_queue.lock().unwrap();
      account_queue.retain(|_, account| account.msg.time > expired_time);
      println!("account queue len: {}", account_queue.len());
    }
  })
}
fn insert_or_update_worker(
  pooled_conn: &MysqlPooledConnection,
  account: &GenericAccount,
  new_msg: &StratumAuthNatsNIM,
) -> Result<WorkerMYSQL, Box<dyn std::error::Error>> {
  // println!(
  //   "gen account: {}, {}, {}",
  //   account.owner_id, account.owner_type, new_msg.worker_name
  // );
  if new_msg.worker_name.len() > 0 {
    if let Ok(mut worker) = get_disconnected_worker_by_worker_name_mysql(
      pooled_conn,
      account.owner_id,
      &account.owner_type,
      &new_msg.worker_name,
    ) {
      // println!("worker: {:?}", worker.id);
      if worker.state != "connected" && worker.state != "active" {
        worker.state = "connected".to_string();
        worker.uuid = new_msg.uuid.to_string();
        worker.time = Some(new_msg.time);
        worker.pid = Some(new_msg.pid);
        worker.stratum_id = new_msg.stratum_id.to_string();
        worker.party_pass = Some(new_msg.party_pass.to_string());
        // println!("Updating worker: {}", worker.worker);
        match update_worker_mysql(pooled_conn, &worker) {
          Ok(w) => w,
          Err(e) => {
            println!("Failed to update worker. e: {}", e);
            return Err(format!("Failed to update worker. e: {}", e))?;
          }
        }
        return Ok(worker);
        // Ok(worker);
      }
    }
  }
  let worker = WorkerMYSQLInsertable {
    coinid: new_msg.coin_id,
    userid: account.owner_id,
    worker: new_msg.worker_name.to_string(),
    hashrate: 0.0,
    difficulty: 0.0,
    owner_id: account.owner_id,
    owner_type: account.owner_type.to_string(),
    uuid: new_msg.uuid.to_string(),
    state: "connected".to_string(),
    ip: new_msg.ip.to_string(),
    version: new_msg.version.to_string(),
    password: new_msg.consensus_mode.to_string(),
    algo: new_msg.algo.to_string(),
    mode: new_msg.mode.to_string(),
    stratum_id: new_msg.stratum_id.to_string(),
    time: new_msg.time,
    pid: new_msg.pid,
    name: new_msg.user_name.to_string(),
    party_pass: new_msg.party_pass.to_string(),
    last_share_time: None,
    shares_per_min: None,
  };
  let rigname = worker.worker.to_string();
  let uuid = worker.uuid.to_string();
  println!("about to insert worker.  {}, {}", rigname, uuid);

  let new_worker = match insert_worker_mysql(pooled_conn, worker) {
    Ok(w) => w,
    Err(e) => {
      println!("Failed to insert worker. e: {}, {}, {}", e, rigname, uuid);
      return Err(format!("Failed to insert worker. e: {}", e))?;
    }
  };
  // println!("Inserting new worker: {}", new_worker.worker);
  Ok(new_worker)
}
