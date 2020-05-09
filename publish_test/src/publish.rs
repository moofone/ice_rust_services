extern crate rand;
extern crate shared;
// use futures::stream::StreamExt;
// use rants::{Address, Client};
// use nats;
use rand::Rng;
// use serde::{Deserialize, Serialize};
// use futures::stream::StreamExt;
use serde_json;
use shared::nats::establish_nats_connection;
use shared::nats::models::{BlockNats, ShareNats};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;

static BLOCKINTERVAL: u64 = 10000;
static SHAREINTERVAL: u64 = 1000;

#[tokio::main]
async fn main() {
  let mut tasks = Vec::new();
  //setup nats
  let mut interval = time::interval(Duration::from_millis(BLOCKINTERVAL));
  let nc = establish_nats_connection();
  // loop {
  //   interval.tick().await;
  //   match nc.publish("Test", "data") {
  //     Ok(val) => println!("success"),
  //     Err(err) => println!("erro: {}", err),
  //   };
  // }
  // let address = Address::new("watlab.icemining.ca", 4222, None);
  // let address = "192.168.2.10".parse().unwrap();
  // let address = "icenats:icenats@watlab.icemining.ca".parse().unwrap();
  // // let address1 = "icenats:icenats@51.161.13.26".parse().unwrap();
  // // let address2 = "icenats:icenats@207.148.27.129:4222".parse().unwrap();
  // println!("WTF");
  // let client = Client::new(vec![address]);

  // println!("test {:?}", client.state().await);
  // // // let mut rng = rand::thread_rng();
  // // println!("about to cononect");
  // client.connect().await;
  // println!("connect");
  // nc.publish("blocks", "test").unwrap();

  //---------------------------BLOCKS------------------------------
  //blocks push
  {
    let nc = nc.clone();
    // let mut rng = rng.clone();
    let task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(BLOCKINTERVAL));
      loop {
        interval.tick().await;

        let mut rng = rand::thread_rng();

        let blocks = create_blocks();
        let channel = format!("blocks");
        // let subject = ">".parse().unwrap();
        // let channel = "blocks".parse().unwrap();
        // let (_, mut subscription) = client.subscribe(&subject, 1024).await.unwrap();

        for mut block in blocks {
          block.id = rng.gen::<i32>().abs();

          let json = serde_json::to_vec(&block).unwrap();
          // match nc.publish(&channel, &json).await {
          //   Ok(_) => println!("pub block"),
          //   Err(err) => println!("block failed"),
          // };
          match nc.publish(&channel, json) {
            Ok(val) => println!("publish block"),
            Err(err) => println!("err: {}", err),
          }
          // nc.publish(&channel, json).unwrap();
        }
      }
    });
    tasks.push(task);
  }
  //----------------------------SHARES--------------------------------
  // shares push
  {
    let nc = nc.clone();
    // let mut rng = rng.clone();

    let task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(SHAREINTERVAL));

      loop {
        interval.tick().await;

        let mut rng = rand::thread_rng();
        let shares = create_shares();

        // push a first block before randomizing the rest
        // set the channel
        let channel = format!("shares.{}", shares[0].coin_id);
        // let channel = format!("shares.{}", shares[0].coin_id).parse().unwrap();
        // json and publish
        let json = serde_json::to_vec(&shares[0]).unwrap();
        match nc.publish(&channel, &json) {
          Ok(_) => println!("pub first share"),
          Err(err) => println!("share first failed"),
        };

        for mut share in shares {
          // randomize the share
          share.user_id = rng.gen_range(1, 1000);
          // share.coin_id = rng.gen::<i32>();
          share.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
          // share.party_pass = format!("{}", rng.gen_range(10000, 20000));

          // set the channel
          let channel = format!("shares.{}", share.coin_id);
          // json and publish
          let json = serde_json::to_vec(&share).unwrap();
          match nc.publish(&channel, &json) {
            Ok(_) => println!("pub next share"),
            Err(err) => println!("share next failed"),
          };
        }
      }
    });
    tasks.push(task);
  }
  for handle in tasks {
    handle.await.unwrap();
  }
}

