// Based upon https://github.com/NixOS/nix/blob/646af7325d93f98802b989f8a8e008a25f7a4788/src/libutil/hash.cc#L87-L107

const BASE32_CHARACTERS: &'static str = "0123456789abcdfghijklmnpqrsvwxyz";

pub fn nar_hash_to_base32(nar_hash: &str) -> Result<String, hex::FromHexError> {
    let nar_hash = hex::decode(&nar_hash[7..])?;
    let mut nar_hash_32 = "".to_string();

    let mut n = (nar_hash.len() * 8 - 1) / 5;
    loop {
        let b = n * 5;
        let i = b / 8;
        let j = b % 8;
        let c = (nar_hash[i] >> j) | if i >= nar_hash.len() - 1 { 0 } else { nar_hash[i + 1] << (8 - j) };
        nar_hash_32.push_str(&BASE32_CHARACTERS[(c & 0x1f) as usize .. ((c & 0x1f) + 1) as usize]);

        if n == 0 {
            break;
        }
        n -= 1;
    }

    Ok(format!("sha256:{}", nar_hash_32))
}
