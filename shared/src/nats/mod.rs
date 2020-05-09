extern crate dotenv;
pub mod models;
use dotenv::dotenv;
use nats; // natsio
          // extern crate nats; // rust-nats
          // use nats::*; // rust-nats
          // use rants::Client; // rants
use std::env;

pub type NatsConnection = nats::Connection; // natsio

pub fn establish_nats_connection() -> NatsConnection {
  // natsio
  // natsio
  // pub fn establish_nats_connection() -> Client { // rust-nats
  // natsio
  dotenv().ok();

  let nats_url = env::var("NATS_URL").expect("NATS_URL must be set"); // natio
                                                                      //let nats_url_full = env::var("NATS_URL_FULL").expect("NATS_URL must be set"); // natio
                                                                      // let nats_url_mtl = env::var("NATS_URL_FULL_MTL").expect("NATS_URL must be set"); // natio
                                                                      // let nats_url_coins2 = env::var("NATS_URL_FULL_COINS2").expect("NATS_URL must be set"); // natio

  let nats_user = env::var("NATS_USER").expect("NATS_USER must be set");
  let nats_pass = env::var("NATS_PASSWORD").expect("NATS_PASSWORD must be set");

  //rust-nats
  // let cluster = vec![nats_url, nats_url_mtl, nats_url_coins2];
  // nats::Client::new(cluster).unwrap()
  // natsio

  println!(
    "url: {}, user: {}, pass: {}",
    nats_url, nats_user, nats_pass
  );
  let con = nats::ConnectionOptions::new().with_user_pass(&nats_user, &nats_pass);

  println!("{:?}", con);

  con.connect(&nats_url).expect("Nats connection failed")
  // rants
  // let client = Client::new(vec![&nats_url_full])
  //nats::connect(&nats_url).expect("Nats Connection failed")
}
