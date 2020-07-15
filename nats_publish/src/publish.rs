extern crate rand;

use rmp_serde;
extern crate shared;
use dotenv::dotenv;
// use rand::Rng;
use shared::nats::establish_nats_connection;
use shared::nats::models::{BlockNats, KDABlockNats, ShareNats};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
static BLOCKINTERVAL: u64 = 8000;
static SHAREINTERVAL: u64 = 2;

#[tokio::main]
async fn main() {
  let mut tasks = Vec::new();
  // Initilize the nats connection
  let nc = match establish_nats_connection() {
    Ok(n) => n,
    Err(e) => {
      println!("Nats did not connect: {}", e);
      panic!("Nats did not connect: {}", e);
    }
  };
  println!(
    "Pub 3 blocks every: {}s, pub 10 shares {} times/s",
    BLOCKINTERVAL as f64 / 1000.0,
    1000.0 / SHAREINTERVAL as f64
  );
  //---------------------------BLOCKS------------------------------
  //blocks push
  {
    let nc = nc.clone();
    // let mut rng = rng.clone();
    // let task = tokio::spawn(async move {
    //   let mut interval = time::interval(Duration::from_millis(BLOCKINTERVAL));
    //   loop {
    //     interval.tick().await;

    //     // tokio::spawn(async move {
    //     //   println!("hi");
    //     //   return;
    //     //   println!("bye")
    //     // });
    //     let mut rng = rand::thread_rng();

    //     let blocks = create_blocks();
    //     let channel = format!("kdablocks");

    //     let kdablock = KDABlockNats {
    //       coin_id: 69,
    //       height: 69,
    //       time: 69,
    //       userid: 69,
    //       workerid: 69,
    //       confirmations: 69,
    //       amount: 1.0,
    //       difficulty: 1.0,
    //       difficulty_user: 1.0,
    //       blockhash: "poopy".to_string(),
    //       algo: "poopy".to_string(),
    //       category: "poopy".to_string(),
    //       stratum_id: "poopy".to_string(),
    //       chainid: 69,
    //       node_id: "poopy".to_string(),
    //       mode: "poopy".to_string(),
    //       party_pass: "poopy".to_string(),
    //       duration: 69,
    //       shares: 69,
    //     };
    //     let msgpack_data = rmp_serde::to_vec(&kdablock).unwrap();
    //     match nc.publish(&channel, msgpack_data) {
    //       Ok(val) => println!("pubbed"),
    //       Err(err) => println!("err: {}", err),
    //     }
    //     // for mut block in blocks {
    //     //   // block.id = rng.gen::<i32>().abs();

    //     //   let msgpack_data = rmp_serde::to_vec(&block).unwrap();

    //     //   match nc.publish(&channel, msgpack_data) {
    //     //     Ok(val) => (),
    //     //     Err(err) => println!("err: {}", err),
    //     //   }
    //     // }
    //   }
    // });
    // tasks.push(task);
  }
  //----------------------------SHARES--------------------------------
  // shares push
  {
    let nc = nc.clone();

    let task = tokio::spawn(async move {
      let mut interval = time::interval(Duration::from_millis(SHAREINTERVAL));

      loop {
        interval.tick().await;

        let mut rng = rand::thread_rng();
        let shares = create_shares();

        // push a first block before randomizing the rest
        // set the channel
        let channel = format!("shares.{}", shares[0].coin_id);
        // json and publish
        // let json = rmp_serde::to_vec(&shares[0]).unwrap();
        // match nc.publish(&channel, &json) {
        //   Ok(_) => (),
        //   Err(err) => println!("share first failed"),
        // };

        for mut share in shares {
          // randomize the share
          // share.user_id = rng.gen_range(1, 1000);
          // share.coin_id = rng.gen::<i32>();
          share.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
          // share.party_pass = format!("{}", rng.gen_range(10000, 15000));

          // set the channel
          let channel = format!("shares.{}", share.coin_id);
          // json and publish
          let json = rmp_serde::to_vec(&share).unwrap();
          match nc.publish(&channel, &json) {
            Ok(_) => (),
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

// fn create_blocks() -> Vec<BlockNats> {
//   dotenv().ok();
//   let stratum_id = env::var("STRATUM_ID")
//     .expect("STRATUM_ID must be set")
//     .parse::<i16>()
//     .unwrap();
//   let mut blocks: Vec<BlockNats> = Vec::new();
//   blocks.push(BlockNats {
//     // id: 100,
//     coin_id: 2422, // i32,
//     height: 100,   // i32,
//     time: SystemTime::now()
//       .duration_since(UNIX_EPOCH)
//       .unwrap()
//       .as_secs() as i64, //i64,
//     userid: 11111, //i32,
//     workerid: 100, //i32,
//     confirmations: 100, //i32,
//     amount: 1000.0, //f64,
//     difficulty: 100.0, //f64,
//     difficulty_user: 100.0, //f64,
//     blockhash: "234".to_string(), //String,
//     algo: 2,       //i8,
//     category: "null".to_string(), //String,
//     stratum_id: stratum_id, //String,
//     mode: 1,       //i8,
//     party_pass: "12345".to_string(), //String,
//     duration: 10,
//     shares: 10,
//   });
//   blocks.push(BlockNats {
//     // id: 100,
//     coin_id: 2422, // i32,
//     height: 100,   // i32,
//     time: SystemTime::now()
//       .duration_since(UNIX_EPOCH)
//       .unwrap()
//       .as_secs() as i64, //i64,
//     userid: 11111, //i32,
//     workerid: 100, //i32,
//     confirmations: 100, //i32,
//     amount: 1000.0, //f64,
//     difficulty: 100.0, //f64,
//     difficulty_user: 100.0, //f64,
//     blockhash: "234".to_string(), //String,
//     algo: 2,       //i8,
//     category: "null".to_string(), //String,
//     stratum_id: stratum_id, //String,
//     mode: 1,       //i8,
//     party_pass: "12345".to_string(), //String,
//     duration: 10,
//     shares: 10,
//   });
//   blocks.push(BlockNats {
//     // id: 100,
//     coin_id: 2422, // i32,
//     height: 100,   // i32,
//     time: SystemTime::now()
//       .duration_since(UNIX_EPOCH)
//       .unwrap()
//       .as_secs() as i64, //i64,
//     userid: 11111, //i32,
//     workerid: 100, //i32,
//     confirmations: 100, //i32,
//     amount: 1000.0, //f64,
//     difficulty: 100.0, //f64,
//     difficulty_user: 100.0, //f64,
//     blockhash: "234".to_string(), //String,
//     algo: 2,       //i8,
//     category: "null".to_string(), //String,
//     stratum_id: stratum_id, //String,
//     mode: 1,       //i8,
//     party_pass: "12345".to_string(), //String,
//     duration: 10,
//     shares: 10,
//   });
//   blocks
// }
fn create_shares() -> Vec<ShareNats> {
  dotenv().ok();
  let stratum_id = env::var("STRATUM_ID")
    .expect("STRATUM_ID must be set")
    .parse::<i16>()
    .unwrap();
  let mut shares: Vec<ShareNats> = Vec::new();
  shares.push(ShareNats {
    user_id: 11111,
    worker_id: 21162647,
    coin_id: 2422,
    timestamp: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64,
    difficulty: 2.3,
    share_diff: 34.45064541,
    block_diff: 1936142.823664896,
    block_reward: 500000000000.0,
    algo: 2,
    mode: 1,
    party_pass: "12345".to_string(),
    stratum_id: stratum_id,
  });
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
    stratum_id: stratum_id,
  });
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
    mode: 1,
    party_pass: "12345".to_string(),
    stratum_id: stratum_id,
  });
  shares.push(ShareNats {
    user_id: 11111,
    worker_id: 184,
    coin_id: 2422,
    timestamp: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64,
    difficulty: 2.3,
    share_diff: 801335.0,
    block_diff: 8900987.0,
    block_reward: 10.0,
    algo: 2,
    mode: 0,
    party_pass: "12345".to_string(),
    stratum_id: stratum_id,
  });
  shares.push(ShareNats {
    user_id: 22222,
    worker_id: 184,
    coin_id: 2122,
    timestamp: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64,
    difficulty: 2.3,
    share_diff: 801335.0,
    block_diff: 8900987.0,
    block_reward: 10.0,
    algo: 2,
    mode: 0,
    party_pass: "12345".to_string(),
    stratum_id: stratum_id,
  });
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
  //   stratum_id: stratum_id,
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
  //   stratum_id: stratum_id,
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
  //   stratum_id: stratum_id,
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
  //   stratum_id: stratum_id,
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
  //   stratum_id: stratum_id,
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
  //   stratum_id: stratum_id,
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
