use diesel::{mysql::MysqlConnection, prelude::*};
// use shared::dbconn;

use shared::{helpers::getallusers}

#[get("/users")]
pub fn users(conn: &MysqlConnection) -> &'static str {
  RUSTSTRINGIFY(getallusers())
}
