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
    algo -> Smallint,
    category -> Text,
    stratum_id -> Smallint,
    mode -> Smallint,
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
    mode -> Smallint,
    stratum_id -> Smallint,
  }
}

table! {
  shares (id) {
    id -> Int8,
    user_id -> Int4,
    worker_id -> Int4,
    coin_id -> Int2,
    time -> Int8,
    difficulty -> Double,
    share_diff -> Double,
    block_diff -> Double,
    algo -> Int2,
    mode -> Int2,
    block_reward -> Double,
    party_pass -> Text,
    stratum_id -> Int2,
  }
}
