/*
  need to get worker values out of workers table

  need to rollup into workers, accounts, (subaccounts) , (users), coins

  1 worker table get, send the data to each function for rollup? then insert into each?


  2 concepts:
    - 1 module for each of the types to be rolled up
      - each type has its rollup code for both scalar and graph ( minus workers which is just graph)
      - main function can run 2 threads, 1 for scalar, 1 for graph
      - update graph function gets workers and passes aray into each of the modules
      - update scalar function does the same
    - 1 function for each of the 5 needed (update acount, update hashaccount, update coin, update hashcoin, update hashworker)
*/

// main needs to listen to event-scheduler event to fire off
// listen in a queue group
fn main(){

  // loop interval
  
  // run update
  update_graph_values();
}

fn update_graph_values(){
  // run get workers to grab an array
let workers = get_workers().await;
  // pass array into each of the rollup functions
  rollup_workers(workers);
  rollup_accounts(workers);
  rollup_coins(workers);
}

// get workers with full data to be collapsed into each rollup group
// need helpers made in shared>mysql
fn get_workers()-> Vec<WorkerMYSQL>{
  // get workers from table

  // return vec?
}

// generates a key for the dictionary 
fn gen_rollup_key(rollup_type: String)->String{
  // match rollup type, generate key for each group
  match rollup_type{
    // coin -> coin - algo 
    // account -> coin - algo - account_id
    // worker -> None
  }
}

// 
fn rollup_workers(Vec<WorkerMYSQL>){
  // create hashmap 
  // more or less copy workers from workers table to hash_worker

  // insert
}


fn rollup_accounts(Vec<WorkerMYSQL>){
  // create hashmap
  // generate key for each worker, add to dict
  // generate hash_account vec

  // insert
}
fn rollup_coins(Vec<WorkerMYSQL>){
  // create hashmap
  // generate key for each worker, add to dict
  // generate hash_coin vec

  // insert
}

