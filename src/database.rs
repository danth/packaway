// This module queries the Nix database directly to avoid spawning a subprocess for every request.
// SQL statements based upon https://github.com/NixOS/nix/blob/ddb82ffda993d237d62d59578f7808a9d98c77fe/src/libstore/local-store.cc#L343-L412

use rocket::futures::TryStreamExt;
use rocket_db_pools::{sqlx, Database, Connection};

#[derive(Database)]
#[database("nix")]
pub struct Db(sqlx::SqlitePool);

pub struct StorePath(String);
impl StorePath {
    pub fn new(path: &str) -> Self {
        if let Some(path) = path.strip_prefix("/nix/store/") {
            Self(path.to_string())
        } else {
            Self(path.to_string())
        }
    }

    pub fn without_prefix(&self) -> String {
        self.0.clone()
    }

    pub fn with_prefix(&self) -> String {
        format!("/nix/store/{}", self.0)
    }
}

pub struct PathInfo {
    id: i64,
    pub path: StorePath,
    pub nar_hash: String,
    pub nar_size: i64,
    pub deriver: Option<StorePath>
}

fn nar_hash_to_base32(hash: &str) -> anyhow::Result<String> {
    let bytes = hex::decode(&hash[7..])?;
    let hash = nix_base32::to_nix_base32(&bytes);
    Ok(format!("sha256:{}", hash))
}

pub async fn get_path_info(db: &mut Connection<Db>, hash: &str) -> anyhow::Result<Option<PathInfo>> {
    let wanted_path = format!("/nix/store/{}", hash);

    let path_info = sqlx::query!(
        "SELECT id, path, hash, narSize, deriver FROM ValidPaths WHERE path >= ? LIMIT 1",
        wanted_path
    )
        .fetch_optional(&mut **db)
        .await?
        .map(|record| PathInfo {
            id: record.id,
            path: StorePath::new(&record.path),
            nar_hash: nar_hash_to_base32(&record.hash).unwrap(),
            nar_size: record.narSize.unwrap(),
            deriver: record.deriver.as_ref().map(|deriver| StorePath::new(deriver))
        })
        .filter(|path_info| {
            // If no direct match was found, the query may return the next path in alphabetical order
            path_info.path.with_prefix().starts_with(&wanted_path)
        });

    Ok(path_info)
}

pub async fn get_references(db: &mut Connection<Db>, path_info: &PathInfo) -> anyhow::Result<Vec<StorePath>> {
    let mut references = sqlx::query!(
        "SELECT path FROM Refs JOIN ValidPaths ON reference = id WHERE referrer = ?",
        path_info.id
    )
        .fetch(&mut **db)
        .map_ok(|record| StorePath::new(&record.path))
        .try_collect::<Vec<_>>()
        .await?;

    // Nix expects references in alphabetical order, especially for signatures
    references.sort_by_key(|path| path.without_prefix());

    Ok(references)
}
