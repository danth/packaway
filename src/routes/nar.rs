use rocket::get;
use rocket::http::Status;
use rocket::response::Body;
use rocket::response::stream::ByteStream;
use std::path::Path;
use std::io::Read;

use crate::database::StorePath;

#[get("/nar/<path>")]
#[allow(unused_must_use)]
pub fn get_nar(path: &str) -> Result<ByteStream![Vec<u8>], Status> {
    let path = StorePath::new(path).with_prefix();
    let path = Path::new(&path);

    if !path.exists() {
        return Err(Status::NotFound);
    }

    let mut encoder = nix_nar::Encoder::new(path);

    Ok(ByteStream! {
        loop {
            let mut total_bytes_read = 0;
            let mut chunk = vec![0; Body::DEFAULT_MAX_CHUNK];

            // encoder.read returns a maximum of one operation per call. This becomes inefficient
            // when there are lots of small files to encode. To mitigate this, we repeatedly call
            // encoder.read until we have a full chunk.
            while total_bytes_read < Body::DEFAULT_MAX_CHUNK {
                match encoder.read(&mut chunk[total_bytes_read..]).unwrap() {
                    0 => break,
                    bytes_read => total_bytes_read += bytes_read
                }
            }

            if total_bytes_read > 0 {
                chunk.truncate(total_bytes_read);
                yield chunk;
            } else {
                break;
            }
        }
    })
}