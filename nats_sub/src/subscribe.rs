use shared::nats::establish_nats_connection;

#[tokio::main]
async fn main() {
  let nc = establish_nats_connection();
  let sub = nc.queue_subscribe("blocks", "block_workers").unwrap();
  for msg in sub.messages() {
    println!("msg: {}", msg);
  }
}
