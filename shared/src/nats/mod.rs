extern crate dotenv;
pub mod models;
use dotenv::dotenv;
use nats;
use std::env;

pub type NatsConnection = nats::Connection;

pub fn establish_nats_connection() -> NatsConnection {
  dotenv().ok();

  let nats_url = env::var("NATS_URL").expect("NATS_URL must be set");
  nats::connect(&nats_url).expect("Nats Connection failed")
}
