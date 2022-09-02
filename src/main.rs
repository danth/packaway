extern crate anyhow;
extern crate base64;
extern crate ed25519_dalek;
extern crate hex;
extern crate nix_base32;
extern crate nix_nar;
extern crate rocket;
extern crate sqlite;

mod database;
mod signature;

use rocket::{get, launch, routes, Responder};
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::response::stream::ByteStream;

use std::io::Read;
use std::path::Path;

use crate::database::{StorePath, get_path_info, get_references};
use crate::signature::Key;

#[derive(Responder)]
#[response(content_type = "text/x-nix-cache-info")]
struct CacheInfoResponse(&'static str);

#[get("/nix-cache-info")]
fn cache_info() -> CacheInfoResponse {
    CacheInfoResponse("StoreDir: /nix/store\nWantMassQuery: 1\nPriority: 20\n")
}

struct NarInfoRequest<'r>(&'r str);
impl<'r> FromParam<'r> for NarInfoRequest<'r> {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if let Some(hash) = param.strip_suffix(".narinfo") {
            Ok(Self(hash))
        } else {
            Err(param)
        }
    }
}

#[derive(Responder)]
#[response(content_type = "text/x-nix-narinfo")]
struct NarInfoResponse(String);

#[get("/<hash>")]
fn nar_info(hash: NarInfoRequest) -> Result<NarInfoResponse, Status> {
    let path_info = get_path_info(hash.0)
        .map_err(|error| {
            eprintln!("SQLite error: {}", error);
            Status::InternalServerError
        })?
        .ok_or(Status::NotFound)?;

    let references = get_references(&path_info)
        .map_err(|error| {
            eprintln!("SQLite error: {}", error);
            Status::InternalServerError
        })?;

    let key = Key::load()
        .map_err(|error| {
            eprintln!("Signature error: {}", error);
            Status::InternalServerError
        })?;
    let signature = key.sign(&path_info, &references)
        .map_err(|error| {
            eprintln!("Signature error: {}", error);
            Status::InternalServerError
        })?;

    let text = match path_info.deriver {
        Some(deriver) => format!(
            "StorePath: {}\nURL: nar/{}\nCompression: none\nFileHash: {}\nFileSize: {}\nNarHash: {}\nNarSize: {}\nReferences: {}\nDeriver: {}\nSig: {}\n",
            path_info.path.with_prefix(),
            path_info.path.without_prefix(),
            path_info.nar_hash,
            path_info.nar_size,
            path_info.nar_hash,
            path_info.nar_size,
            references.iter().map(|r| r.without_prefix()).collect::<Vec<String>>().join(" "),
            deriver.without_prefix(),
            signature
        ),
        None => format!(
            "StorePath: {}\nURL: nar/{}\nCompression: none\nFileHash: {}\nFileSize: {}\nNarHash: {}\nNarSize: {}\nReferences: {}\nSig: {}\n",
            path_info.path.with_prefix(),
            path_info.path.without_prefix(),
            path_info.nar_hash,
            path_info.nar_size,
            path_info.nar_hash,
            path_info.nar_size,
            references.iter().map(|r| r.without_prefix()).collect::<Vec<String>>().join(" "),
            signature
        )
    };

    Ok(NarInfoResponse(text))
}

#[get("/nar/<path>")]
#[allow(unused_must_use)]
fn nar(path: &str) -> Result<ByteStream![Vec<u8>], Status> {
    let path = StorePath::new(path).with_prefix();
    let path = Path::new(&path);

    if !path.exists() {
        return Err(Status::NotFound);
    }

    let mut encoder = nix_nar::Encoder::new(path);

    Ok(ByteStream! {
        loop {
            // Stream chunks of no more than 10MiB
            let mut buffer = vec![0; 1024 * 1024 * 10];

            let bytes_read = encoder.read(&mut buffer).unwrap();

            if bytes_read > 0 {
                buffer.truncate(bytes_read);
                yield buffer;
            } else {
                break;
            }
        }
    })
}

#[launch]
fn launch() -> _ {
    rocket::build().mount("/", routes![cache_info, nar_info, nar])
}
