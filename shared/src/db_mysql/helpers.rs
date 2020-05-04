// use std::time::SystemTime;

// use diesel::mysql::MysqlConnection;

// use crate::common::types::Error;
// use super::models::Earning;
// use std::ops::{Bound, RangeBounds};

// // select * from shares where timestamp > 2hrs ago
// fn get_2hrs_of_shares_from_pg(conn: PgConnection)-> result or Vec<Share>{
//   time2hrsago = ;

//   let shares :Vec<Share> = conn.select("SELECT * FROM shares WHERE time or timestamp > timeh2hrsago")
// return shares
// }
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
