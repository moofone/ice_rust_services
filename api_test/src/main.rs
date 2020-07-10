#![feature(decl_macro, proc_macro_hygiene)]
#[macro_use] extern crate rocket;
mod users;
mod routes;
use shared::db_mysql::{
  establish_mysql_connection, helpers::kdablocks::insert_kdablocks_mysql,
  models::KDABlockMYSQLInsertable, MysqlPool,
};
// mod users_routes;

#[get("/test")]
fn hello() -> &'static str {
  "Hello, world!"
}

#[get("/donkey/test")]
fn donkey() -> &'static str {
  println!("OMFG HI");
  "Hello, Donkey!"
}


fn main() {
 
  //setup msqyl
  let mysql_pool = match establish_mysql_connection() {
    Ok(p) => p,
    Err(e) => panic!("MYSQL FAILED: {}", e),
  };


  rocket::ignite().manage(mysql_pool).mount("/", routes![
      hello, 
      donkey, 
      users::test::cow, 
      routes::users::users
    ]).launch();
}
