table! {
  blocks (id) {
    id -> Integer,
    coin_id -> Integer,
    height -> Integer,
    time -> Integer,
    userid -> Integer,
    workerid -> Integer,
    confirmations -> Integer,
    amount -> Double,
    difficulty -> Double,
    difficulty_user -> Double,
    blockhash -> Text,
    algo -> Text,
    category -> Text,
    stratum_id -> SmallInt,
    mode -> Text,
    party_pass -> Text,
  }
}

table! {
  earnings (id) {
    id -> Integer,
    userid -> Integer,
    coinid -> Integer,
    blockid -> Integer,
    create_time -> Integer,
    amount -> Double,
    status -> Integer,
    mode -> Text,
    algo -> SmallInt,
    stratum_id -> SmallInt,
    party_pass -> Text,
  }
}

table! {
  shares (id) {
    id -> Integer,
    user_id -> Integer,
    worker_id -> Integer,
    coin_id -> Integer,
    timestamp -> BigInt,
    algo -> Text,
    difficulty -> Double,
    share_diff -> Double,
    block_reward -> Double,
    block_diff -> Double,
    mode -> Text,
    party_pass -> Text,
    stratum_id -> SmallInt,
  }
}

table! {
  workers(id){
    id -> Int4,
    coin_id -> Int2,
    user_id -> Int4,
    worker_id -> Int4,
    worker_name -> Text,
    hashrate -> Double,
  }
}
