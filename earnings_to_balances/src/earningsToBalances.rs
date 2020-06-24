extern crate shared;

use sentry::{capture_message, integrations::failure::capture_error, Level};
use shared::db_mysql::{
  establish_mysql_connection,
  helpers::{
    accounts::{get_account_mysql, update_account_balance_mysql},
    earnings::{
      delete_earning_mysql, get_earnings_unprocessed_mysql, update_earning_processed_mysql,
    },
  },
  models::{AccountMYSQL, EarningMYSQL},
  MysqlPool,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
const PROCESS_INTERVAL: u64 = 30;

#[tokio::main]
async fn main() {
  let _guard =
    sentry::init("https://b26323c569294eea999ae5d37436f0d7@sentry.watlab.icemining.ca/11");
  capture_message("Earnings to balance is now live", Level::Info);

  // setup tasks
  let mut tasks = Vec::new();
  //setup msqyl
  let mysql_pool = match establish_mysql_connection() {
    Ok(p) => p,
    Err(e) => panic!("MYSQL FAILED: {}", e),
  };

  //-----------------------interval loop--------------------------------
  {
    // spawn a thread for the loop
    let process_task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_secs(PROCESS_INTERVAL));

      let mysql_pool = mysql_pool.clone();
      loop {
        interval.tick().await;
        println!("Starting earnings processor");

        // grab a mysql pool connection
        let conn = match mysql_pool.get() {
          Ok(c) => c,
          Err(e) => {
            println!("cant get mysql connection? ");
            // no mysql, go to next loop
            continue;
          }
        };

        let earnings: Vec<EarningMYSQL> = match get_earnings_unprocessed_mysql(&conn) {
          Ok(earnings) => earnings,
          Err(e) => {
            // wont error on 0 earnings, this is a different error
            println!("err getting earnings: {}", e);
            // continue to next loop
            continue;
          }
        };

        // shouldnt need this as no earnigns is just an empty iterable vec
        // no earnings, next loop
        if earnings.len() < 1 {
          // #[cfg(debug_assertions)]
          println!("no earnings to process");
          continue;
        }

        // loop through the earnings
        for earning in earnings {
          // #[cfg(debug_assertions)]
          // println!("Processing Earning: {}", &earning.id);

          // TODO find if the coinid exists, then detelete the earning if not

          // update the earning to processed
          match update_earning_processed_mysql(&conn, &earning) {
            Ok(updated) => (),
            Err(e) => {
              println!("earning failed to update: {}", &earning.id);
              // move to next earning, dont update this balance
              continue;
            }
          }

          // get account
          let account: AccountMYSQL = match get_account_mysql(&conn, earning.userid, earning.coinid)
          {
            Ok(a) => a,
            Err(e) => {
              // #[cfg(debug_assertions)]
              println!(
                "user not found: {}, deleting earning: {}, error: {}",
                &earning.userid, &earning.id, e
              );

              // move to next user
              // bad user, delete earning and move on
              match delete_earning_mysql(&conn, &earning) {
                Ok(deleted) => (),
                Err(e) => println!("WTF delete failed: {}", &earning.id),
              }
              continue;
            }
          };

          // if user found, print the id
          // #[cfg(debug_assertions)]
          // println!("Updating account: {:?}", account.id);

          // maybe need a match
          // update the account balance
          match update_account_balance_mysql(&conn, &account, earning.amount) {
            Ok(updated) => (),
            Err(e) => {
              println!(
                "account failed to update balance. Account: {}, Earning: {}",
                &account.id, &earning.id
              );
              // move to next user
              // might need to redo earning
              continue;
            }
          }
          // #[cfg(debug_assertions)]
          // println!("Account balance updated: {}", account.id);
        }
      }
    });
    tasks.push(process_task);
  }
  for handle in tasks {
    handle.await.unwrap();
  }
}
