extern crate dotenv;
pub mod models;
use dotenv::dotenv;
use nats;
use std::env;

pub type NatsConnection = nats::Connection;

pub fn establish_nats_connection() -> Result<NatsConnection, std::io::Error> {
  dotenv().ok();

  let nats_url = env::var("NATS_URL").expect("NATS_URL must be set");
  let nats_user = env::var("NATS_USER").expect("NATS_USER must be set");
  let nats_pass = env::var("NATS_PASSWORD").expect("NATS_PASSWORD must be set");

  println!(
    "NATS --- url: {}, user: {}, pass: {}",
    nats_url, nats_user, nats_pass
  );
  let con = nats::Options::with_user_pass(&nats_user, &nats_pass);
  //   // .set_disconnect_callback(test)
  //   // .set_close_callback(test)
  //   .max_reconnects(Some(2));

  // // .set_reconnect_callback(&test)

  con.connect(&nats_url)
}
pub fn test() {
  println!("HI");
}
