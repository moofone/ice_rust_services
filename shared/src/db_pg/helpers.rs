pub mod shares {

  #[cfg(test)]
  use diesel::debug_query;
  use diesel::dsl::sql;
  use diesel::insert_into;
  use diesel::pg::PgConnection;
  use diesel::prelude::*;
  use diesel::result::Error;
  use diesel::*;

  // use super::super::{
  //   models::EarningInsertable,
  //   schema::earnings::{self, dsl},
  // };
  // use super::super::{models::SharePGInsertable, schema::shares::dsl};
  use super::super::schema::shares::dsl::*;

  use super::super::models::{SharePGInsertable, SharePg};
  /// Inserts block to PG database.
  pub fn insert_shares_pg(
    conn: &PgConnection,
    sharesVec: Vec<SharePGInsertable>,
  ) -> Result<(), Error> {
    insert_into(shares)
      .values(&sharesVec)
      .execute(conn)
      .map(|_| ())
  }

  pub fn select_shares_count_pg(conn: &PgConnection) -> i64 {
    let select_count = shares.select(sql::<sql_types::BigInt>("COUNT(*)"));
    let get_count = || select_count.clone().first::<i64>(conn);
    return get_count().unwrap();
  }

  pub fn select_shares_newer_pg(
    conn: &PgConnection,
    time_greater_than: i64,
    time_less_than: i64,
  ) -> Vec<SharePg> {
    let res = shares
      .filter(time.gt(time_greater_than))
      .filter(time.lt(time_less_than))
      // .limit(5)
      .load::<SharePg>(conn)
      .expect("ffailed loading shares");
    return res;
  }
}
