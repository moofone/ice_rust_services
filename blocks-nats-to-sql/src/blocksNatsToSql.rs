/**
 *
 * TODO
 * - check to see how to do the insert properly
 *
 */
extern crate shared;

// use sentry::{capture_message, integrations::failure::capture_error, Level};
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::accounts::get_account_by_username_mysql,
  helpers::blocks::insert_blocks_mysql,
  helpers::kdablocks::insert_kdablocks_mysql,
  models::{BlockMYSQLInsertable, KDABlockMYSQLInsertable},
  MysqlPool, MysqlPooledConnection,
};
use shared::nats::establish_nats_connection;
use shared::nats::models::{BlockNats, KDABlockNats};
// use std::time::{Duration, SystemTime, UNIX_EPOCH};
// use tokio::time;

// const INSERTINTERVAL: u64 = 50;
// const DELETEINTERVAL: u64 = 2000;
// const WINDOW_LENGTH: u64 = 2 * 60 * 60;

#[tokio::main]
async fn main() {
  // let _guard =
  //   sentry::init("https://f8ee06fb619843b1ae923d9111d855a9@sentry.watlab.icemining.ca/10");

  let mut tasks = Vec::new();
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

  // capture_message("KDA Blocks listening", Level::Info);

  //-----------------------KDA BLOCKS LISTENER--------------------------------
  {
    // grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();
    // setup nats channel
    let subject;
    let env = "dev";
    if env == "dev" {
      subject = format!("dev.stratum.kdablocks");
    } else {
      subject = format!("stratum.kdablocks");
    }
    // let subject = format!("dev.stratum.kdablocks");
    let sub = match nc.queue_subscribe(&subject, "kdablocks_worker") {
      // let sub = match nc.subscribe(&subject) {
      Ok(sub) => sub,
      Err(e) => panic!("Queue kdablock failed: {}", e),
    };

    println!("spawning block task");
    // spawn a thread for this channel to listen to shares
    let blocks_task = tokio::task::spawn(async move {
      // grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();
      println!("about to listen loop sub");

      for msg in sub.messages() {
        println!("kdablock from nats");

        // // grab a copy to be passed into the thread
        let mysql_pool = mysql_pool.clone();
        println!("about to spawn thread");

        // spawn a thread for the block
        tokio::task::spawn_blocking(move || {
          println!("processing block");
          // grab a mysql pool connection
          let conn = match mysql_pool.get() {
            Ok(conn) => conn,
            Err(e) => {
              // crash and sentry BIG ISSUE
              println!("Error mysql conn. e: {}", e);
              panic!("error getting mysql connection.e: {}", e);
            }
          };
          // parse the block
          let kdablock = match parse_kdablock(&msg.data, &conn) {
            Ok(val) => val,
            Err(e) => {
              // massive sentry error
              println!("Error parsing kdablock: {}, subject: {}", e, &msg.subject);
              // return from tokio async block and move on
              return;
            }
          };

          // let account_id = get_account_by_username_mysql(&conn, kdablock.user)
          // create a queue of blocks ( incase we want to scale or bulk insert)
          let mut kdablocks: Vec<KDABlockMYSQLInsertable> = Vec::new();
          let height = kdablock.height;
          kdablocks.push(kdablock);
          match insert_kdablocks_mysql(&conn, kdablocks) {
            Ok(_) => (),
            Err(e) => {
              // yell to sentry that we failed to insert a block
              println!("block not inserted. e: {}", e);
              return Err(format!("Insert failed: {}, {}", height, e)).unwrap();
            }
          };
          println!("block inserted\n");
        });
      }
      println!("done listening to messages");
    });
    tasks.push(blocks_task);
    // }
  }

  //-----------------------BLOCKS LISTENER--------------------------------
  {
    // grab a copy fo the pool to passed into the thread
    let mysql_pool = mysql_pool.clone();
    // setup nats channel
    let subject;
    let env = "dev";
    if env == "dev" {
      subject = format!("dev.stratum.blocks");
    } else {
      subject = format!("stratum.blocks");
    }
    let sub = match nc.queue_subscribe(&subject, "blocks_worker") {
      // let sub = match nc.subscribe(&subject) {
      Ok(sub) => sub,
      Err(e) => panic!("Queue block failed: {}", e),
    };

    println!("spawning block task");
    // spawn a thread for this channel to listen to shares
    let blocks_task = tokio::task::spawn(async move {
      // grab a copy fo the pool to passed into the thread
      let mysql_pool = mysql_pool.clone();
      println!("about to listen loop sub");

      for msg in sub.messages() {
        println!("block from nats");

        // // grab a copy to be passed into the thread
        let mysql_pool = mysql_pool.clone();
        println!("about to spawn thread");

        // spawn a thread for the block
        tokio::task::spawn_blocking(move || {
          println!("processing block");
          // parse the block

          // grab a mysql pool connection
          let conn = match mysql_pool.get() {
            Ok(conn) => conn,
            Err(e) => {
              // crash and sentry BIG ISSUE
              println!("Error mysql conn. e: {}", e);
              panic!("error getting mysql connection.e: {}", e);
            }
          };
          let block = match parse_block(&msg.data, &conn) {
            Ok(val) => val,
            Err(e) => {
              // massive sentry error
              println!("Error parsing block: {}, subject: {}", e, &msg.subject);
              // return from tokio async block and move on
              return;
            }
          };

          // create a queue of blocks ( incase we want to scale or bulk insert)
          let mut blocks: Vec<BlockMYSQLInsertable> = Vec::new();
          let height = block.height;
          blocks.push(block);
          match insert_blocks_mysql(&conn, blocks) {
            Ok(_) => (),
            Err(e) => {
              // yell to sentry that we failed to insert a block
              println!("block not inserted. e: {}", e);
              return Err(format!("Insert failed: {}, {}", height, e)).unwrap();
            }
          };
          println!("block inserted\n");
        });
      }
      println!("done listening to messages");
    });
    tasks.push(blocks_task);
    // }
  }
  // loop {}
  for handle in tasks {
    handle.await.unwrap();
  }
}

