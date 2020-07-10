// pub mod users_routes{
  pub fn dog() -> &'static str {
    "Hello, dog!"
  }
use rocket;
  #[get("/cow")]
  pub fn cow() -> &'static str {
    println!("OMFG HI");
    "Hello, cow!"
  }

  pub fn create_route(router: rocket::Rocket){
    router.mount("/", routes![cow]);
  }
// }