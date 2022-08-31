// This module queries the Nix database directly to avoid spawning a subprocess for every request.
// SQL statements based upon https://github.com/NixOS/nix/blob/ddb82ffda993d237d62d59578f7808a9d98c77fe/src/libstore/local-store.cc#L343-L412

use crate::base32::nar_hash_to_base32;

#[derive(Debug)]
pub struct PrefixError;
impl std::fmt::Display for PrefixError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "database result was not prefixed with /nix/store")
    }
}
impl std::error::Error for PrefixError {}

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

fn open_database() -> anyhow::Result<sqlite::Connection> {
    let flags = sqlite::OpenFlags::new().set_read_only();
    let connection = sqlite::Connection::open_with_flags("/nix/var/nix/db/db.sqlite", flags)?;
    Ok(connection)
}

pub struct PathInfo {
    id: i64,
    pub path: StorePath,
    pub nar_hash: String,
    pub nar_size: i64,
    pub deriver: StorePath
}

pub fn get_path_info(hash: &str) -> anyhow::Result<Option<PathInfo>> {
    let wanted_path = format!("/nix/store/{}", hash);

    let connection = open_database()?;

    let mut statement = connection
        .prepare("SELECT id, path, hash, narSize, deriver FROM ValidPaths WHERE path >= ? LIMIT 1")?
        .bind(1, &*wanted_path)?;

    if let sqlite::State::Row = statement.next()? {
        let path_info = PathInfo {
            id: statement.read::<i64>(0)?,
            path: StorePath::new(&statement.read::<String>(1)?),
            nar_hash: nar_hash_to_base32(&statement.read::<String>(2)?)?,
            nar_size: statement.read::<i64>(3)?,
            deriver: StorePath::new(&statement.read::<String>(4)?)
        };

        // If no direct match was found, the query may return the next path in alphabetical order
        if path_info.path.with_prefix().starts_with(&wanted_path) {
            Ok(Some(path_info))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub fn get_references(path_info: &PathInfo) -> anyhow::Result<Vec<StorePath>> {
    let connection = open_database()?;

    let mut statement = connection
        .prepare("SELECT path FROM Refs JOIN ValidPaths ON reference = id WHERE referrer = ?")?
        .bind(1, path_info.id)?;

    let mut references = Vec::new();

    while let sqlite::State::Row = statement.next()? {
        let path = StorePath::new(&statement.read::<String>(0)?);
        references.push(path);
    }

    Ok(references)
}
