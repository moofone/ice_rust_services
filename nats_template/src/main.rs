use futures::future;
use futures::join;
mod server;
mod server2;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task;
use tokio::time;
extern crate shared;
use shared::nats::establish_nats_connection;
use shared::nats::NatsConnection;

// type ArcVecDeque = Arc<Mutex<VecDeque<i32>>>;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      // crash and sentry BIG
      panic!("Nats did not connect: {}", e);
    }
  };

  let sub = nc
    .subscribe("dev.stratum.auth.2408")?
    .with_handler(move |msg| {
      println!("received {}", &msg);
      Ok(())
    });
  // Ok(())
  // let items: server::ArcVecDeque = Arc::new(Mutex::new(VecDeque::new()));

  // let arr = vec![
  //   1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 1, 2, 3, 4,
  //   5, 6, 7, 8, 9, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9,
  // ];
  // for i in arr {
  //   tokio::spawn(async move {
  //     println!("i - {}", i);
  //   });
  //   println!("done with i - {}", i);
  // }
  // println!("done with loop");
  // let s2 = server2::run_server(items.clone());
  // let s1 = server::run_server(items.clone());
  // join!(s1, s2);
}

pub fn add(a: i32, b: i32) -> i32 {
  a + b
}

// async fn run_server(items: ArcVecDeque) {
//   let items3 = items.clone();
//   let c = loop_3(items3.clone());
//   let items1 = items.clone();
//   let a = loop_1(items1);
//   let items2 = items.clone();
//   let b = loop_2(items2);

//   join!(a, b, c);
// }
// fn loop_1(items: ArcVecDeque) -> task::JoinHandle<()> {
//   // spawn a thread for first loop
//   tokio::spawn(async move {
//     let mut interval = time::interval(Duration::from_millis(1000));

//     loop {
//       interval.tick().await;

//       // lock items
//       let mut items = items.lock().unwrap();
//       // do somethign to items
//       items.push_back(5);
//       println!("Inside loop 1. pushed: 5");
//       // std::process::exit(1);
//     }
//   })
// }
// fn loop_2(items: ArcVecDeque) -> tokio::task::JoinHandle<()> {
//   // spawn a new thead for second loop
//   tokio::spawn(async move {
//     let mut interval = time::interval(Duration::from_millis(2000));

//     // loop waiting for an interval
//     loop {
//       interval.tick().await;

//       let items = items.clone();
//       // spawn a 3rd thread to run something longer
//       tokio::spawn(async move {
//         let mut interval2 = time::interval(Duration::from_millis(4000));
//         interval2.tick().await;
//         interval2.tick().await;

//         let mut items = items.lock().unwrap();
//         println!(
//           "inside loop 2. Popped after 4s: {}",
//           items.pop_front().unwrap()
//         );
//       });
//     }
//   })
// }

// fn loop_3(items: ArcVecDeque) -> task::JoinHandle<()> {
//   tokio::spawn(async move {
//     let mut interval = time::interval(Duration::from_millis(1000));

//     loop {
//       interval.tick().await;

//       println!("inside loop 3. items: {:?}", items);
//     }
//   })
// }

// fn threaded_calc_1(items: ArcVecDeque) {
//   tokio::spawn(async move {
//     let mut interval = time::interval(Duration::from_millis(5000));

//     // interval.tick().await;
//     interval.tick().await;

//     println!("threaded calc 1. items: {:?}", items);
//   });
// }

#[cfg(test)]
mod tests {

  #[test]
  fn test0() {
    assert_eq!(2 + 2, 4);
  }

  #[test]
  fn test1() {
    assert_eq!(3 + 1, 4);
  }
}
