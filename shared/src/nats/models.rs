use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

//toodo mo to utils
extern crate chrono;
extern crate chrono_humanize;

// use chrono::{Duration, Local};
// use chrono_humanize::HumanTime;

// use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareNats {
  pub user_id: i32, //owner_id
  pub worker_id: i32,
  pub coin_id: i16,
  pub timestamp: i64,
  pub difficulty: f64,
  pub share_diff: f64,
  pub block_diff: f64,
  pub algo: i16,
  pub mode: i16,
  pub block_reward: f64,
  pub party_pass: String,
  pub stratum_id: i16,
}

fn pretty_print(i: f64) -> String {
  //println!("i is {}", i);
  if i < 1_000.0 {
    let s = format!("{:.2}", i as f64);
    s
  } else if i >= 1_000.0 && i < 1_000_000.0 {
    let s = format!("{:.2}", i as f64 / 1_000.0);
    //let s = format!("{:.*}{}", i as f64 / 1_000.0, "k", precision);
    s
  } else if i >= 1_000_000.0 && i < 1_000_000_000.0 {
    let s = format!("{:.2}{}", i as f64 / 1_000_000.0, "M");
    s
  } else if i >= 1_000_000_000.0 && i < 1_000_000_000_000.0 {
    let s = format!("{:.2}{}", i as f64 / 1_000_000_000.0, "G");
    s
  } else if i >= 1_000_000_000_000.0 && i < 1_000_000_000_000_000.0 {
    let s = format!("{:.2}{}", i as f64 / 1_000_000_000_000.0, "T");
    s
  } else if i >= 1_000_000_000_000_000.0 && i < 1_000_000_000_000_000_000.0 {
    let s = format!("{:.2}{}", i as f64 / 1_000_000_000_000_000.0, "P");
    s
  } else {
    "invalid".to_string()
  }
}

// // todo move this to appropriate utils in shared
// fn timestamp_to_string(ts: i32) -> String {
//   // let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
//   let dt = Local::now() + Duration::seconds(3500);
//   let ht = HumanTime::from(dt);
//   let english = format!("{}", ht);
//   english
//   //assert_eq!("in a month", english);
// }

impl Display for ShareNats {
  fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
    let difficulty = pretty_print(self.difficulty as f64);
    let share_diff = pretty_print(self.share_diff as f64);
    let block_diff = pretty_print(self.block_diff as f64);
    let block_reward = pretty_print(self.block_reward as f64);

    if self.mode == 2 {
      write!(
        fmt,
        "{},{},{},{},d:{},sd:{},bd:{},algo:{},mode:{},rwd:{},party:{},id:{}",
        self.user_id,
        self.worker_id,
        self.coin_id,
        self.timestamp,
        difficulty,
        share_diff,
        self.block_diff,
        self.algo,
        self.party_pass,
        block_reward,
        self.party_pass,
        self.stratum_id
      )
    } else {
      write!(
        fmt,
        "{},{},{},{},diff:{},sdiff:{},bdiff:{},algo:{},reward:{},id:{}",
        self.user_id,
        self.worker_id,
        self.coin_id,
        self.timestamp,
        difficulty,
        share_diff,
        block_diff,
        self.algo,
        block_reward,
        self.stratum_id
      )
    }
  }
}

/// DPPLNS Block model.
#[derive(Debug, Serialize, Deserialize)]
pub struct DPPLNSBlockNats {
  pub id: i32,
  pub coin_id: i32,
  pub height: i32,
  pub time: i64,
  pub userid: i32,
  pub workerid: i32,
  pub confirmations: i32,
  pub amount: f64,
  pub difficulty: f64,
  pub difficulty_user: f64,
  pub blockhash: String,
  pub algo: String,
  pub category: String,
  pub stratum_id: String,
  pub mode: String,
  pub party_pass: String,
}

/// Block model.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockNats {
  pub coin_id: i16,
  pub height: i32,
  pub time: i64,
  pub userid: i32,
  pub workerid: i32,
  pub confirmations: i32,
  pub amount: f64,
  pub difficulty: f64,
  pub difficulty_user: f64,
  pub blockhash: String,
  pub algo: String,
  pub category: String,
  pub stratum_id: String,
  pub mode: String,
  pub party_pass: String,
  pub duration: i32,
  pub shares: i64,
}

/// Block model.
#[derive(Debug, Serialize, Deserialize)]
pub struct KDABlockNats {
  pub coin_id: i16,
  pub height: i32,
  pub time: i64,
  pub userid: i32,
  pub workerid: i32,
  pub confirmations: i32,
  pub amount: f64,
  pub difficulty: f64,
  pub difficulty_user: f64,
  pub blockhash: String,
  pub algo: String,
  pub category: String,
  pub stratum_id: String,
  pub chainid: i16,
  pub node_id: String,
  pub mode: String,
  pub party_pass: String,
  pub duration: i64,
  pub shares: i64,
}

/// Auth model.
#[derive(Debug, Serialize, Deserialize)]
pub struct StratumAuthNats {
  pub ip: String,
  pub version: String,
  pub port: i32,
  pub worker_name: String,
  pub password: String,
  pub uuid: String,
  pub stratum_id: String,
  pub coin_id: i16,
  pub mode: String,
}
// impl StratumAuthNats {
//   fn parse_worker_name(self) -> (String, String) {
//     match self.worker_name.find('.') {
//       Some(index) => (&self.worker_name[..index], &self.worker_name[index + 1..]),
//       None => (self.worker_name, "default"),
//     }
//   }
// }

// nim strauth auth model
#[derive(Debug, Serialize, Deserialize)]
pub struct StratumAuthNatsNIM {
  pub username: String,
  pub coin_id: i16,
  pub ip: String,
  pub difficulty: f64,
  pub version: String,
  pub consensus_mode: String,
  pub worker_name: String,
  pub uuid: String,
  pub algo: String,
  pub time: i32,
  pub stratum_id: String,
  pub mode: String,
  pub password: String,
  pub party_pass: String,
  pub pid: i32,
}

// stratum start
#[derive(Debug, Serialize, Deserialize)]
pub struct StratumStartNats {
  pub pid: i32,
  pub time: i32,
  pub started: i32,
  pub algo: String,
  pub workers: i32,
  pub port: i16,
  pub symbol: String,
  pub stratum_id: String,
}

// stratum difficulty update
#[derive(Debug, Serialize, Deserialize)]
pub struct StratumDifficultyNats {
  pub username: String,
  pub uuid: String,
  pub difficulty: f64,
  pub compact_target: u64,
}

// stratum model
#[derive(Debug, Serialize, Deserialize)]
pub struct StratumDisconnectNats {
  pub username: String,
  pub uuid: String,
}

// stratum model
#[derive(Debug, Serialize, Deserialize)]
pub struct StratumDevfeeNats {
  pub uuid: String,
}

// stratum model
#[derive(Debug, Serialize)]
pub struct StratumAuthResponseNats {
  pub owner_id: i32,
  pub worker_id: i32,
  pub uuid: String,
}

// /// Auth model.
// #[derive(Debug, Serialize, Deserialize)]
// pub struct StratumAuthNats {
//   pub ip: String,
//   pub version: String,
//   pub port: i32,
//   pub worker_name: String,
//   pub password: String,
//   pub uuid: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Stratum
// subscribe model.
// TODO
// #[derive(Debug, Serialize, Deserialize)]
// pub struct StratumSubcribeNats {
//   pub ip: String,
//   pub version: String,
//   pub port: i32,
//   pub worker_name: String,
//   pub password: String,
//   pub uuid: String,
// }
