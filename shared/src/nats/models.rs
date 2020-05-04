use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct ShareNats {
  pub user_id: i32,
  pub worker_id: i32,
  pub coin_id: i32,
  pub timestamp: i64,
  pub difficulty: f64,
  pub share_diff: f64,
  pub block_reward: f64,
  pub block_diff: f64,
  pub algo: i8,
  pub mode: i8,
  pub party_pass: String,
  pub stratum_id: i8,
}

/// Block model.
#[derive(Queryable, Serialize, Deserialize)]
pub struct BlockNats {
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
  pub algo: i8,
  pub category: String,
  pub stratum_id: String,
  pub mode: i8,
  pub party_pass: String,
}
