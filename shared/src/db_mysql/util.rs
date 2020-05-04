use std::time::SystemTime;

/// Creates unix timestamp.
pub fn unix_timestamp() -> i64 {
  SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64
}