fn create_blocks() -> Vec<BlockNats> {
  let mut blocks: Vec<BlockNats> = Vec::new();
  blocks.push(BlockNats {
    id: 100,
    coin_id: 2422, // i32,
    height: 100,   // i32,
    time: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64, //i64,
    userid: 11111, //i32,
    workerid: 100, //i32,
    confirmations: 100, //i32,
    amount: 1000.0, //f64,
    difficulty: 100.0, //f64,
    difficulty_user: 100.0, //f64,
    blockhash: "234".to_string(), //String,
    algo: 2,       //i8,
    category: "null".to_string(), //String,
    stratum_id: "alpha".to_string(), //String,
    mode: 1,       //i8,
    party_pass: "12345".to_string(), //String,
  });
  blocks.push(BlockNats {
    id: 100,
    coin_id: 2422, // i32,
    height: 100,   // i32,
    time: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64, //i64,
    userid: 11111, //i32,
    workerid: 100, //i32,
    confirmations: 100, //i32,
    amount: 1000.0, //f64,
    difficulty: 100.0, //f64,
    difficulty_user: 100.0, //f64,
    blockhash: "234".to_string(), //String,
    algo: 2,       //i8,
    category: "null".to_string(), //String,
    stratum_id: "alpha".to_string(), //String,
    mode: 1,       //i8,
    party_pass: "12345".to_string(), //String,
  });
  blocks.push(BlockNats {
    id: 100,
    coin_id: 2422, // i32,
    height: 100,   // i32,
    time: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64, //i64,
    userid: 11111, //i32,
    workerid: 100, //i32,
    confirmations: 100, //i32,
    amount: 1000.0, //f64,
    difficulty: 100.0, //f64,
    difficulty_user: 100.0, //f64,
    blockhash: "234".to_string(), //String,
    algo: 2,       //i8,
    category: "null".to_string(), //String,
    stratum_id: "alpha".to_string(), //String,
    mode: 1,       //i8,
    party_pass: "12345".to_string(), //String,
  });
  blocks
}
fn create_shares() -> Vec<ShareNats> {
  let mut shares: Vec<ShareNats> = Vec::new();
  shares.push(ShareNats {
    user_id: 11111,
    worker_id: 1000,
    coin_id: 2422,
    timestamp: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64,
    difficulty: 2.3,
    share_diff: 0.0,
    block_diff: 5.0,
    block_reward: 10.0,
    algo: 2,
    mode: 0,
    party_pass: "12345".to_string(),
    stratum_id: 0,
  });
  // shares.push(ShareNats {
  //   user_id: 11111,
  //   worker_id: 1000,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 0.0,
  //   block_diff: 5.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 0,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 11111,
  //   worker_id: 1000,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 0.0,
  //   block_diff: 5.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 1,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 11111,
  //   worker_id: 184,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 801335.0,
  //   block_diff: 8900987.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 0,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 22222,
  //   worker_id: 184,
  //   coin_id: 2122,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 801335.0,
  //   block_diff: 8900987.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 0,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 333333,
  //   worker_id: 184,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 801335.0,
  //   block_diff: 8900987.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 0,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 444444,
  //   worker_id: 184,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 801335.0,
  //   block_diff: 8900987.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 1,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 444444,
  //   worker_id: 184,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 801335.0,
  //   block_diff: 8900987.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 1,
  //   party_pass: "54321".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 555555,
  //   worker_id: 184,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 801335.0,
  //   block_diff: 8900987.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 2,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 666666,
  //   worker_id: 184,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 801335.0,
  //   block_diff: 8900987.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 2,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });
  // shares.push(ShareNats {
  //   user_id: 777777,
  //   worker_id: 184,
  //   coin_id: 2422,
  //   timestamp: SystemTime::now()
  //     .duration_since(UNIX_EPOCH)
  //     .unwrap()
  //     .as_secs() as i64,
  //   difficulty: 2.3,
  //   share_diff: 801335.0,
  //   block_diff: 8900987.0,
  //   block_reward: 10.0,
  //   algo: 2,
  //   mode: 2,
  //   party_pass: "12345".to_string(),
  //   stratum_id: 0,
  // });

  shares
}
// let nats = require("nats");
// let block_interval = 4000;
// let share_interval = 2;
// let start = () => {
//   console.log(
//     `Publishing-- Blocks: 3 every ${block_interval / 1000}s, Shares: ${
//       (shares.length * 1000) / share_interval
//     }/s`
//   );
//   let block_id = 2100;
//   setInterval(() => {
//     stan.publish(
//       `blocks`,
//       JSON.stringify({
//         coin_id: 2422, // = parseInt(block.coin_id); //string
//         time: Math.floor(Date.now() / 1000), // = parseInt(block.time); // `${block.time}`; //parseInt(block.time);
//         difficulty: 100, //= parseFloat(block.difficulty);
//         user_id: 11111, //= parseInt(block.userid);
//         block_id: block_id, //= block.blockid;
//         algo: "secondary", //= block.algo;
//         mode: 0, //= block.mode;
//         amount: 1000, //= block.amount;
//         party_pass: "12345",
//         reward: 10,
//       }),
//       (err, guid) => {
//         if (err) {
//           console.log("error: " + err);
//         } else {
//           // console.log("published: " + guid);
//         }
//       }
//     );
//     block_id++;
//     stan.publish(
//       `blocks`,
//       JSON.stringify({
//         coin_id: 2422, // = parseInt(block.coin_id); //string
//         time: Math.floor(Date.now() / 1000), // = parseInt(block.time); // `${block.time}`; //parseInt(block.time);
//         difficulty: 100, //= parseFloat(block.difficulty);
//         user_id: 2222, //= parseInt(block.userid);
//         block_id: block_id, //= block.blockid;
//         algo: "secondary", //= block.algo;
//         mode: 0, //= block.mode;
//         amount: 1000, //= block.amount;
//         party_pass: "12345",
//         reward: 10,
//       }),
//       (err, guid) => {
//         if (err) {
//           console.log("error: " + err);
//         } else {
//           // console.log("published: " + guid);
//         }
//       }
//     );
//     block_id++;
//     stan.publish(
//       `blocks`,
//       JSON.stringify({
//         coin_id: 2422, // = parseInt(block.coin_id); //string
//         time: Math.floor(Date.now() / 1000), // = parseInt(block.time); // `${block.time}`; //parseInt(block.time);
//         difficulty: 100, //= parseFloat(block.difficulty);
//         user_id: 2222, //= parseInt(block.userid);
//         block_id: block_id, //= block.blockid;
//         algo: "secondary", //= block.algo;
//         mode: 0, //= block.mode;
//         amount: 1000, //= block.amount;
//         party_pass: "12345",
//         reward: 10,
//       }),
//       (err, guid) => {
//         if (err) {
//           console.log("error: " + err);
//         } else {
//           // console.log("published: " + guid);
//         }
//       }
//     );
//     block_id++;
//     stan.publish(
//       `blocks`,
//       JSON.stringify({
//         coin_id: 2422, // = parseInt(block.coin_id); //string
//         time: Math.floor(Date.now() / 1000), // = parseInt(block.time); // `${block.time}`; //parseInt(block.time);
//         difficulty: 100, //= parseFloat(block.difficulty);
//         user_id: 2222, //= parseInt(block.userid);
//         block_id: block_id, //= block.blockid;
//         algo: "secondary", //= block.algo;
//         mode: 0, //= block.mode;
//         amount: 1000, //= block.amount;
//         party_pass: "12345",
//         reward: 10,
//       }),
//       (err, guid) => {
//         if (err) {
//           console.log("error: " + err);
//         } else {
//           // console.log("published: " + guid);
//         }
//       }
//     );
//     block_id++;
//     stan.publish(
//       `blocks`,
//       JSON.stringify({
//         coin_id: 2422, // = parseInt(block.coin_id); //string
//         time: Math.floor(Date.now() / 1000), // = parseInt(block.time); // `${block.time}`; //parseInt(block.time);
//         difficulty: 100, //= parseFloat(block.difficulty);
//         user_id: 2222, //= parseInt(block.userid);
//         block_id: block_id, //= block.blockid;
//         algo: "secondary", //= block.algo;
//         mode: 0, //= block.mode;
//         amount: 1000, //= block.amount;
//         party_pass: "12345",
//         reward: 10,
//       }),
//       (err, guid) => {
//         if (err) {
//           console.log("error: " + err);
//         } else {
//           // console.log("published: " + guid);
//         }
//       }
//     );
//     block_id++;
//     stan.publish(
//       `blocks`,
//       JSON.stringify({
//         coin_id: 2422, // = parseInt(block.coin_id); //string
//         time: Math.floor(Date.now() / 1000), // = parseInt(block.time); // `${block.time}`; //parseInt(block.time);
//         difficulty: 100, //= parseFloat(block.difficulty);
//         user_id: 2222, //= parseInt(block.userid);
//         block_id: block_id, //= block.blockid;
//         algo: "secondary", //= block.algo;
//         mode: 0, //= block.mode;
//         amount: 1000, //= block.amount;
//         party_pass: "12345",
//         reward: 10,
//       }),
//       (err, guid) => {
//         if (err) {
//           console.log("error: " + err);
//         } else {
//           // console.log("published: " + guid);
//         }
//       }
//     );
//     block_id++;
//     stan.publish(
//       `blocks`,
//       JSON.stringify({
//         coin_id: 2422, // = parseInt(block.coin_id); //string
//         time: Math.floor(Date.now() / 1000), // = parseInt(block.time); // `${block.time}`; //parseInt(block.time);
//         difficulty: 100, //= parseFloat(block.difficulty);
//         user_id: 2222, //= parseInt(block.userid);
//         block_id: block_id, //= block.blockid;
//         algo: "secondary", //= block.algo;
//         mode: 0, //= block.mode;
//         amount: 1000, //= block.amount;
//         party_pass: "12345",
//         reward: 10,
//       }),
//       (err, guid) => {
//         if (err) {
//           console.log("error: " + err);
//         } else {
//           // console.log("published: " + guid);
//         }
//       }
//     );
//     block_id++;
//     // stan.publish(
//     //   `blocks`,
//     //   JSON.stringify({
//     //     coin_id: 2422, // = parseInt(block.coin_id); //string
//     //     time: Math.floor(Date.now() / 1000), // = parseInt(block.time); // `${block.time}`; //parseInt(block.time);
//     //     difficulty: 100, //= parseFloat(block.difficulty);
//     //     user_id: 36333, //= parseInt(block.userid);
//     //     block_id: Math.floor(Date.now() / 1000) % 100000, //= block.blockid;
//     //     algo: "primary", //= block.algo;
//     //     mode: 2, //= block.mode;
//     //     amount: 1000.0, //= block.amount;
//     //     party_pass: "12345",
//     //     reward: 10,
//     //   }),
//     //   (err, guid) => {
//     //     if (err) {
//     //       console.log("error: " + err);
//     //     } else {
//     //       // console.log("published: " + guid);
//     //     }
//     //   }
//     // );
//   }, block_interval);
//   setInterval(() => {
//     // for (let i = 0; i < 2; i++) {
//     // let coinsList = ["NIM"];
//     let coinsList = [2422];
//     stan.publish(`shares.2422`, JSON.stringify(shares[0]), (err, guid) => {
//       if (err) {
//         console.log("error: " + err);
//       } else {
//         // console.log("published: " + guid);
//       }
//     });
//     coinsList.forEach((coin) => {
//       let userid = Math.floor(Math.random() * Math.floor(1000));
//       let party_pass = `${Math.floor(Math.random() * Math.floor(10000))}`;
//       // console.log("pub");
//       shares.forEach((share) => {
//         share[0] = userid;
//         share[3] = Date.now();
//         share[11] = party_pass;
//         stan.publish(`shares.${coin}`, JSON.stringify(share), (err, guid) => {
//           if (err) {
//             console.log("error: " + err);
//           } else {
//             // console.log("published: " + guid);
//           }
//         });
//       });
//     });

