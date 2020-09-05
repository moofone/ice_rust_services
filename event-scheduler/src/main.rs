/*
TODO
  - split up into modules based on service
    - dpplns
    - database maintenance? maybe?
    - share-processor
      - add in event to fire off workers scalars

*/

extern crate shared;

// use sentry::{capture_message, integrations::failure::capture_error, Level};
use dotenv::dotenv;
use shared::nats::establish_nats_connection;
use std::env;
use tokio::time::{interval_at, Duration, Instant};
// EVENT TIMERS
const DPPLNS_RUN_INTERVAL: u64 = 10;
const WORKER_CLEANUP_INTERVAL: u64 = 10; // s
const RTT_TIMER: u64 = 30;

#[tokio::main]
async fn main() {
  dotenv().ok();
  let env_mode = env::var("ENVIRONMENT_MODE").expect("ENVIRONMENT_MODE not set");
  println!("Running in mode: {}", &env_mode);
  // let _guard =
  //   sentry::init("https://f8ee06fb619843b1ae923d9111d855a9@sentry.watlab.icemining.ca/10");

  let mut tasks = Vec::new();
  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      panic!("Nats did not connect: {}", e);
    }
  };

  //-----------------------DPPLNS EVENT_--------------------------
  {
    // let nc = nc.clone();
    // tasks.push(tokio::spawn(async move {
    //   let mut interval = interval_at(
    //     Instant::now() + Duration::from_secs(DPPLNS_RUN_INTERVAL),
    //     Duration::from_secs(DPPLNS_RUN_INTERVAL),
    //   );

    //   loop {
    //     interval.tick().await;

    //     println!("Firing off dpplns");
    //     let msgpack_data = rmp_serde::to_vec("eventnow").unwrap();
    //     match nc.publish("dev.events.dpplns", msgpack_data) {
    //       Ok(_) => (),
    //       Err(err) => println!("err: {}", err),
    //     }
    //   }
    // }))
  }
  //-----------------------worker cleanup EVENT_--------------------------
  {
    let nc = nc.clone();
    tasks.push(tokio::spawn(async move {
      let mut interval = interval_at(
        Instant::now() + Duration::from_secs(WORKER_CLEANUP_INTERVAL),
        Duration::from_secs(WORKER_CLEANUP_INTERVAL),
      );
      let env = "dev";
      loop {
        interval.tick().await;

        println!("Firing off worker cleanup");
        let msgpack_data = rmp_serde::to_vec("eventnow").unwrap();
        let subject;
        if env == "dev" {
          subject = format!("dev.events.maintenance.workers");
        } else {
          subject = format!("events.maintenance.workers");
        }
        match nc.publish(&subject, msgpack_data) {
          Ok(_) => (),
          Err(err) => println!("err: {}", err),
        }
      }
    }))
  }

  //-----------------------ping timer_--------------------------
  {
    tasks.push(tokio::spawn(async move {
      let mut interval = interval_at(
        Instant::now() + Duration::from_secs(RTT_TIMER),
        Duration::from_secs(RTT_TIMER),
      );

      loop {
        interval.tick().await;

        println!("Rtt: {:?}", nc.rtt());
      }
    }))
  }

  for handle in tasks {
    handle.await.unwrap();
  }
}
