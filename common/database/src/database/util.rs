use std::time::SystemTime;

/// Creates unix timestamp.
pub fn unix_timestamp() -> u64 {
  SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs()
}
