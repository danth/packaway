use rocket::{get, Responder, request::FromParam, http::Status};
use rocket_db_pools::Connection;

use crate::database::{Db, get_path_info, get_references};
use crate::signature::Key;

pub struct NarInfoRequest<'r>(&'r str);
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
pub struct NarInfoResponse(String);

#[get("/<hash>")]
pub async fn get_nar_info(mut db: Connection<Db>, hash: NarInfoRequest<'_>) -> Result<NarInfoResponse, Status> {
    let path_info = get_path_info(&mut db, hash.0)
        .await
        .map_err(|error| {
            eprintln!("SQLite error: {}", error);
            Status::InternalServerError
        })?
        .ok_or(Status::NotFound)?;

    let references = get_references(&mut db, &path_info)
        .await
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