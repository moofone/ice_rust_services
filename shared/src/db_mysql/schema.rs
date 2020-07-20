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
    duration -> Integer,
    shares -> BigInt,
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
    username -> Text,

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
    worker -> Text,
    hashrate -> Double,
    owner_id -> Int4,
    owner_type -> Text,
    uuid -> Text,
    state -> Text,
    ip_address -> Text,
    version -> Text,
    password -> Text,
    algo -> Text,
    mode -> Text,
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
