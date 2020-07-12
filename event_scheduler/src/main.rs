extern crate shared;

// use sentry::{capture_message, integrations::failure::capture_error, Level};
use shared::nats::establish_nats_connection;
use tokio::time::{interval_at, Duration, Instant};

// EVENT TIMERS
const DPPLNS_RUN_INTERVAL: u64 = 10;
const RTT_TIMER: u64 = 30;

#[tokio::main]
async fn main() {
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
    let nc = nc.clone();
    tasks.push(tokio::spawn(async move {
      let mut interval = interval_at(
        Instant::now() + Duration::from_secs(DPPLNS_RUN_INTERVAL),
        Duration::from_secs(DPPLNS_RUN_INTERVAL),
      );

      loop {
        interval.tick().await;

        println!("Firing off dpplns");
        let msgpack_data = rmp_serde::to_vec("eventnow").unwrap();
        match nc.publish("events.dpplns", msgpack_data) {
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
