use rocket::{routes, Route};

mod cache_info;
mod nar;
mod nar_info;

pub fn all_routes() -> Vec<Route> {
    routes![cache_info::get_cache_info, nar::get_nar, nar_info::get_nar_info]
}