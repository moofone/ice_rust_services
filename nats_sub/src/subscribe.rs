use sentry::{capture_message, integrations::failure::capture_error, Level};

extern crate termion;

use termion::{clear, color, style};

use rmp_serde::*;
use shared::nats::{establish_nats_connection, models::KDABlockNats, models::ShareNats};

fn parse_share(msg: &Vec<u8>) -> Result<ShareNats, rmp_serde::decode::Error> {
  // Some JSON input data as a &str. Maybe this comes from the user.
  // Parse the string of data into serde_json::Value.
  // println!("msg: {:?}", msg);
  let s: ShareNats = match rmp_serde::from_read_ref(&msg) {
    Ok(s) => s,
    Err(err) => return Err(err),
  };
  // println!("Share: {:?}", &s);
  //let share = sharenats_to_sharepginsertable(s);
  Ok(s)
}

fn parse_kdablock(msg: &Vec<u8>) -> Result<KDABlockNats, rmp_serde::decode::Error> {
  // Some JSON input data as a &str. Maybe this comes from the user.
  // Parse the string of data into serde_json::Value.
  // println!("msg: {:?}", msg);
  let s: KDABlockNats = match rmp_serde::from_read_ref(&msg) {
    Ok(s) => s,
    Err(err) => return Err(err),
  };
  // println!("Share: {:?}", &s);
  //let share = sharenats_to_sharepginsertable(s);
  Ok(s)
}

#[tokio::main]
async fn main() {
  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      panic!("Nats did not connect: {}", e);
    }
  };

  // let sub = nc.subscribe("kdablocks").unwrap();
  let sub = nc.subscribe(">").unwrap();
  //let kdablocks_sub = nc.subscribe("kdablocks").unwrap();
  // loop {
  //   if let Some(msg) = sub.next() {}
  // }
  for msg in sub.messages() {
    let subject = &msg.subject;
    //println!("subject: {}", subject);
    // if subject.starts_with("shares.2423") {
    //   let share = match parse_share(&msg.data) {
    //     Ok(val) => val,
    //     Err(err) => {
    //       println!("Error parsing share: {}", err);
    //       continue;
    //     }
    //   };
    //   println!(
    //     "{}Share:{}{}",
    //     color::Fg(color::Rgb(255, 0, 255)),
    //     color::Fg(color::Reset),
    //     share
    //   );
    // } else {
    //   //if subject.starts_with("kdablocks") {
    //   let kdablock = match parse_kdablock(&msg.data) {
    //     Ok(val) => val,
    //     Err(err) => {
    //       println!("Error parsing kdablock: {}", err);
    //       continue;
    //     }
    //   };
    //   println!(
    //     "{}{:?}{}",
    //     color::Fg(color::Rgb(255, 50, 255)),
    //     kdablock,
    //     color::Fg(color::Reset)
    //   );
  }

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

  println!("done!")
  //  for msg in kdablocks_sub.messages() {
  //   let kdablock = match parse_kdablock(&msg.data) {
  //     Ok(val) => val,
  //     Err(err) => {
  //       println!("Error parsing kdablock: {}", err);
  //       continue;
  //     }
  //   };
  //   println!("KDABLock {:?}", kdablock);
  // }
}
