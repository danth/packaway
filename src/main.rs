extern crate anyhow;
extern crate base64;
extern crate ed25519_dalek;
extern crate hex;
extern crate nix_base32;
extern crate nix_nar;
extern crate rocket;
extern crate rocket_db_pools;

mod database;
mod routes;
mod signature;

use database::Db;
use rocket::launch;
use rocket_db_pools::Database;
use routes::all_routes;

#[launch]
fn launch() -> _ {
    rocket::build()
        .attach(Db::init())
        .mount("/", all_routes())
}