//     // }
//   }, share_interval);
// };
// // let stan = nats.connect("me", "mark-pub", {
// //   url: "nats://192.168.2.10:4222",
// // });
// let stan = nats.connect({
//   url: "nats://192.168.2.10:4222",
// });

// stan.on("connect", (c) => {
//   start();
//   // stan.publish("foo", "OMFG HI", (err, guid) => {
//   //   if (err) {
//   //     console.log("error: " + err);
//   //   } else {
//   //     console.log("published: " + guid);
//   //   }
//   // });

//   // var opts = stan.subscriptionOptions().setStartWithLastReceived();
//   // var subscription = stan.subscribe("foo", opts);
//   // subscription.on("message", function (msg) {
//   //   console.log(
//   //     "Received a message [" + msg.getSequence() + "] " + msg.getData()
//   //   );
//   // });
// });

// let shares = [
//   [
//     11111, // userid
//     184, //workerid
//     2422, //coinid
//     0, //time
//     "secondary", // algo
//     1, //valid
//     4, //difficulty
//     801335, //share_diff
//     0.6, //reward
//     8900987, //blockDiff
//     0, //mode
//     "12345", //party_pass
//   ],
//   [
//     22222, // userid
//     184, //workerid
//     2122, //coinid
//     0, //time
//     "secondary", // algo
//     1, //valid
//     4, //difficulty
//     801335, //share_diff
//     0.6, //reward
//     8900987, //blockDiff
//     0, //mode
//     "12345", //party_pass
//   ],
//   [
//     333333, // userid
//     184, //workerid
//     2422, //coinid
//     0, //time
//     "secondary", // algo
//     1, //valid
//     4, //difficulty
//     801335, //share_diff
//     0.6, //reward
//     8900987, //blockDiff
//     0, //mode
//     "12345", //party_pass
//   ],
//   [
//     444444, // userid
//     184, //workerid
//     2422, //coinid
//     0, //time
//     "secondary", // algo
//     1, //valid
//     4, //difficulty
//     801335, //share_diff
//     0.6, //reward
//     8900987, //blockDiff
//     1, //mode
//     "12345", //party_pass
//   ],
//   [
//     555555, // userid
//     184, //workerid
//     2422, //coinid
//     0, //time
//     "secondary", // algo
//     1, //valid
//     4, //difficulty
//     801335, //share_diff
//     0.6, //reward
//     8900987, //blockDiff
//     2, //mode
//     "12345", //party_pass
//   ],
//   [
//     666666, // userid
//     184, //workerid
//     2422, //coinid
//     0, //time
//     "secondary", // algo
//     1, //valid
//     4, //difficulty
//     801335, //share_diff
//     0.6, //reward
//     8900987, //blockDiff
//     2, //mode
//     "12345", //party_pass
//   ],
//   [
//     777777, // userid
//     184, //workerid
//     2422, //coinid
//     0, //time
//     "secondary", // algo
//     1, //valid
//     4, //difficulty
//     801335, //share_diff
//     0.6, //reward
//     8900987, //blockDiff
//     2, //mode
//     "12345", //party_pass
//   ],
// ];
