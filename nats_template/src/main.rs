use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;

#[tokio::main]
async fn main() {
  let mut tasks = Vec::new();

  // setup our items queueu
  let items: VecDeque<i32> = VecDeque::new();
  let items = Arc::new(Mutex::new(items));

  // loop 1
  {
    let items = items.clone();

    // spawn a thread for first loop
    tasks.push(tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(1000));

      loop {
        interval.tick().await;

        // lock items
        let mut items = items.lock().unwrap();
        // do somethign to items
        items.push_back(5);
        println!("pushed: 5");
      }
    }))
  }

  // loop 2
  {
    let items = items.clone();

    // spawn a new thead for second loop
    tasks.push(tokio::spawn(async move {
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
          println!("Popped after 4s: {}", items.pop_front().unwrap());
        });
      }
    }))
  }

  // join tasks to ensure everything keeps running
  for handle in tasks {
    handle.await.unwrap();
  }
}

pub fn add(a: i32, b: i32) -> i32 {
  a + b
}
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
