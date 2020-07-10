table! {
  blocks (id) {
    id -> Integer,
    coin_id -> Integer,
    height -> Integer,
    time -> BigInt,
    userid -> Integer,
    workerid -> Integer,
    confirmations -> Integer,
    amount -> Double,
    difficulty -> Double,
    difficulty_user -> Double,
    blockhash -> Text,
    algo -> Text,
    category -> Text,
    stratum_id -> Text,
    mode -> Text,
    party_pass -> Text,
    state -> Integer,
  }
}
table! {
  kdablocks (id) {
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
    stratum_id -> Text,
    mode -> Text,
    party_pass -> Text,
    chainid -> SmallInt,
    node_id -> Text,
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
    mode -> Nullable<Text>,
    algo -> Nullable<SmallInt>,
    stratum_id -> Nullable<SmallInt>,
    party_pass -> Nullable<Text>,
  }
}

table! {
  accounts (id){
    id -> Integer,
    coinid -> Integer,
    balance -> Nullable<Double>,

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

table! {
  coins(id){
    id-> Int4,
    symbol -> Text,
    enable -> Int4,
  }
}

table! {
  algorithms(id){
    id -> Int4,
    name -> Text,
    multiplier -> Int4,
  }
}

table! {
  modes(id){
    id-> Int4,
    name -> Text,
  }
}

table! {
  users (id) {
    id -> Int4,
    display_name -> Text,
    email -> Text,
    password -> Text,
  }
}
