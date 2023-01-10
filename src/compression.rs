use serde::Deserialize;
use serde::Serialize;
use snap::raw::Decoder;
use snap::raw::Encoder;

pub fn compress<T>(x: &T) -> Vec<u8>
where
    T: Serialize,
{
    let serialized = postcard::to_stdvec(x).unwrap(); // Only errors on OOM
    compress_snap(&serialized)
}

pub fn decompress<T>(x: &[u8]) -> T
where
    T: for<'a> Deserialize<'a>,
{
    let decompressed = decompress_snap(x);
    postcard::from_bytes(&decompressed).unwrap() // Only errors on OOM
}

fn compress_snap(x: &[u8]) -> Vec<u8> {
    Encoder::new().compress_vec(x).unwrap()
}

fn decompress_snap(x: &[u8]) -> Vec<u8> {
    Decoder::new().decompress_vec(x).unwrap()
}
