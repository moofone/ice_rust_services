extern crate shared;
use shared::db_mysql::{establish_mysql_connection, helpers::coins::get_coins_mysql, models::Coin};
fn main() {
  let mysql_pool = establish_mysql_connection();
  let mut conn = mysql_pool.get().unwrap();
  let coins: Vec<Coin> = get_coins_mysql(&conn);
  println!("coins: {:?}", coins);
  println!("coin: {}", coins[0].id)
}
