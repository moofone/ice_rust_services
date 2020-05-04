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
    algo -> TinyInt,
    category -> Text,
    stratum_id -> Text,
    mode -> TinyInt,
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
    mode -> TinyInt,
    stratum -> Text,
  }
}

table! {
  shares (id) {
    id -> Integer,
    user_id -> Integer,
    worker_id -> Integer,
    coin_id -> Integer,
    timestamp -> BigInt,
    algo -> TinyInt,
    difficulty -> Double,
    share_diff -> Double,
    block_reward -> Double,
    block_diff -> Double,
    mode -> TinyInt,
    party_pass -> Text,
    stratum_id -> Integer,
  }
}
