pub mod earnings {
  use diesel::result::Error;
  use diesel::{insert_into, mysql::MysqlConnection, prelude::*};

  // use super::super::{
  //   models::EarningInsertable,
  //   schema::earnings::{self, dsl},
  // };
  use super::super::{models::EarningMYSQLInsertable, schema::earnings::dsl};
  /// Inserts block to MySQL database.
  pub fn insert_earnings_mysql(
    conn: &MysqlConnection,
    earnings: Vec<EarningMYSQLInsertable>,
  ) -> Result<(), Error> {
    insert_into(dsl::earnings)
      .values(&earnings)
      .execute(conn)
      .map(|_| ())
  }
}

pub mod shares {}

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
