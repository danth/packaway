use rocket::{get, Responder};

#[derive(Responder)]
#[response(content_type = "text/x-nix-cache-info")]
pub struct CacheInfoResponse(&'static str);

#[get("/nix-cache-info")]
pub fn get_cache_info() -> CacheInfoResponse {
    CacheInfoResponse("StoreDir: /nix/store\nWantMassQuery: 1\nPriority: 20\n")
}