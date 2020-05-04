use super::schema::{earnings, shares};
use super::util::unix_timestamp;
use serde::{Deserialize, Serialize};
// use diesel::deserialize::Queryable;
// type MySQLDB = diesel::mysql::Mysql;

#[derive(Queryable, Serialize, Deserialize)]
pub struct Share {
  pub id: i32,
  pub user_id: i32,
  pub worker_id: i32,
  pub coin_id: i32,
  pub timestamp: i64,
  pub difficulty: f64,
  pub share_diff: f64,
  pub block_reward: f64,
  pub block_diff: f64,
  pub algo: String,
  pub mode: String,
  pub party_pass: String,
  pub stratum_id: i8,
}

impl Default for Share {
  fn default() -> Self {
    Share {
      id: 0,
      user_id: 0,
      worker_id: 0,
      coin_id: 0,
      timestamp: unix_timestamp(),
      difficulty: 0.0,
      share_diff: 0.0,
      block_reward: 0.6,
      //block_reward: 2.38095238,
      block_diff: 0.0,
      algo: "".to_string(),
      mode: "".to_string(),
      party_pass: "".to_string(),
      stratum_id: 0,
    }
  }
}

impl Share {
  /// Constructs new share data for worker.
  pub fn new(
    user_id: i32,
    worker_id: i32,
    difficulty: f64,
    share_diff: f64,
    block_diff: f64,
    algo: String,
    mode: String,
    party_pass: String,
    stratum_id: i8,
  ) -> Self {
    Share {
      user_id: user_id,
      worker_id: worker_id,
      difficulty: difficulty,
      share_diff: share_diff,
      block_diff: block_diff,
      algo: algo,
      mode: mode,
      party_pass: party_pass,
      stratum_id,
      ..Share::default()
    }
  }
}

impl ToString for Share {
  fn to_string(&self) -> String {
    format!(
      "[{},{},{},{},{},{},{},{},{},{},{},{}]",
      self.user_id,
      self.worker_id,
      self.coin_id,
      self.timestamp,
      self.algo,
      self.difficulty,
      self.share_diff,
      self.block_reward,
      self.block_diff,
      self.mode,
      self.party_pass,
      self.stratum_id,
    )
  }
}

/// Earning model for inserts.
#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "shares"]
pub struct ShareMYSQLInsertable {
  pub user_id: i32,
  pub worker_id: i32,
  pub coin_id: i32,
  pub difficulty: f64,
  pub share_diff: f64,
  pub block_diff: f64,
  pub algo: String,
  pub mode: String,
  pub block_reward: f64,
  pub party_pass: String,
  pub stratum_id: i32,
  pub timestamp: i64,
}

/// Earning model for queries.
#[derive(Queryable)]
pub struct Earning {
  pub id: i32,
  pub userid: i32,
  pub coinid: i32,
  pub blockid: i32,
  pub create_time: i32,
  pub amount: f64,
  pub status: i32,
  pub mode: String,
  pub stratum: String,
}

impl Default for Earning {
  fn default() -> Self {
    Earning {
      id: 0,
      userid: 0,
      coinid: 0,
      blockid: 0,
      create_time: 0,
      amount: 0.0,
      status: 0,
      mode: "".to_string(),
      stratum: "".to_string(),
    }
  }
}

/// Earning model for inserts.
#[derive(Insertable)]
#[table_name = "earnings"]
pub struct EarningMYSQLInsertable {
  pub userid: i32,
  pub coinid: i32,
  pub blockid: i32,
  pub amount: f64,
  pub status: i32,
  pub mode: String,
  pub stratum: String,
}

/// Block model.
#[derive(Queryable, Serialize, Deserialize)]
pub struct Block {
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

impl Default for Block {
  fn default() -> Self {
    Block {
      id: 0,
      coin_id: 0,
      height: 0,
      time: unix_timestamp() as i64,
      userid: 0,
      workerid: 0,
      confirmations: 0,
      amount: 0.0,
      difficulty: 0.0,
      difficulty_user: 0.0,
      blockhash: "".to_string(),
      algo: "".to_string(),
      category: "new".to_string(),
      stratum_id: "".to_string(),
      mode: "norm".to_string(),
      party_pass: "".to_string(),
    }
  }
}
