use nats; // natsio

fn main() {
  let con = nats::ConnectionOptions::new()
    .with_user_pass("icenats", "icenats")
    .no_echo();

  println!("{:?}", con);

  let nc = con
    .connect("192.168.2.10:4222,51.161.15.26,207.148.27.129")
    .expect("Nats connection failed");
  println!("{:?}", nc);
  let sub = nc.subscribe("*").unwrap();
  for msg in sub.messages() {
    println!("{}", msg)
  }
}
