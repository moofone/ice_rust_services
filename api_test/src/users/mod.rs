//#![allow(proc_macro_derive_resolution_fallback)]
//use shared::db_mysql::schema::users;
//use shared::db_mysql::schema::users;

extern crate shared;

pub mod handler;
pub mod router;
pub mod test;


// mark todo: Structs for User, etc shoudl all go in Shared
// pub mod repository;

// #[derive(Queryable, AsChangeset, Serialize, Deserialize)]
// #[table_name = "users"]
// pub struct User {
//     pub id: i32,
//     pub display_name: String,
//     pub email: String,
//     pub password: String,
// }

// move this to shared?
// struct InsertableUser {
//     pub display_name: String,
//     pub email: String,
//     pub password: String,
// }

// impl InsertableUser {
//   fn from_user(user: User) -> InsertableUser {
//       InsertableUser {
//         display_name: user.display_name,
//         email: user.email,
//         password: user.password,
//       }
//   }
// }