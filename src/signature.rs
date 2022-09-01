use ed25519_dalek::{Keypair, Signer};

use std::{env, fs};

use crate::database::{PathInfo, StorePath};

#[derive(Debug)]
pub struct KeyFormatError;

impl std::fmt::Display for KeyFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "secret key should be in the format «name»:«base64 encoded key»")
    }
}

impl std::error::Error for KeyFormatError {}

pub struct Key {
    name: String,
    key: Keypair
}

impl Key {
    pub fn load() -> anyhow::Result<Key> {
        let path = env::var("NIX_SECRET_KEY_FILE")?;
        let text = fs::read_to_string(&path)?;

        let (name, encoded_bytes) = text.split_once(':').ok_or(KeyFormatError)?;

        let bytes = base64::decode(encoded_bytes).map_err(|_| KeyFormatError)?;
        let key = Keypair::from_bytes(&bytes).map_err(|_| KeyFormatError)?;

        Ok(Key { name: name.to_string(), key })
    }

    pub fn sign(&self, path_info: &PathInfo, references: &[StorePath]) -> anyhow::Result<String> {
        let fingerprint = format!(
            "1;{};{};{};{}",
            path_info.path.with_prefix(),
            path_info.nar_hash,
            path_info.nar_size,
            references.iter().map(|r| r.with_prefix()).collect::<Vec<String>>().join(",")
        );

        let signature = self.key.sign(fingerprint.as_bytes());
        let signature = base64::encode(signature.to_bytes());

        Ok(format!("{}:{}", self.name, signature))
    }
}
