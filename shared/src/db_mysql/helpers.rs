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
