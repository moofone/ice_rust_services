pub mod accounts {
  use super::super::{models::AccountMYSQL, schema::accounts::dsl::*};
  use diesel::result::Error;
  use diesel::{mysql::MysqlConnection, prelude::*, update};

  // gets account
  pub fn get_account_mysql(
    conn: &MysqlConnection,
    account_id: i32,
    coin_id: i32,
  ) -> Result<AccountMYSQL, Error> {
    accounts
      .filter(id.eq(account_id))
      .filter(coinid.eq(coin_id))
      .first::<AccountMYSQL>(conn)
  }

  // update account balance
  pub fn update_account_balance_mysql(
    conn: &MysqlConnection,
    account: &AccountMYSQL,
    amount: f64,
  ) -> Result<(), Error> {
    update(accounts.filter(id.eq(account.id)))
      .set(balance.eq(balance + amount))
      .execute(conn);
    Ok(())
  }
}

pub mod earnings {
  use super::super::{
    models::{EarningMYSQL, EarningMYSQLInsertable},
    schema::earnings::dsl::*,
  };
  use diesel::result::Error;
  use diesel::{delete, insert_into, mysql::MysqlConnection, prelude::*, update};
  use std::time::{SystemTime, UNIX_EPOCH};

  /// Inserts earnings to MySQL database.
  pub fn insert_earnings_mysql(
    conn: &MysqlConnection,
    earnings_vec: Vec<EarningMYSQLInsertable>,
  ) -> Result<(), Error> {
    insert_into(earnings)
      .values(&earnings_vec)
      .execute(conn)
      .map(|_| ())
  }

  // get the earnings with status 1
  pub fn get_earnings_unprocessed_mysql(
    conn: &MysqlConnection,
  ) -> Result<Vec<EarningMYSQL>, Error> {
    earnings.filter(status.eq(1)).load::<EarningMYSQL>(conn)
  }

  // update earnings to status 2
  pub fn update_earning_processed_mysql(
    conn: &MysqlConnection,
    earning: &EarningMYSQL,
  ) -> Result<(), Error> {
    update(earnings.filter(id.eq(earning.id)))
      .set(status.eq(2))
      .execute(conn);
    Ok(())
  }

  // delete bad earnings
  pub fn delete_earning_mysql(conn: &MysqlConnection, earning: &EarningMYSQL) -> Result<(), Error> {
    delete(earnings.filter(id.eq(earning.id))).execute(conn);
    Ok(())
  }
}
pub mod kdablocks {
  use super::super::{models::KDABlockMYSQLInsertable, schema::kdablocks::dsl};
  use diesel::result::Error;
  use diesel::{insert_into, mysql::MysqlConnection, prelude::*};

  /// Inserts block to MySQL database.
  pub fn insert_kdablocks_mysql(
    conn: &MysqlConnection,
    kdablocks: Vec<KDABlockMYSQLInsertable>,
  ) -> Result<(), Error> {
    insert_into(dsl::kdablocks)
      .values(&kdablocks)
      .execute(conn)
      .map(|_| ())
  }
}

pub mod shares {}

pub mod blocks {
  use super::super::{models::BlockMYSQL, schema::blocks::dsl::*};
  use diesel::result::Error;
  use diesel::{mysql::MysqlConnection, prelude::*};
  use std::time::{SystemTime, UNIX_EPOCH};

  pub fn get_blocks_unprocessed_mysql(conn: &MysqlConnection) -> Result<Vec<BlockMYSQL>, Error> {
    // select blocks that have not been sent to dpplns
    let mut time_start = (SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs()
      - 3600) as i64;
    blocks
      .filter(category.eq("new"))
      .filter(time.ge(time_start))
      .load::<BlockMYSQL>(conn)
  }

  pub fn update_block_to_unconfirmed_mysql(
    conn: &MysqlConnection,
    block: &BlockMYSQL,
  ) -> Result<(), Error> {
    diesel::update(blocks.filter(id.eq(block.id)))
      .set((category.eq("unconfirmed"), confirmations.eq(0)))
      .execute(conn);
    Ok(())
  }
}
pub mod coins {
  use super::super::{models::Coin, schema::coins::dsl::*};
  use diesel::result::Error;
  use diesel::{mysql::MysqlConnection, prelude::*};

  pub fn get_coins_mysql(conn: &MysqlConnection) -> Vec<Coin> {
    //select id, symbol, whatever other info from coins;
    // for each row, create a new Coin struct and push to vec
    // return vec
    let res = coins
      .filter(enable.eq(1))
      // .limit(5)
      .load::<Coin>(conn)
      .expect("failed loading coins");
    return res;
  }
}
pub mod algorithms {
  use super::super::{models::AlgorithmMYSQL, schema::algorithms::dsl::*};
  use diesel::result::Error;
  use diesel::{mysql::MysqlConnection, prelude::*};
  use std::collections::HashMap;

  pub fn get_algorithms_mysql(
    conn: &MysqlConnection,
  ) -> Result<HashMap<String, AlgorithmMYSQL>, Error> {
    // select blocks that have not been sent to dpplns
    let algosVec = match algorithms.load::<AlgorithmMYSQL>(conn) {
      Ok(a) => a,
      Err(e) => panic!("failed to get algos"),
    };
    let mut algosMap = HashMap::new();
    for algo in algosVec {
      algosMap.insert(algo.name.to_string(), algo);
    }
    Ok(algosMap)
  }
}
pub mod modes {
  use super::super::{models::ModeMYSQL, schema::modes::dsl::*};
  use diesel::result::Error;
  use diesel::{mysql::MysqlConnection, prelude::*};
  use std::collections::HashMap;

  pub fn get_modes_mysql(conn: &MysqlConnection) -> Result<HashMap<String, ModeMYSQL>, Error> {
    // select blocks that have not been sent to dpplns
    let modesVec = match modes.load::<ModeMYSQL>(conn) {
      Ok(m) => m,
      Err(e) => panic!("failed to get modes"),
    };
    let mut modesMap = HashMap::new();
    for mode in modesVec {
      modesMap.insert(mode.name.to_string(), mode);
    }
    Ok(modesMap)
  }
}

// }
// pub mod workers {
//   use diesel::result::Error;
//   use diesel::{mysql::MysqlConnection, prelude::*, update};

//   // use super::super::{
//   //   models::EarningInsertable,
//   //   schema::earnings::{self, dsl},
//   // };
//   use super::super::{models::Worker::dsl::*, schema::workers::dsl};
//   /// Inserts block to MySQL database.
//   pub fn update_worker_mysql(conn: &MysqlConnection, worker: Worker) -> Result<(), Error> {
//     update(dsl::workers.filter(id.eq(worker.id)))
//       .set(hashrate.eq(worker.hashrate))
//       .get_result(conn)
//   }
// }
