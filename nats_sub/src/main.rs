// use nats; // natsio
use futures::stream::StreamExt;
use rants::Client;
use shared::nats::models::ShareNats;
#[tokio::main]
async fn main() {
  let address = "icenats:icenats@watlab.icemining.ca:4222".parse().unwrap();
  let address1 = "icenats:icenats@51.161.15.26:4222".parse().unwrap();
  let address2 = "icenats:icenats@207.148.27.129:4222".parse().unwrap();
  let client = Client::new(vec![address, address1, address2]);

  client.connect().await;
  println!("HI");
  let subject = ">".parse().unwrap();

  let (_, mut subscription) = client.subscribe(&subject, 1024).await.unwrap();

  loop {
    let message = subscription.next().await.unwrap();
    // println!("{:?}", message.into_payload());
    // let share: ShareNats = serde_json::from_slice(&message.into_payload()).unwrap();
    let message = String::from_utf8(message.into_payload()).unwrap();
    println!("{}", message);
  }
  println!("test");
  // let nc = con
  //   .connect("watlab.icemining.ca:4222,51.161.15.26:4222,207.148.27.129:4222")
  //   .expect("Nats connection failed");
  // println!("{:?}", nc);

  // nc.subscribe("bar")?.with_handler(move |msg| {
  //   println!("Received {}", &msg);
  //   Ok(())
  // });
  // let sub = match nc.subscribe(">") {
  //   Ok(sub) => sub,
  //   Err(err) => panic!("fail"),
  // };

  // for msg in sub.messages() {
  //   println!("{}", msg)
  // }
}
