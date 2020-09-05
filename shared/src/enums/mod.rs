use std::fmt;

// #[derive(Debug, Copy, Clone)]
// pub enum Coins {
//   MWC = 2422,
//   VTC = 2122,
//   SECONDARY = 2,
//   CUCKOO = 3,
//   CUCKOO29 = 4,
//   EQUIHASH144 = 5,
//   BEAMHASHII = 6,
//   ARGON2D = 7,
//   RANDOMX = 8,
//   PROGPOW = 9,
//   CUCKAROO = 10,
//   CUCKATOO = 11,
//   BLAKE2S = 12,
// }

pub struct DpplnsCoinConfig {
  coin_id: i16,
  algo: String,
  normal_fee: f64,
  solo_fee: f64,
  party_fee: f64,
  window_length: u64,
}
impl DpplnsCoinConfig {
  fn new() -> DpplnsCoinConfig {
    DpplnsCoinConfig {
      coin_id: 0,
      algo: "".to_string(),
      normal_fee: 1.0,
      solo_fee: 1.0,
      party_fee: 1.0,
      window_length: 300,
    }
  }
}

// pub struct DpplnsConfigs {
//   configs: HashMap<DpplnsCoinConfig>,
// }
// impl DpplnsConfigs {
//   fn new() -> DpplnsConfigs {
//     //setup array
//     let configs = Vec::new();
//     let default = DpplnsCoinConfig::new();
//     configs.push(default);
//     // rvn
//     let rvn = DpplnsCoinConfig::new();
//     rvn.algo = "x16r".to_string();
//     rvn.coin_id = 2500;
//     configs.push(rvn);

//     //nim
//     let nim = DpplnsCoinConfig::new();
//     nim.algo = "argon2d".to_string();
//     nim.coin_id = 2500;
//     configs.push(nim);
//     //nim 2
//     let nim = DpplnsCoinConfig::new();
//     nim.algo = "argon3d".to_string();
//     nim.coin_id = 2500;
//     configs.push(nim);

//     DpplnsConfigs { configs: configs }
//   }
// }

#[derive(Debug, Copy, Clone)]
pub enum Algos {
  DEFAULT,
  PRIMARY,
  SECONDARY,
  CUCKOO,
  CUCKOO29,
  EQUIHASH144,
  BEAMHASHII,
  ARGON2D,
  RANDOMX,
  PROGPOW,
  CUCKAROO,
  CUCKATOO,
  BLAKE2S,
}
impl Algos {
  pub fn from_string(value: &str) -> Algos {
    match value {
      "" => Algos::DEFAULT,
      "primary" => Algos::PRIMARY,
      "secondary" => Algos::SECONDARY,
      _ => Algos::DEFAULT,
    }
  }
  pub fn from_i16(value: i16) -> Algos {
    match value {
      0 => Algos::DEFAULT,
      23 => Algos::PRIMARY,
      35 => Algos::SECONDARY,
      28 => Algos::BLAKE2S,
      _ => Algos::DEFAULT,
    }
  }
  pub fn get_target(algo: &Algos) -> i32 {
    match algo {
      Algos::PRIMARY => 41000,
      Algos::SECONDARY => 20000,
      Algos::CUCKOO => 2100,
      Algos::CUCKOO29 => 200,
      Algos::EQUIHASH144 => 8600000,
      Algos::BEAMHASHII => 8600000,
      Algos::ARGON2D => 65536000,
      Algos::RANDOMX => 1000,
      Algos::PROGPOW => 1000,
      Algos::CUCKAROO => 1024,
      Algos::CUCKATOO => 1024,
      Algos::BLAKE2S => 1000,
      _ => (2 as i32).pow(42),
    }
  }
}
impl fmt::Display for Algos {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShareModes {
  NORMAL = 0,
  PARTY = 1,
  SOLO = 2,
}
impl ShareModes {
  pub fn from_string(value: &str) -> ShareModes {
    match value {
      "" => ShareModes::NORMAL,
      "party" => ShareModes::PARTY,
      "solo" => ShareModes::SOLO,
      _ => panic!("Unknown Value: {}", value),
    }
  }
  pub fn from_i16(value: i16) -> ShareModes {
    match value {
      0 => ShareModes::NORMAL,
      1 => ShareModes::PARTY,
      2 => ShareModes::SOLO,
      _ => panic!("Unknown Value: {}", value),
    }
  }
}
impl fmt::Display for ShareModes {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
