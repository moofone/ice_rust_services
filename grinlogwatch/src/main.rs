extern crate logwatcher;
use logwatcher::{LogWatcher,LogWatcherAction};
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};
use std::process::Command;
use chrono::prelude::*;
use tokio::*;

// extern crate sentry;
// use sentry::integrations::panic::register_panic_handler;
// use sentry::integrations::failure::capture_error;

pub type ArcLogTime = Arc<Mutex<LogTime>>;

const COIN_NAME: &str = "EPIC";
const pm2_name: &str = "promtail";
const TIMEOUT_WARN_LOG : Duration = Duration::from_secs(10);
const TIMEOUT_WARN_SHARE : Duration = Duration::from_secs(30);
const TIMEOUT_RESTART_LOG : Duration = Duration::from_secs(5);
const TIMEOUT_RESTART_SHARE : Duration = Duration::from_secs(240);
const RESTART_TIME : Duration = Duration::from_secs(120);

const LOGFILE: &str = "/root/.epic/main/epic-server.log";


struct LogTime {
  share_now: SystemTime,
  log_activity_now: SystemTime,
}

impl LogTime {
   pub fn new() -> Self {
     Self {
        share_now: SystemTime::now(),
        log_activity_now: SystemTime::now(),
     }
   }
}


fn restart() {
  let output = Command::new("pm2")
      .arg("restart")
      .arg(pm2_name)
      .output()
      .expect("Failed to restart");

    if output.status.success() {
      println!("Restarted {} Successfully", COIN_NAME);
    }

    //println!("status: {}", output.status);
    //println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    //println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    //assert_eq!(b"d\n", output.stdout.as_slice());
}


#[tokio::main]
async fn main() {
  let _guard = sentry::init(("http://d1f907c2dc4745429e2ddd900a1f701d@192.168.2.118:9000/2", sentry::ClientOptions {
    max_breadcrumbs: 50,
    debug: false,
    ..Default::default()
  }));
  sentry::capture_message("Started log check", sentry::Level::Info);

  println!("Started log check {:?}", Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
  let mut tasks = Vec::new();
  let log_time = Arc::new(Mutex::new(LogTime::new()));
  
  {
    let log_time = log_time.clone();
    let check_task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(1000));
      loop {
        interval.tick().await;
        let mut log_time = log_time.lock().unwrap();
        
        match log_time.log_activity_now.elapsed() {
          Ok(elapsed) => {
            if elapsed > TIMEOUT_WARN_LOG {
              println!("Warning: no log activity in {:?}s", elapsed.as_secs());
            } 
            if elapsed > TIMEOUT_RESTART_LOG {
              println!("RESTARTING node after no log activity in {:?}s", elapsed.as_secs());
              // Give it time to restart
              log_time.share_now = SystemTime::now() + RESTART_TIME;
              log_time.log_activity_now = SystemTime::now() + RESTART_TIME;
              restart();
            } 
          }
          Err(e) => {
            // println!("error: {}", e);
          }
        }

        match log_time.share_now.elapsed() {
         Ok(elapsed) => {
            if elapsed > TIMEOUT_WARN_SHARE {
              println!("Warning: no share in {:?}s", elapsed.as_secs());
            } 
            if elapsed > TIMEOUT_RESTART_SHARE {
              println!("RESTARTING node after no shares in {:?}s", elapsed.as_secs());
              log_time.share_now = SystemTime::now()+ RESTART_TIME;
              log_time.log_activity_now = SystemTime::now() + RESTART_TIME;
              restart();
            } 
          }
          Err(e) => {
            //println!("err");
          }
        }
      }
    });
    tasks.push(check_task);
  }

  {
    let log_time = log_time.clone();
    let parse_task = tokio::spawn(async move {
        let mut log_watcher = LogWatcher::register(LOGFILE.to_string()).unwrap();
        log_watcher.watch(&mut move |line: String| {
          if line.contains("Got share") {
            //print!(".");
            let mut log_time = log_time.lock().unwrap();
            log_time.log_activity_now = SystemTime::now();
            log_time.share_now = SystemTime::now();
          }
          if line.contains("REORG") {
            println!("{} REORG Detected", COIN_NAME);
            sentry::capture_message("REORG", sentry::Level::Warning);
          }
          if line.contains(" DIFFICULTY 0 Job") {
            println!("{} Difficulty 0 Job", COIN_NAME);
            sentry::capture_message("0 Difficulty Job Received", sentry::Level::Warning);
          }
          if line.contains("Invalid Root") {
            println!("{} Invalid Root", COIN_NAME);
            sentry::capture_message("Invalid Root", sentry::Level::Warning);
          }
          if line.contains("Stratum server started") {
            println!("{} Stratum server started", COIN_NAME);
            sentry::capture_message("Stratum server started", sentry::Level::Info);
          }
          LogWatcherAction::None
        });
    });
    tasks.push(parse_task);
  }

  for handle in tasks {
    handle.await.unwrap();
  }
}