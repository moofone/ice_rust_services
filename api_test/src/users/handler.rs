//use crate::connection::DbConn;
// use diesel::result::Error;
use std::env;
use crate::users;
// use crate::users::User;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::Json;

// #[get("/")]
// pub fn all() -> Result<Json<Vec<User>>, Status> {
   
// }

#[get("/")]
pub fn hello() -> &'static str {
    "Hello, world!"
}

//     // users::repository::all(&connection)
//     //     .map(|users| Json(users))
//     //     .map_err(|error| error_status(error))
// }

// fn error_status(error: Error) -> Status {
//     match error {
//         Error::NotFound => Status::NotFound,
//         _ => Status::InternalServerError
//     }
// }

// #[get("/<id>")]
// pub fn get(id: i32, connection: DbConn) -> Result<Json<User>, Status> {
//     users::repository::get(id, &connection)
//         .map(|user| Json(user))
//         .map_err(|error| error_status(error))
// }

// fn host() -> String {
//     env::var("ROCKET_ADDRESS").expect("ROCKET_ADDRESS must be set")
// }

// fn port() -> String {
//     env::var("ROCKET_PORT").expect("ROCKET_PORT must be set")
// }