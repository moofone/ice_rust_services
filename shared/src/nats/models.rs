use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ShareNats {
  pub user_id: i32,
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

/// Block model.
#[derive(Serialize, Deserialize)]
pub struct BlockNats {
  pub id: i32,
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
  pub algo: i16,
  pub category: String,
  pub stratum_id: i16,
  pub mode: i16,
  pub party_pass: String,
}
