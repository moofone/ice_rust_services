pub mod accounts {
  use super::super::{models::AccountMYSQL, schema::accounts::dsl::*};
  use diesel::result::Error;
  use diesel::{insert_into, mysql::MysqlConnection, prelude::*, update};

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

  // get account by username
  pub fn get_account_by_username_mysql(
    conn: &MysqlConnection,
    _username: &String,
  ) -> Result<AccountMYSQL, Error> {
    accounts
      .filter(username.like(_username))
      .first::<AccountMYSQL>(conn)
  }

  // insert account by username
  pub fn insert_account_mysql(
    conn: &MysqlConnection,
    _username: &String,
    _coin_id: i32,
  ) -> Result<AccountMYSQL, Error> {
    insert_into(accounts)
      .values((username.eq(&_username), coinid.eq(_coin_id)))
      .execute(conn)?;

    accounts
      .filter(username.eq(&_username))
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
      .execute(conn)
      .map(|_| ())
  }
}

pub mod earnings {
  use super::super::{
    models::{EarningMYSQL, EarningMYSQLInsertable},
    schema::earnings::dsl::*,
  };
  use diesel::result::Error;
  use diesel::{delete, insert_into, mysql::MysqlConnection, prelude::*, update};
  // use std::time::{SystemTime, UNIX_EPOCH};

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
      .execute(conn)
      .map(|_| ())
  }

  // delete bad earnings
  pub fn delete_earning_mysql(conn: &MysqlConnection, earning: &EarningMYSQL) -> Result<(), Error> {
    delete(earnings.filter(id.eq(earning.id)))
      .execute(conn)
      .map(|_| ())
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
  use super::super::{
    models::{BlockMYSQL, BlockMYSQLInsertable},
    schema::blocks::dsl,
    schema::blocks::dsl::*,
  };
  use diesel::result::Error;
  use diesel::{insert_into, mysql::MysqlConnection, prelude::*};
  use std::time::{SystemTime, UNIX_EPOCH};

  /// Inserts block to MySQL database.
  pub fn insert_blocks_mysql(
    conn: &MysqlConnection,
    blocks_vec: Vec<BlockMYSQLInsertable>,
  ) -> Result<(), Error> {
    insert_into(dsl::blocks)
      .values(&blocks_vec)
      .execute(conn)
      .map(|_| ())
  }

  pub fn get_blocks_unprocessed_mysql(conn: &MysqlConnection) -> Result<Vec<BlockMYSQL>, Error> {
    // select blocks that have not been sent to dpplns
    let time_start = (SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs()
      - 3600) as i64;
    blocks
      .filter(state.eq(0))
      .filter(time.ge(time_start))
      .load::<BlockMYSQL>(conn)
  }

  pub fn update_block_to_processed_mysql(
    conn: &MysqlConnection,
    block: &BlockMYSQL,
  ) -> Result<(), Error> {
    diesel::update(blocks.filter(id.eq(block.id)))
      .set(state.eq(1))
      .execute(conn)
      .map(|_| ())
  }
}
pub mod coins {
  use super::super::{models::Coin, schema::coins::dsl::*};
  // use diesel::result::Error;
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
    let algos_vec = match algorithms.load::<AlgorithmMYSQL>(conn) {
      Ok(a) => a,
      Err(e) => panic!("failed to get algos: {}", e),
    };
    let mut algos_map = HashMap::new();
    for algo in algos_vec {
      algos_map.insert(algo.name.to_string(), algo);
    }
    Ok(algos_map)
  }
}

pub mod modes {
  use super::super::{models::ModeMYSQL, schema::modes::dsl::*};
  use diesel::result::Error;
  use diesel::{mysql::MysqlConnection, prelude::*};
  use std::collections::HashMap;

  pub fn get_modes_mysql(conn: &MysqlConnection) -> Result<HashMap<String, ModeMYSQL>, Error> {
    // select blocks that have not been sent to dpplns
    let modes_vec = match modes.load::<ModeMYSQL>(conn) {
      Ok(m) => m,
      Err(e) => panic!("failed to get modes : {}", e),
    };
    let mut modes_map = HashMap::new();
    for mode in modes_vec {
      modes_map.insert(mode.name.to_string(), mode);
    }
    Ok(modes_map)
  }
}

// }
pub mod workers {
  use super::super::{
    models::{WorkerMYSQL, WorkerMYSQLInsertable},
    schema::workers::dsl::*,
  };
  use diesel::result::Error;
  use diesel::{insert_into, mysql::MysqlConnection, prelude::*, update};

