table! {
  blocks (id) {
    id -> Integer,
    coin_id -> Integer,
    height -> Integer,
    time -> BigInt,
    userid -> Nullable<Integer>,
    workerid -> Nullable<Integer>,
    confirmations -> Nullable<Integer>,
    amount -> Double,
    difficulty -> Double,
    difficulty_user -> Double,
    blockhash -> Nullable<Text>,
    algo -> Text,
    category -> Text,
    stratum_id -> Text,
    mode -> Text,
    party_pass -> Nullable<Text>,
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
    coinid -> Int2,
    userid -> Int4,
    worker -> Text,
    hashrate -> Double,
    difficulty -> Double,
    owner_id -> Int4,
    owner_type -> Text,
    uuid -> Text,
    state -> Text,
    ip -> Text,
    version -> Text,
    password -> Text,
    algo -> Text,
    mode -> Text,
    stratum_id -> Text,
    time -> Nullable<Int4>,
    pid -> Nullable<Int4>,
    name -> Nullable<Text>,
    last_share_time -> Nullable<Int4>,
    shares_per_min -> Nullable<Double>,
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

table! {
  stratums(pid){
    pid-> Int4,
    time -> Int4,
    started -> Int4,
    algo -> Text,
    workers -> Int4,
    port -> Int2,
    symbol -> Text,
    stratum_id -> Text,
  }
}
