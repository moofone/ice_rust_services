/**
  All rust services and stratum will get their startup configuration from this service.
  This service will interface with mysql cluster to supply this configuration.
  This way we have the configuration in a highly available fashion, and avoids dilicate data 

  TODO
  - need to load in share processor data
    - needs window time per coin-algo combo
    - needs difficulty per algo
**/
use sentry::{capture_message, integrations::failure::capture_error, Level};

extern crate termion;
use termion::{color, clear, style};

//use rmp_serde::*;
use shared::nats::{establish_nats_connection, models::ConfigurationNats};

use serde::{Deserialize, Serialize};
use serde_json::Result;

// fn parse_config(msg: &Vec<u8>) -> Result<ShareNats, rmp_serde::decode::Error> {
//   // Some JSON input data as a &str. Maybe this comes from the user.
//   // Parse the string of data into serde_json::Value.
//   // // println!("msg: {:?}", msg);
//   // let s: ShareNats = match rmp_serde::from_read_ref(&msg) {
//   //   Ok(s) => s,
//   //   Err(err) => return Err(err),
//   // };
//   // println!("Share: {:?}", &s);
//   //let share = sharenats_to_sharepginsertable(s);
//   Ok(s)
// }

// 

#[tokio::main]
async fn main() {
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      panic!("Nats did not connect: {}", e);
    }
  };


    /*
    let share = match parse_share(&msg.data) {
      Ok(val) => val,
      Err(err) => {
        println!("Error parsing share: {}", err);
        continue;
      }
    };
    */
    //println!("Share: {}", share);
  }
}