  // get worker by worker name and owner
  pub fn get_disconnected_worker_by_worker_name_mysql(
    conn: &MysqlConnection,
    _owner_id: i32,
    _owner_type: &String,
    _worker: &String,
  ) -> Result<WorkerMYSQL, Error> {
    workers
      .filter(owner_id.eq(_owner_id))
      .filter(owner_type.eq(_owner_type))
      .filter(worker.eq(_worker))
      .filter(state.eq("disconnected").or(state.eq("devfee")))
      .first::<WorkerMYSQL>(conn)
    // println!(
    //   "query: {}",
    //   diesel::debug_query::<diesel::mysql::Mysql, _>(&query)
    // );
    // query.first::<WorkerMYSQL>(conn)
  }

  // get worker by uuid
  pub fn get_worker_by_uuid_mysql(
    conn: &MysqlConnection,
    _uuid: &String,
  ) -> Result<WorkerMYSQL, Error> {
    workers.filter(uuid.eq(_uuid)).first::<WorkerMYSQL>(conn)
  }

  // insert_worker_mysql(conn, WorkerMYSQLInsertable)
  // inserts a new worker row with state connected
  // insert account by username
  pub fn insert_worker_mysql(
    conn: &MysqlConnection,
    _worker: WorkerMYSQLInsertable,
  ) -> Result<WorkerMYSQL, Error> {
    insert_into(workers).values(&_worker).execute(conn)?;

    workers
      .filter(uuid.eq(_worker.uuid))
      .first::<WorkerMYSQL>(conn)
  }

  // update_worker_by_id_mysql(conn, WorkerMYSQL, state)
  // update state using id

  // update_worker_by_uuid_mysql(conn, uuid, state)
  // update state using uuid
  pub fn update_worker_by_uuid_mysql(
    conn: &MysqlConnection,
    _uuid: &String,
    _state: &String,
  ) -> Result<(), Error> {
    update(workers.filter(uuid.eq(_uuid)))
      .set((state.eq(_state)))
      .execute(conn)
      .map(|_| ())
  }

  // update worker by full id
  pub fn update_worker_mysql(conn: &MysqlConnection, _worker: &WorkerMYSQL) -> Result<(), Error> {
    update(workers.filter(id.eq(_worker.id)))
      .set((
        state.eq(&_worker.state),
        uuid.eq(&_worker.uuid),
        difficulty.eq(&_worker.difficulty),
      ))
      .execute(conn)
      .map(|_| ())
  }

  pub fn update_worker_hashrate(
    conn: &MysqlConnection,
    _id: i32,
    _hashrate: f64,
  ) -> Result<(), Error> {
    update(workers.filter(id.eq(_id)))
      .set(hashrate.eq(_hashrate))
      .execute(conn)
      .map(|_| ())
  }

  // set all workers of stratum_id to disconnected
  pub fn update_workers_on_stratum_connect_mysql(
    conn: &MysqlConnection,
    _stratum_id: &String,
    _coin_id: i16,
  ) -> Result<(), Error> {
    update(
      workers
        .filter(stratum_id.eq(_stratum_id))
        .filter(coinid.eq(_coin_id)),
    )
    .set(state.eq("disconnected"))
    .execute(conn)
    .map(|_| ())
  }
}

pub mod stratums {
  use super::super::{models::StratumMYSQLInsertable, schema::stratums::dsl::*};
  use diesel::result::Error;
  use diesel::{insert_into, mysql::MysqlConnection, prelude::*, update};

  // insert_stratum_mysql
  pub fn insert_stratum_mysql(
    conn: &MysqlConnection,
    _stratum: &StratumMYSQLInsertable,
  ) -> Result<(), Error> {
    insert_into(stratums)
      .values(_stratum)
      .execute(conn)
      .map(|_| ())
  }
}

// todo !!
// pub mod users {

//   #![allow(proc_macro_derive_resolution_fallback)]
//   use super::super::schema::users;

//   use super::super::{models::UserMYSQL, schema::users::dsl::*};
//   use diesel::result::Error;
//   use diesel::{mysql::MysqlConnection, prelude::*};
//   // use std::collections::HashMap;

//   //odes.load::<ModeMYSQL>(conn) {
//   //   Ok(m) => m,
//   //   Err(e) => panic!("failed to get modes"),
//   // };
//   // /
//   pub fn get_users_mysql(conn: &MysqlConnection) -> Result<Vec<UserMYSQL>, Error> {
//     users.load::<UserMYSQL>(conn)
//   }

//   // get

//   // insert

//   // update

//   // delete

//   //     .map(|users| Json(users))
//   //     .map_err(|error| error_status(error))

//   // pub fn all(connection: &MysqlConnection) -> QueryResult<Vec<User>> {
//   //   users::table.load::<User>(&*connection)
//   // }

//   // fn error_status(error: Error) -> Status {
//   //   match error {
//   //       Error::NotFound => Status::NotFound,
//   //       _ => Status::InternalServerError
//   //   }
// }
