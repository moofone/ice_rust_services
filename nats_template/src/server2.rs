use futures::join;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task;
use tokio::time;
pub type ArcVecDeque = Arc<Mutex<VecDeque<i32>>>;

pub async fn run_server(items: ArcVecDeque) {
  let items3 = items.clone();
  let c = loop_3(items3.clone());
  let items1 = items.clone();
  let a = loop_1(items1);
  let items2 = items.clone();
  let b = loop_2(items2);
  join!(a, b, c);
}
fn loop_1(items: ArcVecDeque) -> task::JoinHandle<()> {
  // spawn a thread for first loop
  tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_millis(1000));
    loop {
      interval.tick().await;
      // lock items
      let mut items = items.lock().unwrap();
      // do somethign to items
      items.push_back(5);
      println!("Server2Inside loop 1. pushed: 5");
      // std::process::exit(1);
    }
  })
}
fn loop_2(items: ArcVecDeque) -> tokio::task::JoinHandle<()> {
  // spawn a new thead for second loop
  tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_millis(2000));
    // loop waiting for an interval
    loop {
      interval.tick().await;
      let items = items.clone();
      // spawn a 3rd thread to run something longer
      tokio::spawn(async move {
        let mut interval2 = time::interval(Duration::from_millis(4000));
        interval2.tick().await;
        interval2.tick().await;
        let mut items = items.lock().unwrap();
        println!(
          "inside loop 2. Popped after 4s: {}",
          items.pop_front().unwrap()
        );
      });
    }
  })
}
fn loop_3(items: ArcVecDeque) -> task::JoinHandle<()> {
  tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_millis(1000));
    loop {
      interval.tick().await;
      println!("inside loop 3. items: {:?}", items);
    }
  })
}
fn threaded_calc_1(items: ArcVecDeque) {
  tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_millis(5000));
    // interval.tick().await;
    interval.tick().await;
    println!("threaded calc 1. items: {:?}", items);
  });
}
