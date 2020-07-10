extern crate shared;
use rocket;
// use users;

use shared::db_mysql::{
  establish_mysql_connection,
  MysqlPool,
};

#[get("/")]
fn hello() -> &'static str {
    share::mysql_get_user()
}

pub fn create_routes() {
  let mysql_pool = match establish_mysql_connection() {
    Ok(p) => p,
    Err(e) => panic!("MYSQL FAILED: {}", e),
  };

   rocket::ignite()
      .manage(mysql_pool)
      .mount("/users",
              routes![hello,
                  //.. todo ... 
                  ],
      ).launch();

  // rocket::ignite().mount("/", routes![hello]).launch();
}

// fn main() {
//   rocket::ignite().mount("/", routes![hello]).launch();
// }
//   // rocket::ignite()
  //     .manage(mysql_pool)
  //     .mount("/users",
  //             routes![users::handler::hello,
  //                 //.. todo ...
  //                 ],
  //     ).launch();
// }