// converts nats message to KDABlockMYSQLInsertable
fn parse_kdablock(
  msg: &Vec<u8>,
  conn: &MysqlPooledConnection,
) -> Result<KDABlockMYSQLInsertable, rmp_serde::decode::Error> {
  let b: KDABlockNats = match rmp_serde::from_read_ref(&msg) {
    Ok(b) => b,
    Err(err) => return Err(err),
  };

  let block = kdablocknats_to_blockmysqlinsertable(b, conn);
  Ok(block)
}
// converts nats message to BlockMYSQLInsertable
fn parse_block(
  msg: &Vec<u8>,
  conn: &MysqlPooledConnection,
) -> Result<BlockMYSQLInsertable, rmp_serde::decode::Error> {
  let b: BlockNats = match rmp_serde::from_read_ref(&msg) {
    Ok(b) => b,
    Err(err) => return Err(err),
  };
  let mut block = blocknats_to_blockmysqlinsertable(b, conn);
  if block.mode.eq("") {
    block.mode = "normal".to_string();
  }
  Ok(block)
}

fn kdablocknats_to_blockmysqlinsertable(
  b: KDABlockNats,
  conn: &MysqlPooledConnection,
) -> KDABlockMYSQLInsertable {
  let user_id: Option<i32> = match get_account_by_username_mysql(conn, &b.user_name) {
    Ok(account) => Some(account.id),
    Err(_) => None,
  };
  KDABlockMYSQLInsertable {
    coin_id: b.coin_id as i32,
    height: b.height,
    time: b.time as i32,
    userid: user_id,
    // workerid: b.workerid,
    rigname: b.rig_name,
    confirmations: b.confirmations,
    amount: b.amount,
    difficulty: b.difficulty,
    difficulty_user: b.difficulty_user,
    blockhash: b.blockhash,
    algo: b.algo,
    category: b.category,
    stratum_id: b.stratum_id,
    mode: b.mode,
    party_pass: b.party_pass,
    chainid: b.chainid,
    node_id: b.node_id,
  }
}
fn blocknats_to_blockmysqlinsertable(
  b: BlockNats,
  conn: &MysqlPooledConnection,
) -> BlockMYSQLInsertable {
  let user_id: Option<i32> = match get_account_by_username_mysql(conn, &b.user_name) {
    Ok(account) => Some(account.id),
    Err(_) => None,
  };
  BlockMYSQLInsertable {
    coin_id: b.coin_id as i32,
    height: b.height as i32,
    time: b.time,
    userid: user_id,
    rigname: b.rig_name,
    // workerid: b.workerid,
    confirmations: b.confirmations,
    amount: b.amount,
    difficulty: b.difficulty,
    difficulty_user: b.difficulty_user,
    blockhash: b.blockhash,
    algo: b.algo,
    category: b.category,
    stratum_id: b.stratum_id,
    mode: b.mode,
    party_pass: b.party_pass,
    state: 0,
    duration: b.duration,
    shares: b.shares
    // chainid: b.chainid,
    // node_id: b.node_id,
  }
}
